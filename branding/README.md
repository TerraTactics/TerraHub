# TerraTactics branding

Assets for the **TerraTactics** parent brand plus product sub-brands **TerraHub**
and **TerraLink**, matched to [terratactics.com.au](https://terratactics.com.au):
olive / ivory / gold / lime, Jost wordmarks.

## Palette

| Token | Hex | Use |
|-------|-----|-----|
| Lime | `#DDF34A` | Mark discs (all brands) |
| Olive 900 | `#1C2415` | Icon fill on lime; app-icon fields |
| Olive | `#344126` | Dark-mark discs; secondary |
| Ink | `#17251D` | Wordmark on light |
| Ivory | `#F6F3EA` | Light mono mark / light app field |
| Gold | `#D9A521` | Lockup tagline on light |

See `palette.json` for the machine-readable set.

## Marks

All primary marks share the same language: **lime disc + monochrome olive icon**.

| Brand | Symbol |
|-------|--------|
| TerraTactics | Lightbulb — rim, field bands, grain stalk, screw base (hand-authored geometry from approved option-1 concept) |
| TerraHub | Hub-and-spoke — field gateway |
| TerraLink | Three-node mesh + radio arcs — LoRa field mesh |

## Files (each of `terratactics/`, `terrahub/`, `terralink/`)

| File | Purpose |
|------|---------|
| `logo.svg` | Primary wordmark + mark (dark on light) |
| `logo-on-dark.svg` | Same, white wordmark for dark UI / headers |
| `lockup.svg` | README / docs / site header lockup + tagline |
| `lockup-on-dark.svg` | Lockup for dark backgrounds |
| `mark.svg` | Primary mark — lime disc + olive icon |
| `mark-dark.svg` | Mono dark (olive disc + ivory icon) — for light backgrounds |
| `mark-light.svg` | Mono light (ivory disc + olive icon) — for dark backgrounds |
| `icon.svg` | Square app / org icon |
| `icon-light.svg` | Square icon on ivory |
| `png/` | Raster exports: `mark-{32..512}`, `icon-{32..512}`, `logo-{1x,2x}`, `lockup-{1x,2x}` (+ `-on-dark`) |

Wordmarks are outlined Jost paths (no live font dependency). The TerraTactics
bulb is **hand-authored SVG geometry** in `branding/_tools/generate.py` (fair arcs,
capsules, ellipses) — not a contour trace. Reference concept PNG:
`terratactics/_ref-option1.png`. Legacy letter-**T** remains available as mark kind `t`.

## Website / org drop-in

For [terratactics.com.au](https://terratactics.com.au) and the GitHub org avatar:

| Need | Use |
|------|-----|
| Site header (dark bar) | `terratactics/lockup-on-dark.svg` or `logo-on-dark.svg` |
| Light surfaces | `terratactics/lockup.svg` / `logo.svg` |
| Favicon / org avatar | `terratactics/icon.svg` or `png/icon-512.png` |
| Color mark alone | `terratactics/mark.svg` |

## Usage notes

- Prefer SVG in docs and the site; use PNG where hosts strip SVG.
- Do not recolor the lime disc to purple or generic SaaS blues.
- Product marks share the lime-disc language; keep the tactics lightbulb exclusive to TerraTactics.

## Regenerate

```bash
# from repo root (needs Python fonttools + Node @resvg/resvg-js under branding/_tools)
python branding/_tools/generate.py
```

Jost variable font + OFL license live under `branding/_fonts/` (OFL).
