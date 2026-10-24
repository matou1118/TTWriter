#!/usr/bin/env python3
"""单测大写金额算法与参数校验（校验失败须干净退出、无堆栈）。"""
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from jietiao import to_capital  # noqa: E402

CASES = [
    (50000, "人民币伍萬元整"),
    (100000, "人民币壹拾萬元整"),
    (12345.67, "人民币壹萬贰仟叁佰肆拾伍元陆角柒分"),
    (100001, "人民币壹拾萬零壹元整"),
    (105.5, "人民币壹佰零伍元伍角"),
    (0.05, "人民币零伍分"),
    (0, "人民币零元整"),
    (1.005, "人民币壹元零壹分"),
    (7000000, "人民币柒佰萬元整"),
    (100500000, "人民币壹亿零伍拾萬元整"),
    (200000000001, "人民币贰仟亿零壹元整"),
]

BASE = [sys.executable, str(Path(__file__).parent / "jietiao.py"), "-o", "x.docx",
        "--borrower", "李四", "--borrower-id", "110101199001011234",
        "--lender", "张三", "--lender-id", "110101198805052345", "--amount", "50000"]
BAD = [
    ["--rate", "-1"],                      # 负利率不得写成无息
    ["--amount", "nan"],                   # 非数值金额不得崩栈
    ["--years", "10000"],                  # 期限溢出不得崩栈
    ["--borrower-id", "110101199913011234"],  # 13 月身份证号
]

bad = 0
for a, want in CASES:
    got = to_capital(a)
    ok = got == want
    bad += 0 if ok else 1
    print(("OK " if ok else "FAIL") + f" {a:>13} -> {got}" + ("" if ok else f"   want {want}"))
for extra in BAD:
    r = subprocess.run(BASE + extra, capture_output=True, text=True, encoding="utf-8", errors="replace")
    ok = r.returncode != 0 and "Traceback" not in r.stderr
    bad += 0 if ok else 1
    print(("OK " if ok else "FAIL") + f" reject {extra} (exit {r.returncode})")
print(f"{bad} failures / {len(CASES) + len(BAD)} cases")
sys.exit(1 if bad else 0)
