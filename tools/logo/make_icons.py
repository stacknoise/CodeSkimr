"""Draws the CodeSkimr app logo (code brackets with three skim lines) and writes the icon files.

Run from the repository root: python3 tools/logo/make_icons.py
Output: crates/app/assets/codeskimr.ico (Windows exe icon) and tray-icon.rgba (raw RGBA, TRAY_SIZE square).
Colours equal the tokens `accent` of the petrol theme (PETROL) and of the dark theme (HIGHLIGHT) in
design/tokens.toml. They are set by hand here: if the tokens change, change them here and run the script again.
"""
from pathlib import Path
from PIL import Image, ImageDraw

SIZE = 1024
TRAY_SIZE = 64
PETROL = "#0C727B"
WHITE = "#FFFFFF"
HIGHLIGHT = "#3BB6C1"
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
OUT = Path("crates/app/assets")


def draw_logo() -> Image.Image:
    im = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((40, 40, SIZE - 40, SIZE - 40), radius=210, fill=PETROL)
    d.line([(330, 300), (190, 512), (330, 724)], fill=WHITE, width=64, joint="curve")
    d.line([(694, 300), (834, 512), (694, 724)], fill=WHITE, width=64, joint="curve")
    for y, x2, colour in [(400, 624, WHITE), (512, 560, HIGHLIGHT), (624, 600, WHITE)]:
        d.rounded_rectangle((400, y - 30, x2, y + 30), radius=30, fill=colour)
    return im


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    logo = draw_logo()
    logo.save(OUT / "codeskimr.ico", sizes=[(s, s) for s in ICO_SIZES])
    (OUT / "tray-icon.rgba").write_bytes(logo.resize((TRAY_SIZE, TRAY_SIZE), Image.LANCZOS).tobytes())


if __name__ == "__main__":
    main()
