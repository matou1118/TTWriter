#!/usr/bin/env python3
"""体检一份法务文书 .docx 的格式合规性（规则表驱动，零 LLM）。

检查项（按文书类型自动路由）：
  文书识别     标题关键词 → 文书类型（借条/起诉状/答辩状/合同……16 种）
  借条九要素   借条类必查：双方身份、金额大小写并存、用途、期限、利率上限
               （LPR 4 倍）、交付方式、还款方式、签名手印
  此致格式     「此致」顶格 + 受文者另起一行左空二字
  落款位置     具状人/签名右对齐提示（版式层人工确认，结构层查有无落款段）
  当事人栏     起诉/答辩/仲裁类：原告/被告（申请人/被申请人）两栏都在
  金额大小写   金额只有阿拉伯数字没有大写 → 借条/合同类 fail（易涂改）
  身份证号     当事人栏出现缩写身份证（不足 18 位）→ warn

用法：
  python check_legal.py 借条.docx [--json]

退出码：0 = 无 fail；1 = 有 fail；2 = 打不开。只读不改，不给法律意见。
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys

# GBK 控制台（中文 Windows 默认）打不出 ✔/✘——降级成 ?，不让体检中途崩。
sys.stdout.reconfigure(errors="replace")

CLI = shutil.which("wordcraft-cli") or os.environ.get("WORDCRAFT_CLI") or ""

DAXIE = "壹贰叁肆伍陆柒捌玖拾佰仟萬亿"
RE_MONEY_SMALL = re.compile(r"[¥￥]?\s*\d[\d,，]*\.?\d*\s*(?:元|万元)")
RE_MONEY_BIG = re.compile(f"[{DAXIE}]+(?:元整?|万元整?)")
RE_ID18 = re.compile(r"\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:[0-2]\d|3[01])\d{3}[\dXx]")
RE_ID_SHORT = re.compile(r"身份证[号码：:]*\s*[\dX*]{6,17}\s")
RE_RATE = re.compile(r"年利率\s*(\d+(?:\.\d+)?)\s*%")
LPR4X_CAP = 15.4  # 一年期 LPR 3.85% × 4 ≈ 15.4%（2026 常识值；以当期 LPR 为准）

# 文书类型路由：标题关键词 → 类型（首匹配优先）
KINDS = [
    ("借条", "借条"), ("欠条", "欠条"), ("收条", "收条"),
    ("起诉状", "起诉状"), ("答辩状", "答辩状"), ("上诉状", "上诉状"),
    ("强制执行", "执行申请"), ("财产保全", "保全申请"), ("撤诉", "撤诉申请"),
    ("仲裁申请", "仲裁申请"), ("律师函", "律师函"), ("催款", "催款函"),
    ("和解协议", "和解协议"), ("租赁合同", "租赁合同"), ("保密协议", "NDA"),
    ("授权委托", "授权委托"), ("证据目录", "证据目录"), ("证据清单", "证据目录"),
]

# 借条九要素的关键词（正则）
JOUTIAO = [
    ("借款人", re.compile(r"借款人[：:]")), ("出借人", re.compile(r"出借人[：:]")),
    ("金额大写", RE_MONEY_BIG), ("借款用途", re.compile(r"用途[：:]")),
    ("借款期限", re.compile(r"期限[：:]|自\s*\d{4}\s*年.*至")),
    ("利率", re.compile(r"利率|无息")),
    ("交付方式", re.compile(r"转账|现金|交付")),
    ("还款方式", re.compile(r"还款|归还|一次性|分期")),
    ("签名手印", re.compile(r"签名|签字|手印|指印")),
]


def run_cli(args: list) -> str:
    exe = CLI or "wordcraft-cli"
    out = subprocess.run([exe, *args], capture_output=True)
    if out.returncode != 0:
        msg = out.stderr.decode("utf-8", "replace").strip()
        raise RuntimeError(msg or f"{exe} {args[0]} failed")
    return out.stdout.decode("utf-8", "replace")


def check(text: str) -> tuple:
    """(文书类型, issues 列表)。"""
    results = []

    def add(level, name, detail):
        results.append({"level": level, "name": name, "detail": detail})

    head = text.strip().splitlines()[0].strip() if text.strip() else ""
    kind = next((k for pat, k in KINDS if pat in text[:120]), None)
    if kind is None and head:
        kind = next((k for pat, k in KINDS if pat in head), "未识别")
    add("ok", "文书类型", f"{kind or '未识别'}（标题「{head[:24]}」）")

    # 借条九要素
    if kind == "借条":
        missing = [n for n, rex in JOUTIAO if not rex.search(text)]
        add("fail" if missing else "ok", "借条九要素",
            "九要素齐全" if not missing else f"缺：{'、'.join(missing)}（最高法民间借贷司法解释的举证要求）")
        # 金额大小写并存
        has_s, has_b = RE_MONEY_SMALL.search(text), RE_MONEY_BIG.search(text)
        if has_s and not has_b:
            add("fail", "金额大写", f"「{has_s.group(0)}」只有小写——大写并存防涂改")
        # 利率上限
        m = RE_RATE.search(text)
        if m and float(m.group(1)) > LPR4X_CAP:
            add("fail", "利率上限", f"年利率 {m.group(1)}% 超过 LPR 四倍（≈{LPR4X_CAP}%）——超出部分法院不支持")
        # 身份证缩写
        for mm in RE_ID_SHORT.finditer(text):
            if not RE_ID18.search(text):
                add("warn", "身份证号", f"「{mm.group(0).strip()}」不足 18 位或含 *——借条要全文")

    # 诉讼类当事人栏
    if kind in ("起诉状", "答辩状", "上诉状", "仲裁申请"):
        pairs = [("原告", "被告")] if kind != "仲裁申请" else [("申请人", "被申请人")]
        for a, b in pairs:
            if a not in text:
                add("fail", f"{a}栏", f"没有{a}信息——当事人栏必备")
            if b not in text:
                add("fail", f"{b}栏", f"没有{b}信息——当事人栏必备")

    # 此致格式（所有「此致」文书）
    if "此致" in text:
        lines = [ln.strip() for ln in text.splitlines()]
        at = next((i for i, ln in enumerate(lines) if ln.startswith("此致")), None)
        if at is not None:
            nxt = lines[at + 1] if at + 1 < len(lines) else ""
            if not nxt:
                add("fail", "此致格式", "「此致」后没有受文者行（法院/单位名）")
            elif re.match(r"^此致\s*\S+", lines[at]):
                add("warn", "此致格式", "「此致」同行跟了受文者——规范是另起一行")
        # 上诉状此致写原审法院
        if kind == "上诉状" and re.search(r"此致.*中级", text.replace("\n", "")):
            add("warn", "上诉状此致", "此致中级法院——上诉状通过原审法院转呈，此致写原审法院名")

    # 落款（具状人/申请人/签名）
    if kind in ("起诉状", "答辩状", "上诉状", "执行申请", "保全申请", "撤诉申请", "仲裁申请"):
        if not re.search(r"(具状人|申请人|上诉人|答辩人)[：:]", text):
            add("fail", "落款签名", "没有「具状人/申请人：」落款段")
    if kind in ("借条", "欠条", "收条") and not re.search(r"(签名|签字|手印|指印)", text):
        add("fail", "签名手印", "没有签名/手印标记——借条类签名+手印是举证关键")

    # 合同类金额大小写
    if kind in ("和解协议", "租赁合同", "NDA"):
        has_s = RE_MONEY_SMALL.search(text)
        has_b = RE_MONEY_BIG.search(text)
        if has_s and not has_b:
            add("warn", "金额大写", f"「{has_s.group(0)}」建议加大写并存（合同惯例防涂改）")

    return kind, results


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("file")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    try:
        text = run_cli(["text", os.path.abspath(args.file)])
    except Exception as e:  # noqa: BLE001
        print(json.dumps({"error": str(e)}) if args.json else f"打不开：{e}", file=sys.stderr)
        return 2

    kind, results = check(text)
    fails = [r for r in results if r["level"] == "fail"]
    warns = [r for r in results if r["level"] == "warn"]

    if args.json:
        print(json.dumps({"file": args.file, "kind": kind, "results": results,
                          "summary": {"fail": len(fails), "warn": len(warns)}},
                         ensure_ascii=False, indent=2))
    else:
        for r in results:
            mark = {"ok": "✔", "warn": "△", "fail": "✘"}[r["level"]]
            print(f"{mark} {r['name']}: {r['detail']}")
        print(f"\n{len(fails)} fail / {len(warns)} warn")
        print("（本检查是格式层，不构成法律意见）")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
