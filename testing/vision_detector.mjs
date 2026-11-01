/**
 * Visual Artifact Detector for Rust J2ME Emulator
 * Analyzes rendered frame pixel buffers for visual anomalies:
 * 1. TEXT_SCANLINE_GAP_BUG: Interlacing / horizontal scanline drops on vertical stems.
 * 2. MEMORY_GARBAGE_RENDER: Uninitialized buffer noise with high entropy (>40% pixels).
 * 3. BLACK_SCREEN: >99.9% blank/black pixels after startup period.
 */

/**
 * Calculates perceived luminance (0-255) for an RGBA pixel.
 */
export function getLuminance(r, g, b) {
  return 0.299 * r + 0.587 * g + 0.114 * b;
}

/**
 * Detects horizontal scanline gaps / comb artifacts (TEXT_SCANLINE_GAP_BUG).
 * Anomaly occurs when alternate lines along a vertical stroke drop to background color
 * with a high brightness delta (Delta > 150), oscillating for multiple lines across columns.
 *
 * @param {Uint8Array|Uint8ClampedArray} pixels - RGBA pixel buffer
 * @param {number} width - Canvas width
 * @param {number} height - Canvas height
 * @param {object} options
 * @returns {object} { detected, confidence, count, message }
 */
export function detectScanlineGaps(pixels, width, height, options = {}) {
  const minDelta = options.minDelta ?? 140; // Luminance contrast threshold
  const maxSimilarity = options.maxSimilarity ?? 60; // Max difference between line y and y+2
  const minOscillations = options.minOscillations ?? 3; // Number of alternating stripes
  const minAffectedColumns = options.minAffectedColumns ?? 4; // Distinct columns affected

  if (!pixels || width < 4 || height < 6) {
    return { detected: false, confidence: 0, count: 0, message: "Canvas too small" };
  }

  // Precompute luminance grid for speed and simplicity
  const lum = new Float32Array(width * height);
  for (let i = 0; i < width * height; i++) {
    const idx = i * 4;
    const a = pixels[idx + 3];
    if (a < 128) {
      lum[i] = 0.0;
    } else {
      lum[i] = getLuminance(pixels[idx], pixels[idx + 1], pixels[idx + 2]);
    }
  }

  let affectedColumns = 0;
  let totalOscillatingRuns = 0;

  for (let x = 0; x < width; x++) {
    let columnHasBug = false;
    let y = 0;

    while (y < height - 5) {
      // Check for alternating parity oscillation:
      // y (stroke), y+1 (gap), y+2 (stroke), y+3 (gap), etc.
      let runLength = 0;
      let curY = y;

      while (curY < height - 2) {
        const l0 = lum[curY * width + x];
        const l1 = lum[(curY + 1) * width + x];
        const l2 = lum[(curY + 2) * width + x];

        const delta01 = Math.abs(l0 - l1);
        const delta12 = Math.abs(l1 - l2);
        const diff02 = Math.abs(l0 - l2);

        // Even lines resemble each other, odd lines differ strongly from even lines
        if (delta01 >= minDelta && delta12 >= minDelta && diff02 <= maxSimilarity) {
          runLength++;
          curY += 2;
        } else {
          break;
        }
      }

      if (runLength >= minOscillations) {
        totalOscillatingRuns++;
        columnHasBug = true;
        y = curY;
      } else {
        y++;
      }
    }

    if (columnHasBug) {
      affectedColumns++;
    }
  }

  const detected = affectedColumns >= minAffectedColumns;
  const confidence = Math.min(1.0, affectedColumns / (minAffectedColumns * 2));

  return {
    detected,
    confidence,
    affectedColumns,
    totalOscillatingRuns,
    message: detected
      ? `TEXT_SCANLINE_GAP_BUG detected across ${affectedColumns} columns (${totalOscillatingRuns} oscillating vertical runs)`
      : "No scanline gap anomalies detected",
  };
}

/**
 * Detects uninitialized memory garbage / raw buffer noise (MEMORY_GARBAGE_RENDER).
 * Memory garbage exhibits maximum spatial entropy: adjacent pixels lack local correlation
 * in all 4 directions, unlike real fonts, game sprites, or procedural textures.
 *
 * @param {Uint8Array|Uint8ClampedArray} pixels - RGBA pixel buffer
 * @param {number} width - Canvas width
 * @param {number} height - Canvas height
 * @param {object} options
 * @returns {object} { detected, noiseRatio, entropy, message }
 */
