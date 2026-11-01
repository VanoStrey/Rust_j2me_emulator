/**
 * Reference Runner for FreeJ2ME using Headless Chrome & CheerpJ
 * Executes deterministic scenarios against the FreeJ2ME golden oracle.
 */

import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import http from 'http';
import { fileURLToPath } from 'url';
import { encodePpm } from './visual_diff.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

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

export class FreeJ2meHeadlessRunner {
  constructor(port = 8930, cdpPort = 9270) {
    this.port = port;
    this.cdpPort = cdpPort;
    this.server = null;
    this.chromeProc = null;
    this.ws = null;
    this.msgId = 1;
    this.pending = new Map();
  }

  async start() {
    // 1. Static file server
    this.server = http.createServer((req, res) => {
      let reqPath = req.url.split('?')[0];
      if (reqPath === '/') reqPath = '/index.html';
      const filePath = path.join(__dirname, '../../web', reqPath);

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
          'Cache-Control': 'no-cache, no-store, must-revalidate'
        });
        res.end(data);
      });
    });

    await new Promise((resolve) => this.server.listen(this.port, resolve));

    // 2. Launch Chrome
    const tmpProfile = fs.mkdtempSync('/tmp/j2me_free_runner_');
    const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    const chromeArgs = [
      '--headless=new',
      `--remote-debugging-port=${this.cdpPort}`,
      `--user-data-dir=${tmpProfile}`,
      '--disable-gpu',
      '--no-first-run',
      '--no-default-browser-check',
      '--disable-background-timer-throttling',
      '--disable-renderer-backgrounding',
      `http://127.0.0.1:${this.port}/freej2me/run.html`
    ];

    this.chromeProc = spawn(chromePath, chromeArgs, { stdio: 'ignore' });

    // 3. Connect CDP
    let wsUrl = null;
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 150));
      try {
        const resp = await fetch(`http://127.0.0.1:${this.cdpPort}/json/list`);
        if (resp.ok) {
          const tabs = await resp.json();
          const tab = tabs.find((t) => t.url.includes('run.html')) || tabs[0];
          if (tab && tab.webSocketDebuggerUrl) {
            wsUrl = tab.webSocketDebuggerUrl;
            break;
          }
        }
      } catch (_) {}
    }

    if (!wsUrl) {
      throw new Error('Failed to discover Chrome CDP WebSocket for FreeJ2ME');
    }

    this.ws = new WebSocket(wsUrl);
    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.id && this.pending.has(data.id)) {
          const { resolve, reject } = this.pending.get(data.id);
          this.pending.delete(data.id);
          if (data.error) reject(data.error);
          else resolve(data.result);
        }
      } catch (e) {
        console.error('[CDP Parse Error]', e);
      }
    };

    await new Promise((resolve) => {
      this.ws.onopen = resolve;
    });
  }

  send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = this.msgId++;
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  async eval(expression) {
    const res = await this.send('Runtime.evaluate', {
      expression,
      returnByValue: true,
      awaitPromise: true
    });
    return res.result ? res.result.value : null;
  }

  async runScenario(scenario, outDir) {
    const jarUrl = `http://127.0.0.1:${this.port}/freej2me/run.html?jar=${encodeURIComponent(scenario.jar_file)}`;
    await this.send('Page.navigate', { url: jarUrl });

    // Wait until CheerpJ has loaded and canvas is created
    let ready = false;
    for (let i = 0; i < 60; i++) {
      await new Promise((r) => setTimeout(r, 200));
      ready = await this.eval(`
        (() => {
          const d = document.getElementById("display");
          return Boolean(d && d.width > 0 && d.height > 0 && document.getElementById("loading").hidden);
        })()
      `);
      if (ready) break;
    }

    if (!ready) {
      console.warn(`[WARN] FreeJ2ME canvas not fully ready after 12s for ${scenario.jar_file}, proceeding with fallback`);
    }

    const snapshots = [];
    const stepIntervalMs = scenario.tick_step_ms || 16.66667;

    // Index inputs by tick
    const inputMap = new Map();
    for (const inp of scenario.inputs) {
      if (!inputMap.has(inp.tick)) inputMap.set(inp.tick, []);
      inputMap.get(inp.tick).push(inp);
    }

    // Step through virtual ticks
    for (let tick = 1; tick <= scenario.total_ticks; tick++) {
      // Dispatch any scheduled input
      if (inputMap.has(tick)) {
        for (const ev of inputMap.get(tick)) {
          const isDown = ev.action.startsWith('p') || ev.action.includes('down') || ev.action.includes('press');
          const codeMap = {
            'Enter': 13, 'ArrowLeft': 37, 'ArrowUp': 38, 'ArrowRight': 39, 'ArrowDown': 40,
            'Digit1': 49, 'Digit2': 50, 'Digit3': 51, 'Digit4': 52, 'Digit5': 53, 'Digit6': 54, 'Digit7': 55, 'Digit8': 56, 'Digit9': 57, 'Digit0': 48,
            'F1': 112, 'F2': 113, 'SoftLeft': 112, 'SoftRight': 113
          };
          const keyNum = codeMap[ev.free_key] || 13;
          await this.eval(`
            (() => {
              if (window.evtQueue) {
                window.evtQueue.queueEvent({
                  kind: ${isDown ? '"keydown"' : '"keyup"'},
                  args: [${keyNum}, '\x00', false, false]
                });
              }
            })()
          `);
        }
      }

      if (scenario.snapshot_ticks.includes(tick)) {
        // Read canvas pixels
        const rawBase64 = await this.eval(`
          (() => {
            const d = document.getElementById("display");
            if (!d) return null;
            const ctx = d.getContext("2d");
            const imgData = ctx.getImageData(0, 0, ${scenario.resolution.width}, ${scenario.resolution.height});
            let binary = "";
            const bytes = new Uint8Array(imgData.data.buffer);
            const len = bytes.byteLength;
            for (let i = 0; i < len; i++) {
              binary += String.fromCharCode(bytes[i]);
            }
            return btoa(binary);
          })()
        `);

        if (rawBase64) {
          const rawBuf = Buffer.from(rawBase64, 'base64');
          const rawName = `ref_tick_${String(tick).padStart(4, '0')}.raw`;
          const ppmName = `ref_tick_${String(tick).padStart(4, '0')}.ppm`;
          const rawPath = path.join(outDir, rawName);
          const ppmPath = path.join(outDir, ppmName);

          fs.writeFileSync(rawPath, rawBuf);
          const ppmBuf = encodePpm(new Uint8Array(rawBuf), scenario.resolution.width, scenario.resolution.height);
          fs.writeFileSync(ppmPath, ppmBuf);

          snapshots.push({
            tick,
            rawFile: rawName,
            ppmFile: ppmName,
            buffer: rawBuf
          });
        }
      }

      // Small tick cadence sleep to allow CheerpJ event loop to pump
      if (tick % 10 === 0) {
        await new Promise((r) => setTimeout(r, 5));
      }
    }

    return snapshots;
  }

  async close() {
    if (this.chromeProc) {
      this.chromeProc.kill();
    }
    if (this.server) {
      this.server.close();
    }
  }
}
