import http from 'http';
import fs from 'fs';
import path from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WEB_DIR = path.resolve(__dirname, '../web');
const PORT = 8095;
const CDP_PORT = 9229;

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
  if (reqPath === '/') reqPath = '/test_runner_phase43.html';
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
    '--user-data-dir=/tmp/j2me_chrome_phase43_profile',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--autoplay-policy=no-user-gesture-required',
    `http://127.0.0.1:${PORT}/test_runner_phase43.html`,
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
        const tab = tabs.find((t) => t.url.includes('test_runner_phase43.html')) || tabs[0];
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

    const maxPolls = 180;
    for (let poll = 0; poll < maxPolls; poll++) {
      if (testResults) break;
      await new Promise((r) => setTimeout(r, 1000));
      if (poll % 10 === 0) {
        console.log(`[Progress] Browser test running... elapsed ~${poll}s`);
      }
    }

    if (!testResults) {
      console.error('[ERROR] Timeout waiting for Phase 4.3 browser tests to complete');
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
      console.log(' PHASE 4.3 BROWSER VERIFICATION REPORT (Chrome Headless)');
      console.log('============================================================');
      console.log(`User Agent:           ${testResults.browser_user_agent}`);
      console.log(`All Tests Passed:     ${testResults.all_passed}`);
      console.log(`WASM Load Time:       ${testResults.tests.wasm_load?.load_time_ms?.toFixed(2)} ms`);
      console.log(`Memory Cycles:        ${testResults.memory_audit.cycles?.growth_pages} pages growth across 20 cycles`);
      console.log(`Test A Throughput:    ${testResults.test_a_throughput?.ticks_per_second} ticks/s (${testResults.test_a_throughput?.instructions_executed} instrs)`);
      console.log(`Test A Step Time:     Median=${testResults.test_a_throughput?.vm_step_median_ms?.toFixed(3)} ms | P95=${testResults.test_a_throughput?.vm_step_p95_ms?.toFixed(3)} ms | P99=${testResults.test_a_throughput?.vm_step_p99_ms?.toFixed(3)} ms`);
      console.log(`Test B Real-Time:     Wall=${testResults.test_b_realtime_bounce_tales?.wall_time_ms?.toFixed(1)} ms, Guest=${testResults.test_b_realtime_bounce_tales?.guest_time_ms?.toFixed(1)} ms (Ratio=${testResults.test_b_realtime_bounce_tales?.guest_wall_ratio?.toFixed(6)}, Drift=${testResults.test_b_realtime_bounce_tales?.time_drift_ms?.toFixed(2)} ms)`);
      console.log(`Test B Step Time:     Median=${testResults.test_b_realtime_bounce_tales?.vm_step_median_ms?.toFixed(3)} ms | P95=${testResults.test_b_realtime_bounce_tales?.vm_step_p95_ms?.toFixed(3)} ms | P99=${testResults.test_b_realtime_bounce_tales?.vm_step_p99_ms?.toFixed(3)} ms`);
      console.log(`Test B rAF Interval:  Median=${testResults.test_b_realtime_bounce_tales?.raf_median_ms?.toFixed(2)} ms | P95=${testResults.test_b_realtime_bounce_tales?.raf_p95_ms?.toFixed(2)} ms | P99=${testResults.test_b_realtime_bounce_tales?.raf_p99_ms?.toFixed(2)} ms`);
      console.log(`Render Submissions:   ${testResults.test_b_realtime_bounce_tales?.render_submissions} / ${testResults.test_b_realtime_bounce_tales?.total_frames} frames`);
      console.log(`GC & Heap Audit:      Before GC=${testResults.test_b_realtime_bounce_tales?.live_objects_before_gc}, Freed=${testResults.test_b_realtime_bounce_tales?.freed_objects_by_gc}, After GC=${testResults.test_b_realtime_bounce_tales?.live_objects_after_gc}`);
      console.log(`Audio Breakdown:      Quantum=${testResults.audio_telemetry_breakdown?.audioworklet_quantum_samples} samples | Queries=${testResults.audio_telemetry_breakdown?.ring_buffer_queries_total} | Silence=${testResults.audio_telemetry_breakdown?.intentional_silence_duration_seconds}s (${testResults.audio_telemetry_breakdown?.intentional_silence_missing_samples} samples) | Active=${testResults.audio_telemetry_breakdown?.active_playback_duration_seconds}s | Starvation Underruns=${testResults.audio_telemetry_breakdown?.active_playback_underruns}`);
      console.log(`Audio Metrics:        DC Offset=${testResults.audio_telemetry_breakdown?.dc_offset?.toFixed(6)}, RMS=${testResults.audio_telemetry_breakdown?.rms?.toFixed(6)}, Peak=${testResults.audio_telemetry_breakdown?.peak?.toFixed(6)}`);
      console.log(`Test C Spyro:         Paint Calls=${testResults.test_c_realtime_spyro?.paint_calls}, FB Publishes=${testResults.test_c_realtime_spyro?.framebuffer_publications}, Threads=${testResults.test_c_realtime_spyro?.active_threads}, Active Rendering Verified=${testResults.test_c_realtime_spyro?.active_rendering_verified}`);
      console.log(`48000 Hz Status:      ${testResults.audio_48khz_status}`);
      console.log(`Safari Status:        ${testResults.safari_status}`);
      console.log('============================================================\n');

      const outDir = path.resolve(__dirname, '../experiments/exp_phase43_active_rendering');
      fs.mkdirSync(outDir, { recursive: true });
      const outPath = path.join(outDir, 'browser_realtime_results.json');
      fs.writeFileSync(outPath, JSON.stringify(testResults, null, 2));
      console.log(`[SUCCESS] Saved Phase 4.3 browser results to: ${outPath}`);
    }

    ws.close();
    chromeProc.kill();
    server.close();
    process.exit(testResults && testResults.all_passed ? 0 : 1);
  };
});
