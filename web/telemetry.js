/**
 * Unified Telemetry & Performance Monitoring Engine
 * Shared between Rust+WASM J2ME, FreeJ2ME, and Side-by-Side Benchmark
 */

export class TelemetryTracker {
  constructor(engineName = "Rust WASM") {
    this.engineName = engineName;
    this.startTime = performance.now();
    this.coldBootTime = null;
    this.jarLoadStartTime = null;
    this.firstPaintTime = null;
    this.runtimeFootprintKB = 0;

    // Frame metrics
    this.frameTimes = []; // ms per frame
    this.tickTimes = [];  // ms per emu_tick
    this.frameTimestamps = [];
    this.lastFrameTime = performance.now();
    this.fps = 0;
    this.fpsLow1Pct = 0;
    this.paintCallsCount = 0;
    this.paintCallsPerSec = 0;
    this.lastPaintReportTime = performance.now();
    this.lastPaintReportCount = 0;

    // Instructions
    this.totalInstructions = 0;
    this.lastInstReportTime = performance.now();
    this.lastInstReportCount = 0;
    this.ips = 0;

    // Audio
    this.audioPeak = 0;
    this.audioRms = 0;
    this.audioClippedSamples = 0;
    this.audioUnderruns = 0;

    // Memory
    this.jsHeapMB = 0;
    this.wasmHeapMB = 0;
    this.vmHeapObjects = 0;
    this.gcRuns = 0;

    // Framebuffer
    this.nonBlackPixelsPct = 0;

    // Rolling history for export and graphs (max 300 entries = ~15s-60s)
    this.history = {
      timestamps: [],
      fps: [],
      frameTimeMs: [],
      cpuLoadPct: [],
      audioPeak: [],
      memoryMB: []
    };

    // Behavioral Parity State
    this.activeScreen = "Canvas";
    this.isFullScreen = false;
    this.canvasWidth = 240;
    this.canvasHeight = 320;
    this.clipRect = { x: 0, y: 0, w: 240, h: 320 };
    this.translate = { tx: 0, ty: 0 };
    this.drawCalls = {
      drawImage: 0,
      drawRegion: 0,
      fillRect: 0,
      drawString: 0,
      drawPixels: 0,
      total: 0
    };
    this.activeThreads = 0;
    this.activeTimers = 0;
    this.lastException = null;

    this.detectRuntimeFootprint();
  }

  recordParityState(state = {}) {
    if (state.active_screen !== undefined) this.activeScreen = state.active_screen;
    if (state.is_fullscreen !== undefined) this.isFullScreen = state.is_fullscreen;
    if (state.canvas_width !== undefined) this.canvasWidth = state.canvas_width;
    if (state.canvas_height !== undefined) this.canvasHeight = state.canvas_height;
    if (state.clip_rect) this.clipRect = state.clip_rect;
    if (state.translate) this.translate = state.translate;
    if (state.draw_calls) this.drawCalls = state.draw_calls;
    if (state.active_threads !== undefined) this.activeThreads = state.active_threads;
    if (state.active_timers !== undefined) this.activeTimers = state.active_timers;
    if (state.last_exception !== undefined) this.lastException = state.last_exception;
  }

  detectRuntimeFootprint() {
    try {
      const resources = performance.getEntriesByType("resource");
      let totalBytes = 0;
      for (const res of resources) {
        if (res.name.endsWith(".wasm") || res.name.endsWith(".jar") || res.name.includes("cj3loader") || res.name.includes("cheerpj")) {
          totalBytes += (res.transferSize || res.encodedBodySize || res.decodedBodySize || 0);
        }
      }
      this.runtimeFootprintKB = Math.round(totalBytes / 1024);
    } catch (_) {}
  }

  recordColdBoot() {
    this.coldBootTime = Math.round(performance.now() - this.startTime);
  }

  recordJarLoadStart() {
    this.jarLoadStartTime = performance.now();
    this.firstPaintTime = null;
  }

  recordFirstPaint() {
    if (this.jarLoadStartTime && this.firstPaintTime === null) {
      this.firstPaintTime = Math.round(performance.now() - this.jarLoadStartTime);
    }
  }

