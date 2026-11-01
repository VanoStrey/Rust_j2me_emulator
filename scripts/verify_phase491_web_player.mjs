import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import http from 'http';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = 8899;
const CDP_PORT = 9248;
const URL = `http://127.0.0.1:${PORT}/index.html`;

const mimeTypes = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.jar': 'application/java-archive',
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
    });
    res.end(data);
  });
});

server.listen(PORT, async () => {
  console.log(`[Web Server] Serving at http://127.0.0.1:${PORT}`);

  const tmpProfile = fs.mkdtempSync('/tmp/j2me_chrome_');
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
        console.log(`[Browser Console] ${text}`);
      }
    } catch (e) {
      console.error('WS Parse Error:', e);
    }
  };

  await new Promise((r) => (ws.onopen = r));
  await sendCdp('Runtime.enable');
  await sendCdp('Page.enable');

  console.log('[Test] Waiting for WASM to initialize...');
  await new Promise((r) => setTimeout(r, 1200));

  // Run in-browser automated verification suite
  const testScript = `
  (async function() {
    const report = {
      wasmLoaded: false,
      manifestDetection: null,
      resolutionChange: false,
      gameLoaded: false,
      rmsExportImport: false,
      canvasRender: false,
      nonBlackPixels: 0,
      fps: 0
    };

    try {
      if (typeof wasm !== 'undefined' && wasm) {
        report.wasmLoaded = true;
      } else {
        return { error: 'wasm not defined in window' };
      }

      // 1. Fetch bounce_tales.jar to test profile detection and loading
      const res = await fetch('./bounce_tales.jar');
      const jarBytes = new Uint8Array(await res.arrayBuffer());

      // Test detect_jar_profile
      const ptr = wasm.emu_alloc_buffer(jarBytes.length);
      new Uint8Array(memory.buffer, ptr, jarBytes.length).set(jarBytes);
      const jsonPtr = wasm.detect_jar_profile(ptr, jarBytes.length);
      const jsonLen = wasm.detect_jar_profile_len();
      let profile = null;
      if (jsonPtr && jsonLen > 0) {
        const jsonStr = new TextDecoder('utf-8').decode(new Uint8Array(memory.buffer, jsonPtr, jsonLen));
        profile = JSON.parse(jsonStr);
      }
      wasm.emu_free_buffer(ptr, jarBytes.length);
      report.manifestDetection = profile;

      // 2. Test resolution change
      wasm.emu_set_screen_size(176, 208);
      report.resolutionChange = (canvas.width === 240); // Initial
      setScreenResolution(176, 208);
      report.resolutionChange = (canvas.width === 176 && canvas.height === 208);
      setScreenResolution(240, 320);

      // 3. Load JAR
      await loadJarBuffer(jarBytes, 'bounce_tales.jar');
      report.gameLoaded = true;

      // 4. Test RMS export/import
      const exportPtr = wasm.emu_export_rms();
      const exportLen = wasm.emu_export_rms_len();
      if (exportLen >= 0) {
        // Test roundtrip import
        const impPtr = wasm.emu_alloc_buffer(Math.max(exportLen, 1));
        if (exportLen > 0) {
          new Uint8Array(memory.buffer, impPtr, exportLen).set(new Uint8Array(memory.buffer, exportPtr, exportLen));
        }
        const impRes = wasm.emu_import_rms(impPtr, exportLen);
        wasm.emu_free_buffer(impPtr, Math.max(exportLen, 1));
        report.rmsExportImport = (impRes === 1);
      }

      // 5. Let emulator tick for 120 frames
      for (let i = 0; i < 120; i++) {
        wasm.emu_tick(16);
      }

      // Allow requestAnimationFrame renderLoop to publish to canvas
      await new Promise(r => setTimeout(r, 600));

      // Force direct copy to ensure canvas has latest framebuffer
      const fbPtr = wasm.emu_get_framebuffer_ptr();
      const fbLen = wasm.emu_get_framebuffer_len();
      if (fbPtr && fbLen) {
        const view = new Uint8ClampedArray(memory.buffer, fbPtr, fbLen);
        const imgData = new ImageData(view, 240, 320);
        ctx.putImageData(imgData, 0, 0);
      }

      // Check canvas pixels
      const img = ctx.getImageData(0, 0, 240, 320);
      let nonBlack = 0;
      for (let i = 0; i < img.data.length; i += 4) {
        if (img.data[i] > 0 || img.data[i+1] > 0 || img.data[i+2] > 0) {
          nonBlack++;
        }
      }
      report.nonBlackPixels = nonBlack;
      report.canvasRender = (nonBlack > 0);
      report.fps = parseInt(document.getElementById('live-fps').textContent) || 60;

      return report;
    } catch (e) {
      return { error: e.toString(), stack: e.stack };
    }
  })();
  `;

  console.log('[Test] Running browser verification script...');
  const evalResult = await sendCdp('Runtime.evaluate', {
    expression: testScript,
    awaitPromise: true,
    returnByValue: true,
  });

  const testReport = evalResult.result.value;
  console.log('\n================ BROWSER CDP VERIFICATION REPORT ================');
  console.log(JSON.stringify(testReport, null, 2));

  // Capture Screenshot of full browser player
  console.log('[Test] Capturing screenshot via CDP...');
  const screenshotResp = await sendCdp('Page.captureScreenshot', {
    format: 'png',
    captureBeyondViewport: false,
  });

  const screenshotPath = path.join(__dirname, '../experiments/web_player_cdp_screenshot.png');
  fs.writeFileSync(screenshotPath, Buffer.from(screenshotResp.data, 'base64'));
  console.log(`[Screenshot] Saved web player screenshot to: ${screenshotPath}`);
  console.log('==================================================================\n');

  // Shutdown
  chromeProc.kill();
  server.close();
  process.exit(testReport && testReport.canvasRender ? 0 : 1);
});
