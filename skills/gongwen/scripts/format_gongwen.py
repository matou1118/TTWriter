#!/usr/bin/env python3
"""Apply the GB/T 9704-2012 official-document layout to a .docx, using WordCraft.

The content is left alone: this only sets the page box, the styles, the line
spacing and the page numbers. Run it on a document whose text is already final.

    python format_gongwen.py content.docx -o gongwen.docx
    python format_gongwen.py content.docx -o gongwen.docx --pdf
    python format_gongwen.py content.docx -o gongwen.docx --wenzhong gongwen

It drives wordcraft-cli; it never touches the document text. Requires the
wordcraft-cli binary on PATH.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# GB/T 9704-2012: the text block is 156 x 225 mm on A4.
# Margins in points (1 mm = 72 / 25.4 pt).
MM = 72 / 25.4
MARGINS = {
    "top": round(37 * MM, 1),      # 上白边 37mm
    "bottom": round(35 * MM, 1),   # 下白边 35mm
    "left": round(28 * MM, 1),     # 左白边（订口）28mm
    "right": round(26 * MM, 1),    # 右白边 26mm
}
LINE_SPACING = {"rule": "exactly", "value": 28}   # 固定值 28 磅 → 每面 22 行
BODY_SIZE = 16.0    # 3 号
TITLE_SIZE = 22.0   # 2 号
PAGE_SIZE = 14.0    # 4 号

# 正文 3 号仿宋；标题 2 号小标宋；一级 3 号黑体；二级 3 号楷体。
#
# 每个角色是一串候选「(family, files)」：取第一个字体文件真在机器上的。
# 名字必须取**安装后的 family 名**——现代 Windows 上 `仿宋_GB2312` 这个名字
# 并不存在（simfang.ttf 的 family 是 FangSong / 仿宋），写了会静默回退成黑体，
# 公文的仿宋/楷体区分就没了。英文名在前因为它跨语言稳定，中文名作后备。
FONTS = {
    "body": [("FangSong", ["simfang.ttf"]), ("仿宋", ["simfang.ttf"]), ("仿宋_GB2312", ["simfang.ttf"])],
    "h1": [("SimHei", ["simhei.ttf"]), ("黑体", ["simhei.ttf"])],
    "h2": [("KaiTi", ["simkai.ttf"]), ("楷体", ["simkai.ttf"]), ("楷体_GB2312", ["simkai.ttf"])],
    # 方正小标宋简体不是 Windows 自带字体；华文中宋/宋体是可行的替身。
    "title": [
        ("方正小标宋简体", ["fxbsjw.ttf", "fzxbsjw.ttf", "方正小标宋简体.ttf"]),
        ("FZXiaoBiaoSong-B05S", ["fzxbsjw.ttf"]),
        ("华文中宋", ["STZHONGS.TTF"]),
        ("STZhongsong", ["STZHONGS.TTF"]),
    ],
    # 兜底是宋体不是黑体：GB/T 9704 的标题是小标宋（宋体一类），黑体是一级标题的字体。
    "title_fallback": [("SimSun", ["simsun.ttc"]), ("宋体", ["simsun.ttc"])],
    "latin": "Times New Roman",
}


def cli() -> str:
    exe = shutil.which("wordcraft-cli")
    if not exe:
        sys.exit("wordcraft-cli not found on PATH")
    return exe


def run(exe: str, args: list[str]) -> str:
    proc = subprocess.run([exe, *args], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        sys.exit(f"wordcraft-cli {' '.join(args)} failed:\n{proc.stdout}\n{proc.stderr}")
    return proc.stdout


def installed(family: str, files: list[str]) -> bool:
    """Whether a Chinese font face is present on this machine.

    The layout can only ask for a face that is installed; a missing one falls back
    silently, so it is checked here rather than discovered in the render.
    """
    fonts = Path(os.environ.get("WINDIR", "C:/Windows")) / "Fonts"
    return any((fonts / f).exists() for f in files)


def pick_face(role: str) -> tuple[str, str | None]:
    """The installed family to ask for in `role`, and a note if it had to fall back."""
    for family, files in FONTS[role]:
        if installed(family, files):
            return family, None
    family = FONTS[role][0][0] if role != "title" else FONTS["title_fallback"][0][0]
    return family, f"未找到 {role} 字体，原样写入 {family}，渲染可能回退。"


def pick_title_face(preferred: str | None) -> tuple[str, str | None]:
    """The title face to ask for, and a note if it had to fall back."""
    if preferred:
        return preferred, None
    for family, files in FONTS["title"]:
        if installed(family, files):
            return family, None
    family = FONTS["title_fallback"][0][0]
    return family, (
        f"未找到标题字体，回退为 {family}。"
        f"（GB/T 9704 要求 2 号小标宋；本机没有方正小标宋简体时用 --title-font 指定）"
    )


def build_commands(wenzhong: str | None, title_face: str) -> list[dict]:
    body = pick_face("body")[0]
    h1 = pick_face("h1")[0]
    h2 = pick_face("h2")[0]
    cmds: list[dict] = [
        # 版心：靠边距实现，纸仍是 A4。
        {"command": "layout.margins", "params": MARGINS},
        # 正文：3 号仿宋，固定行距 28 磅，首行缩进 2 字符。公文是黑白的。
        {"command": "styles.modify", "params": {
            "style": "Normal",
            "chr": {"font": FONTS["latin"], "fontEa": body, "size": BODY_SIZE, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 2},
        }},
        # 标题：2 号小标宋，居中，顶格，回行不断尾由内容控制。
        {"command": "styles.modify", "params": {
            "style": "Title",
            "chr": {"font": FONTS["latin"], "fontEa": title_face, "size": TITLE_SIZE, "bold": False, "color": "000000"},
            "para": {"align": "center", "lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }},
        # 一级标题 一、：3 号黑体，顶格。
        {"command": "styles.modify", "params": {
            "style": "Heading1",
            "chr": {"font": FONTS["latin"], "fontEa": h1, "size": BODY_SIZE, "bold": False, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }},
        # 二级标题 （一）：3 号楷体，顶格。
        {"command": "styles.modify", "params": {
            "style": "Heading2",
            "chr": {"font": FONTS["latin"], "fontEa": h2, "size": BODY_SIZE, "bold": False, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }},
    ]
    if wenzhong:
        cmds.append({"command": "styles.modify", "params": {
            "style": "Subtitle",
            "chr": {"font": FONTS["latin"], "fontEa": body, "size": BODY_SIZE, "color": "000000"},
            "para": {"align": "center", "lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }})
    return cmds


def main() -> int:
    ap = argparse.ArgumentParser(description="Apply the GB/T 9704-2012 layout to a .docx.")
    ap.add_argument("source", type=Path, help="the content .docx (text already final)")
    ap.add_argument("-o", "--out", type=Path, required=True, help="the output .docx")
    ap.add_argument("--pdf", action="store_true", help="also export a PDF")
    ap.add_argument("--wenzhong", metavar="NAME", help="the document type, e.g. 通知 (recorded in the title style)")
    ap.add_argument("--title-font", default=None, help="override the title face (e.g. 方正小标宋简体)")
    args = ap.parse_args()

    if not args.source.exists():
        sys.exit(f"no such file: {args.source}")
    exe = cli()

    title_face, note = pick_title_face(args.title_font)
    if note:
        print(f"[注意] {note}", file=sys.stderr)
    print(f"[版式] 标题字体 = {title_face}", file=sys.stderr)

    saves = [str(args.out.resolve())]
    if args.pdf:
        saves.append(str(args.out.with_suffix(".pdf").resolve()))

    # The script's `file` and `save` paths resolve against the script file's own folder,
    # and this script lives in a temp directory — so both must be absolute.
    script = {
        "file": str(args.source.resolve()),
        "commands": build_commands(args.wenzhong, title_face),
        "save": saves,
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(script, f, ensure_ascii=False, indent=2)
        script_path = f.name

    try:
        print(run(exe, ["run", "--script", script_path]))
    finally:
        Path(script_path).unlink(missing_ok=True)

    print(f"wrote {args.out}")
    print("Now check the layout: wordcraft-cli render %s page1.png --page 1" % args.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
