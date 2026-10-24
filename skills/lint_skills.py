#!/usr/bin/env python3
"""Lint the skills/ directory: structure, references, script safety, metadata.

Rules are grouped and severity-graded (any FAIL breaks the build; WARN doesn't):

  结构 STRUCTURE
    S1  每个技能目录有 SKILL.md                          FAIL
    S2  目录里只有约定形态的文件：SKILL.md、references/*.md、
        scripts/*.py——别的（尤其 __pycache__、*.pyc、临时文件）是脏     FAIL
    S3  SKILL.md 非空且不太长（> 64KB 的 SKILL.md 塞满了上下文窗口）    WARN

  元数据 METADATA
    M1  frontmatter 有 name/description/license/compatibility        FAIL
    M2  frontmatter 的 name == 目录名（安装后按名字找）                FAIL
    M3  license 是认识的开源许可（MIT/Apache-2.0/CC0-1.0/CC-BY-4.0/
        CC-BY-SA-4.0/BSD-3-Clause/ISC/OFL-1.1）                      FAIL
    M4  description 非空且含 "Use when"（agent 的触发句式）            WARN
    M5  description ≤ 1024 字符（太长装不进索引）                      WARN

  引用 REFERENCES
    R1  SKILL.md 里提到的 references/ 与 scripts/ 相对路径真实存在    FAIL
    R2  scripts/*.py 引用的模板/数据文件（同目录或 ../references 的
        相对路径）真实存在                                              FAIL

  脚本安全 SCRIPT SAFETY
    P1  scripts/*.py 无危险调用：os.system、subprocess 的 shell=True、
        eval/exec、rm -rf 字样                                        FAIL
    P2  scripts/*.py 编译通过（py_compile；语法错=技能根本跑不了）      FAIL
    P3  scripts/*.py 无网络外呼（urllib/requests/http.client）——
        技能是离线的（compatibility 都写了 No network access）        WARN

用法：
    python skills/lint_skills.py            # 检查 skills/ 下全部技能
    python skills/lint_skills.py gongwen    # 只查一个

退出码：0 全过（WARN 不算）；1 有 FAIL；2 用法错。
"""

from __future__ import annotations

import py_compile
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"

# 约定的技能目录形态：这些之外的东西都是脏文件。
KNOWN_TOP = {"SKILL.md", "references", "scripts", "evidence.md", "assets"}
KNOWN_LICENSES = {
    "MIT",
    "Apache-2.0",
    "CC0-1.0",
    "CC-BY-4.0",
    "CC-BY-SA-4.0",
    "BSD-3-Clause",
    "ISC",
    "OFL-1.1",
}
# 脚本安全：出现即 FAIL 的调用形态。
DANGEROUS = [
    re.compile(r"\bos\.system\s*\("),
    re.compile(r"shell\s*=\s*True"),
    re.compile(r"\beval\s*\("),
    re.compile(r"\bexec\s*\("),
    re.compile(r"rm\s+-rf"),
]
# 网络外呼（WARN）：技能按约定离线。
NETWORK = [
    re.compile(r"\burllib\b"),
    re.compile(r"\brequests\b"),
    re.compile(r"\bhttp\.client\b"),
    re.compile(r"\bsocket\b"),
]
MAX_SKILL_MD = 64 * 1024
MAX_DESCRIPTION = 1024

FAILS: list[str] = []
WARNS: list[str] = []


def fail(skill: str, code: str, msg: str) -> None:
    FAILS.append(f"[{skill}] {code}: {msg}")


def warn(skill: str, code: str, msg: str) -> None:
    WARNS.append(f"[{skill}] {code}: {msg}")


def frontmatter(text: str) -> dict[str, str]:
    """SKILL.md 开头的 YAML frontmatter（只取顶层键；metadata: 的嵌套不管）。"""
    if not text.startswith("---"):
        return {}
    end = text.find("\n---", 3)
    if end < 0:
        return {}
    out: dict[str, str] = {}
    for line in text[3:end].splitlines():
        m = re.match(r"^([a-zA-Z_][\w-]*):\s*(.*)$", line)
        if m and not m.group(2).startswith('"'):
            out[m.group(1)] = m.group(2).strip()
        elif m and m.group(2).startswith('"'):
            out[m.group(1)] = m.group(2).strip().strip('"')
    return out


