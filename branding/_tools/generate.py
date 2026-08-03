#!/usr/bin/env python3
"""Generate TerraTactics / TerraHub / TerraLink branding SVGs and PNG exports."""

from __future__ import annotations

import json
import math
import subprocess
import sys
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parents[1]
FONTS = ROOT / "_fonts"
OUT = {
    "terratactics": ROOT / "terratactics",
    "terrahub": ROOT / "terrahub",
    "terralink": ROOT / "terralink",
}

# TerraTactics visual tokens (matched to live site / TerraHub admin UI)
LIME = "#DDF34A"
OLIVE_900 = "#1C2415"
OLIVE = "#344126"
INK = "#17251D"
IVORY = "#F6F3EA"
GOLD = "#D9A521"
WHITE = "#FFFFFF"

PRODUCTS = {
    "terratactics": {
        "name": "TerraTactics",
        "prefix": "Terra",
        "suffix": "Tactics",
        "tag": "SOIL, UNDERSTOOD.",
        "mark": "tactics",
    },
    "terrahub": {
        "name": "TerraHub",
        "prefix": "Terra",
        "suffix": "Hub",
        "tag": "FIELD GATEWAY",
        "mark": "hub",
    },
    "terralink": {
        "name": "TerraLink",
        "prefix": "Terra",
        "suffix": "Link",
        "tag": "FIELD MESH",
        "mark": "link",
    },
}


def load_font(weight: float) -> TTFont:
    base = TTFont(FONTS / "Jost-Variable.ttf")
    return instantiateVariableFont(base, {"wght": weight})


def glyph_path(font: TTFont, name: str) -> str:
    glyph_set = font.getGlyphSet()
    pen = SVGPathPen(glyph_set)
    glyph_set[name].draw(pen)
    return pen.getCommands()


def text_to_paths(
    text: str,
    font: TTFont,
    *,
    x: float,
    baseline: float,
    size: float,
    tracking_em: float = 0.0,
) -> tuple[str, float]:
    """Return SVG path data and advance width for `text` at font size `size`."""
    glyph_set = font.getGlyphSet()
    cmap = font.getBestCmap()
    units = font["head"].unitsPerEm
    scale = size / units
    parts: list[str] = []
    cursor = 0.0
    for ch in text:
        code = ord(ch)
        if code not in cmap:
            raise KeyError(f"Missing glyph for {ch!r}")
        mapped = cmap[code]
        name = mapped if isinstance(mapped, str) else font.getGlyphName(mapped)
        path = glyph_path(font, name)
        tx = x + cursor
        ty = baseline
        parts.append(
            f'<path transform="translate({tx:.3f} {ty:.3f}) scale({scale:.6f} {-scale:.6f})" d="{path}"/>'
        )
        advance = glyph_set[name].width * scale
        cursor += advance + tracking_em * size
    if text:
        cursor -= tracking_em * size
    return "\n".join(parts), cursor


