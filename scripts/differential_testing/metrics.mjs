/**
 * Visual Differential Testing Metrics Engine (Milestone 2)
 * Implements 5 mathematical visual parity metrics:
 * 1. Exact Pixel Match (%)
 * 2. Mean Absolute Error (MAE)
 * 3. Root Mean Square Error (RMSE)
 * 4. Percentage of Changed Pixels (ΔRGB > tolerance)
 * 5. Structural Similarity Index (SSIM)
 */

/**
 * Calculates exact pixel match percentage (RGBA equality).
 * @param {Uint8Array} bufA - RGBA buffer A
 * @param {Uint8Array} bufB - RGBA buffer B
 * @returns {number} Percentage in [0.0 .. 100.0]
 */
export function exactPixelMatch(bufA, bufB) {
  const totalPixels = Math.min(bufA.length, bufB.length) / 4;
  if (totalPixels === 0) return 0;

  let matching = 0;
  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    if (
      bufA[idx] === bufB[idx] &&
      bufA[idx + 1] === bufB[idx + 1] &&
      bufA[idx + 2] === bufB[idx + 2]
    ) {
      matching++;
    }
  }
  return +((matching / totalPixels) * 100).toFixed(2);
}

/**
 * Calculates Mean Absolute Error (MAE) normalized per channel [0.0 .. 255.0].
 * @param {Uint8Array} bufA
 * @param {Uint8Array} bufB
 * @returns {number}
 */
export function meanAbsoluteError(bufA, bufB) {
  const totalPixels = Math.min(bufA.length, bufB.length) / 4;
  if (totalPixels === 0) return 0;

  let totalDiff = 0;
  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    const dr = Math.abs(bufA[idx] - bufB[idx]);
    const dg = Math.abs(bufA[idx + 1] - bufB[idx + 1]);
    const db = Math.abs(bufA[idx + 2] - bufB[idx + 2]);
    totalDiff += (dr + dg + db) / 3.0;
  }
  return +(totalDiff / totalPixels).toFixed(3);
}

/**
 * Calculates Root Mean Square Error (RMSE) across RGB channels [0.0 .. 255.0].
 * @param {Uint8Array} bufA
 * @param {Uint8Array} bufB
 * @returns {number}
 */
export function rootMeanSquareError(bufA, bufB) {
  const totalPixels = Math.min(bufA.length, bufB.length) / 4;
  if (totalPixels === 0) return 0;

  let totalSqDiff = 0;
  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    const dr = bufA[idx] - bufB[idx];
    const dg = bufA[idx + 1] - bufB[idx + 1];
    const db = bufA[idx + 2] - bufB[idx + 2];
    totalSqDiff += (dr * dr + dg * dg + db * db) / 3.0;
  }
  return +Math.sqrt(totalSqDiff / totalPixels).toFixed(3);
}

/**
 * Calculates percentage of pixels exceeding a per-channel color tolerance.
 * @param {Uint8Array} bufA
 * @param {Uint8Array} bufB
 * @param {number} tolerance - Max allowed delta per channel (e.g. 0 for strict, 8 for tolerant)
 * @returns {number} Percentage of changed pixels [0.0 .. 100.0]
 */
export function percentageChangedPixels(bufA, bufB, tolerance = 8) {
  const totalPixels = Math.min(bufA.length, bufB.length) / 4;
  if (totalPixels === 0) return 0;

  let changed = 0;
  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    const dr = Math.abs(bufA[idx] - bufB[idx]);
    const dg = Math.abs(bufA[idx + 1] - bufB[idx + 1]);
    const db = Math.abs(bufA[idx + 2] - bufB[idx + 2]);
    if (dr > tolerance || dg > tolerance || db > tolerance) {
      changed++;
    }
  }
  return +((changed / totalPixels) * 100).toFixed(2);
}

/**
 * Calculates Structural Similarity Index (SSIM) on luminance channel Y [0.0 .. 1.0].
 * Using 8x8 block-based computation.
 * @param {Uint8Array} bufA
 * @param {Uint8Array} bufB
 * @param {number} width
 * @param {number} height
 * @returns {number} SSIM score
 */
export function structuralSimilarityIndex(bufA, bufB, width, height) {
  const totalPixels = width * height;
  if (totalPixels === 0) return 0;

  // Convert RGB to Luminance Y = 0.299R + 0.587G + 0.114B
  const lumA = new Float32Array(totalPixels);
  const lumB = new Float32Array(totalPixels);
  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4;
    lumA[i] = 0.299 * bufA[idx] + 0.587 * bufA[idx + 1] + 0.114 * bufA[idx + 2];
    lumB[i] = 0.299 * bufB[idx] + 0.587 * bufB[idx + 1] + 0.114 * bufB[idx + 2];
  }

  // Standard SSIM constants (k1=0.01, k2=0.03, L=255)
  const c1 = (0.01 * 255) ** 2; // 6.5025
  const c2 = (0.03 * 255) ** 2; // 58.5225

  const blockSize = 8;
  const numBlocksX = Math.floor(width / blockSize);
  const numBlocksY = Math.floor(height / blockSize);
  if (numBlocksX === 0 || numBlocksY === 0) return 0;

  let totalSsim = 0;
  let blockCount = 0;

  for (let by = 0; by < numBlocksY; by++) {
    for (let bx = 0; bx < numBlocksX; bx++) {
      let meanA = 0;
      let meanB = 0;
      const startX = bx * blockSize;
      const startY = by * blockSize;

      for (let y = 0; y < blockSize; y++) {
        for (let x = 0; x < blockSize; x++) {
          const pIdx = (startY + y) * width + (startX + x);
          meanA += lumA[pIdx];
          meanB += lumB[pIdx];
        }
      }
      const n = blockSize * blockSize;
      meanA /= n;
      meanB /= n;

      let varA = 0;
      let varB = 0;
      let covAB = 0;

      for (let y = 0; y < blockSize; y++) {
        for (let x = 0; x < blockSize; x++) {
          const pIdx = (startY + y) * width + (startX + x);
          const diffA = lumA[pIdx] - meanA;
          const diffB = lumB[pIdx] - meanB;
          varA += diffA * diffA;
          varB += diffB * diffB;
          covAB += diffA * diffB;
        }
      }
      varA /= (n - 1);
      varB /= (n - 1);
      covAB /= (n - 1);

      const ssim =
        ((2 * meanA * meanB + c1) * (2 * covAB + c2)) /
        ((meanA * meanA + meanB * meanB + c1) * (varA + varB + c2));

      totalSsim += ssim;
      blockCount++;
    }
  }

  const avgSsim = blockCount > 0 ? totalSsim / blockCount : 0;
  return +Math.max(0, Math.min(1.0, avgSsim)).toFixed(4);
}

/**
 * Computes all 5 metrics in a single pass.
 */
export function computeAllMetrics(bufA, bufB, width, height) {
  return {
    exactPixelMatchPct: exactPixelMatch(bufA, bufB),
    meanAbsoluteError: meanAbsoluteError(bufA, bufB),
    rootMeanSquareError: rootMeanSquareError(bufA, bufB),
    percentageChangedPixelsStrict: percentageChangedPixels(bufA, bufB, 0),
    percentageChangedPixelsTolerant: percentageChangedPixels(bufA, bufB, 8),
    structuralSimilarityIndex: structuralSimilarityIndex(bufA, bufB, width, height)
  };
}
