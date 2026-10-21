#!/usr/bin/env python3
"""Draw the app icon and write every committed icon file.

One geometry, one script: the SVG (the source of truth `packaging/icons.sh` renders from) and
the committed PNG/ICO are all produced here, so they cannot drift apart. Run it after changing
`MARK`/`BG`:

    python tools/make_icons.py

Needs Pillow. `packaging/icons.sh` regenerates the same files on macOS via resvg + iconutil;
this script is the cross-platform path for the committed outputs.
"""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "app-icon"

# 蜻蜓文档 TTWriter: 朱砂 rounded square with a white 文 (亠 + 乂). Cinnabar is the site's
# default accent (#b33420), and 文 is the shortest word for "writing".
BG = "#b33420"
FG = "#ffffff"
RADIUS = 112
WIDTH = 34

# (x1, y1), (x2, y2) in a 512×512 box: the 点, the 横, and the two strokes of the 乂.
MARK = [
    ((256, 96), (256, 144)),
    ((136, 186), (376, 186)),
    ((232, 232), (368, 404)),
    ((280, 232), (144, 404)),
]

SIZES = [16, 24, 32, 48, 64, 128, 256, 512]


def svg() -> str:
    lines = [
        f'  <rect width="512" height="512" rx="{RADIUS}" fill="{BG}"/>',
        f'  <g stroke="{FG}" stroke-width="{WIDTH}" stroke-linecap="round" fill="none">',
    ]
    lines += [f'    <line x1="{a[0]}" y1="{a[1]}" x2="{b[0]}" y2="{b[1]}"/>' for a, b in MARK]
    lines.append("  </g>")
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">\n'
        "  <!-- 蜻蜓文档 TTWriter app icon (original; Apache-2.0). Drawn by tools/make_icons.py. -->\n"
        + "\n".join(lines)
        + "\n</svg>\n"
    )


def render(size: int) -> Image.Image:
    """The icon at `size` px, drawn at 4× and downsampled so the strokes antialias."""
    f = 4
    px = size * f
    k = px / 512.0
    im = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, px - 1, px - 1], radius=RADIUS * k, fill=BG)
    w = max(1, round(WIDTH * k))
    for (x1, y1), (x2, y2) in MARK:
        a = (x1 * k, y1 * k)
        b = (x2 * k, y2 * k)
        d.line([a, b], fill=FG, width=w)
        for x, y in (a, b):  # round caps
            r = w / 2
            d.ellipse([x - r, y - r, x + r, y + r], fill=FG)
    return im.resize((size, size), Image.LANCZOS)


# ICNS types whose payload is a PNG: type code -> pixel size.
ICNS_PNG = {"ic11": 32, "ic12": 64, "ic07": 128, "ic13": 256, "ic08": 256, "ic14": 512, "ic09": 512, "ic10": 1024}


def write_icns(path: Path) -> None:
    """A modern ICNS: PNG payloads only, which every macOS that runs this app understands."""
    import io
    import struct

    chunks = b""
    for code, size in ICNS_PNG.items():
        buf = io.BytesIO()
        render(size).save(buf, format="PNG")
        png = buf.getvalue()
        chunks += code.encode("ascii") + struct.pack(">I", len(png) + 8) + png
    path.write_bytes(b"icns" + struct.pack(">I", len(chunks) + 8) + chunks)



def main() -> int:
    art = svg()
    for p in [OUT / "wordcraft.svg", OUT / "hicolor" / "scalable" / "apps" / "ai.storyteller.wordcraft.svg"]:
        p.write_text(art, encoding="utf-8", newline="\n")

    big = render(1024)
    big.save(OUT / "wordcraft-1024.png")
    render(512).save(OUT / "wordcraft-macos-512.png")
    for n in SIZES:
        d = OUT / "hicolor" / f"{n}x{n}" / "apps"
        d.mkdir(parents=True, exist_ok=True)
        render(n).save(d / "ai.storyteller.wordcraft.png")

    render(256).save(OUT / "wordcraft.ico", sizes=[(n, n) for n in SIZES if n <= 256])
    write_icns(OUT / "wordcraft.icns")
    print(f"wrote {len(SIZES) + 4} files under {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
