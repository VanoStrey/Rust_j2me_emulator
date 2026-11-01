import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { fileURLToPath } from 'node:url';
import {
  detectScanlineGaps,
  detectMemoryGarbage,
  detectBlackScreen,
  analyzeFrame,
} from './vision_detector.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');
const WEB_DIR = path.join(ROOT_DIR, 'web');
const MARKET_DIR = '/Users/foolvan/Desktop/j2me_market';
const EXPERIMENTS_DIR = path.join(ROOT_DIR, 'experiments', 'bug_hunter_artifacts');

if (!fs.existsSync(EXPERIMENTS_DIR)) {
  fs.mkdirSync(EXPERIMENTS_DIR, { recursive: true });
}

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.jar': 'application/java-archive',
  '.jad': 'text/plain',
  '.png': 'image/png',
  '.css': 'text/css',
  '.json': 'application/json',
};

/**
 * Creates an HTTP server capable of serving web player assets and game JARs.
 */
export function createTestServer(port = 8925) {
  const server = http.createServer((req, res) => {
    let reqUrl = req.url.split('?')[0];
    if (reqUrl === '/') reqUrl = '/index.html';

    let targetFile = null;
    if (reqUrl.startsWith('/market/')) {
      targetFile = path.join(MARKET_DIR, reqUrl.replace('/market/', ''));
    } else if (reqUrl.startsWith('/artifacts/')) {
      targetFile = path.join(EXPERIMENTS_DIR, reqUrl.replace('/artifacts/', ''));
    } else if (path.isAbsolute(reqUrl) && fs.existsSync(reqUrl)) {
      targetFile = reqUrl;
    } else {
      targetFile = path.join(WEB_DIR, reqUrl);
    }

    if (!fs.existsSync(targetFile) || fs.statSync(targetFile).isDirectory()) {
      res.writeHead(404, { 'Access-Control-Allow-Origin': '*' });
      res.end(`Not found: ${reqUrl}`);
      return;
    }

    const ext = path.extname(targetFile).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    res.writeHead(200, {
      'Content-Type': contentType,
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'no-cache, no-store, must-revalidate',
      Pragma: 'no-cache',
      Expires: '0',
    });
    fs.createReadStream(targetFile).pipe(res);
  });

  return new Promise((resolve) => {
    server.listen(port, () => {
      resolve({ server, port });
    });
  });
}

/**
 * Simple 32-bit FNV-1a hash of a Uint8Array or string for instant frame difference detection.
 */
function fnv1a(data) {
  let hash = 0x811c9dc5;
  for (let i = 0; i < data.length; i++) {
    hash ^= data[i];
    hash = Math.imul(hash, 0x01000193);
  }
  return hash >>> 0;
}

/**
 * Bug-Hunting & Behavioral Fuzzing Bot
 */
export class BugHunterAgent {
  constructor(options = {}) {
    this.port = options.port ?? 8925;
    this.cdpPort = options.cdpPort ?? 9260;
    this.durationSec = options.durationSec ?? 30;
    this.headless = options.headless !== false;
    this.server = null;
    this.chromeProc = null;
    this.tmpProfile = null;
    this.ws = null;
    this.msgId = 1;
    this.pendingRequests = new Map();
  }