def hub_icon(cx: float, cy: float, r: float, fill: str) -> str:
    """Central hub with four terminal nodes — field gateway."""
    arm = r * 0.50
    node_r = r * 0.175
    hub_r = r * 0.24
    stroke = r * 0.11
    pts = [
        (cx, cy - arm),
        (cx + arm, cy),
        (cx, cy + arm),
        (cx - arm, cy),
    ]
    lines = []
    nodes = []
    for x, y in pts:
        lines.append(
            f'<line x1="{cx:.2f}" y1="{cy:.2f}" x2="{x:.2f}" y2="{y:.2f}" '
            f'stroke="{fill}" stroke-width="{stroke:.2f}" stroke-linecap="round"/>'
        )
        nodes.append(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{node_r:.2f}" fill="{fill}"/>')
    return "\n".join(lines) + f'\n<circle cx="{cx:.2f}" cy="{cy:.2f}" r="{hub_r:.2f}" fill="{fill}"/>\n' + "\n".join(nodes)


def link_icon(cx: float, cy: float, r: float, fill: str) -> str:
    """Three-node mesh triangle with a radio tick — LoRa field mesh."""
    node_r = r * 0.155
    stroke = r * 0.10
    # Equilateral-ish triangle, slightly top-weighted
    top = (cx, cy - r * 0.36)
    bl = (cx - r * 0.38, cy + r * 0.28)
    br = (cx + r * 0.38, cy + r * 0.28)
    edges = [(top, bl), (bl, br), (br, top)]
    lines = []
    for (x1, y1), (x2, y2) in edges:
        lines.append(
            f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{y2:.2f}" '
            f'stroke="{fill}" stroke-width="{stroke:.2f}" stroke-linecap="round"/>'
        )
    nodes = [
        f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{node_r:.2f}" fill="{fill}"/>'
        for x, y in (top, bl, br)
    ]
    # Small radio arcs above the top node
    arc_r1 = r * 0.22
    arc_r2 = r * 0.32
    arcs = []
    for ar in (arc_r1, arc_r2):
        # arc from -50° to -130° (SVG: degrees from +x, Y down so negative is up-left/right)
        a0 = math.radians(-140)
        a1 = math.radians(-40)
        x0 = top[0] + ar * math.cos(a0)
        y0 = top[1] + ar * math.sin(a0)
        x1 = top[0] + ar * math.cos(a1)
        y1 = top[1] + ar * math.sin(a1)
        arcs.append(
            f'<path d="M {x0:.2f} {y0:.2f} A {ar:.2f} {ar:.2f} 0 0 1 {x1:.2f} {y1:.2f}" '
            f'fill="none" stroke="{fill}" stroke-width="{stroke * 0.75:.2f}" stroke-linecap="round"/>'
        )
    return "\n".join(lines + arcs + nodes)


def _polar(cx: float, cy: float, rad: float, deg_from_top: float) -> tuple[float, float]:
    """Point on a circle; 0° = top, positive = clockwise."""
    a = math.radians(deg_from_top)
    return cx + rad * math.sin(a), cy - rad * math.cos(a)


def _rounded_bar(cx: float, cy: float, w: float, h: float) -> str:
    """Capsule (rounded rect) centered at (cx, cy); works for horizontal or vertical."""
    x = cx - w / 2
    y = cy - h / 2
    rx = min(w, h) / 2
    return (
        f'<rect x="{x:.2f}" y="{y:.2f}" width="{w:.2f}" height="{h:.2f}" '
        f'rx="{rx:.2f}" ry="{rx:.2f}"/>'
    )


def tactics_icon(cx: float, cy: float, r: float, fill: str) -> str:
    """Lightbulb emblem — rim, field bands, grain stalk, screw base.

    Hand-authored fair curves (arcs / capsules / ellipses). Monochrome olive on
    the shared lime disc (same language as Hub/Link). Proportions follow the
    approved option-1 PNG; craft matches Hub/Link polish.
    """
    # Fit taller-than-wide bulb inside the disc with breathing room
    R = r * 0.66
    by = cy - r * 0.08
    rim_t = R * 0.148
    Ri = R - rim_t
    a0, a1 = -108.0, 108.0

    parts: list[str] = [f'<g fill="{fill}">']

    # --- Bulb rim (round-capped stroke arc — same craft language as Hub/Link) ---
    rc = R - rim_t / 2
    x0, y0 = _polar(cx, by, rc, a0)
    x1, y1 = _polar(cx, by, rc, a1)
    span = (a1 - a0) % 360.0
    large = 1 if span > 180.0 else 0
    parts.append(
        f'  <path d="M {x0:.2f} {y0:.2f} A {rc:.2f} {rc:.2f} 0 {large} 1 {x1:.2f} {y1:.2f}" '
        f'fill="none" stroke="{fill}" stroke-width="{rim_t:.2f}" stroke-linecap="round"/>'
    )

    # --- Field bands: two side hills + lower center paddock ---
    # Side lobes curve up toward the rim and meet at a center V for the stalk.
    left = (
        f"M {cx - Ri * 0.90:.2f} {by + Ri * 0.28:.2f} "
        f"C {cx - Ri * 0.92:.2f} {by + Ri * 0.02:.2f} "
        f"{cx - Ri * 0.55:.2f} {by - Ri * 0.02:.2f} "
        f"{cx - Ri * 0.14:.2f} {by + Ri * 0.16:.2f} "
        f"C {cx - Ri * 0.08:.2f} {by + Ri * 0.28:.2f} "
        f"{cx - Ri * 0.10:.2f} {by + Ri * 0.42:.2f} "
        f"{cx - Ri * 0.18:.2f} {by + Ri * 0.52:.2f} "
        f"C {cx - Ri * 0.42:.2f} {by + Ri * 0.62:.2f} "
        f"{cx - Ri * 0.78:.2f} {by + Ri * 0.58:.2f} "
        f"{cx - Ri * 0.90:.2f} {by + Ri * 0.28:.2f} Z"
    )
    right = (
        f"M {cx + Ri * 0.90:.2f} {by + Ri * 0.28:.2f} "
        f"C {cx + Ri * 0.92:.2f} {by + Ri * 0.02:.2f} "
        f"{cx + Ri * 0.55:.2f} {by - Ri * 0.02:.2f} "
        f"{cx + Ri * 0.14:.2f} {by + Ri * 0.16:.2f} "
        f"C {cx + Ri * 0.08:.2f} {by + Ri * 0.28:.2f} "
        f"{cx + Ri * 0.10:.2f} {by + Ri * 0.42:.2f} "
        f"{cx + Ri * 0.18:.2f} {by + Ri * 0.52:.2f} "
        f"C {cx + Ri * 0.42:.2f} {by + Ri * 0.62:.2f} "
        f"{cx + Ri * 0.78:.2f} {by + Ri * 0.58:.2f} "
        f"{cx + Ri * 0.90:.2f} {by + Ri * 0.28:.2f} Z"
    )
    center = (
        f"M {cx - Ri * 0.48:.2f} {by + Ri * 0.48:.2f} "
        f"C {cx - Ri * 0.22:.2f} {by + Ri * 0.38:.2f} "
        f"{cx + Ri * 0.22:.2f} {by + Ri * 0.38:.2f} "
        f"{cx + Ri * 0.48:.2f} {by + Ri * 0.48:.2f} "
        f"C {cx + Ri * 0.55:.2f} {by + Ri * 0.62:.2f} "
        f"{cx + Ri * 0.42:.2f} {by + Ri * 0.78:.2f} "
        f"{cx:.2f} {by + Ri * 0.82:.2f} "
        f"C {cx - Ri * 0.42:.2f} {by + Ri * 0.78:.2f} "
        f"{cx - Ri * 0.55:.2f} {by + Ri * 0.62:.2f} "
        f"{cx - Ri * 0.48:.2f} {by + Ri * 0.48:.2f} Z"
    )
    for d in (left, right, center):
        parts.append(f'  <path d="{d}"/>')

    # --- Grain stalk / filament (mono olive; thicker geometry, not a 2nd color) ---
    stalk_w = max(R * 0.065, 1.35)
    node_r = R * 0.068
    head_r = R * 0.100
    stalk_top = by - Ri * 0.58
    stalk_bot = by + Ri * 0.36
    parts.append(f"  {_rounded_bar(cx, (stalk_top + stalk_bot) / 2, stalk_w, stalk_bot - stalk_top)}")
    for t in (0.30, 0.52, 0.74):
        ny = stalk_top + (stalk_bot - stalk_top) * t
        parts.append(f'  <circle cx="{cx:.2f}" cy="{ny:.2f}" r="{node_r:.2f}"/>')
    head_y = stalk_top + head_r * 0.15
    parts.append(f'  <circle cx="{cx:.2f}" cy="{head_y:.2f}" r="{head_r:.2f}"/>')
    # Grain-head halo — thicker stroke so it reads at 32px without a second fill
    halo_r = head_r * 1.58
    halo_sw = max(R * 0.078, 1.5)
    ha0, ha1 = math.radians(-148), math.radians(-32)
    hx0 = cx + halo_r * math.cos(ha0)
    hy0 = head_y + halo_r * math.sin(ha0)
    hx1 = cx + halo_r * math.cos(ha1)
    hy1 = head_y + halo_r * math.sin(ha1)
    parts.append(
        f'  <path d="M {hx0:.2f} {hy0:.2f} A {halo_r:.2f} {halo_r:.2f} 0 0 1 {hx1:.2f} {hy1:.2f}" '
        f'fill="none" stroke="{fill}" stroke-width="{halo_sw:.2f}" stroke-linecap="round"/>'
    )

    # --- Screw base: four tapering capsules ---
    bar_h = R * 0.092
    bar_gap = R * 0.052
    bar_y0 = by + R * 0.96
    widths = [R * 1.18, R * 0.98, R * 0.78, R * 0.58]
    for i, bw in enumerate(widths):
        by_bar = bar_y0 + i * (bar_h + bar_gap)
        parts.append(f"  {_rounded_bar(cx, by_bar, bw, bar_h)}")

    parts.append("</g>")
    return "\n".join(parts)


def letter_icon(cx: float, cy: float, r: float, fill: str, font: TTFont, letter: str = "T") -> str:
    """Legacy letter mark — lime disc + Jost capital (previous site mark)."""
    glyph_set = font.getGlyphSet()
    cmap = font.getBestCmap()
    mapped = cmap[ord(letter)]
    name = mapped if isinstance(mapped, str) else font.getGlyphName(mapped)
    bounds_pen = BoundsPen(glyph_set)
    glyph_set[name].draw(bounds_pen)
    if bounds_pen.bounds is None:
        raise RuntimeError(f"No bounds for letter {letter!r}")
    xmin, ymin, xmax, ymax = bounds_pen.bounds
    gw = xmax - xmin
    gh = ymax - ymin
    target = r * 1.04
    scale = target / max(gw, gh)
    path = glyph_path(font, name)
    ox = cx - (xmin + gw / 2) * scale
    oy = cy + (ymin + gh / 2) * scale - r * 0.02
    return (
        f'<g fill="{fill}" transform="translate({ox:.3f} {oy:.3f}) '
        f'scale({scale:.6f} {-scale:.6f})">\n'
        f'  <path d="{path}"/>\n'
        f"</g>"
    )


def icon_for(kind: str, cx: float, cy: float, r: float, fill: str, font_bold: TTFont) -> str:
    if kind == "hub":
        return hub_icon(cx, cy, r, fill)
    if kind == "link":
        return link_icon(cx, cy, r, fill)
    if kind == "tactics":
        return tactics_icon(cx, cy, r, fill)
    if kind == "t":
        return letter_icon(cx, cy, r, fill, font_bold, "T")
    raise ValueError(f"Unknown mark kind: {kind}")


def mark_svg(
    kind: str,
    *,
    bg: str,
    fg: str,
    font_bold: TTFont,
    size: int = 512,
    padding: float = 0.06,
) -> str:
    r = size / 2
    pad = size * padding
    cr = r - pad
    icon = icon_for(kind, r, r, cr, fg, font_bold)
    return f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" role="img" aria-label="mark">
  <title>Mark</title>
  <circle cx="{r}" cy="{r}" r="{cr}" fill="{bg}"/>
  {icon}
</svg>
'''


def square_app_icon(
    kind: str,
    *,
    bg: str,
    fg: str,
    disc: str,
    font_bold: TTFont,
    size: int = 512,
) -> str:
    """Square app icon: solid field with centered lime-disc mark."""
    r = size / 2
    disc_r = size * 0.38
    icon = icon_for(kind, r, r, disc_r, fg, font_bold)
    return f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" role="img" aria-label="app icon">
  <title>App icon</title>
  <rect width="{size}" height="{size}" fill="{bg}"/>
  <circle cx="{r}" cy="{r}" r="{disc_r}" fill="{disc}"/>
  {icon}
</svg>
'''


def compose_logo(
    product: dict,
    font_reg: TTFont,
    font_bold: TTFont,
    *,
    word_fill: str,
    tag_fill: str,
    mark_bg: str,
    mark_fg: str,
    with_tag: bool,
    mark_size: float = 96.0,
    word_size: float = 44.0,
    gap: float = 18.0,
) -> str:
    prefix_paths, prefix_w = text_to_paths(
        product["prefix"], font_reg, x=0, baseline=0, size=word_size, tracking_em=0.01
    )
    suffix_paths, suffix_w = text_to_paths(
        product["suffix"], font_bold, x=prefix_w, baseline=0, size=word_size, tracking_em=0.01
    )
    word_w = prefix_w + suffix_w

    tag_paths = ""
    tag_w = 0.0
    tag_size = word_size * 0.32
    if with_tag:
        tag_paths, tag_w = text_to_paths(
            product["tag"], font_reg, x=0, baseline=0, size=tag_size, tracking_em=0.16
        )

    content_w = max(word_w, tag_w)
    total_w = mark_size + gap + content_w
    if with_tag:
        block_h = word_size + tag_size * 1.85
        total_h = max(mark_size, block_h)
        word_baseline = (total_h - block_h) / 2 + word_size * 0.78
        tag_baseline = word_baseline + tag_size * 1.55
    else:
        total_h = mark_size
        word_baseline = mark_size * 0.62
        tag_baseline = 0

    pad_x, pad_y = 8, 8
    vb_w = total_w + pad_x * 2
    vb_h = total_h + pad_y * 2

    mark_y = pad_y + (total_h - mark_size) / 2
    text_x = pad_x + mark_size + gap

    mark_cx = pad_x + mark_size / 2
    mark_cy = mark_y + mark_size / 2
    mark_r = mark_size / 2

    mark_icon = icon_for(product["mark"], mark_cx, mark_cy, mark_r, mark_fg, font_bold)
    mark_disc = (
        f'<circle cx="{mark_cx:.2f}" cy="{mark_cy:.2f}" r="{mark_r:.2f}" fill="{mark_bg}"/>\n  '
    )

    tag_group = ""
    if with_tag:
        tag_group = f'''
  <g fill="{tag_fill}" transform="translate({text_x:.3f} {pad_y + tag_baseline:.3f})">
    {tag_paths}
  </g>'''

    return f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vb_w:.2f} {vb_h:.2f}" role="img" aria-label="{product["name"]}">
  <title>{product["name"]}</title>
  {mark_disc}{mark_icon}
  <g fill="{word_fill}" transform="translate({text_x:.3f} {pad_y + word_baseline:.3f})">
    {prefix_paths}
    {suffix_paths}
  </g>{tag_group}
