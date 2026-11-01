import http from 'http';
import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WEB_DIR = path.resolve(__dirname, '../web');
const PORT = 8089;
const CDP_PORT = 9224;

// 1. Simple HTTP static server
const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.wasm': 'application/wasm',
  '.jar': 'application/java-archive',
  '.png': 'image/png',
  '.css': 'text/css',
};

const server = http.createServer((req, res) => {
  let reqPath = req.url.split('?')[0];
  if (reqPath === '/') reqPath = '/test_runner.html';
  const filePath = path.join(WEB_DIR, reqPath);

  if (!fs.existsSync(filePath)) {
    res.writeHead(404);
    res.end('Not Found');
    return;
  }

  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';
  const stat = fs.statSync(filePath);

  res.writeHead(200, {
    'Content-Type': contentType,
    'Content-Length': stat.size,
    'Access-Control-Allow-Origin': '*',
  });
  fs.createReadStream(filePath).pipe(res);
});

server.listen(PORT, async () => {
  console.log(`[HTTP Server] Serving ${WEB_DIR} on http://127.0.0.1:${PORT}`);

  const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  const chromeArgs = [
    '--headless=new',
    `--remote-debugging-port=${CDP_PORT}`,
    '--user-data-dir=/tmp/j2me_chrome_test_profile',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--autoplay-policy=no-user-gesture-required',
    `http://127.0.0.1:${PORT}/test_runner.html`,
  ];

  console.log(`[Chrome] Launching: ${chromePath}`);
  const chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

  // Wait for CDP port to be open
  let wsUrl = null;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((r) => setTimeout(r, 200));
    try {
      const resp = await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`);
      if (resp.ok) {
        const tabs = await resp.json();
        const tab = tabs.find((t) => t.url.includes('test_runner.html')) || tabs[0];
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
    const data = JSON.parse(event.data);
    if (data.id && pendingRequests.has(data.id)) {
      const { resolve, reject } = pendingRequests.get(data.id);
      pendingRequests.delete(data.id);
      if (data.error) reject(data.error);
      else resolve(data.result);
    }
  };

  ws.onopen = async () => {
    console.log('[CDP] Attached! Polling test results from window.__TEST_RESULTS__...');
    await sendCdp('Runtime.enable');
    await sendCdp('Console.enable');

    ws.addEventListener('message', (event) => {
      try {
        const d = JSON.parse(event.data);
        if (d.method === 'Console.messageAdded') {
          console.log(`[Browser Console] ${d.params.message.text}`);
        } else if (d.method === 'Runtime.consoleAPICalled') {
          const text = d.params.args.map(a => a.value || a.description || '').join(' ');
          console.log(`[Browser Console] ${text}`);
        }
      } catch (_) {}
    });

    // Poll window.__TEST_RESULTS__
    let testResults = null;
    const maxPolls = 120; // 60 seconds max
    for (let poll = 0; poll < maxPolls; poll++) {
      await new Promise((r) => setTimeout(r, 500));
      const res = await sendCdp('Runtime.evaluate', {
        expression: 'JSON.stringify(window.__TEST_RESULTS__)',
        returnByValue: true,
      });

      if (res && res.result && res.result.value) {
        const parsed = JSON.parse(res.result.value);
        if (parsed.all_passed === true || parsed.error !== undefined) {
          testResults = parsed;
          break;
        }
      }
    }

    if (!testResults) {
      console.error('[ERROR] Timeout waiting for browser tests to complete');
      // Fetch logs from logEl
      const logRes = await sendCdp('Runtime.evaluate', {
        expression: 'document.getElementById("log") ? document.getElementById("log").textContent : "no log"',
        returnByValue: true,
      });
      console.log('[Browser DOM Log]:\n' + (logRes?.result?.value || ''));
    } else if (testResults.error) {
      console.error(`[ERROR in Browser Test]: ${testResults.error}`);
      const logRes = await sendCdp('Runtime.evaluate', {
        expression: 'document.getElementById("log") ? document.getElementById("log").textContent : "no log"',
        returnByValue: true,
      });
      console.log('[Browser DOM Log]:\n' + (logRes?.result?.value || ''));
    } else {
      console.log('\n============================================================');
      console.log(' BROWSER VERIFICATION REPORT (Google Chrome Headless)');
      console.log('============================================================');
      console.log(`User Agent:        ${testResults.browser_user_agent}`);
      console.log(`All Tests Passed:  ${testResults.all_passed}`);
      console.log(`WASM Load Time:    ${testResults.tests.wasm_load?.load_time_ms?.toFixed(2)} ms`);
      console.log(`WASM Memory Pages: ${testResults.tests.wasm_load?.initial_memory_pages}`);
      console.log(`Key Press Latency: ${testResults.tests.key_events?.press_latency_ms?.toFixed(3)} ms`);
      console.log(`Memory Cycles:     ${testResults.memory_leak_audit?.cycles_tested} cycles (growth: ${testResults.memory_leak_audit?.growth_pages} pages)`);
      console.log(`Benchmark (60s):   ${testResults.metrics?.throughput_frames_per_sec} fps throughput`);
      console.log(`Tick Duration:     Median: ${testResults.metrics?.median_tick_ms?.toFixed(3)} ms | p95: ${testResults.metrics?.p95_tick_ms?.toFixed(3)} ms`);
      console.log(`Audio Samples:     ${testResults.audio_metrics?.total_samples_captured} captured, ${testResults.audio_metrics?.non_zero_samples} non-zero (Peak: ${testResults.audio_metrics?.peak_amplitude})`);
      console.log('============================================================\n');

      const outDir = path.resolve(__dirname, '../catalog');
      const outPath = path.join(outDir, 'browser_test_results.json');
      fs.writeFileSync(outPath, JSON.stringify(testResults, null, 2));
      console.log(`[SUCCESS] Saved browser results to: ${outPath}`);
    }

    ws.close();
    chromeProc.kill();
    server.close();
    process.exit(testResults && testResults.all_passed ? 0 : 1);
  };
});
