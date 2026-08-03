#!/usr/bin/env node
/**
 * Rasterize an SVG to PNG at given widths.
 * Usage: node rasterize.mjs <svg> <outdir> <stem> <w1> [w2...]
 *
 * Special: widths 1 and 2 mean 1x/2x by intrinsic height
 * (1x ≈ 48px tall for logos, 40px for lockups).
 */
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join, basename } from "node:path";
import { Resvg } from "@resvg/resvg-js";

const [svgPath, outDir, stem, ...widthArgs] = process.argv.slice(2);
if (!svgPath || !outDir || !stem || widthArgs.length === 0) {
  console.error("Usage: node rasterize.mjs <svg> <outdir> <stem> <w1> [w2...]");
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });
const svg = readFileSync(svgPath);

const viewBoxMatch = svg.toString().match(/viewBox="([^"]+)"/);
const [, , vbW, vbH] = viewBoxMatch
  ? viewBoxMatch[1].split(/\s+/).map(Number)
  : [0, 0, 512, 512];
if (!Number.isFinite(vbW) || !Number.isFinite(vbH) || vbH === 0) {
  console.error("Could not parse viewBox from", svgPath);
  process.exit(1);
}

for (const wArg of widthArgs) {
  const n = Number(wArg);
  let width;
  let label;
  if (n === 1 || n === 2) {
    // Target height: lockups ~40px @1x, logos ~48px @1x
    const isLockup = stem.includes("lockup");
    const h1 = isLockup ? 48 : 64;
    const height = h1 * n;
    width = Math.round((vbW / vbH) * height);
    label = `${stem}-${n}x.png`;
  } else {
    width = n;
    label = `${stem}-${n}.png`;
  }

  const resvg = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
    background: "rgba(0,0,0,0)",
  });
  const png = resvg.render().asPng();
  const outPath = join(outDir, label);
  writeFileSync(outPath, png);
  console.log(`    ${basename(outPath)} (${width}px wide)`);
}