  recordFrame(tickDurationMs = 0) {
    const now = performance.now();
    const delta = now - this.lastFrameTime;
    this.lastFrameTime = now;

    if (delta > 0 && delta < 1000) {
      this.frameTimes.push(delta);
      if (this.frameTimes.length > 120) this.frameTimes.shift();

      this.frameTimestamps.push(now);
      while (this.frameTimestamps.length > 0 && this.frameTimestamps[0] < now - 1000) {
        this.frameTimestamps.shift();
      }
      this.fps = this.frameTimestamps.length;
    }

    if (tickDurationMs > 0) {
      this.tickTimes.push(tickDurationMs);
      if (this.tickTimes.length > 60) this.tickTimes.shift();
    }
  }

  recordPaint() {
    this.paintCallsCount++;
    this.recordFirstPaint();
  }

  recordInstructions(total) {
    this.totalInstructions = total;
  }

  updateMetrics(stats = {}) {
    const now = performance.now();

    // Calculate 1% Low FPS
    if (this.frameTimes.length >= 10) {
      const sorted = [...this.frameTimes].sort((a, b) => b - a); // slowest first
      const count1Pct = Math.max(1, Math.floor(sorted.length * 0.05));
      const worstTimes = sorted.slice(0, count1Pct);
      const avgWorst = worstTimes.reduce((acc, v) => acc + v, 0) / count1Pct;
      this.fpsLow1Pct = avgWorst > 0 ? Math.round(1000 / avgWorst) : this.fps;
    } else {
      this.fpsLow1Pct = this.fps;
    }

    // Paint calls per sec
    const paintSec = (now - this.lastPaintReportTime) / 1000;
    if (paintSec >= 0.5) {
      this.paintCallsPerSec = Math.round((this.paintCallsCount - this.lastPaintReportCount) / paintSec);
      this.lastPaintReportTime = now;
      this.lastPaintReportCount = this.paintCallsCount;
    }

    // IPS
    const instSec = (now - this.lastInstReportTime) / 1000;
    if (instSec >= 0.5) {
      this.ips = Math.round((this.totalInstructions - this.lastInstReportCount) / instSec);
      this.lastInstReportTime = now;
      this.lastInstReportCount = this.totalInstructions;
    }

    // Memory
    if (performance.memory) {
      this.jsHeapMB = +(performance.memory.usedJSHeapSize / (1024 * 1024)).toFixed(1);
    }
    if (stats.wasmMemoryBuffer) {
      this.wasmHeapMB = +(stats.wasmMemoryBuffer.byteLength / (1024 * 1024)).toFixed(1);
    }
    if (stats.vmHeapObjects !== undefined) this.vmHeapObjects = stats.vmHeapObjects;
    if (stats.gcRuns !== undefined) this.gcRuns = stats.gcRuns;

    // Audio
    if (stats.audioPeak !== undefined) this.audioPeak = stats.audioPeak;
    if (stats.audioRms !== undefined) this.audioRms = stats.audioRms;
    if (stats.audioClippedSamples !== undefined) this.audioClippedSamples = stats.audioClippedSamples;
    if (stats.audioUnderruns !== undefined) this.audioUnderruns = stats.audioUnderruns;

    // Non-black pixels
    if (stats.nonBlackPixelsPct !== undefined) this.nonBlackPixelsPct = stats.nonBlackPixelsPct;

    // CPU load calculation
    const avgTick = this.getAverageTickTime();
    const cpuLoad = Math.min(100, Math.round((avgTick / 16.67) * 100));

    // Rolling history
    this.history.timestamps.push(now);
    this.history.fps.push(this.fps);
    this.history.frameTimeMs.push(this.getAverageFrameTime());
    this.history.cpuLoadPct.push(cpuLoad);
    this.history.audioPeak.push(this.audioPeak);
    this.history.memoryMB.push(this.jsHeapMB + this.wasmHeapMB);

    if (this.history.timestamps.length > 200) {
      for (const k in this.history) {
        this.history[k].shift();
      }
    }
  }