</svg>
'''


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")
    print(f"  wrote {path.relative_to(ROOT)}")


def export_pngs(svg_path: Path, out_dir: Path, stem: str, widths: list[int]) -> None:
    """Rasterize SVG via @resvg/resvg-js."""
    script = ROOT / "_tools" / "rasterize.mjs"
    cmd = ["node", str(script), str(svg_path), str(out_dir), stem, *[str(w) for w in widths]]
    subprocess.check_call(cmd)


def main() -> int:
    font_reg = load_font(400)
    font_bold = load_font(600)

    for key, product in PRODUCTS.items():
        out = OUT[key]
        out.mkdir(parents=True, exist_ok=True)
        png = out / "png"
        png.mkdir(exist_ok=True)
        kind = product["mark"]
        print(f"\n== {product['name']} ==")

        # --- Marks (lime disc + olive icon for all brands) ---
        write(out / "mark.svg", mark_svg(kind, bg=LIME, fg=OLIVE_900, font_bold=font_bold))
        write(out / "mark-dark.svg", mark_svg(kind, bg=OLIVE, fg=IVORY, font_bold=font_bold))
        write(out / "mark-light.svg", mark_svg(kind, bg=IVORY, fg=OLIVE_900, font_bold=font_bold))
        write(
            out / "icon.svg",
            square_app_icon(kind, bg=OLIVE, fg=OLIVE_900, disc=LIME, font_bold=font_bold),
        )
        write(
            out / "icon-light.svg",
            square_app_icon(kind, bg=IVORY, fg=OLIVE_900, disc=LIME, font_bold=font_bold),
        )
        write(
            out / "logo.svg",
            compose_logo(
                product,
                font_reg,
                font_bold,
                word_fill=INK,
                tag_fill=OLIVE,
                mark_bg=LIME,
                mark_fg=OLIVE_900,
                with_tag=False,
            ),
        )
        write(
            out / "logo-on-dark.svg",
            compose_logo(
                product,
                font_reg,
                font_bold,
                word_fill=WHITE,
                tag_fill=LIME,
                mark_bg=LIME,
                mark_fg=OLIVE_900,
                with_tag=False,
            ),
        )
        write(
            out / "lockup.svg",
            compose_logo(
                product,
                font_reg,
                font_bold,
                word_fill=INK,
                tag_fill=GOLD,
                mark_bg=LIME,
                mark_fg=OLIVE_900,
                with_tag=True,
                mark_size=72,
                word_size=36,
                gap=14,
            ),
        )
        write(
            out / "lockup-on-dark.svg",
            compose_logo(
                product,
                font_reg,
                font_bold,
                word_fill=WHITE,
                tag_fill=LIME,
                mark_bg=LIME,
                mark_fg=OLIVE_900,
                with_tag=True,
                mark_size=72,
                word_size=36,
                gap=14,
            ),
        )

        # Rasterize
        print("  rasterizing…")
        export_pngs(out / "mark.svg", png, "mark", [32, 64, 128, 256, 512])
        export_pngs(out / "icon.svg", png, "icon", [32, 64, 128, 256, 512])
        export_pngs(out / "logo.svg", png, "logo", [1, 2])
        export_pngs(out / "logo-on-dark.svg", png, "logo-on-dark", [1, 2])
        export_pngs(out / "lockup.svg", png, "lockup", [1, 2])
        export_pngs(out / "lockup-on-dark.svg", png, "lockup-on-dark", [1, 2])

    # Palette reference
    palette = {
        "ink": INK,
        "olive": OLIVE,
        "olive_900": OLIVE_900,
        "lime": LIME,
        "ivory": IVORY,
        "gold": GOLD,
        "fonts": {"display": "Jost", "body": "DM Sans"},
        "source": "https://terratactics.com.au",
    }
    write(ROOT / "palette.json", json.dumps(palette, indent=2) + "\n")
    print("\nDone.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
