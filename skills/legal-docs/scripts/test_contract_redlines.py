#!/usr/bin/env python3
"""contract_gen 红线自检：不合规的试用期/负数必须拒绝生成（不落盘、不依赖 wordcraft-cli）。

拒绝发生在渲染之前——exit 1 且不产出 -o 文件即为通过。跑法：
    python test_contract_redlines.py
"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = Path(tempfile.gettempdir()) / "contract_redline_never.docx"

CASES: list[tuple[str, list[str]]] = [
    ("不得超过 1 个月", ["--kind", "labor", "--term-months", "11", "--probation-months", "3"]),
    ("不得约定试用期", ["--kind", "labor", "--term-months", "2", "--probation-months", "1"]),
    ("要有合同期限", ["--kind", "labor", "--probation-months", "1"]),
    ("期限不能为负", ["--kind", "labor", "--term-months", "-1", "--probation-months", "1"]),
    ("不能为负", ["--kind", "lease", "--rent", "-5"]),
]


def main() -> int:
    bad = 0
    for want, args in CASES:
        OUT.unlink(missing_ok=True)
        # 子进程走管道时按 locale（中文 Windows = cp936）写字节——钉成 utf-8 再断言消息。
        r = subprocess.run(
            [sys.executable, str(HERE / "contract_gen.py"), *args, "-o", str(OUT)],
            capture_output=True,
            env={**os.environ, "PYTHONIOENCODING": "utf-8"},
        )
        out = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")
        ok = r.returncode == 1 and want in out and not OUT.exists()
        print(("ok   " if ok else f"FAIL ") + " ".join(args) + (f"  ({want})" if ok else f"  want {want!r} exit={r.returncode} out={out[:160]!r}"))
        bad += not ok
    print(f"{len(CASES) - bad}/{len(CASES)} red-line cases pass")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
