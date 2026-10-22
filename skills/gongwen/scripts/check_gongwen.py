#!/usr/bin/env python3
"""体检一份 .docx 的公文合规性：版式层（GB/T 9704）+ 文本规范层（GB/T 15834/15835 与行文惯例）。

版式层（可程序化断言的国标数值）：
  版心       上 37mm±1 / 下 35mm±1 / 左 28mm±1 / 右 26mm±1 → 104.9 / 99.2 / 79.4 / 73.7 磅
  行距       正文行距固定值 28 磅（±2 容差）
  字号       正文 3 号（16pt）
  中文字体   正文中文字体应为仿宋（含朱雀仿宋兜底名）
  标题       2 号（22pt）居中
  页码       页脚存在

文本规范层（退文高频项，规则表驱动、零 LLM）：
  发文字号   〔2026〕15 号：六角括号〔〕不是【】；年份四位；序号不编虚位（15 不写 015）
  成文日期   用阿拉伯数字（2026 年 10 月 22 日）；汉字日期（二〇二六年）是 2012 版之前的写法
  层次序数   一、→（一）→ 1. →（1）：只能逐级深入，不能跳级（一、下面直接 1.）
  概数       二三个/三四天：相邻数字概数中间不加顿号；公文量词前「2」作概数宜作「两」
  高频错词   其它→其他；截止/截至混用；制订/制定混用；权力/权利混用（词表可扩）
  标点       句内括号后直接接句号；书名号嵌套用〈〉；「：」后再用「：」

用法：
  python check_gongwen.py 公文.docx              # 人类可读报告
  python check_gongwen.py 公文.docx --json       # 机器可读（供 CI / agent）
  python check_gongwen.py 公文.docx --fix -o 修.docx
                                                 # 修复确定性错误（括号/虚位/其它/概数顿号），
                                                 # 版式与其余问题仍报告

退出码：0 = 无 fail；1 = 有 fail（或 --fix 后仍有 fail）；2 = 文件打不开。--fix 只改文本层确定性错误，从不动版式、不动需要语境判断的项（日期、跳级、易混词）。
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

MM_TO_PT = 72.0 / 25.4

# 国标版心（毫米）→ 磅（四舍五入到 0.1）
MARGINS_PT = {
    "top": round(37 * MM_TO_PT, 1),
    "bottom": round(35 * MM_TO_PT, 1),
    "left": round(28 * MM_TO_PT, 1),
    "right": round(26 * MM_TO_PT, 1),
}
TOL = 3.0  # 磅；国标 ±1mm ≈ 2.8pt，取整为 3

FANGSONG = ("仿宋", "仿宋_GB2312", "朱雀仿宋", "FangSong", "Fangsong_GB2312")

# wordcraft-cli on PATH, or the WORDCRAFT_CLI environment variable.
CLI = shutil.which("wordcraft-cli") or os.environ.get("WORDCRAFT_CLI") or ""


def run_cli(args: list) -> str:
    """Run wordcraft-cli, raising RuntimeError with its message on failure."""
    exe = CLI or "wordcraft-cli"
    out = subprocess.run([exe, *args], capture_output=True)
    if out.returncode != 0:
        msg = out.stderr.decode("utf-8", "replace").strip()
        raise RuntimeError(msg or f"{exe} {args[0]} failed (exit {out.returncode})")
    return out.stdout.decode("utf-8", "replace")


def inspect(path: str) -> dict:
    """wordcraft-cli inspect 的 JSON（结构、段落属性）。"""
    return json.loads(run_cli(["inspect", path]))


def text_of(path: str) -> str:
    return run_cli(["text", path])


def fix_commands(text: str) -> list:
    """确定性文本修复的 wordcraft 命令（edit.replaceAll）。

    只收无歧义替换：括号形状、虚位序号、其它→其他、概数顿号。
    日期/跳级/易混词需要语境或结构调整，不属于「确定性」，不自动改。
    """
    cmds = []
    if RE_WENHAO_SQUARE.search(text):
        # 【2026】15 号 → 〔2026〕15 号：对每个出现的方头括号年份单独替换。
        for m in RE_WENHAO_SQUARE.finditer(text):
            cmds.append({"command": "edit.replaceAll", "params": {
                "text": f"【{m.group(1)}】", "with": f"〔{m.group(1)}〕", "matchCase": True}})
    for m in RE_WENHAO_PADDED.finditer(text):
        num = m.group(2).lstrip("0") or "0"
        cmds.append({"command": "edit.replaceAll", "params": {
            "text": f"〔{m.group(1)}〕{m.group(2)}号", "with": f"〔{m.group(1)}〕{num}号", "matchCase": True}})
    if "其它" in text:
        cmds.append({"command": "edit.replaceAll", "params": {"text": "其它", "with": "其他", "matchCase": True}})
    for m in RE_GAISHU.finditer(text):
        cmds.append({"command": "edit.replaceAll", "params": {
            "text": m.group(0).replace(" ", ""), "with": m.group(0).replace("、", "").replace(" ", ""), "matchCase": True}})
    return cmds


def apply_fix(src: str, out: str, text: str) -> int:
    """把 fix_commands 写成脚本跑掉，返回修复条数。"""
    cmds = fix_commands(text)
    if not cmds:
        return 0
    # run 脚本的 file/save 都按脚本所在目录（temp）解析——必须传绝对路径。
    script = {"file": os.path.abspath(src), "commands": cmds, "save": [os.path.abspath(out)]}
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(script, f, ensure_ascii=False, indent=1)
        path = f.name
    try:
        exe = CLI or "wordcraft-cli"
        r = subprocess.run([exe, "run", "--script", path], capture_output=True)
        if r.returncode != 0:
            msg = r.stderr.decode("utf-8", "replace")
            print(f"修复执行失败：{msg}", file=sys.stderr)
            return 0
    finally:
        os.unlink(path)
    return len(cmds)


# ------------------------------------------------------------------
# 文本规范层（GB/T 15834 / 15835 / 行文惯例）

# 发文字号：机关代字 + 〔年份〕 + 序号。错的括号、虚位序号都报。
RE_WENHAO = re.compile(r"[【〔\[（(]\s*(\d{4})\s*[】〕\]）)]\s*[\s〔【\[]?(\d{1,4})\s*号")
RE_WENHAO_SQUARE = re.compile(r"【\s*(\d{4})\s*】\s*(\d{1,4})\s*号")
RE_WENHAO_PADDED = re.compile(r"[〔【[]\s*(\d{4})\s*[〕】\]]\s*0*(\d{3})\s*号")

# 成文日期：汉字年份（二〇二六）是 2012 前旧式；阿拉伯日期为正。
RE_HAN_DATE = re.compile(r"[〇一二三四五六七八九]{4}\s*年")
RE_ARAB_DATE = re.compile(r"\d{4}\s*年\s*\d{1,2}\s*月\s*\d{1,2}\s*日")

# 层次序数：汉字序数段（一、）、带括序数段（（一））、数字序数段（1.）。
RE_LEVEL1 = re.compile(r"^([一二三四五六七八九十]+)\s*、")
RE_LEVEL2 = re.compile(r"^（([一二三四五六七八九十]+)）")
RE_LEVEL3 = re.compile(r"^(\d+)\s*[.、]")

# 概数：相邻汉字数字 + 顿号 + 相邻汉字数字（二、三个 → 二三个）。
RE_GAISHU = re.compile(r"[一二三四五六七八九]\s*、\s*[一二三四五六七八九十]")

# 高频错词（误 → 正）。词表可扩；报 warn 级（语境依赖，人工确认）。
WORD_PAIRS = [
    ("其它", "其他"),
    ("做 出决定", "作出决定"),
]
CONFUSABLE = {
    "截止": "截至（截至 + 时间点；截止 + 时间段/日期）",
    "制订": "制定（制定=定成不得更动的；制订=创制拟订的）",
    "权力": "权利（权力=职权；权利=法律赋予的利益）",
    "附件∶": "附件：（公文里冒号用全角）",
}


def check_text(text: str) -> list:
    """文本规范检查（每个 issue 一条，带位置线索）。"""
    issues = []

    def hit(level, name, where, fix):
        issues.append({"level": level, "name": name, "detail": f"{where} —— {fix}"})

    lines = text.splitlines()

    # 发文字号
    for m in RE_WENHAO_SQUARE.finditer(text):
        hit("fail", "发文字号括号",
            f"【{m.group(1)}】{m.group(2)}号",
            "用六角括号〔〕，不是方头括号【】（GB/T 9704 4.4.2 发文字号）")
    for m in RE_WENHAO_PADDED.finditer(text):
        hit("fail", "发文字号虚位",
            f"〔{m.group(1)}〕{m.group(2)}号",
            "序号不编虚位：15 号不写 015 号")
    has_wenhao = bool(RE_WENHAO.search(text))

    # 成文日期
    han = RE_HAN_DATE.search(text)
    if han and not RE_ARAB_DATE.search(text):
        hit("fail", "成文日期数字",
            han.group(0),
            "2012 版国标改用阿拉伯数字：写 2026 年 10 月 22 日，不写二〇二六年十月二十二日")
    elif han:
        hit("warn", "成文日期数字",
            han.group(0),
            "发现汉字年份；若它是成文日期应改阿拉伯数字（古文引用则保留）")

    # 层次序数跳级：一、下面直接（1. 或（1）），跳过（一）
    last_level = 0  # 0=无, 1=一、, 2=（一）, 3=1.
    for i, ln in enumerate(lines, 1):
        ln = ln.strip()
        if RE_LEVEL1.match(ln):
            last_level = 1
        elif RE_LEVEL2.match(ln):
            if last_level not in (0, 1, 2):
                hit("fail", "层次序数跳级", f"第{i}行「{ln[:12]}」",
                    "（一）出现在 1. 之后：层次序数应逐级一、→（一）→ 1. →（1）")
            last_level = 2
        elif RE_LEVEL3.match(ln):
            if last_level == 0:
                hit("warn", "层次序数", f"第{i}行「{ln[:12]}」",
                    "顶层直接用 1.——顶层应用一、（1. 是第三层）")
            elif last_level == 1:
                hit("fail", "层次序数跳级", f"第{i}行「{ln[:12]}」",
                    "一、下面直接 1.：应先（一）再 1.")
            last_level = 3

    # 概数顿号
    for m in RE_GAISHU.finditer(text):
        hit("fail", "概数顿号", m.group(0).replace(" ", ""),
            "相邻数字概数中间不加顿号：写「二三个」，不写「二、三个」（GB/T 15835）")

    # 高频错词
    for wrong, right in WORD_PAIRS:
        if wrong in text:
            hit("fail", "高频错词", wrong, f"应为「{right}」")
    for w, note in CONFUSABLE.items():
        if w in text:
            hit("warn", "易混词", w, note + "——语境自检")

    # 标点：句内括号后句号
    for m in re.finditer(r"）\s*。\s*[\u4e00-\u9fff]", text):
        hit("warn", "括号后句号", f"…{m.group(0)}…",
            "句内括号（…）后接正文时不再加句号；只有括号收束全句时才句号")
    # 书名号嵌套
    for m in re.finditer(r"《[^《》]*《", text):
        hit("warn", "书名号嵌套", m.group(0)[:20],
            "书名号里再套书名号用单书名号〈〉：《条例》→《〈…〉》")

    if not issues and has_wenhao:
        issues.append({"level": "ok", "name": "文本规范", "detail": "未发现发文字号/日期/序数/错词问题"})
    return issues


# ------------------------------------------------------------------
# 版式层（GB/T 9704 数值）


def check(doc: dict) -> list:
    """每项检查返回 dict（level/name/detail）：level ∈ ok|warn|fail。"""
    results = []

    def add(level, name, ok, detail):
        results.append({"level": "ok" if ok else level, "name": name, "detail": detail})

    # 版心（section 属性）
    sections = doc.get("sections") or []
    if not sections:
        add("fail", "版心", False, "文档没有 section 信息（inspect 输出结构变了？）")
    else:
        s = sections[0].get("props") or {}
        for side, want in MARGINS_PT.items():
            got = s.get(f"margin{side.capitalize()}")
            if got is None:
                add("fail", f"版心·{side}", False, "inspect 未报告该边距")
                continue
            got = round(float(got), 1)
            ok = abs(got - want) <= TOL
            add("fail" if not ok else "ok", f"版心·{side}", ok,
                f"{got}pt（标准 {want}pt，容差 ±{TOL}）")

    # 正文行距 / 字号 / 字体（抽查段落；属性在 props 下，字号字体可被样式给出）
    paras = [b for b in (doc.get("blocks") or []) if b.get("type") == "paragraph"]
    paras = [b for b in paras if "eading" not in (b.get("style") or "") and (b.get("style") or "") != "Title"]
    if not paras:
        add("warn", "正文", False, "没有可检查的正文段落")
    else:
        bad_line = bad_size = bad_font = 0
        for p in paras:
            pr = p.get("props") or {}
            ls = pr.get("lineSpacing") or {}
            if ls.get("rule") == "exactly":
                if abs(float(ls.get("value", 0)) - 28.0) > 2.0:
                    bad_line += 1
            else:
                bad_line += 1
            size = pr.get("size")
            if size is not None and abs(float(size) - 16.0) > 0.5:
                bad_size += 1
            elif size is None:
                bad_size += 1
            if str(pr.get("fontEa") or "") not in FANGSONG:
                bad_font += 1
        n = len(paras)
        add("fail", "行距固定值 28pt", bad_line == 0, f"{n - bad_line}/{n} 段合规（缺或非固定值的都算）")
        add("fail", "正文 3 号（16pt）", bad_size == 0, f"{n - bad_size}/{n} 段合规（属性缺省时按样式，需人工确认）")
        add("warn", "正文仿宋", bad_font == 0, f"{n - bad_font}/{n} 段合规（兜底字体也算）")

    # 标题
    heads = [b for b in (doc.get("blocks") or []) if (b.get("style") or "") in ("Title", "Heading1")]
    if not heads:
        add("warn", "标题", False, "没有 Title/Heading1 段落——公文标题用样式排才能被程序看到")
    else:
        h = heads[0].get("props") or {}
        sz, align = h.get("size"), h.get("align")
        ok = (sz is not None and abs(float(sz) - 22.0) <= 1.0 and align == "center") or (sz is None and align == "center")
        add("fail", "标题 2 号居中", ok, f"size={sz} align={align}（size 缺省走样式时需人工确认）")

    # 页脚（页码）：section 的 footers 指针非空
    footers = (sections[0].get("props") or {}).get("footers") if sections else None
    has_footer = bool(footers and any(v is not None for v in footers.values() if not isinstance(v, (int,)) or v is not None)) or bool(footers)
    add("fail", "页码（页脚）", has_footer, "页脚存在" if has_footer else "没有页脚——公文必须编页码")

    return results


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("file")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--layout-only", action="store_true", help="只跑版式层，不跑文本规范层")
    ap.add_argument("--fix", action="store_true", help="修复确定性文本错误（需 -o 输出路径）")
    ap.add_argument("-o", "--out", default=None, help="--fix 的输出文件（缺省在原名后加 -fixed）")
    args = ap.parse_args()

    try:
        doc = inspect(args.file)
        text = "" if args.layout_only else text_of(args.file)
    except Exception as e:  # noqa: BLE001 - 报告给用户，不炸
        print(json.dumps({"error": str(e)}) if args.json else f"打不开：{e}", file=sys.stderr)
        return 2

    if args.fix:
        out = args.out or re.sub(r"(\.\w+)?$", r"-fixed\1", args.file, count=1)
        n = apply_fix(args.file, out, text)
        if n:
            print(f"[fix] {n} 处确定性错误已修复 → {out}")
            # 修复后重查：剩余问题（版式层 + 需语境判断项）仍报告
            try:
                doc = inspect(out)
                text = "" if args.layout_only else text_of(out)
            except Exception as e:  # noqa: BLE001
                print(f"[fix] 复查失败：{e}", file=sys.stderr)
        else:
            print("[fix] 没有可自动修复的确定性错误")

    results = check(doc)
    if not args.layout_only:
        results += check_text(text)

    fails = [r for r in results if r["level"] == "fail"]
    warns = [r for r in results if r["level"] == "warn"]

    if args.json:
        print(json.dumps({"file": args.file, "results": results,
                          "summary": {"fail": len(fails), "warn": len(warns),
                                      "ok": len(results) - len(fails) - len(warns)}},
                         ensure_ascii=False, indent=2))
    else:
        for r in results:
            mark = {"ok": "✔", "warn": "△", "fail": "✘"}[r["level"]]
            print(f"{mark} {r['name']}: {r['detail']}")
        print(f"\n{len(fails)} fail / {len(warns)} warn / {len(results) - len(fails) - len(warns)} ok")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
