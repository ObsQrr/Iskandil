"""Iskandil uygulama ikonunu uretir (ICO + PNG).

Tasarim: murekkep zemin, esmerkezli derinlik halkalari, vermilyon iskandil ipi.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

INK = (0x16, 0x19, 0x15, 255)
PAPER = (0xF1, 0xF2, 0xED, 255)
ACCENT = (0xB2, 0x3A, 0x2B, 255)
SIZE = 1024


def render() -> Image.Image:
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    radius = int(SIZE * 0.19)
    draw.rounded_rectangle((0, 0, SIZE - 1, SIZE - 1), radius=radius, fill=INK)

    cx, cy = SIZE * 0.17, SIZE * 0.86
    for r in (0.32, 0.53, 0.74):
        radius_px = SIZE * r
        width = max(2, int(SIZE * 0.017))
        draw.ellipse(
            (cx - radius_px, cy - radius_px, cx + radius_px, cy + radius_px),
            outline=PAPER,
            width=width,
        )

    # Iskandil ipi
    x = SIZE * 0.72
    w = int(SIZE * 0.027)
    draw.rectangle((x - w / 2, SIZE * 0.12, x + w / 2, SIZE * 0.90), fill=ACCENT)
    # Agirlik
    m = SIZE * 0.062
    draw.rectangle((x - m, SIZE * 0.82 - m, x + m, SIZE * 0.82 + m), fill=ACCENT)
    # Ust halka (tutamak)
    hr = SIZE * 0.038
    draw.ellipse(
        (x - hr, SIZE * 0.11 - hr, x + hr, SIZE * 0.11 + hr),
        outline=ACCENT,
        width=max(2, int(SIZE * 0.018)),
    )
    return img.resize((256, 256), Image.LANCZOS)


def main() -> None:
    root = Path(__file__).parent.parent
    assets = root / "assets"
    assets.mkdir(exist_ok=True)
    icon = render()
    icon.save(assets / "iskandil-256.png")
    icon.save(
        assets / "iskandil.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )
    print(f"yazildi: {assets / 'iskandil.ico'}")


if __name__ == "__main__":
    main()