export function detectMemoryGarbage(pixels, width, height, options = {}) {
  const noiseThreshold = options.threshold ?? 0.4; // 40% threshold required by spec
  const deltaThreshold = options.deltaThreshold ?? 60;

  if (!pixels || width < 4 || height < 4) {
    return { detected: false, noiseRatio: 0, entropy: 0, message: "Canvas too small" };
  }

  const totalPixels = width * height;
  let uncorrelatedPixels = 0;

  // We sample pixels inside borders
  for (let y = 1; y < height - 1; y++) {
    for (let x = 1; x < width - 1; x++) {
      const idx = (y * width + x) * 4;
      const r = pixels[idx];
      const g = pixels[idx + 1];
      const b = pixels[idx + 2];

      // Neighbors: left, right, top, bottom
      const leftIdx = (y * width + (x - 1)) * 4;
      const rightIdx = (y * width + (x + 1)) * 4;
      const topIdx = ((y - 1) * width + x) * 4;
      const botIdx = ((y + 1) * width + x) * 4;

      const dLeft = Math.abs(r - pixels[leftIdx]) + Math.abs(g - pixels[leftIdx + 1]) + Math.abs(b - pixels[leftIdx + 2]);
      const dRight = Math.abs(r - pixels[rightIdx]) + Math.abs(g - pixels[rightIdx + 1]) + Math.abs(b - pixels[rightIdx + 2]);
      const dTop = Math.abs(r - pixels[topIdx]) + Math.abs(g - pixels[topIdx + 1]) + Math.abs(b - pixels[topIdx + 2]);
      const dBot = Math.abs(r - pixels[botIdx]) + Math.abs(g - pixels[botIdx + 1]) + Math.abs(b - pixels[botIdx + 2]);

      // Average channel difference across the 4 neighbors
      const avgNeighborDiff = (dLeft + dRight + dTop + dBot) / 12.0;
      if (avgNeighborDiff > deltaThreshold) {
        uncorrelatedPixels++;
      }
    }
  }

  const innerPixels = (width - 2) * (height - 2);
  const noiseRatio = innerPixels > 0 ? uncorrelatedPixels / innerPixels : 0;
  const detected = noiseRatio >= noiseThreshold;

  return {
    detected,
    noiseRatio: Number(noiseRatio.toFixed(4)),
    uncorrelatedPixels,
    message: detected
      ? `MEMORY_GARBAGE_RENDER detected: ${(noiseRatio * 100).toFixed(1)}% pixels exhibit uninitialized memory noise (>40% threshold)`
      : `Normal buffer coherence: ${(noiseRatio * 100).toFixed(1)}% noise`,
  };
}

/**
 * Detects blank / completely black or transparent screen (BLACK_SCREEN).
 *
 * @param {Uint8Array|Uint8ClampedArray} pixels - RGBA pixel buffer
 * @param {number} width - Canvas width
 * @param {number} height - Canvas height
 * @param {number} threshold - Minimum ratio of black/transparent pixels (default 0.999 = 99.9%)
 * @returns {object} { detected, blackRatio, message }
 */
export function detectBlackScreen(pixels, width, height, threshold = 0.999) {
  if (!pixels || width === 0 || height === 0) {
    return { detected: true, blackRatio: 1.0, message: "Empty pixel buffer" };
  }

  const totalPixels = width * height;
  let blackCount = 0;

  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    const r = pixels[idx];
    const g = pixels[idx + 1];
    const b = pixels[idx + 2];
    const a = pixels[idx + 3];

    // Black or transparent
    if (a <= 10 || (r <= 5 && g <= 5 && b <= 5)) {
      blackCount++;
    }
  }

  const blackRatio = blackCount / totalPixels;
  const detected = blackRatio >= threshold;

  return {
    detected,
    blackRatio: Number(blackRatio.toFixed(5)),
    blackCount,
    totalPixels,
    message: detected
      ? `BLACK_SCREEN detected: ${(blackRatio * 100).toFixed(2)}% pixels are blank/black`
      : `Screen active: ${(blackRatio * 100).toFixed(2)}% black pixels`,
  };
}

/**
 * High-level frame artifact analyzer combining all detectors.
 */
export function analyzeFrame(pixels, width, height, options = {}) {
  const elapsedMs = options.elapsedMs ?? 4000;
  const artifacts = [];

  // 1. Scanline Gaps
  const scanline = detectScanlineGaps(pixels, width, height, options.scanlineOptions);
  if (scanline.detected) {
    artifacts.push("TEXT_SCANLINE_GAP_BUG");
  }

  // 2. Memory Garbage
  const garbage = detectMemoryGarbage(pixels, width, height, options.garbageOptions);
  if (garbage.detected) {
    artifacts.push("MEMORY_GARBAGE_RENDER");
  }

  // 3. Black Screen (only evaluated if elapsedMs >= 3000ms as per spec)
  const black = detectBlackScreen(pixels, width, height);
  if (elapsedMs >= 3000 && black.detected) {
    artifacts.push("BLACK_SCREEN");
  }

  return {
    hasArtifacts: artifacts.length > 0,
    artifacts,
    details: {
      scanline,
      garbage,
      black,
    },
  };
}
