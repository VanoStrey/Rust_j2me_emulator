import assert from "node:assert";
import {
  detectScanlineGaps,
  detectMemoryGarbage,
  detectBlackScreen,
  analyzeFrame,
} from "./vision_detector.mjs";

console.log("=== Running Unit Tests for Visual Artifact Detector ===");

// -------------------------------------------------------------
// Helper: create RGBA buffer
// -------------------------------------------------------------
function createBuffer(w, h, fill = [0, 0, 0, 255]) {
  const buf = new Uint8Array(w * h * 4);
  for (let i = 0; i < w * h; i++) {
    buf[i * 4] = fill[0];
    buf[i * 4 + 1] = fill[1];
    buf[i * 4 + 2] = fill[2];
    buf[i * 4 + 3] = fill[3];
  }
  return buf;
}

// -------------------------------------------------------------
// Test 1: Scanline Gap Detector - Normal Text (No Gaps)
// -------------------------------------------------------------
{
  const w = 100, h = 50;
  const buf = createBuffer(w, h, [0, 0, 0, 255]); // black background

  // Draw 6 solid white vertical stems (like letters 'M', 'I', 'L')
  for (let col = 10; col <= 60; col += 10) {
    for (let x = col; x < col + 2; x++) {
      for (let y = 10; y < 35; y++) {
        const idx = (y * w + x) * 4;
        buf[idx] = 255;
        buf[idx + 1] = 255;
        buf[idx + 2] = 255;
      }
    }
  }

  const result = detectScanlineGaps(buf, w, h);
  assert.strictEqual(result.detected, false, "Solid stems should not trigger scanline bug");
  console.log("✔ Test 1 PASS: Normal text with solid strokes does not trigger TEXT_SCANLINE_GAP_BUG");
}

// -------------------------------------------------------------
// Test 2: Scanline Gap Detector - Interlaced / Gapped Text Stems
// -------------------------------------------------------------
{
  const w = 100, h = 50;
  const buf = createBuffer(w, h, [10, 10, 10, 255]); // dark background

  // Draw stems with dropped scanlines (even y = white, odd y = dark background, Delta > 200)
  for (let col = 10; col <= 60; col += 10) {
    for (let x = col; x < col + 2; x++) {
      for (let y = 10; y < 35; y++) {
        const idx = (y * w + x) * 4;
        if (y % 2 === 0) {
          buf[idx] = 250;
          buf[idx + 1] = 250;
          buf[idx + 2] = 250;
        } else {
          buf[idx] = 10;
          buf[idx + 1] = 10;
          buf[idx + 2] = 10;
        }
      }
    }
  }

  const result = detectScanlineGaps(buf, w, h);
  assert.strictEqual(result.detected, true, "Scanline gaps must be detected!");
  assert(result.affectedColumns >= 6, `Expected >= 6 affected columns, got ${result.affectedColumns}`);
  console.log("✔ Test 2 PASS: Artificial scanline-gapped text accurately triggers TEXT_SCANLINE_GAP_BUG");
}

// -------------------------------------------------------------
// Test 3: Memory Garbage Detector - Random High-Entropy Noise
// -------------------------------------------------------------
{
  const w = 80, h = 80;
  const buf = new Uint8Array(w * h * 4);

  // Fill with pseudo-random uncorrelated noise (high entropy)
  let seed = 12345;
  function rnd() {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed & 0xff;
  }

  for (let i = 0; i < w * h * 4; i += 4) {
    buf[i] = rnd();
    buf[i + 1] = rnd();
    buf[i + 2] = rnd();
    buf[i + 3] = 255;
  }

  const result = detectMemoryGarbage(buf, w, h);
  assert.strictEqual(result.detected, true, "High entropy noise must be detected as memory garbage");
  assert(result.noiseRatio > 0.40, `Noise ratio must exceed 40%, got ${(result.noiseRatio * 100).toFixed(1)}%`);
  console.log(`✔ Test 3 PASS: Random noise detected as MEMORY_GARBAGE_RENDER (${(result.noiseRatio * 100).toFixed(1)}% > 40%)`);
}

