import http from 'http';
import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WEB_DIR = path.resolve(__dirname, '../web');
const PORT = 8094;
const CDP_PORT = 9228;

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
  if (reqPath === '/') reqPath = '/test_runner_postcontract.html';
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
    '--user-data-dir=/tmp/j2me_chrome_postcontract_profile',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--autoplay-policy=no-user-gesture-required',
    `http://127.0.0.1:${PORT}/test_runner_postcontract.html`,
  ];

  console.log(`[Chrome] Launching: ${chromePath}`);
  const chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

  let wsUrl = null;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((r) => setTimeout(r, 200));
    try {
      const resp = await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`);
      if (resp.ok) {
        const tabs = await resp.json();
        const tab = tabs.find((t) => t.url.includes('test_runner_postcontract.html')) || tabs[0];
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

  let testResults = null;

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.id && pendingRequests.has(data.id)) {
        const { resolve, reject } = pendingRequests.get(data.id);
        pendingRequests.delete(data.id);
        if (data.error) reject(data.error);
        else resolve(data.result);
      }
      if (data.method === 'Console.messageAdded') {
        const text = data.params.message.text;
        if (text.startsWith('__RESULTS_JSON__')) {
          testResults = JSON.parse(text.substring('__RESULTS_JSON__'.length));
        } else {
          console.log(`[Browser Console] ${text}`);
        }
      } else if (data.method === 'Runtime.consoleAPICalled') {
        const text = data.params.args.map((a) => a.value || a.description || '').join(' ');
        if (text.startsWith('__RESULTS_JSON__')) {
          testResults = JSON.parse(text.substring('__RESULTS_JSON__'.length));
        } else {
          console.log(`[Browser Console] ${text}`);
        }
      }
    } catch (_) {}
  };

  ws.onopen = async () => {
    console.log('[CDP] Attached! Polling test results from window.__TEST_RESULTS__...');
    await sendCdp('Runtime.enable');
    await sendCdp('Console.enable');

    const maxPolls = 150;
    for (let poll = 0; poll < maxPolls; poll++) {
      if (testResults) break;
      await new Promise((r) => setTimeout(r, 1000));
      if (poll % 10 === 0) {
        console.log(`[Progress] Browser test running... elapsed ~${poll}s`);
      }
    }

    if (!testResults) {
      console.error('[ERROR] Timeout waiting for post-contract browser tests to complete');
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
      console.log(' POST-CONTRACT BROWSER VERIFICATION REPORT (Chrome Headless)');
      console.log('============================================================');
      console.log(`User Agent:           ${testResults.browser_user_agent}`);
      console.log(`All Tests Passed:     ${testResults.all_passed}`);
      console.log(`WASM Load Time:       ${testResults.tests.wasm_load?.load_time_ms?.toFixed(2)} ms`);
      console.log(`Memory Cycles:        ${testResults.memory_audit.cycles?.growth_pages} pages growth across 20 cycles`);
      console.log(`Test A Throughput:    ${testResults.test_a_throughput?.ticks_per_second} ticks/s (3600 frames in ${testResults.test_a_throughput?.wall_time_ms?.toFixed(1)} ms)`);
      console.log(`Test A VM Step:       Median=${testResults.test_a_throughput?.vm_step_median_ms?.toFixed(3)} ms | P95=${testResults.test_a_throughput?.vm_step_p95_ms?.toFixed(3)} ms`);
      console.log(`Test B Real-Time:     Wall=${testResults.test_b_realtime_bounce_tales?.wall_time_ms?.toFixed(1)} ms, Guest=${testResults.test_b_realtime_bounce_tales?.guest_time_ms?.toFixed(1)} ms (Ratio=${testResults.test_b_realtime_bounce_tales?.guest_wall_ratio?.toFixed(6)}, Drift=${testResults.test_b_realtime_bounce_tales?.time_drift_ms?.toFixed(2)} ms)`);
      console.log(`Test B Audio Window:  ${testResults.test_b_realtime_bounce_tales?.audio_window_samples} samples (60s @ 44100 Hz continuous stream)`);
      console.log(`Audio Ring Balance:   gen=${testResults.test_b_realtime_bounce_tales?.audio_generated_samples}, cons=${testResults.test_b_realtime_bounce_tales?.audio_consumed_samples}, drop=${testResults.test_b_realtime_bounce_tales?.audio_dropped_samples}`);
      console.log(`Audio Metrics:        DC Offset=${testResults.test_b_realtime_bounce_tales?.dc_offset?.toFixed(6)}, RMS=${testResults.test_b_realtime_bounce_tales?.rms?.toFixed(6)}, Peak=${testResults.test_b_realtime_bounce_tales?.peak?.toFixed(6)}`);
      console.log(`GC & Heap Audit:      Before GC=${testResults.test_b_realtime_bounce_tales?.live_objects_before_gc}, Freed=${testResults.test_b_realtime_bounce_tales?.freed_objects_by_gc}, After GC=${testResults.test_b_realtime_bounce_tales?.live_objects_after_gc}`);
      console.log(`48000 Hz Support:     Supported=${testResults.test_b_audio_48khz_check?.supported}, Samples=${testResults.test_b_audio_48khz_check?.generated_samples}`);
      console.log(`Test B Rev2:          Throughput=${testResults.test_b_realtime_rev2?.throughput_ticks_per_second} ticks/s (${testResults.test_b_realtime_rev2?.nominal_ticks} ticks in ${testResults.test_b_realtime_rev2?.unthrottled_wall_time_ms?.toFixed(1)} ms), threads=${testResults.test_b_realtime_rev2?.active_threads}`);
      console.log(`Safari Status:        ${testResults.safari_status} (${testResults.safari_automation_protocol})`);
      console.log('============================================================\n');

      const outDir = path.resolve(__dirname, '../experiments/exp_phase42_postcontract');
      fs.mkdirSync(outDir, { recursive: true });
      const outPath = path.join(outDir, 'browser_realtime_results.json');
      fs.writeFileSync(outPath, JSON.stringify(testResults, null, 2));
      console.log(`[SUCCESS] Saved post-contract browser results to: ${outPath}`);
    }

    ws.close();
    chromeProc.kill();
    server.close();
    process.exit(testResults && testResults.all_passed ? 0 : 1);
  };
});