  getAverageFrameTime() {
    if (this.frameTimes.length === 0) return 0;
    const sum = this.frameTimes.reduce((acc, v) => acc + v, 0);
    return +(sum / this.frameTimes.length).toFixed(2);
  }

  getFramePercentiles() {
    if (this.frameTimes.length === 0) return { min: 0, avg: 0, max: 0, p95: 0, p99: 0 };
    const sorted = [...this.frameTimes].sort((a, b) => a - b);
    const min = +sorted[0].toFixed(2);
    const max = +sorted[sorted.length - 1].toFixed(2);
    const avg = this.getAverageFrameTime();
    const p95 = +sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * 0.95))].toFixed(2);
    const p99 = +sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * 0.99))].toFixed(2);
    return { min, avg, max, p95, p99 };
  }

  getFrameJitter() {
    if (this.frameTimes.length < 2) return 0;
    const avg = this.getAverageFrameTime();
    const variance = this.frameTimes.reduce((acc, v) => acc + Math.pow(v - avg, 2), 0) / this.frameTimes.length;
    return +Math.sqrt(variance).toFixed(2);
  }

  getAverageTickTime() {
    if (this.tickTimes.length === 0) return 0;
    const sum = this.tickTimes.reduce((acc, v) => acc + v, 0);
    return +(sum / this.tickTimes.length).toFixed(2);
  }

  exportJSON() {
    return JSON.stringify({
      engine: this.engineName,
      timestamp: new Date().toISOString(),
      coldBootTimeMs: this.coldBootTime,
      firstPaintTimeMs: this.firstPaintTime,
      runtimeFootprintKB: this.runtimeFootprintKB,
      fps: this.fps,
      fpsLow1Pct: this.fpsLow1Pct,
      frameStats: this.getFramePercentiles(),
      frameJitterMs: this.getFrameJitter(),
      cpuLoadPct: Math.min(100, Math.round((this.getAverageTickTime() / 16.67) * 100)),
      totalInstructions: this.totalInstructions,
      ips: this.ips,
      paintCallsPerSec: this.paintCallsPerSec,
      totalPaintCalls: this.paintCallsCount,
      nonBlackPixelsPct: this.nonBlackPixelsPct,
      memory: {
        jsHeapMB: this.jsHeapMB,
        wasmHeapMB: this.wasmHeapMB,
        totalMB: +(this.jsHeapMB + this.wasmHeapMB).toFixed(1),
        vmObjects: this.vmHeapObjects,
        gcRuns: this.gcRuns
      },
      audio: {
        peakLevel: +this.audioPeak.toFixed(3),
        peakDbFs: this.audioPeak > 0 ? +(20 * Math.log10(this.audioPeak)).toFixed(1) : -96,
        clippedSamples: this.audioClippedSamples,
        underruns: this.audioUnderruns
      },
      parity: {
        activeScreen: this.activeScreen,
        isFullScreen: this.isFullScreen,
        canvasWidth: this.canvasWidth,
        canvasHeight: this.canvasHeight,
        clipRect: this.clipRect,
        translate: this.translate,
        drawCalls: this.drawCalls,
        activeThreads: this.activeThreads,
        activeTimers: this.activeTimers,
        lastException: this.lastException
      },
      history: this.history
    }, null, 2);
  }

  exportCSV() {
    let csv = "TimestampMs,FPS,FrameTimeMs,CPULoadPct,AudioPeak,TotalMemoryMB\n";
    for (let i = 0; i < this.history.timestamps.length; i++) {
      csv += `${Math.round(this.history.timestamps[i])},${this.history.fps[i]},${this.history.frameTimeMs[i]},${this.history.cpuLoadPct[i]},${this.history.audioPeak[i]},${this.history.memoryMB[i]}\n`;
    }
    return csv;
  }
}

/**
 * Visual Renderer for Telemetry HUD & Sparklines
 */
export class TelemetryUI {
  constructor(tracker, containerId = "telemetry-container") {
    this.tracker = tracker;
    this.container = document.getElementById(containerId);
    this.sparklineCanvas = null;
    this.vuCanvas = null;
    this.initDOM();
  }

