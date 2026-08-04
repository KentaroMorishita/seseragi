from pathlib import Path
import copy
import hashlib
import xml.etree.ElementTree as ET

import cairosvg
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets" / "brand"
SOURCE = BRAND / "source"
PUBLIC = BRAND / "public" / "brand"
SOCIAL = BRAND / "social"


def strip_namespace(element: ET.Element) -> ET.Element:
    clone = copy.deepcopy(element)
    for node in clone.iter():
        if "}" in node.tag:
            node.tag = node.tag.split("}", 1)[1]
    return clone


def build_social_svg() -> str:
    logo_root = ET.parse(SOURCE / "seseragi-logo-dark.svg").getroot()
    children = "".join(
        ET.tostring(strip_namespace(child), encoding="unicode")
        for child in list(logo_root)
    )
    return f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630" role="img" aria-labelledby="title desc">
  <title id="title">Seseragi programming language</title>
  <desc id="desc">Seseragi logo and product surfaces.</desc>
  <rect width="1200" height="630" fill="#08100e"/>
  <rect width="1200" height="10" fill="#009aad"/>
  <svg x="100" y="112" width="1000" height="300" viewBox="300 420 1400 420" preserveAspectRatio="xMidYMid meet">
{children}
  </svg>
  <text x="600" y="495" text-anchor="middle" fill="#b9ccc4" font-family="DejaVu Sans, sans-serif" font-size="28" font-weight="600" letter-spacing="1.2">TYPES · EFFECTS · SIGNALS · DOM</text>
  <text x="600" y="548" text-anchor="middle" fill="#69a28f" font-family="DejaVu Sans, sans-serif" font-size="20" font-weight="600" letter-spacing="3">PLAYGROUND · TOUR · VS CODE</text>
</svg>
'''


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    PUBLIC.mkdir(parents=True, exist_ok=True)
    SOCIAL.mkdir(parents=True, exist_ok=True)

    social_svg = SOCIAL / "seseragi-social-preview.svg"
    social_svg.write_text(build_social_svg(), encoding="utf-8")

    cairosvg.svg2png(
        url=str(social_svg),
        write_to=str(PUBLIC / "seseragi-social-preview.png"),
        output_width=1200,
        output_height=630,
    )
    cairosvg.svg2png(
        url=str(SOURCE / "seseragi-icon.svg"),
        write_to=str(PUBLIC / "apple-touch-icon.png"),
        output_width=180,
        output_height=180,
    )

    social = Image.open(PUBLIC / "seseragi-social-preview.png")
    apple = Image.open(PUBLIC / "apple-touch-icon.png")
    assert social.size == (1200, 630) and social.mode == "RGB"
    assert apple.size == (180, 180) and apple.mode == "RGBA"
    assert "<image" not in social_svg.read_text(encoding="utf-8")

    print("social", sha256(PUBLIC / "seseragi-social-preview.png"))
    print("apple", sha256(PUBLIC / "apple-touch-icon.png"))


if __name__ == "__main__":
    main()
