#!/usr/bin/env python3
"""体检一份 .docx 的 GB/T 9704 版式合规性（自动核对，不用人眼盯 PNG）。

检查项（每项都是可程序化断言的国标数值）：
  版心       上 37mm±1 / 下 35mm±1 / 左 28mm±1 / 右 26mm±1 → 104.9 / 99.2 / 79.4 / 73.7 磅
  行距       正文行距固定值 28 磅（±2 容差）
  字号       正文 3 号（16pt）
  中文字体   正文中文字体应为仿宋（含朱雀仿宋兜底名）
  标题       2 号（22pt）居中
  页码       页脚存在

用法：
  python check_gongwen.py 公文.docx              # 人类可读报告
  python check_gongwen.py 公文.docx --json       # 机器可读（供 CI / agent）

退出码：0 = 全过；1 = 有 FAIL；2 = 文件打不开。它只读不改。
"""
import argparse
import json
import subprocess
import sys

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

import os
import shutil

# wordcraft-cli on PATH, or the repo's own release build, or beside this script's install.
CLI = (
    shutil.which("wordcraft-cli")
    or os.environ.get("WORDCRAFT_CLI")
    or ""
)


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


def check(doc: dict) -> list:
    """每项检查返回 (level, name, ok, detail)：level ∈ ok|warn|fail。"""
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
    args = ap.parse_args()

    try:
        doc = inspect(args.file)
    except Exception as e:  # noqa: BLE001 - 报告给用户，不炸
        print(json.dumps({"error": str(e)}) if args.json else f"打不开：{e}", file=sys.stderr)
        return 2

    results = check(doc)
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