  async start() {
    // 1. Start Server
    const srv = await createTestServer(this.port);
    this.server = srv.server;

    // 2. Launch Chrome
    this.tmpProfile = fs.mkdtempSync('/tmp/j2me_bughunter_');
    const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    const chromeArgs = [
      '--headless=new',
      '--remote-debugging-port=0',
      `--user-data-dir=${this.tmpProfile}`,
      '--disable-gpu',
      '--no-first-run',
      '--no-default-browser-check',
      '--disable-background-timer-throttling',
      '--disable-renderer-backgrounding',
      `http://127.0.0.1:${this.port}/index.html`,
    ];

    this.chromeProc = spawn(chromePath, chromeArgs);

    // 3. Connect CDP WebSocket via stderr dynamic port discovery
    let boundPort = null;
    const portPromise = new Promise((resolve) => {
      this.chromeProc.stderr.on('data', (d) => {
        const text = d.toString();
        const m = text.match(/ws:\/\/127\.0\.0\.1:(\d+)\//);
        if (m && !boundPort) {
          boundPort = parseInt(m[1], 10);
          resolve(boundPort);
        }
      });
    });

    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Chrome startup timed out after 10s')), 10000)
    );

    const actualPort = await Promise.race([portPromise, timeoutPromise]);
    this.cdpPort = actualPort;

    // Give browser a few milliseconds to initialize tabs
    let wsUrl = null;
    for (let attempt = 0; attempt < 30; attempt++) {
      await new Promise((r) => setTimeout(r, 100));
      try {
        const resp = await fetch(`http://127.0.0.1:${actualPort}/json/list`);
        if (resp.ok) {
          const tabs = await resp.json();
          if (Array.isArray(tabs) && tabs.length > 0) {
            const tab = tabs.find((t) => t.url && t.url.includes('index.html')) || tabs[0];
            if (tab && tab.webSocketDebuggerUrl) {
              wsUrl = tab.webSocketDebuggerUrl;
              break;
            }
          }
        }
      } catch (_) {}
    }

    if (!wsUrl) {
      throw new Error(`Failed to discover Chrome tab WebSocket URL on dynamic port ${actualPort}`);
    }

    this.ws = new WebSocket(wsUrl);
    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.id && this.pendingRequests.has(data.id)) {
          const { resolve, reject } = this.pendingRequests.get(data.id);
          this.pendingRequests.delete(data.id);
          if (data.error) reject(data.error);
          else resolve(data.result);
        }
      } catch (err) {
        console.error('CDP message parse error:', err);
      }
    };

    await new Promise((r) => (this.ws.onopen = r));
    await this.sendCdp('Runtime.enable');
    await this.sendCdp('Page.enable');

    // Wait 1.5s for page and WASM core initialization
    await new Promise((r) => setTimeout(r, 1500));

    // Inject Vision Detector directly into browser page context
    const visionCode = fs.readFileSync(path.join(__dirname, 'vision_detector.mjs'), 'utf-8');
    const sanitizedVisionCode = visionCode.replace(/export function/g, 'function');
    await this.eval(`
      window.__vision = (() => {
        ${sanitizedVisionCode}
        return { detectScanlineGaps, detectMemoryGarbage, detectBlackScreen, analyzeFrame };
      })();
    `);
  }

  sendCdp(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = this.msgId++;
      this.pendingRequests.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  async eval(expression) {
    const res = await this.sendCdp('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (res.exceptionDetails) {
      throw new Error(`Eval error: ${JSON.stringify(res.exceptionDetails)}`);
    }
    return res.result ? res.result.value : undefined;
  }

  async captureScreenshot() {
    const res = await this.sendCdp('Page.captureScreenshot', { format: 'png' });
    return Buffer.from(res.data, 'base64');
  }

  async close() {
    if (this.ws) {
      try {
        this.ws.close();
      } catch (_) {}
    }
    if (this.chromeProc) {
      this.chromeProc.kill();
    }
    if (this.server) {
      this.server.close();
    }
    if (this.tmpProfile && fs.existsSync(this.tmpProfile)) {
      try {
        fs.rmSync(this.tmpProfile, { recursive: true, force: true });
      } catch (_) {}
    }
  }

  /**
   * Runs the Smart Fuzzing Agent on a single game for the given duration.
   *
   * @param {object} gameConfig { id, name, jarPath, jadPath }
   * @param {number} durationSec
   * @returns {Promise<object>} Fuzzing session report with detected anomalies
   */
  async runGame(gameConfig, durationSec = this.durationSec) {
    const gameId = gameConfig.id || path.basename(gameConfig.jarPath, '.jar');
    const gameName = gameConfig.name || gameId;

    console.log(`\n🎮 [BugHunterAgent] Starting fuzzing session: ${gameName} (${durationSec}s)...`);

    // 1. Fresh page navigation per game for clean sandbox isolation
    let loadUrl = gameConfig.jarPath;
    if (!loadUrl.startsWith('http://') && !loadUrl.startsWith('/')) {
      loadUrl = `/` + loadUrl;
    } else if (loadUrl.startsWith(MARKET_DIR)) {
      loadUrl = '/market/' + path.relative(MARKET_DIR, loadUrl);
    }

    const targetPageUrl = `http://127.0.0.1:${this.port}/index.html?game=${encodeURIComponent(loadUrl)}`;
    await this.sendCdp('Page.navigate', { url: targetPageUrl });

    // Wait 1.5s for page reload and game fetch
    await new Promise((r) => setTimeout(r, 1500));

    // Inject Vision Detector into the refreshed page
    const visionCode = fs.readFileSync(path.join(__dirname, 'vision_detector.mjs'), 'utf-8');
    const sanitizedVisionCode = visionCode.replace(/export function/g, 'function');
    await this.eval(`
      window.__vision = (() => {
        ${sanitizedVisionCode}
        return { detectScanlineGaps, detectMemoryGarbage, detectBlackScreen, analyzeFrame };
      })();
    `);

    // Wait another 1.5s for MIDlet to unpack and initialize first view
    await new Promise((r) => setTimeout(r, 1500));

    // Fuzzing Key definitions
    // Key codes: FIRE (-5), SOFT1 (-6), SOFT2 (-7), UP (-1), DOWN (-2), LEFT (-3), RIGHT (-4), Digits 0-9 (48..57)
    const CORE_KEYS = [-5, -6, -7, -1, -2, -3, -4, 53, 49, 51, 55, 57, 48];

    const sessionStart = Date.now();
    const sessionDurationMs = durationSec * 1000;

    let previousFrameHash = 0;
    let lastFrameChangeTime = Date.now();
    let softlockDetected = false;
    let maskedExceptionDetected = false;
    let autoHealingCount = 0;
    let lastAutoHealingSig = '';
    const detectedAnomalies = new Set();
    const timelineLog = [];

    let loopCount = 0;
    let currentKeyHeld = null;

    while (Date.now() - sessionStart < sessionDurationMs) {
      loopCount++;
      const elapsedMs = Date.now() - sessionStart;

      // 1. State-Aware Key Press / Fuzzing
      // Cycle between action keys (FIRE/SOFT1 to skip titles/prompts) and exploratory movement
      let chosenKey;
      if (loopCount % 3 === 0) {
        chosenKey = -5; // FIRE (common confirmation)
      } else if (loopCount % 5 === 0) {
        chosenKey = -6; // SOFT 1 (Menu/Start)
      } else if (loopCount % 7 === 0) {
        chosenKey = -2; // DOWN (scroll options)
      } else {
        // Random key from core keypad
        chosenKey = CORE_KEYS[Math.floor(Math.random() * CORE_KEYS.length)];
      }

      // Send key down and simulate realistic release
      await this.eval(`
        if (window.sendKey) {
          window.sendKey(${chosenKey}, true);
        }
      `);
      currentKeyHeld = chosenKey;

      // Hold for 70ms
      await new Promise((r) => setTimeout(r, 70));

      await this.eval(`
        if (window.sendKey && ${currentKeyHeld !== null}) {
          window.sendKey(${currentKeyHeld}, false);
        }
      `);
      currentKeyHeld = null;

      // Wait remaining slice until 500ms interval
      await new Promise((r) => setTimeout(r, 430));

      // 2. Query Telemetry & In-Browser Frame Analysis
      const telem = await this.eval(`
        (() => {
          const w = window.wasm;
          if (!w) return null;
          const inst = Number(w.emu_get_instructions_executed ? w.emu_get_instructions_executed() : 0);
          const pub = w.emu_get_framebuffer_publications ? w.emu_get_framebuffer_publications() : 0;
          const uncaught = w.emu_get_uncaught_exceptions_count ? w.emu_get_uncaught_exceptions_count() : 0;
          const autoHealing = window.emu_get_auto_healing_count ? window.emu_get_auto_healing_count() : 0;
          const autoSig = window.emu_get_last_auto_healing_signature ? window.emu_get_last_auto_healing_signature() : "";
          
          let canvasW = 0, canvasH = 0;
          let curHash = 0;
          let visionRes = null;
          const cvs = window.canvas;
          if (cvs && cvs.width > 0 && cvs.height > 0) {
            canvasW = cvs.width;
            canvasH = cvs.height;
            const ctx = window.ctx;
            if (ctx) {
              const imgData = ctx.getImageData(0, 0, canvasW, canvasH);
              const data = imgData.data;

              // Fast 32-bit FNV-1a hash of pixels
              let h = 0x811c9dc5;
              for (let i = 0; i < data.length; i += 8) {
                h ^= data[i] | (data[i + 1] << 8) | (data[i + 2] << 16);
                h = Math.imul(h, 0x01000193);
              }
              curHash = h >>> 0;

              // Heuristic vision detector execution
              if (window.__vision && window.__vision.analyzeFrame) {
                visionRes = window.__vision.analyzeFrame(data, canvasW, canvasH, { elapsedMs: ${elapsedMs} });
              }
            }
          }

          return {
            instructions: inst,
            publishes: pub,
            uncaught,
            autoHealing,
            autoSig,
            canvasW,
            canvasH,
            curHash,
            visionRes,
          };
        })()
      `);

      if (!telem) continue;

      // Check Uncaught Crash
      if (telem.uncaught > 0) {
        detectedAnomalies.add('GUEST_EXCEPTION_CRASH');
      }

      // Check Auto-Healing Sniffer (WASM Telemetry Bridge)
      if (telem.autoHealing > 0) {
        maskedExceptionDetected = true;
        autoHealingCount = telem.autoHealing;
        lastAutoHealingSig = telem.autoSig;
        detectedAnomalies.add('MASKED_EXCEPTION');
      }

      // Check Frame Hash for Softlock / Deadlock
      if (telem.curHash !== 0) {
        if (telem.curHash !== previousFrameHash) {
          previousFrameHash = telem.curHash;
          lastFrameChangeTime = Date.now();
        } else {
          // If frame unchanged for > 5000ms while active clicking
          const stagnantMs = Date.now() - lastFrameChangeTime;
          if (stagnantMs >= 5000 && telem.instructions > 5000) {
            softlockDetected = true;
            detectedAnomalies.add('SOFTLOCK_DEADLOCK');
          }
        }
      }

      // 3. Vision detector anomalies
      if (telem.visionRes && telem.visionRes.artifacts) {
        for (const art of telem.visionRes.artifacts) {
          detectedAnomalies.add(art);
        }
      }

      timelineLog.push({
        elapsedMs,
        instructions: telem.instructions,
        publishes: telem.publishes,
        autoHealing: telem.autoHealing,
        anomalies: Array.from(detectedAnomalies),
      });

      // If a hard crash or extreme anomaly detected, record and continue
      if (detectedAnomalies.has('GUEST_EXCEPTION_CRASH')) {
        break;
      }
    }

    // Capture artifact screenshot and telemetry report
    const screenshotBuf = await this.captureScreenshot();
    const screenshotPath = path.join(EXPERIMENTS_DIR, `${gameId}_artifact.png`);
    fs.writeFileSync(screenshotPath, screenshotBuf);

    const report = {
      gameId,
      gameName,
      durationSec,
      anomalies: Array.from(detectedAnomalies),
      hasBugs: detectedAnomalies.size > 0,
      autoHealingCount,
      lastAutoHealingSig,
      softlockDetected,
      screenshotPath,
      summary: detectedAnomalies.size === 0 ? 'CLEAN_PASS' : Array.from(detectedAnomalies).join(', '),
      finalTelemetry: timelineLog[timelineLog.length - 1] || null,
    };

    const reportJsonPath = path.join(EXPERIMENTS_DIR, `${gameId}_telemetry.json`);
    fs.writeFileSync(reportJsonPath, JSON.stringify(report, null, 2));

    console.log(`   └─ Status: ${report.summary}`);
    if (maskedExceptionDetected) {
      console.log(`   └─ [Auto-Healing Sniffer] Healed: ${autoHealingCount}x | Signature: ${lastAutoHealingSig}`);
    }

    return report;
  }
}
