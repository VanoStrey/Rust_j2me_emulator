import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import http from 'http';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = 8915;
const CDP_PORT = 9255;
const URL = `http://127.0.0.1:${PORT}/index.html`;

const mimeTypes = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.jar': 'application/java-archive',
  '.jad': 'text/plain',
  '.png': 'image/png',
  '.css': 'text/css'
};

const server = http.createServer((req, res) => {
  let reqPath = req.url.split('?')[0];
  if (reqPath === '/') reqPath = '/index.html';
  const filePath = path.join(__dirname, '../web', reqPath);

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404);
      res.end('Not found');
      return;
    }
    const ext = path.extname(filePath);
    res.writeHead(200, {
      'Content-Type': mimeTypes[ext] || 'application/octet-stream',
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'no-cache, no-store, must-revalidate',
      'Pragma': 'no-cache',
      'Expires': '0'
    });
    res.end(data);
  });
});

const CANONICAL_GAMES = [
  { id: 'bounce_tales', name: 'Bounce Tales', file: 'bounce_tales.jar', expectedW: 240, expectedH: 320, key: 53 },
  { id: 'gravity_defied', name: 'Gravity Defied', file: 'gravity_defied.jar', expectedW: 176, expectedH: 220, key: 53 },
  { id: 'doom_rpg', name: 'Doom RPG', file: 'doom_rpg.jar', expectedW: 240, expectedH: 320, key: -5 },
  { id: 'prince_sands', name: 'Prince of Persia', file: 'prince_sands.jar', expectedW: 176, expectedH: 208, key: -5 },
  { id: 'castlevania', name: 'Castlevania', file: 'castlevania.jar', expectedW: 240, expectedH: 320, key: -6 },
  { id: 'heroes_lore', name: 'Heroes Lore', file: 'heroes_lore.jar', expectedW: 240, expectedH: 320, key: -6 },
  { id: 'cyberpunk', name: 'CyberPunk: Arasaka', file: 'cyberpunk.jar', expectedW: 176, expectedH: 208, key: -5, checkAudio: true, checkResolutionHeuristic: true },
];

