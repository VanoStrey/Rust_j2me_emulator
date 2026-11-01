import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CDP_PORT = 9230;
const URL = 'http://127.0.0.1:8899/test_runner_phase46.html';

const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const chromeArgs = [
  '--headless=new',
  `--remote-debugging-port=${CDP_PORT}`,
  '--user-data-dir=/tmp/j2me_chrome_phase46_profile',
  '--disable-gpu',
  '--no-first-run',
  '--no-default-browser-check',
  '--disable-background-timer-throttling',
  '--disable-renderer-backgrounding',
  '--autoplay-policy=no-user-gesture-required',
  URL,
];

console.log(`[Chrome] Launching Headless Chrome: ${chromePath}`);
const chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

async function main() {
  let wsUrl = null;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((r) => setTimeout(r, 200));
    try {
      const resp = await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`);
      if (resp.ok) {
        const tabs = await resp.json();
        const tab = tabs.find((t) => t.url.includes('test_runner_phase46.html')) || tabs[0];
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
    console.log('[CDP] Attached! Enabling Runtime, Console, Page...');
    await sendCdp('Runtime.enable');
    await sendCdp('Console.enable');
    await sendCdp('Page.enable');
    console.log('[CDP] Triggering fresh page reload...');
    await sendCdp('Page.reload', { ignoreCache: true });

    // 2 benchmarks * 60 seconds + load/lifecycle = ~130s. Allow up to 360 seconds.
    const maxPolls = 360;
    for (let poll = 0; poll < maxPolls; poll++) {
      if (testResults) break;
      await new Promise((r) => setTimeout(r, 1000));
      if (poll % 15 === 0 && poll > 0) {
        console.log(`[Progress] Headless Chrome test running... elapsed ~${poll}s`);
      }
    }

    if (!testResults) {
      console.error('[ERROR] Timeout waiting for Phase 4.6 browser tests to complete');
      const logRes = await sendCdp('Runtime.evaluate', {
        expression: 'document.getElementById("log") ? document.getElementById("log").textContent : "no log"',
        returnByValue: true,
      });
      console.log('[Browser DOM Log]:\n' + (logRes?.result?.value || ''));
    } else if (testResults.error) {
      console.error(`[ERROR in Browser Test]: ${testResults.error}`);
    } else {
      console.log('\n============================================================');
      console.log(' PHASE 4.6 BROWSER REAL-TIME VERIFICATION REPORT');
      console.log('============================================================');
      console.log(`User Agent:              ${testResults.user_agent}`);
      console.log(`All Tests Passed:        ${testResults.all_passed}`);
      console.log(`Initial Memory Pages:    ${testResults.memory_audit.initial_memory_pages}`);
      console.log(`Peak Memory Pages:       ${testResults.lifecycle_test.wasm_peak_pages} (${testResults.lifecycle_test.wasm_peak_memory_mb.toFixed(2)} MB)`);
      console.log(`Bounce Tales (60s):      Wall=${(testResults.bounce_tales_60s.wall_time_ms / 1000).toFixed(2)}s | Guest Virtual=${(testResults.bounce_tales_60s.guest_virtual_time_ms / 1000).toFixed(2)}s`);
      console.log(`BT VM Tick:              Median=${testResults.bounce_tales_60s.tick_median_ms.toFixed(3)} ms | p95=${testResults.bounce_tales_60s.tick_p95_ms.toFixed(3)} ms | p99=${testResults.bounce_tales_60s.tick_p99_ms.toFixed(3)} ms`);
      console.log(`BT rAF Intervals:        Median=${testResults.bounce_tales_60s.raf_median_ms.toFixed(2)} ms | p95=${testResults.bounce_tales_60s.raf_p95_ms.toFixed(2)} ms | p99=${testResults.bounce_tales_60s.raf_p99_ms.toFixed(2)} ms`);
      console.log(`BT Graphics & Audio:     FB Publishes=${testResults.bounce_tales_60s.framebuffer_publishes} | Long Tasks (Browser=${testResults.bounce_tales_60s.browser_long_tasks}, VM=${testResults.bounce_tales_60s.vm_long_tasks}) | Audio Active=${testResults.bounce_tales_60s.audio_active_frames} | Audio Underruns=${testResults.bounce_tales_60s.audio_playback_underruns}`);
      console.log(`Doom RPG (60s):          Wall=${(testResults.advanced_game_60s.wall_time_ms / 1000).toFixed(2)}s | Guest Virtual=${(testResults.advanced_game_60s.guest_virtual_time_ms / 1000).toFixed(2)}s`);
      console.log(`Doom RPG VM Tick:        Median=${testResults.advanced_game_60s.tick_median_ms.toFixed(3)} ms | p95=${testResults.advanced_game_60s.tick_p95_ms.toFixed(3)} ms | p99=${testResults.advanced_game_60s.tick_p99_ms.toFixed(3)} ms`);
      console.log(`Doom RPG rAF Intervals:  Median=${testResults.advanced_game_60s.raf_median_ms.toFixed(2)} ms | p95=${testResults.advanced_game_60s.raf_p95_ms.toFixed(2)} ms | p99=${testResults.advanced_game_60s.raf_p99_ms.toFixed(2)} ms`);
      console.log(`Doom RPG Graphics:       FB Publishes=${testResults.advanced_game_60s.framebuffer_publishes} | Long Tasks (Browser=${testResults.advanced_game_60s.browser_long_tasks}, VM=${testResults.advanced_game_60s.vm_long_tasks})`);
      console.log(`Stop->Reset->Reload:     Passed=${testResults.lifecycle_test.stop_reset_reload_passed} (Publishes=${testResults.lifecycle_test.reload_publishes}, Exceptions=${testResults.lifecycle_test.reload_exceptions})`);
      console.log('============================================================\n');

      const outPath = path.resolve(__dirname, '../experiments/exp_phase46_stability_blockers/browser_verification_results.json');
      fs.writeFileSync(outPath, JSON.stringify(testResults, null, 2));
      console.log(`[SUCCESS] Saved browser results to: ${outPath}`);
    }

    ws.close();
    chromeProc.kill();
    process.exit(testResults && testResults.all_passed ? 0 : 1);
  };
}

main().catch((e) => {
  console.error('[FATAL]', e);
  chromeProc.kill();
  process.exit(1);
});
