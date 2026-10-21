#!/usr/bin/env python3
"""Lay out a Chinese legal document (起诉状 / 律师函 / 法律意见书 / 合同审查报告).

The content is taken verbatim from a JSON file produced by the content layer
(for example WorkMind's complaint/opinion generators). This script only sets
the page box, the styles, the line spacing, the page numbers and the
signature block — it never writes or edits the text.

    python format_legal.py complaint.json -o 民事起诉状-张三诉李四.docx
    python format_legal.py complaint.json -o out.docx --kind complaint --pdf
    python format_legal.py opinion.json  -o out.docx --kind opinion --masked

The JSON input:
{
  "kind": "complaint",              # complaint | letter | opinion | review
  "title": "民事起诉状",
  "blocks": [
    {"type": "paragraph", "text": "原告：……"},
    {"type": "numbered",  "items": ["请求一……", "请求二……"]},
    {"type": "table",     "header": ["证据名称", "证明对象"],
     "rows": [["借条", "存在借贷关系"]]},
    {"type": "sign",      "lines": ["具状人：张三", "2026年10月21日"]},
    {"type": "quote",     "text": "此致"}
  ],
  "masked": false
}

It drives wordcraft-cli; it never touches the text.
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
    "top": round(37 * MM, 1),
    "bottom": round(35 * MM, 1),
    "left": round(28 * MM, 1),
    "right": round(26 * MM, 1),
}
LINE_SPACING = {"rule": "multiple", "value": 1.5}
BODY_SIZE = 12.0     # 小四
TITLE_SIZE = 22.0    # 二号
H1_SIZE = 16.0       # 三号
FONTS = {"body": "宋体", "head": "黑体", "latin": "Times New Roman"}
MASK_BANNER = "【对外脱敏版本，内部底稿保留完整主体信息】"


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


def build_commands(data: dict) -> list[dict]:
    cmds: list[dict] = [
        {"command": "layout.margins", "params": MARGINS},
        # 正文：宋体小四，1.5 倍行距，首行缩进 2 字符。
        {"command": "styles.modify", "params": {
            "style": "Normal",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["body"], "size": BODY_SIZE, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 2},
        }},
        # 标题：黑体二号居中，顶格。
        {"command": "styles.modify", "params": {
            "style": "Title",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["head"], "size": TITLE_SIZE, "bold": False, "color": "000000"},
            "para": {"align": "center", "lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }},
        # 一级标题「一、」：黑体三号顶格。
        {"command": "styles.modify", "params": {
            "style": "Heading1",
            "chr": {"font": FONTS["latin"], "fontEa": FONTS["head"], "size": H1_SIZE, "bold": False, "color": "000000"},
            "para": {"lineSpacing": LINE_SPACING, "indentFirstChars": 0},
        }},
    ]

    # The document title comes first, in the Title style.
    if data.get("title"):
        cmds.append({"command": "text.insert", "params": {"text": data["title"]}})
        cmds.append({"command": "styles.apply", "params": {"style": "Title"}})
        cmds.append({"command": "text.newParagraph"})

    for block in data.get("blocks", []):
        kind = block.get("type")
        if kind == "paragraph":
            cmds.append({"command": "text.insert", "params": {"text": block.get("text", "")}})
            cmds.append({"command": "text.newParagraph"})
        elif kind == "heading":
            cmds.append({"command": "text.insert", "params": {"text": block.get("text", "")}})
            cmds.append({"command": "styles.apply", "params": {"style": "Heading1"}})
            cmds.append({"command": "text.newParagraph"})
        elif kind == "numbered":
            for item in block.get("items", []):
                cmds.append({"command": "text.insert", "params": {"text": item}})
                cmds.append({"command": "text.newParagraph"})
        elif kind == "quote":
            # 「此致」顶格，下一行左空二字。
            cmds.append({"command": "text.insert", "params": {"text": block.get("text", "")}})
            cmds.append({"command": "para.set", "params": {"props": {"indentFirstChars": 0}}})
            cmds.append({"command": "text.newParagraph"})
            if block.get("next"):
                cmds.append({"command": "text.insert", "params": {"text": block["next"]}})
                cmds.append({"command": "para.set", "params": {"props": {"indentFirstChars": 2}}})
                cmds.append({"command": "text.newParagraph"})
        elif kind == "table":
            header = block.get("header") or []
            rows = block.get("rows") or []
            if header:
                cmds.append({"command": "insert.table", "params": {"rows": len(rows) + 1, "cols": len(header)}})
                # The cursor is in the first cell here, so table commands still apply.
                cmds.append({"command": "table.repeatHeader", "params": {"enabled": True}})
                cells = list(header) + [c for row in rows for c in row]
                for i, cell in enumerate(cells):
                    cmds.append({"command": "text.insert", "params": {"text": cell}})
                    # caret.right steps to the next cell; skip it after the last one,
                    # which would move the cursor out of the table.
                    if i < len(cells) - 1:
                        cmds.append({"command": "caret.right"})
                cmds.append({"command": "text.newParagraph"})
        elif kind == "sign":
            for line in block.get("lines", []):
                cmds.append({"command": "text.insert", "params": {"text": line}})
                cmds.append({"command": "para.set", "params": {"props": {"align": "right", "indentFirstChars": 0}}})
                cmds.append({"command": "text.newParagraph"})

    if data.get("masked"):
        cmds.append({"command": "insert.header", "params": {"text": MASK_BANNER}})

    cmds.append({"command": "insert.footer", "params": {"preset": "blankThree"}})
    cmds.append({"command": "insert.pageNumber", "params": {"position": "bottom", "align": "center"}})
    return cmds


def main() -> int:
    ap = argparse.ArgumentParser(description="Lay out a Chinese legal document.")
    ap.add_argument("source", type=Path, help="the content JSON")
    ap.add_argument("-o", "--out", type=Path, required=True, help="the output .docx")
    ap.add_argument("--kind", default=None, help="complaint | letter | opinion | review")
    ap.add_argument("--pdf", action="store_true", help="also export a PDF")
    ap.add_argument("--masked", action="store_true", help="the 对外脱敏版: add the banner")
    args = ap.parse_args()

    if not args.source.exists():
        sys.exit(f"no such file: {args.source}")
    data = json.loads(args.source.read_text(encoding="utf-8"))
    if args.kind:
        data["kind"] = args.kind
    if args.masked:
        data["masked"] = True

    exe = cli()
    saves = [str(args.out.resolve())]
    if args.pdf:
        saves.append(str(args.out.with_suffix(".pdf").resolve()))

    # The script's `file` and `save` paths resolve against the script file's own folder,
    # and this script lives in a temp directory — so `save` must be absolute.
    # `file` is unused here (the content is in the JSON), so the document starts blank.
    script = {
        "template": "blank",
        "commands": build_commands(data),
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