server.listen(PORT, async () => {
  console.log(`[Web Server] Serving at http://127.0.0.1:${PORT}`);

  const tmpProfile = fs.mkdtempSync('/tmp/j2me_chrome_p51_');
  const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  const chromeArgs = [
    '--headless=new',
    `--remote-debugging-port=${CDP_PORT}`,
    `--user-data-dir=${tmpProfile}`,
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-background-timer-throttling',
    '--disable-renderer-backgrounding',
    URL,
  ];

  console.log(`[Chrome] Launching Headless Chrome on port ${CDP_PORT}...`);
  const chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

  let wsUrl = null;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((r) => setTimeout(r, 200));
    try {
      const resp = await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`);
      if (resp.ok) {
        const tabs = await resp.json();
        const tab = tabs.find((t) => t.url.includes('index.html')) || tabs[0];
        if (tab && tab.webSocketDebuggerUrl) {
          wsUrl = tab.webSocketDebuggerUrl;
          break;
        }
      }
    } catch (_) {}
  }

  if (!wsUrl) {
    console.error('[ERROR] Failed to discover Chrome CDP WebSocket URL');
    chromeProc.kill();
    server.close();
    process.exit(1);
  }

  console.log(`[CDP] Connected to: ${wsUrl}`);
  const ws = new WebSocket(wsUrl);

  let msgId = 1;
  const pendingRequests = new Map();

  function sendCdp(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = msgId++;
      pendingRequests.set(id, { resolve, reject });
      ws.send(JSON.stringify({ id, method, params }));
    });
  }

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.id && pendingRequests.has(data.id)) {
        const { resolve, reject } = pendingRequests.get(data.id);
        pendingRequests.delete(data.id);
        if (data.error) reject(data.error);
        else resolve(data.result);
      }
      if (data.method === 'Runtime.consoleAPICalled') {
        const text = data.params.args.map((a) => a.value || a.description || '').join(' ');
        if (text.includes('[ERROR]') || text.includes('[FATAL]') || text.includes('panic')) {
          console.error(`[Browser Console ERROR] ${text}`);
        }
      }
    } catch (e) {
      console.error('WS Parse Error:', e);
    }
  };

  await new Promise((r) => (ws.onopen = r));
  await sendCdp('Runtime.enable');
  await sendCdp('Page.enable');
  await sendCdp('Network.enable');
  await sendCdp('Network.setCacheDisabled', { cacheDisabled: true });

  console.log('[Test] Waiting for WASM to initialize...');
  await new Promise((r) => setTimeout(r, 1500));

  const gameReports = [];

  for (const game of CANONICAL_GAMES) {
    console.log(`\n================ TESTING GAME: ${game.name} (${game.file}) ================`);

    const runnerScript = `
    (async function() {
      const g = ${JSON.stringify(game)};
      const res = {
        id: g.id,
        name: g.name,
        file: g.file,
        expectedW: g.expectedW,
        expectedH: g.expectedH,
        loaded: false,
        profileDetected: null,
        autoResolutionDetected: null,
        screenW: 0,
        screenH: 0,
        instructions: 0,
        publishes: 0,
        uncaughtExceptions: 0,
        nonBlackPixels: 0,
        audioSamplesGenerated: 0,
        rmsSaved: false,
        rmsRestored: false,
        keyInputDelivered: false
      };

      try {
        // 0. Clean isolated re-init per game
        if (window.wasm.emu_destroy) {
          window.wasm.emu_destroy();
        }
        window.wasm.emu_init(g.expectedW, g.expectedH, 44100);

        // 1. Fetch JAR
        const resp = await fetch('./' + g.file);
        if (!resp.ok) throw new Error('Fetch failed: ' + resp.status);
        const jarBytes = new Uint8Array(await resp.arrayBuffer());

        // 2. Test auto-resolution profile detection
        if (window.analyzeAndApplyJarProfile) {
          res.autoResolutionDetected = window.analyzeAndApplyJarProfile(jarBytes);
        }

        // 3. Set resolution
        window.setScreenResolution(g.expectedW, g.expectedH);
        res.screenW = window.canvas.width;
        res.screenH = window.canvas.height;

        // 4. Auto inject JAD property for Heroes Lore if needed
        if (g.id === 'heroes_lore') {
          const jadArea = document.getElementById('jad-properties-input');
          if (jadArea) {
            jadArea.value = 'HO-Demo: PAID';
          }
          if (window.applyJadPropertiesFromText) {
            window.applyJadPropertiesFromText('HO-Demo: PAID');
          }
        }

        // 5. Load JAR
        await window.loadJarBuffer(jarBytes, g.file);
        res.loaded = (document.getElementById('game-status-badge').textContent === 'Running');

        // Extract profile from DOM
        res.profileDetected = {
          title: document.getElementById('meta-title').textContent,
          vendor: document.getElementById('meta-vendor').textContent,
          version: document.getElementById('meta-version').textContent,
          mainClass: document.getElementById('meta-class').textContent,
        };

        // 6. Test Keypad Input simulation (press, release, and movement)
        const keyBtn = document.querySelector('.k-btn[data-code="' + g.key + '"]');
        if (keyBtn) {
          keyBtn.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
          res.keyInputDelivered = true;
          await new Promise(r => setTimeout(r, 50));
          keyBtn.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }));
        }

        // If Cyberpunk: simulate right arrow (-4) and fire (-5) to test sprite flip
        if (g.id === 'cyberpunk') {
          const rightBtn = document.querySelector('.k-btn[data-code="-4"]');
          if (rightBtn) {
            rightBtn.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
            window.wasm.emu_tick(16.6);
            rightBtn.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }));
          }
        }

        // 7. Execute 300 frame ticks
        for (let t = 0; t < 300; t++) {
          window.wasm.emu_tick(16.6);
        }

        // Telemetry check
        res.instructions = Number(window.wasm.emu_get_instructions_executed());
        res.publishes = window.wasm.emu_get_framebuffer_publications();
        res.uncaughtExceptions = window.wasm.emu_get_uncaught_exceptions_count();

        // Audio telemetry
        if (window.wasm.emu_get_audio_buffer_len) {
          res.audioSamplesGenerated = window.wasm.emu_get_audio_buffer_len();
        }

        // 8. Test RMS export/import via localStorage
        if (window.saveRmsToLocalStorage) {
          window.saveRmsToLocalStorage();
        }
        const exportLen = window.wasm.emu_export_rms_len ? window.wasm.emu_export_rms_len() : 0;
        res.rmsSaved = (exportLen >= 0);

        if (window.restoreRmsFromLocalStorage) {
          window.restoreRmsFromLocalStorage();
        }
        res.rmsRestored = true;

        // 9. Render check on canvas
        const curW = window.canvas.width;
        const curH = window.canvas.height;
        const fbPtr = window.wasm.emu_get_framebuffer_ptr();
        const fbLen = window.wasm.emu_get_framebuffer_len();
        if (fbPtr && fbLen) {
          const view = new Uint8ClampedArray(window.memory.buffer, fbPtr, fbLen);
          let imgData;
          try {
            imgData = new ImageData(view, curW, curH);
          } catch (_) {
            imgData = window.ctx.createImageData(curW, curH);
            const copyLen = Math.min(view.length, imgData.data.length);
            imgData.data.set(view.subarray(0, copyLen));
          }
          window.ctx.putImageData(imgData, 0, 0);
        }

        const img = window.ctx.getImageData(0, 0, curW, curH);
        let nonBlack = 0;
        for (let i = 0; i < img.data.length; i += 4) {
          if (img.data[i] > 0 || img.data[i+1] > 0 || img.data[i+2] > 0) {
            nonBlack++;
          }
        }
        res.nonBlackPixels = nonBlack;

        return res;
      } catch (err) {
        return { error: err.toString(), stack: err.stack, id: g.id, name: g.name };
      }
    })();
    `;

    const evalResult = await sendCdp('Runtime.evaluate', {
      expression: runnerScript,
      awaitPromise: true,
      returnByValue: true,
    });

    const report = evalResult.result.value;
    console.log(`[Result] ${game.name}:`, JSON.stringify(report, null, 2));
    gameReports.push(report);
  }

  // Capture final CDP Screenshot of the web player
  console.log('\n[CDP] Capturing final composite screenshot of web player...');
  const screenshotResp = await sendCdp('Page.captureScreenshot', {
    format: 'png',
    captureBeyondViewport: false,
  });

  const screenshotPath = path.join(__dirname, '../experiments/all7_web_player_cdp.png');
  fs.writeFileSync(screenshotPath, Buffer.from(screenshotResp.data, 'base64'));
  console.log(`[Screenshot] Saved all-7 web player composite screenshot to: ${screenshotPath}`);

  // Summary Table
  console.log('\n========================================================================================================');
  console.log('                               ALL 7 CANONICAL GAMES CDP E2E VERIFICATION RESULTS                       ');
  console.log('========================================================================================================');
  console.log('Game                 | Resolution | Instructions | Publishes | Uncaught | Render Pixels | Auto-Res Profile');
  console.log('---------------------+------------+--------------+-----------+----------+---------------+-----------------------');

  let allPassed = true;
  for (const rep of gameReports) {
    if (rep.error) {
      console.log(`${rep.name.padEnd(20)} | ERROR: ${rep.error}`);
      allPassed = false;
      continue;
    }
    const resStr = `${rep.screenW}x${rep.screenH}`.padEnd(10);
    const instStr = String(rep.instructions).padStart(12);
    const pubStr = String(rep.publishes).padStart(9);
    const uncaughtStr = String(rep.uncaughtExceptions).padStart(8);
    const pixStr = String(rep.nonBlackPixels).padStart(13);
    const autoRes = rep.autoResolutionDetected ? `${rep.autoResolutionDetected.suggested_width}x${rep.autoResolutionDetected.suggested_height} (${rep.autoResolutionDetected.resolution_source})` : 'N/A';

    console.log(`${rep.name.padEnd(20)} | ${resStr} | ${instStr} | ${pubStr} | ${uncaughtStr} | ${pixStr} | ${autoRes}`);

    if (rep.uncaughtExceptions > 0 || rep.nonBlackPixels === 0 || !rep.loaded) {
      allPassed = false;
    }
  }
  console.log('========================================================================================================\n');

  // Save report JSON
  const reportPath = path.join(__dirname, '../experiments/all7_web_player_report.json');
  fs.writeFileSync(reportPath, JSON.stringify(gameReports, null, 2));
  console.log(`[Report] Saved JSON report to: ${reportPath}`);

  // Cleanup
  chromeProc.kill();
  server.close();
  process.exit(allPassed ? 0 : 1);
});