def check_skill(dir_path: Path) -> None:
    name = dir_path.name
    skill_md = dir_path / "SKILL.md"

    # S1
    if not skill_md.is_file():
        fail(name, "S1", "SKILL.md 缺失")
        return

    text = skill_md.read_text(encoding="utf-8", errors="replace")

    # S2 目录形态
    for entry in dir_path.iterdir():
        if entry.name not in KNOWN_TOP:
            fail(name, "S2", f"意外的顶层条目 {entry.name}（约定只有 SKILL.md / references / scripts / assets / evidence.md）")
    for sub in ("references", "scripts", "assets"):
        p = dir_path / sub
        if p.is_dir():
            for f in p.rglob("*"):
                if f.is_file():
                    bad = f.suffix not in {".md", ".py", ".json", ".txt", ".png", ".svg", ".csv", ".ttf"}
                    cache = "__pycache__" in f.parts or f.suffix == ".pyc"
                    if cache:
                        fail(name, "S2", f"编译缓存进了仓库：{f.relative_to(dir_path)}")
                    elif bad:
                        fail(name, "S2", f"意外的文件类型 {f.relative_to(dir_path)}")

    # S3
    if not text.strip():
        fail(name, "S3", "SKILL.md 是空的")
    elif len(text.encode("utf-8")) > MAX_SKILL_MD:
        warn(name, "S3", f"SKILL.md 有 {len(text.encode('utf-8')) // 1024}KB（>{MAX_SKILL_MD // 1024}KB 会塞满上下文）")

    # M1-M5
    fm = frontmatter(text)
    for key in ("name", "description", "license", "compatibility"):
        if key not in fm or not fm[key]:
            fail(name, "M1", f"frontmatter 缺 {key}")
    if fm.get("name") and fm["name"] != name:
        fail(name, "M2", f"frontmatter name={fm['name']!r} != 目录名 {name!r}")
    if fm.get("license") and fm["license"] not in KNOWN_LICENSES:
        fail(name, "M3", f"不认识的 license {fm['license']!r}（认识：{sorted(KNOWN_LICENSES)}）")
    desc = fm.get("description", "")
    if desc and "Use when" not in desc and "Use this skill when" not in desc:
        warn(name, "M4", "description 没有 'Use when' 触发句式")
    if len(desc) > MAX_DESCRIPTION:
        warn(name, "M5", f"description 有 {len(desc)} 字符（>{MAX_DESCRIPTION}）")

    # R1 SKILL.md 里的 markdown 链接目标（[文本](目标)）与反引号里的路径真实存在。
    # 只查目标不查显示文本：`[references/commands.md](../wordcraft/references/commands.md)`
    # 的显示文本是跨技能引用的描述，实际目标是 ../ 那个。
    refs: set[str] = set()
    for target in re.findall(r"\]\(([^)\s]+)\)", text):
        if target.startswith(("references/", "scripts/", "../", "./")):
            refs.add(target.split("#", 1)[0])  # 剥页内锚（#section 不是文件名）
    for ref in re.findall(r"`((?:\.\./[\w\-]+/)?(?:references|scripts)/[\w\-./\u4e00-\u9fff]+)`", text):
        refs.add(ref)
    for ref in sorted(refs):
        target = (dir_path / ref).resolve()
        if not target.exists():
            fail(name, "R1", f"SKILL.md 提到 {ref} 但文件不存在")
    # R2 脚本里引用的同目录/相对文件（粗扫字符串里的文件名形态）
    scripts_dir = dir_path / "scripts"
    if scripts_dir.is_dir():
        for py in scripts_dir.glob("*.py"):
            src = py.read_text(encoding="utf-8", errors="replace")
            for rel in re.findall(r"""["']((?:[\w\-.\u4e00-\u9fff]+/)*[\w\-.\u4e00-\u9fff]+\.(?:json|txt|csv|md|yaml))["']""", src):
                if "/" in rel and not (py.parent / rel).exists() and not (dir_path / rel).exists():
                    fail(name, "R2", f"{py.name} 引用 {rel} 但不存在")
            # P1 危险调用
            for pat in DANGEROUS:
                if pat.search(src):
                    fail(name, "P1", f"{py.name} 有危险调用 {pat.pattern!r}")
            # P2 编译（字节码落临时文件，查完即删——不留 __pycache__）
            import tempfile

            with tempfile.NamedTemporaryFile(suffix=".pyc", delete=False) as tf:
                pyc_path = tf.name
            try:
                py_compile.compile(str(py), doraise=True, cfile=pyc_path)
            except py_compile.PyCompileError as e:
                fail(name, "P2", f"{py.name} 编译失败：{e}")
            finally:
                Path(pyc_path).unlink(missing_ok=True)
            # P3 网络
            for pat in NETWORK:
                if pat.search(src):
                    warn(name, "P3", f"{py.name} 疑似网络调用 {pat.pattern!r}（技能按约定离线）")


def main() -> int:
    args = sys.argv[1:]
    targets = [SKILLS / a for a in args] if args else sorted(p for p in SKILLS.iterdir() if p.is_dir())
    if not targets:
        print("skills/ 下没有技能目录", file=sys.stderr)
        return 2
    for t in targets:
        if not t.is_dir():
            print(f"没有这个技能：{t}", file=sys.stderr)
            return 2
        check_skill(t)

    for w in WARNS:
        print(f"WARN  {w}")
    for f in FAILS:
        print(f"FAIL  {f}")
    print(f"\n{len(targets)} 个技能：{len(FAILS)} FAIL，{len(WARNS)} WARN")
    if FAILS:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