// -------------------------------------------------------------
// Test 4: Memory Garbage Detector - Natural Game Graphics
// -------------------------------------------------------------
{
  const w = 80, h = 80;
  const buf = createBuffer(w, h, [50, 100, 200, 255]); // blue sky

  // Add terrain gradient and tiled bricks
  for (let y = 40; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const idx = (y * w + x) * 4;
      const isBrickBorder = (x % 16 === 0) || (y % 8 === 0);
      buf[idx] = isBrickBorder ? 80 : 160;
      buf[idx + 1] = isBrickBorder ? 40 : 82;
      buf[idx + 2] = isBrickBorder ? 20 : 45;
    }
  }

  const result = detectMemoryGarbage(buf, w, h);
  assert.strictEqual(result.detected, false, "Natural game graphics should not trigger memory garbage");
  assert(result.noiseRatio < 0.15, `Expected noise ratio < 15%, got ${(result.noiseRatio * 100).toFixed(1)}%`);
  console.log(`✔ Test 4 PASS: Natural game graphics has low noise (${(result.noiseRatio * 100).toFixed(1)}%), not flagged`);
}

// -------------------------------------------------------------
// Test 5: Black Screen Detector - 100% and 99.9% Blank
// -------------------------------------------------------------
{
  const w = 100, h = 100;
  const blankBuf = createBuffer(w, h, [0, 0, 0, 255]);
  const res1 = detectBlackScreen(blankBuf, w, h);
  assert.strictEqual(res1.detected, true);
  assert.strictEqual(res1.blackRatio, 1.0);

  // Put 5 non-black pixels out of 10,000 (99.95% black)
  for (let i = 0; i < 5; i++) {
    blankBuf[i * 4] = 255;
    blankBuf[i * 4 + 1] = 255;
    blankBuf[i * 4 + 2] = 255;
  }
  const res2 = detectBlackScreen(blankBuf, w, h);
  assert.strictEqual(res2.detected, true, "99.95% black pixels should trigger BLACK_SCREEN");

  console.log("✔ Test 5 PASS: Black screen detector correctly identifies 100% and 99.95% blank screens");
}

// -------------------------------------------------------------
// Test 6: Black Screen Detector - Active Game Screen
// -------------------------------------------------------------
{
  const w = 100, h = 100;
  const activeBuf = createBuffer(w, h, [0, 0, 0, 255]);
  // 50% non-black
  for (let i = 0; i < 5000; i++) {
    activeBuf[i * 4] = 200;
    activeBuf[i * 4 + 1] = 100;
    activeBuf[i * 4 + 2] = 50;
  }
  const res = detectBlackScreen(activeBuf, w, h);
  assert.strictEqual(res.detected, false);
  console.log(`✔ Test 6 PASS: Active screen with 50% content correctly not detected as black screen`);
}

// -------------------------------------------------------------
// Test 7: Composite analyzeFrame() function
// -------------------------------------------------------------
{
  const w = 60, h = 60;
  const blankBuf = createBuffer(w, h, [0, 0, 0, 255]);

  // Under 3000ms: BLACK_SCREEN should not be reported yet (boot grace period)
  const bootRes = analyzeFrame(blankBuf, w, h, { elapsedMs: 1500 });
  assert.strictEqual(bootRes.artifacts.includes("BLACK_SCREEN"), false, "Boot grace period should not trigger BLACK_SCREEN");

  // After 3000ms: BLACK_SCREEN should be reported
  const steadyRes = analyzeFrame(blankBuf, w, h, { elapsedMs: 3500 });
  assert.strictEqual(steadyRes.artifacts.includes("BLACK_SCREEN"), true, "After 3s blank screen triggers BLACK_SCREEN");

  console.log("✔ Test 7 PASS: analyzeFrame handles grace periods and composite artifacts correctly");
}

console.log("\nALL VISION DETECTOR UNIT TESTS PASSED (7/7)!");
