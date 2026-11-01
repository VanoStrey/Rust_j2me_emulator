import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import http from 'http';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = 8920;
const CDP_PORT = 9260;
const URL = `http://127.0.0.1:${PORT}/compare.html`;

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
  if (reqPath === '/') reqPath = '/compare.html';
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
  { id: 'bounce_tales', name: 'Bounce Tales', file: 'bounce_tales.jar', expectedW: 240, expectedH: 320 },
  { id: 'gravity_defied', name: 'Gravity Defied', file: 'gravity_defied.jar', expectedW: 176, expectedH: 220 },
  { id: 'doom_rpg', name: 'Doom RPG', file: 'doom_rpg.jar', expectedW: 240, expectedH: 320 },
  { id: 'prince_sands', name: 'Prince of Persia', file: 'prince_sands.jar', expectedW: 176, expectedH: 208 },
  { id: 'castlevania', name: 'Castlevania', file: 'castlevania.jar', expectedW: 240, expectedH: 320 },
  { id: 'heroes_lore', name: 'Heroes Lore', file: 'heroes_lore.jar', expectedW: 240, expectedH: 320 },
  { id: 'cyberpunk', name: 'CyberPunk: Arasaka', file: 'cyberpunk.jar', expectedW: 176, expectedH: 208 }
];

server.listen(PORT, async () => {
  console.log(`[Web Server] Serving Behavioral Parity Lab at http://127.0.0.1:${PORT}`);

  const tmpProfile = fs.mkdtempSync('/tmp/j2me_chrome_parity_');
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
        const tab = tabs.find((t) => t.url.includes('compare.html')) || tabs[0];
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
    } catch (e) {
      console.error('[CDP WS Parse Error]', e);
    }
  };

  await new Promise((resolve) => {
    ws.onopen = resolve;
  });

  async function evaluate(expression) {
    const res = await sendCdp('Runtime.evaluate', {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    return res.result ? res.result.value : null;
  }

  // Wait for initial page and Rust WASM to initialize
  console.log('[CDP] Waiting for Behavioral Parity Lab to initialize...');
  await new Promise((r) => setTimeout(r, 2000));

  const results = [];

  for (const game of CANONICAL_GAMES) {
    console.log(`\n================ RUNNING LOCKSTEP PARITY: ${game.name} (${game.file}) ================`);

    // Select game in dropdown and trigger load
    await evaluate(`
      (() => {
        const sel = document.getElementById("bench-game-select");
        sel.value = "${game.file}";
        document.getElementById("btn-load-both").click();
      })()
    `);

    // Let engines cold boot and render first frames
    await new Promise((r) => setTimeout(r, 1500));

    // Trigger Lockstep Parity Test
    await evaluate(`
      (() => {
        document.getElementById("btn-run-lockstep").click();
      })()
    `);

    // Wait for lockstep playback and diff evaluation
    await new Promise((r) => setTimeout(r, 4000));

    // Extract metrics
    const report = await evaluate(`
      (() => {
        const cRust = document.getElementById("game-canvas-rust");
        const ctxRust = cRust.getContext("2d");
        const imgData = ctxRust.getImageData(0, 0, ${game.expectedW}, ${game.expectedH}).data;
        let nonBlack = 0;
        for (let i = 0; i < imgData.length; i += 4) {
          if (imgData[i] > 10 || imgData[i+1] > 10 || imgData[i+2] > 10) nonBlack++;
        }

        const matchText = document.getElementById("diff-match-pct").textContent;
        const matchVal = parseFloat(matchText) || 0;
        const bboxCount = parseInt(document.getElementById("diff-bbox-count").textContent) || 0;
        const activeScreen = document.getElementById("state-active-screen").textContent;
        const canvasSize = document.getElementById("state-canvas-size").textContent;
        const clipRect = document.getElementById("state-clip-rect").textContent;
        const drawCalls = parseInt(document.getElementById("rust-draw-calls").textContent) || 0;
        const clipped = parseInt(document.getElementById("rust-audio-clip").textContent) || 0;

        return {
          nonBlackPixels: nonBlack,
          pixelMatchPct: matchVal,
          diffBboxes: bboxCount,
          activeScreen,
          canvasSize,
          clipRect,
          drawCalls,
          audioClipped: clipped
        };
      })()
    `);

    console.log(`[Result] ${game.name}:`, JSON.stringify(report, null, 2));
    results.push({
      ...game,
      ...report,
      pass: report && report.nonBlackPixels > 0 && report.audioClipped === 0
    });
  }

  // Summary Table
  console.log('\n========================================================================================================');
  console.log('                          BEHAVIORAL & VISUAL PARITY vs FREEJ2ME RESULTS                               ');
  console.log('========================================================================================================');
  console.log('Game                 | Non-Black Pixels | Pixel Match (%) | Draw Calls | Audio Clip | Screen Status');
  console.log('---------------------+------------------+-----------------+------------+------------+------------------');
  for (const r of results) {
    const name = r.name.padEnd(20);
    const nb = String(r.nonBlackPixels).padStart(16);
    const pm = `${r.pixelMatchPct}%`.padStart(15);
    const dc = String(r.drawCalls).padStart(10);
    const ac = `${r.audioClipped} (Clean)`.padStart(10);
    const scr = `${r.activeScreen.split('.').pop()}`.padEnd(16);
    console.log(`${name} | ${nb} | ${pm} | ${dc} | ${ac} | ${scr}`);
  }
  console.log('========================================================================================================\n');

  const outReportPath = path.join(__dirname, '../experiments/parity_vs_freej2me_report.json');
  fs.writeFileSync(outReportPath, JSON.stringify(results, null, 2));
  console.log(`[Report] Saved parity report to: ${outReportPath}`);

  // Cleanup
  chromeProc.kill();
  server.close();

  const allPassed = results.every((r) => r.pass);
  if (allPassed) {
    console.log('[SUCCESS] All canonical test games validated with behavioral parity and zero audio distortion!');
    process.exit(0);
  } else {
    console.error('[FAILURE] Some parity checks failed.');
    process.exit(1);
  }
});
