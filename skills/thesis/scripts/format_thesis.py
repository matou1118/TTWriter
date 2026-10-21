#!/usr/bin/env python3
"""Apply the GB/T 7713 academic-paper layout to a .docx, using WordCraft.

The content is left alone: this only sets the page box, the styles, the line
spacing and the page numbers. Run it on a document whose text is already final.

    python format_thesis.py content.docx -o thesis.docx
    python format_thesis.py content.docx -o thesis.docx --pdf
    python format_thesis.py content.docx -o thesis.docx --title "论文题目" --header

It drives wordcraft-cli; it never touches the document text.
"""
import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

MM = 72 / 25.4
MARGINS = {
    "top": round(25 * MM, 1),
    "bottom": round(25 * MM, 1),
    "left": round(30 * MM, 1),   # 装订边
    "right": round(20 * MM, 1),
}
LINE_SPACING = {"rule": "exactly", "value": 20}
BODY_SIZE = 12.0    # 小四
TITLE_SIZE = 16.0   # 三号
CAPTION_SIZE = 10.5  # 五号
FONTS = {
    "body": "宋体",
    "title": "黑体",
    "caption": "宋体",
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


def build_commands(args) -> list[dict]:
    cmds: list[dict] = [
        {"command": "layout.margins", "params": MARGINS},
        # 正文：宋体小四，固定行距 20 磅，首行缩进 2 字符。
        {"command": "styles.modify", "params": {
            "style": "Normal",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["body"], "size": BODY_SIZE, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 2},
        }},
        # 题目：黑体三号，居中，段前段后各半行。
        {"command": "styles.modify", "params": {
            "style": "Title",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["title"], "size": TITLE_SIZE, "bold": False, "color": "000000"},
            "para": {"align": "center", "lineSpacing": LINE_SPACING, "indentFirstChars": 0,
                     "spaceBefore": 18, "spaceAfter": 18},
        }},
        # 章标题：黑体三号居中；节标题：黑体四号左空二字。
        {"command": "styles.modify", "params": {
            "style": "Heading1",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["title"], "size": TITLE_SIZE, "bold": False, "color": "000000"},
            "para": {"align": "center", "lineSpacing": LINE_SPACING, "indentFirstChars": 0,
                     "spaceBefore": 18, "spaceAfter": 18},
        }},
        {"command": "styles.modify", "params": {
            "style": "Heading2",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["title"], "size": 14.0, "bold": False, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 2},
        }},
        {"command": "styles.modify", "params": {
            "style": "Heading3",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["title"], "size": BODY_SIZE, "bold": False, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 2},
        }},
        # 题注：宋体五号居中。LineSpacing 只有 multiple/atLeast/exactly 三种规则，
        # 没有 "auto" —— 单倍行距用 multiple 1.0。
        {"command": "styles.modify", "params": {
            "style": "Caption",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["caption"], "size": CAPTION_SIZE, "bold": False, "color": "000000"},
            "para": {"align": "center", "lineSpacing": {"rule": "multiple", "value": 1.0}, "indentFirstChars": 0},
        }},
    ]
    # Body commands first. `insert.header` / `insert.footer` move the caret into the header or
    # footer story, and a command that only works in the main text (references.toc,
    # references.bibliography) fails there with "goes in the main text" — so the header goes last
    # and the footer just before it.
    if args.toc:
        cmds.append({"command": "references.toc", "params": {
            "levels": 3, "title": "目 录", "pageNumbers": True,
            "rightAlign": True, "tabLeader": "dot", "hyperlinks": True}})
    if args.citation_style:
        cmds.append({"command": "references.citationStyle", "params": {"style": args.citation_style}})
        # The bibliography title is hardcoded English in the style table ("References"),
        # and `references.bibliography` takes a title, so pass the Chinese one.
        if args.bibliography_title:
            cmds.append({"command": "references.bibliography", "params": {"title": args.bibliography_title}})
    if args.page_numbers:
        cmds.append({"command": "insert.footer", "params": {"preset": "blankThree"}})
        cmds.append({"command": "insert.pageNumber", "params": {"position": "bottom", "align": "center"}})
    if args.title:
        cmds.append({"command": "insert.header", "params": {"text": args.title}})
    if args.header:
        cmds.append({"command": "insert.header", "params": {"text": args.header}})
    return cmds


def main() -> int:
    ap = argparse.ArgumentParser(description="Apply the GB/T 7713 paper layout to a .docx.")
    ap.add_argument("source", type=Path, help="the content .docx (text already final)")
    ap.add_argument("-o", "--out", type=Path, required=True, help="the output .docx")
    ap.add_argument("--pdf", action="store_true", help="also export a PDF")
    ap.add_argument("--title", help="the paper title (used for the header)")
    ap.add_argument("--header", help="the header text (overrides --title)")
    ap.add_argument("--page-numbers", action="store_true", default=True, help="add centred page numbers (default)")
    ap.add_argument("--no-page-numbers", dest="page_numbers", action="store_false")
    ap.add_argument("--toc", action="store_true", help="insert a table of contents")
    ap.add_argument("--citation-style", default=None, help="e.g. GB/T 7714")
    ap.add_argument("--bibliography-title", default="参考文献",
                    help="the heading above the reference list (default 参考文献)")
    args = ap.parse_args()

    if not args.source.exists():
        sys.exit(f"no such file: {args.source}")
    exe = cli()

    saves = [str(args.out.resolve())]
    if args.pdf:
        saves.append(str(args.out.with_suffix(".pdf").resolve()))

    # The script's `file` and `save` paths resolve against the script file's own folder,
    # and this script lives in a temp directory — so both must be absolute.
    script = {
        "file": str(args.source.resolve()),
        "commands": build_commands(args),
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