  initDOM() {
    if (!this.container) return;
    this.container.innerHTML = `
      <div class="glass-card telemetry-card">
        <div class="card-title">
          <span>📊 Engine Telemetry (${this.tracker.engineName})</span>
          <div class="header-badges">
            <span id="tel-fps" class="badge-fps">-- FPS</span>
            <span id="tel-clipping" class="badge-clean">CLIP: 0</span>
          </div>
        </div>

        <!-- Live Graphs -->
        <div class="graphs-row">
          <div class="sparkline-wrapper">
            <div class="graph-label">Frame Time (ms) & FPS</div>
            <canvas id="tel-sparkline" width="220" height="42"></canvas>
          </div>
          <div class="vu-wrapper">
            <div class="graph-label">Audio Peak Level (dBFS)</div>
            <canvas id="tel-vu" width="90" height="42"></canvas>
          </div>
        </div>

        <!-- Metrics Table -->
        <div class="telemetry-grid">
          <div class="telemetry-box">
            <div class="metric-name">Frame Time (Avg / P95)</div>
            <div class="metric-value" id="tel-frametime">-- / -- ms</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">1% Low FPS / Jitter</div>
            <div class="metric-value" id="tel-lowfps">-- fps / -- ms</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">CPU Load / VM Tick</div>
            <div class="metric-value" id="tel-cpuload">--% / -- ms</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">IPS / Instructions</div>
            <div class="metric-value" id="tel-ips">-- / --</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">Memory (JS + WASM)</div>
            <div class="metric-value" id="tel-memory">-- MB</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">Guest Objects / GCs</div>
            <div class="metric-value" id="tel-guestobjs">-- / --</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">Paints / Non-Black</div>
            <div class="metric-value" id="tel-paints">-- /sec &bull; --%</div>
          </div>
          <div class="telemetry-box">
            <div class="metric-name">Startup (Cold / First Paint)</div>
            <div class="metric-value" id="tel-startup">-- / -- ms</div>
          </div>
        </div>

        <!-- Export Buttons -->
        <div class="export-row" style="margin-top: 12px; display: flex; gap: 8px;">
          <button class="btn-action small" id="btn-export-json">📥 Export JSON</button>
          <button class="btn-action small" id="btn-export-csv">📄 Export CSV</button>
        </div>
      </div>
    `;

    this.sparklineCanvas = this.container.querySelector("#tel-sparkline");
    this.vuCanvas = this.container.querySelector("#tel-vu");

    this.container.querySelector("#btn-export-json").addEventListener("click", () => {
      this.downloadFile(`${this.tracker.engineName.toLowerCase().replace(/[^a-z0-9]/g, "_")}_telemetry.json`, this.tracker.exportJSON(), "application/json");
    });
    this.container.querySelector("#btn-export-csv").addEventListener("click", () => {
      this.downloadFile(`${this.tracker.engineName.toLowerCase().replace(/[^a-z0-9]/g, "_")}_telemetry.csv`, this.tracker.exportCSV(), "text/csv");
    });
  }

