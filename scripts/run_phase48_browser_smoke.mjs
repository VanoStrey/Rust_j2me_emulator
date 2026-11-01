import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import http from 'http';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = 8899;
const CDP_PORT = 9248;
const URL = `http://127.0.0.1:${PORT}/test_runner_phase48_smoke.html`;

// 1. Start simple static server for web/
const mimeTypes = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.jar': 'application/java-archive',
};

const server = http.createServer((req, res) => {
  let reqPath = req.url.split('?')[0];
  if (reqPath === '/') reqPath = '/test_runner_phase48_smoke.html';
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
  console.log(`[Server] Static server listening at http://127.0.0.1:${PORT}`);

  const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  const chromeArgs = [
    '--headless=new',
    `--remote-debugging-port=${CDP_PORT}`,
    '--user-data-dir=/tmp/j2me_chrome_smoke_profile',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-background-timer-throttling',
    '--disable-renderer-backgrounding',
    URL,
  ];

  console.log(`[Chrome] Launching Headless Chrome CDP on port ${CDP_PORT}`);
  const chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

  let wsUrl = null;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((r) => setTimeout(r, 200));
    try {
      const resp = await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`);
      if (resp.ok) {
        const tabs = await resp.json();
        const tab = tabs.find((t) => t.url.includes('test_runner_phase48_smoke.html')) || tabs[0];
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
    console.log('[CDP] Attached! Enabling Runtime, Console, Page...');
    await sendCdp('Runtime.enable');
    await sendCdp('Console.enable');
    await sendCdp('Page.enable');
    await sendCdp('Page.reload', { ignoreCache: true });

    const maxPolls = 120;
    for (let poll = 0; poll < maxPolls; poll++) {
      if (testResults) break;
      await new Promise((r) => setTimeout(r, 500));
    }

    if (!testResults) {
      console.error('[ERROR] Timeout waiting for browser tests to complete');
      const logRes = await sendCdp('Runtime.evaluate', {
        expression: 'document.getElementById("log") ? document.getElementById("log").textContent : "no log"',
        returnByValue: true,
      });
      console.log('[Browser DOM Log]:\n' + (logRes?.result?.value || ''));
    } else if (testResults.error) {
      console.error(`[ERROR in Browser Test]: ${testResults.error}`);
    } else {
      console.log('\n============================================================');
      console.log('   PHASE 4.8 CHROME HEADLESS CDP BROWSER VERIFICATION REPORT');
      console.log('============================================================');
      console.log(`Browser:                 ${testResults.user_agent}`);
      console.log(`All Tests Passed:        ${testResults.all_passed}`);
      console.log(`Bounce Tales:            Passed=${testResults.bounce_tales.passed} | FB Publishes=${testResults.bounce_tales.fb_publishes} | Uncaught=${testResults.bounce_tales.uncaught_exceptions}`);
      console.log(`Prince of Persia:        Passed=${testResults.prince_of_persia.passed} | FB Publishes=${testResults.prince_of_persia.fb_publishes} | Uncaught=${testResults.prince_of_persia.uncaught_exceptions}`);
      console.log('============================================================\n');

      const outPath = path.resolve(__dirname, '../experiments/browser_smoke_phase48_results.json');
      fs.writeFileSync(outPath, JSON.stringify(testResults, null, 2));
      console.log(`[SUCCESS] Saved browser results to: ${outPath}`);
    }

    ws.close();
    chromeProc.kill();
    server.close();
    process.exit(testResults && testResults.all_passed ? 0 : 1);
  };
});
