#!/usr/bin/env python3
"""体检一份学位论文 .docx 的 GB/T 7713/7714 合规性（自动核对）。

检查项（可程序化断言的）：
  图题注     「图 1-1」式编号连续、图题在图下（简化：编号连续性）
  表题注     「表 1-1」式编号连续
  页边距     上下 2.5cm、左（装订）3cm、右 2cm（常见校规，±3pt 容差）
  参考文献   每条 [n] 编号连续；正文引用 [n] 与文献表一一对应（缺失引用报 warn）

用法：
  python check_thesis.py 论文.docx [--json]

退出码：0 = 无 fail；1 = 有 fail；2 = 文件打不开。只读不改。
"""
import argparse
import json
import re
import subprocess
import sys

CM_TO_PT = 72.0 / 2.54
MARGINS_PT = {"top": 70.9, "bottom": 70.9, "left": 85.0, "right": 56.7}
TOL = 3.0

FIG = re.compile(r"^图\s*(\d+)\s*[-–—]\s*(\d+)")
TAB = re.compile(r"^表\s*(\d+)\s*[-–—]\s*(\d+)")
CITE = re.compile(r"\[(\d{1,3})\]")
BIB = re.compile(r"^\[(\d{1,3})\]")


import os
import shutil

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


def text_of(path: str) -> str:
    return run_cli(["text", path])


def inspect(path: str) -> dict:
    return json.loads(run_cli(["inspect", path]))


def number_series(nums: list) -> tuple:
    """(ok, detail)：每章内编号从 1 连续递增。"""
    if not nums:
        return True, "没有编号（也不一定是错——全文无图表）"
    by_ch: dict[int, list[int]] = {}
    for ch, no in nums:
        by_ch.setdefault(ch, []).append(no)
    bad = []
    for ch, nos in sorted(by_ch.items()):
        if nos != list(range(1, len(nos) + 1)):
            bad.append(f"第{ch}章 {nos}")
    return not bad, "各章连续" if not bad else "；".join(bad)


def check(path: str) -> list:
    results = []

    def add(level, name, ok, detail):
        results.append({"level": "ok" if ok else level, "name": name, "detail": detail})

    doc = inspect(path)
    txt = text_of(path)
    lines = [ln.strip() for ln in txt.splitlines()]

    # 页边距
    sections = doc.get("sections") or []
    if sections:
        s = sections[0].get("props") or {}
        for side, want in MARGINS_PT.items():
            got = s.get(f"margin{side.capitalize()}")
            if got is None:
                add("fail", f"页边距·{side}", False, "inspect 未报告该边距")
                continue
            got = round(float(got), 1)
            ok = abs(got - want) <= TOL
            add("fail" if not ok else "ok", f"页边距·{side}", ok,
                f"{got}pt（常见校规 {want}pt ±{TOL}；以本校模板为准）")
    else:
        add("warn", "页边距", False, "没有 section 信息")

    # 图表编号连续性
    figs, tabs, bibs = [], [], []
    for ln in lines:
        if m := FIG.match(ln):
            figs.append((int(m.group(1)), int(m.group(2))))
        if m := TAB.match(ln):
            tabs.append((int(m.group(1)), int(m.group(2))))
        if m := BIB.match(ln):
            bibs.append(int(m.group(1)))

    ok, detail = number_series(figs)
    add("fail", "图编号连续（图 n-m）", ok, detail)
    ok, detail = number_series(tabs)
    add("fail", "表编号连续（表 n-m）", ok, detail)

    # 参考文献：编号连续
    ok = bibs == list(range(1, len(bibs) + 1)) if bibs else False
    add("fail", "文献编号连续 [1..n]", ok, f"共 {len(bibs)} 条：{bibs[:8]}{'…' if len(bibs) > 8 else ''}" if bibs else "未找到 [n] 起行的文献条目")

    # 引用与文献表对应
    if bibs:
        cited = set()
        for m in CITE.finditer(txt):
            cited.add(int(m.group(1)))
        have = set(bibs)
        uncited = sorted(have - cited)
        missing = sorted(cited - have)
        ok = not missing
        detail = f"文献 {len(have)} 条，被引 {len(cited & have)}"
        if uncited:
            detail += f"；未被引用 {uncited[:8]}"
        if missing:
            detail += f"；引了但文献表没有 {missing[:8]}"
        add("fail", "引用与文献一一对应", ok, detail)
        if uncited:
            add("warn", "未被引用的文献", True, "GB/T 7714 要求列出被引文献；未被引的条目考虑删除")
    return results


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("file")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    try:
        results = check(args.file)
    except Exception as e:  # noqa: BLE001 - 报告给用户，不炸
        print(json.dumps({"error": str(e)}) if args.json else f"打不开：{e}", file=sys.stderr)
        return 2

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