  downloadFile(filename, content, type) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  render() {
    if (!this.container) return;
    const t = this.tracker;
    const pct = t.getFramePercentiles();
    const jitter = t.getFrameJitter();
    const avgTick = t.getAverageTickTime();
    const cpuLoad = Math.min(100, Math.round((avgTick / 16.67) * 100));

    // Elements
    const elFps = this.container.querySelector("#tel-fps");
    const elClip = this.container.querySelector("#tel-clipping");
    const elFrameTime = this.container.querySelector("#tel-frametime");
    const elLowFps = this.container.querySelector("#tel-lowfps");
    const elCpuLoad = this.container.querySelector("#tel-cpuload");
    const elIps = this.container.querySelector("#tel-ips");
    const elMemory = this.container.querySelector("#tel-memory");
    const elGuestObjs = this.container.querySelector("#tel-guestobjs");
    const elPaints = this.container.querySelector("#tel-paints");
    const elStartup = this.container.querySelector("#tel-startup");

    if (elFps) elFps.textContent = `${t.fps} FPS`;
    if (elClip) {
      if (t.audioClippedSamples > 0) {
        elClip.className = "badge-danger";
        elClip.textContent = `CLIP: ${t.audioClippedSamples}`;
      } else {
        elClip.className = "badge-clean";
        elClip.textContent = "CLIP: 0";
      }
    }

    if (elFrameTime) elFrameTime.textContent = `${pct.avg} / ${pct.p95} ms`;
    if (elLowFps) elLowFps.textContent = `${t.fpsLow1Pct} fps / ±${jitter} ms`;
    if (elCpuLoad) elCpuLoad.textContent = `${cpuLoad}% / ${avgTick} ms`;
    if (elIps) elIps.textContent = `${(t.ips / 1000).toFixed(1)}k / ${(t.totalInstructions / 1000).toFixed(0)}k`;

    const totalMem = +(t.jsHeapMB + t.wasmHeapMB).toFixed(1);
    if (elMemory) elMemory.textContent = `${totalMem} MB (${t.jsHeapMB} JS + ${t.wasmHeapMB} WASM)`;
    if (elGuestObjs) elGuestObjs.textContent = `${t.vmHeapObjects} objs / ${t.gcRuns} GCs`;
    if (elPaints) elPaints.textContent = `${t.paintCallsPerSec}/s • ${t.nonBlackPixelsPct}% fill`;

    const bootStr = t.coldBootTime ? `${t.coldBootTime}ms` : "--";
    const paintStr = t.firstPaintTime ? `${t.firstPaintTime}ms` : "--";
    if (elStartup) elStartup.textContent = `${bootStr} / ${paintStr}`;

    this.drawSparkline();
    this.drawVU();
  }

  drawSparkline() {
    if (!this.sparklineCanvas) return;
    const ctx = this.sparklineCanvas.getContext("2d");
    const w = this.sparklineCanvas.width;
    const h = this.sparklineCanvas.height;
    ctx.clearRect(0, 0, w, h);

    const history = this.tracker.history.frameTimeMs;
    if (history.length < 2) return;

    // Draw 16.6ms target line
    ctx.strokeStyle = "rgba(0, 229, 255, 0.25)";
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    const y16 = h - (16.67 / 50.0) * h;
    ctx.beginPath();
    ctx.moveTo(0, y16);
    ctx.lineTo(w, y16);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw Frame Times
    ctx.beginPath();
    ctx.strokeStyle = "#00e5ff";
    ctx.lineWidth = 1.8;
    const step = w / (history.length - 1);
    for (let i = 0; i < history.length; i++) {
      const val = Math.min(50, history[i]);
      const y = h - (val / 50.0) * (h - 4) - 2;
      const x = i * step;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  }

  drawVU() {
    if (!this.vuCanvas) return;
    const ctx = this.vuCanvas.getContext("2d");
    const w = this.vuCanvas.width;
    const h = this.vuCanvas.height;
    ctx.clearRect(0, 0, w, h);

    const peak = Math.min(1.0, this.tracker.audioPeak);
    const db = peak > 0 ? 20 * Math.log10(peak) : -60;
    const norm = Math.max(0, Math.min(1, (db + 48) / 48)); // -48dB to 0dB

    // VU meter bar
    const barWidth = Math.round(norm * (w - 10));
    const gradient = ctx.createLinearGradient(0, 0, w - 10, 0);
    gradient.addColorStop(0, "#00e676"); // green
    gradient.addColorStop(0.7, "#ffb300"); // yellow
    gradient.addColorStop(0.95, "#ff5252"); // red

    ctx.fillStyle = "rgba(255, 255, 255, 0.08)";
    ctx.fillRect(4, h / 2 - 8, w - 8, 16);

    ctx.fillStyle = gradient;
    ctx.fillRect(4, h / 2 - 8, barWidth, 16);

    ctx.fillStyle = "#fff";
    ctx.font = "9px monospace";
    ctx.fillText(`${db.toFixed(1)} dB`, 8, h / 2 + 4);
  }
}
