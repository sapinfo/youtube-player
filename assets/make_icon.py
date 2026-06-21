#!/usr/bin/env python3
"""Generate a macOS app icon (1024x1024 PNG) for YouTube Player.

Design: Big Sur-style rounded square in YouTube red with a white play triangle.
Drawn at 4x supersampling then downscaled for smooth anti-aliased edges.
"""
from PIL import Image, ImageDraw
import os

SS = 4               # supersampling factor
SIZE = 1024 * SS
MARGIN = 90 * SS     # transparent padding around the rounded square
RADIUS = 230 * SS    # corner radius of the rounded square

RED_TOP = (255, 64, 56)     # lighter red (top)
RED_BOTTOM = (213, 0, 0)    # deeper red (bottom)


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def main():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

    # --- vertical gradient body, masked to a rounded square ---
    body = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    grad = Image.new("RGB", (1, SIZE))
    for y in range(SIZE):
        grad.putpixel((0, y), lerp(RED_TOP, RED_BOTTOM, y / (SIZE - 1)))
    grad = grad.resize((SIZE, SIZE))

    mask = Image.new("L", (SIZE, SIZE), 0)
    mdraw = ImageDraw.Draw(mask)
    mdraw.rounded_rectangle(
        [MARGIN, MARGIN, SIZE - MARGIN, SIZE - MARGIN],
        radius=RADIUS,
        fill=255,
    )
    body.paste(grad, (0, 0), mask)
    img = Image.alpha_composite(img, body)

    # --- white play triangle, optically centered ---
    draw = ImageDraw.Draw(img)
    cx, cy = SIZE / 2, SIZE / 2
    tw = 250 * SS   # half-width-ish of triangle
    th = 270 * SS   # half-height of triangle
    shift = 30 * SS  # nudge right so it looks centered
    p1 = (cx - tw * 0.55 + shift, cy - th)
    p2 = (cx - tw * 0.55 + shift, cy + th)
    p3 = (cx + tw + shift, cy)
    draw.polygon([p1, p2, p3], fill=(255, 255, 255, 255))

    # --- downscale to 1024 with high-quality resampling ---
    out = img.resize((1024, 1024), Image.LANCZOS)
    here = os.path.dirname(os.path.abspath(__file__))
    path = os.path.join(here, "icon_1024.png")
    out.save(path)
    print("wrote", path)


if __name__ == "__main__":
    main()
