#!/usr/bin/env python3
"""批量发文：一份定稿公文 + 收文单位名单 → 每单位一份带落款版记的印发版。

用法：
  python batch_issue.py 公文.docx 名单.xlsx -o out/            # 名单字段：单位
  python batch_issue.py 公文.docx 名单.csv -o out/ --field 收文单位
  python batch_issue.py 公文.docx 名单.xlsx -o out/ --pdf     # 同时出 PDF

名单字段要求（缺一即报错，不猜）：
  单位（或 --field 指定的字段名）——版记「抄送：______」替换的目标

做什么：把版记占位「抄送：______」逐单位替换。**不改公文正文一个字**——
这是 gongwen×mailmerge 的联动：mailmerge 的名单读取（xlsx/csv/gbk 编码自动探测），
gongwen 的正文不动原则。每单位产出 公文-<单位>.docx（+可选 .pdf）。
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

CLI = shutil.which("wordcraft-cli") or os.environ.get("WORDCRAFT_CLI") or ""
ILLEGAL = re.compile(r'[\\/:*?"<>|\r\n\t]')
PLACEHOLDER = "抄送：______"


def run_script(script: dict, capture: bool = False) -> list:
    """跑 wordcraft-cli run --script（--print 捕获每条命令的 JSON 结果行）。

    脚本里的 file/save 路径按脚本所在目录（temp）解析——统一转绝对路径再写。
    """
    exe = CLI or "wordcraft-cli"
    if "file" in script:
        script["file"] = os.path.abspath(script["file"])
    script["save"] = [os.path.abspath(str(s)) for s in script.get("save") or []]
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(script, f, ensure_ascii=False)
        path = f.name
    try:
        r = subprocess.run([exe, "run", "--script", path, "--print"], capture_output=True)
        if r.returncode != 0:
            msg = r.stderr.decode("utf-8", "replace").strip()
            raise RuntimeError(msg or "wordcraft-cli run failed")
        out = r.stdout.decode("utf-8", "replace")
    finally:
        os.unlink(path)
    results = []
    if capture:
        for line in out.splitlines():
            head, _, rest = line.partition(": ")
            if rest.startswith("{"):
                try:
                    results.append((head, json.loads(rest)))
                except json.JSONDecodeError:
                    pass
    return results


def unit_list(roster: str, field: str) -> list:
    """名单里 field 列的全部值（走 mailings.recipients + editRecipients 读表）。"""
    pairs = run_script({"commands": [
        {"command": "mailings.recipients", "params": {"path": os.path.abspath(roster)}},
        {"command": "mailings.editRecipients", "params": {}},
    ]}, capture=True)
    info = next((v for k, v in pairs if k == "mailings.editRecipients"), None)
    if info is None:
        raise RuntimeError(f"名单读取失败：{roster}")
    headers = info.get("headers") or []
    rows = info.get("rows") or []
    if field not in headers:
        raise RuntimeError(f"名单没有字段「{field}」；可用字段：{headers}")
    col = headers.index(field)
    units = [str(r[col]) for r in rows if col < len(r) and str(r[col]).strip()]
    if not units:
        raise RuntimeError(f"字段「{field}」是空的")
    return units


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("docx", help="定稿公文 .docx（正文不再改）")
    ap.add_argument("roster", help="收文单位名单 .xlsx/.csv/.tsv/.txt")
    ap.add_argument("-o", "--out", type=Path, default=Path("batch-out"), help="输出目录")
    ap.add_argument("--field", default="单位", help="名单里的单位字段名（默认「单位」）")
    ap.add_argument("--pdf", action="store_true", help="同时导出 PDF")
    args = ap.parse_args()

    if not Path(args.docx).exists() or not Path(args.roster).exists():
        sys.exit("输入文件不存在")

    try:
        units = unit_list(args.roster, args.field)
    except RuntimeError as e:
        sys.exit(str(e))

    # 公文里得有占位行；没有就提醒（不擅自添加版记）。cwd 跑 text。
    text = subprocess.run([CLI or "wordcraft-cli", "text", os.path.abspath(args.docx)], capture_output=True)
    body = text.stdout.decode("utf-8", "replace")
    if PLACEHOLDER not in body:
        sys.exit(f"公文里没有占位「{PLACEHOLDER}」——在版记抄送行留 {PLACEHOLDER} 后再跑批量。")

    args.out.mkdir(parents=True, exist_ok=True)
    print(f"[名单] {len(units)} 个收文单位：{'、'.join(units[:5])}{'…' if len(units) > 5 else ''}")

    n_ok = 0
    for unit in units:
        safe = ILLEGAL.sub("_", unit).strip() or "unnamed"
        doc_out = args.out / f"{Path(args.docx).stem}-{safe}.docx"
        saves = [str(doc_out.resolve())]
        if args.pdf:
            saves.append(str(doc_out.with_suffix(".pdf").resolve()))
        try:
            run_script({"file": args.docx, "commands": [
                {"command": "edit.replaceAll", "params": {"text": PLACEHOLDER, "with": f"抄送：{unit}"}},
            ], "save": saves})
            n_ok += 1
        except RuntimeError as e:
            print(f"[失败] {unit}: {e}", file=sys.stderr)

    print(f"[完成] {n_ok}/{len(units)} 份 → {args.out}/")
    print("[提醒] 每份的版记已换单位；跑 check_gongwen.py 抽查版式仍是国标。")
    return 0 if n_ok == len(units) else 1


if __name__ == "__main__":
    sys.exit(main())
