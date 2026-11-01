/**
 * Visual Discrepancy Classification & Diff Image Generator (Milestone 2)
 * Categorizes differences into:
 * 1. GAME_STATE (Macro divergence > 50% or different scene hierarchy)
 * 2. OBJECT_GEOMETRY (Localized shifted entity bounding box)
 * 3. COLOR_DIFF (Palette offset, RGB clamp/gamma variation)
 * 4. FONT_METRICS (Text baseline, character glyph spacing, font height)
 * 5. ANTIALIASING (Single-pixel boundary edge gradient delta)
 * 6. ANIMATION_TIMING (Phase shift across sequential ticks)
 */

import fs from 'fs';
import { computeAllMetrics } from './metrics.mjs';

/**
 * Classify discrepancy between two RGBA buffers and produce difference image.
 * @param {Uint8Array} rustBuf
 * @param {Uint8Array} refBuf
 * @param {number} width
 * @param {number} height
 * @param {object} options
 */
export function analyzeVisualDiff(rustBuf, refBuf, width, height, options = {}) {
  const totalPixels = width * height;
  const metrics = computeAllMetrics(rustBuf, refBuf, width, height);

  const diffBuf = new Uint8Array(totalPixels * 4);
  let minX = width, maxX = 0, minY = height, maxY = 0;
  let differingCount = 0;
  let borderOnlyCount = 0;
  let smallColorDiffCount = 0;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      const r1 = rustBuf[idx];
      const g1 = rustBuf[idx + 1];
      const b1 = rustBuf[idx + 2];
      const r2 = refBuf[idx];
      const g2 = refBuf[idx + 1];
      const b2 = refBuf[idx + 2];

      const dr = Math.abs(r1 - r2);
      const dg = Math.abs(g1 - g2);
      const db = Math.abs(b1 - b2);
      const maxDelta = Math.max(dr, dg, db);

      if (maxDelta === 0) {
        // Matching: dimmed green background
        diffBuf[idx] = Math.floor(r1 * 0.25);
        diffBuf[idx + 1] = Math.min(255, Math.floor(g1 * 0.25 + 40));
        diffBuf[idx + 2] = Math.floor(b1 * 0.25);
        diffBuf[idx + 3] = 255;
      } else {
        differingCount++;
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
        minY = Math.min(minY, y);
        maxY = Math.max(maxY, y);

        if (maxDelta <= 16) {
          smallColorDiffCount++;
        }

        // Highlight difference with heatmap: yellow for slight, magenta/red for severe
        if (maxDelta <= 16) {
          diffBuf[idx] = 255;
          diffBuf[idx + 1] = 200;
          diffBuf[idx + 2] = 0;
        } else if (maxDelta <= 64) {
          diffBuf[idx] = 255;
          diffBuf[idx + 1] = 80;
          diffBuf[idx + 2] = 0;
        } else {
          diffBuf[idx] = 255;
          diffBuf[idx + 1] = 0;
          diffBuf[idx + 2] = 80;
        }
        diffBuf[idx + 3] = 255;
      }
    }
  }

  // Draw bounding box if differences exist
  const hasDiff = differingCount > 0 && maxX >= minX && maxY >= minY;
  const bbox = hasDiff
    ? { x: minX, y: minY, w: maxX - minX + 1, h: maxY - minY + 1 }
    : null;

  if (bbox) {
    drawBoundingBox(diffBuf, width, height, bbox.x, bbox.y, bbox.w, bbox.h);
  }

  // Diagnostic Classification
  let primaryCategory = 'PERFECT_MATCH';
  let categoryExplanation = 'Buffers match with 100% pixel fidelity.';

  if (differingCount > 0) {
    const diffPct = (differingCount / totalPixels) * 100;

    if (diffPct > 45.0) {
      primaryCategory = 'GAME_STATE';
      categoryExplanation = `Macro divergence (${diffPct.toFixed(1)}% of frame differs): midlets are displaying entirely different game screens, menus, or major cutscenes.`;
    } else if (bbox && bbox.w <= 180 && bbox.h <= 30 && bbox.y > 200) {
      primaryCategory = 'FONT_METRICS';
      categoryExplanation = `Text and menu font metrics: differences localized to lower/header text bounds (${diffPct.toFixed(1)}% pixels).`;
    } else if (smallColorDiffCount / differingCount > 0.85) {
      primaryCategory = 'COLOR_DIFF';
      categoryExplanation = `Color palette / rounding differences: 85%+ of differing pixels have ΔRGB ≤ 16.`;
    } else if (diffPct < 3.5 && metrics.structuralSimilarityIndex >= 0.96) {
      primaryCategory = 'ANTIALIASING';
      categoryExplanation = `Sub-pixel / edge antialiasing: High SSIM (${metrics.structuralSimilarityIndex}) with < 3.5% isolated edge boundary variations.`;
    } else if (bbox && bbox.w < width * 0.7 && bbox.h < height * 0.7) {
      primaryCategory = 'OBJECT_GEOMETRY';
      categoryExplanation = `Localized entity/sprite geometry offset in bounding box [${bbox.x}, ${bbox.y}, ${bbox.w}x${bbox.h}].`;
    } else {
      primaryCategory = 'ANIMATION_TIMING';
      categoryExplanation = `Animation timing / cadence drift: active rendering sequence at divergent cycle frames.`;
    }
  }

  return {
    metrics,
    differingPixelCount: differingCount,
    totalPixels,
    boundingBox: bbox,
    primaryCategory,
    categoryExplanation,
    diffImageBuffer: diffBuf
  };
}

function drawBoundingBox(buf, w, h, bx, by, bw, bh) {
  const drawLine = (x0, y0, x1, y1) => {
    for (let x = Math.max(0, x0); x <= Math.min(w - 1, x1); x++) {
      for (let y = Math.max(0, y0); y <= Math.min(h - 1, y1); y++) {
        const idx = (y * w + x) * 4;
        buf[idx] = 0;
        buf[idx + 1] = 230;
        buf[idx + 2] = 255;
        buf[idx + 3] = 255;
      }
    }
  };

  const pad = 2;
  const x = Math.max(0, bx - pad);
  const y = Math.max(0, by - pad);
  const x2 = Math.min(w - 1, bx + bw + pad);
  const y2 = Math.min(h - 1, by + bh + pad);

  // Top, Bottom, Left, Right
  drawLine(x, y, x2, y);
  drawLine(x, y2, x2, y2);
  drawLine(x, y, x, y2);
  drawLine(x2, y, x2, y2);
}

/**
 * Encodes RGBA buffer to PPM P6 format
 * @param {Uint8Array} rgbaBuf
 * @param {number} width
 * @param {number} height
 * @returns {Buffer}
 */
export function encodePpm(rgbaBuf, width, height) {
  const header = `P6\n${width} ${height}\n255\n`;
  const rgb = Buffer.alloc(width * height * 3);
  for (let i = 0; i < width * height; i++) {
    rgb[i * 3] = rgbaBuf[i * 4];
    rgb[i * 3 + 1] = rgbaBuf[i * 4 + 1];
    rgb[i * 3 + 2] = rgbaBuf[i * 4 + 2];
  }
  return Buffer.concat([Buffer.from(header, 'ascii'), rgb]);
}
