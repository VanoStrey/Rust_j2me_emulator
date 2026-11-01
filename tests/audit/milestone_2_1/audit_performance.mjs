import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '../../..');
const WASM_PATH = path.resolve(ROOT, 'web/j2me_wasm.wasm');
const BOUNCE_JAR = path.resolve(ROOT, 'web/bounce_tales.jar');
const DOOM_JAR = path.resolve(ROOT, 'web/doom_rpg.jar');
const GD_JAR = path.resolve(ROOT, 'web/gravity_defied.jar');

function percentile(arr, pct) {
  if (arr.length === 0) return 0;
  const sorted = [...arr].sort((a, b) => a - b);
  const idx = Math.min(sorted.length - 1, Math.floor(sorted.length * (pct / 100)));
  return +sorted[idx].toFixed(4);
}

// 1. WASM Multi-Run Auditor
async function runWasmAudit(gameName, jarPath, targetTicks, runsCount = 5) {
  const wasmBuffer = fs.readFileSync(WASM_PATH);
  const runs = [];

  for (let r = 0; r < runsCount; r++) {
    const wasmModule = await WebAssembly.instantiate(wasmBuffer, {});
    const wasm = wasmModule.instance.exports;

    const jarBytes = fs.readFileSync(jarPath);
    wasm.emu_init(240, 320, 44100);
    wasm.emu_set_strict_mode(1);

    const jarPtr = wasm.emu_alloc_buffer(jarBytes.length);
    new Uint8Array(wasm.memory.buffer, jarPtr, jarBytes.length).set(jarBytes);
    wasm.emu_load_jar(jarPtr, jarBytes.length);
    wasm.emu_free_buffer(jarPtr, jarBytes.length);

    // Warm-up 50 ticks to allow V8 JIT to compile hot loops
    for (let i = 0; i < 50; i++) {
      wasm.emu_tick(16.66667);
    }

    // Measure ticks
    const tickTimes = [];
    const t0 = performance.now();
    for (let i = 0; i < targetTicks; i++) {
      const tTick0 = performance.now();
      wasm.emu_tick(16.66667);
      tickTimes.push(performance.now() - tTick0);
    }
    const wallMs = performance.now() - t0;

    const activeTicks = tickTimes.filter(t => t > 0.05).length;
    const idleTicks = tickTimes.length - activeTicks;

    runs.push({
      run_index: r + 1,
      wall_ms: +wallMs.toFixed(2),
      ticks_per_sec: +(targetTicks / (wallMs / 1000)).toFixed(1),
      mean_ms: +(tickTimes.reduce((a, b) => a + b, 0) / tickTimes.length).toFixed(4),
      median_ms: percentile(tickTimes, 50),
      p95_ms: percentile(tickTimes, 95),
      p99_ms: percentile(tickTimes, 99),
      active_ticks_count: activeTicks,
      idle_ticks_count: idleTicks,
      live_objects: wasm.emu_get_live_objects(),
      gc_count: wasm.emu_get_gc_count(),
      fb_publishes: wasm.emu_get_framebuffer_publications(),
      fb_changes: Number(wasm.emu_get_framebuffer_changes_count())
    });
  }

  // Aggregate across 5 runs
  const avgMean = +(runs.reduce((acc, r) => acc + r.mean_ms, 0) / runsCount).toFixed(4);
  const avgMedian = +(runs.reduce((acc, r) => acc + r.median_ms, 0) / runsCount).toFixed(4);
  const avgP95 = +(runs.reduce((acc, r) => acc + r.p95_ms, 0) / runsCount).toFixed(4);
  const avgP99 = +(runs.reduce((acc, r) => acc + r.p99_ms, 0) / runsCount).toFixed(4);
  const avgTps = +(runs.reduce((acc, r) => acc + r.ticks_per_sec, 0) / runsCount).toFixed(1);

  return {
    game: gameName,
    runtime: 'WASM (V8 Node.js)',
    target_ticks: targetTicks,
    runs_count: runsCount,
    aggregated: {
      ticks_per_sec: avgTps,
      mean_latency_ms: avgMean,
      median_latency_ms: avgMedian,
      p95_latency_ms: avgP95,
      p99_latency_ms: avgP99,
      vm_fraction_of_16ms_budget_pct: Math.min(100.0, +((avgMean / 16.66667) * 100).toFixed(2))
    },
    bimodal_explanation: `Across ${targetTicks} ticks, ${runs[0].idle_ticks_count} ticks were idle sleep ticks (<0.05ms) and ${runs[0].active_ticks_count} were active ticks. Median represents idle ticks; mean reflects blended execution.`,
    individual_runs: runs
  };
}

// 2. Native Multi-Run Auditor
function runNativeScenarioOnce(scenarioFile) {
  return new Promise((resolve, reject) => {
    const scPath = path.resolve(ROOT, `tests/scenarios/${scenarioFile}`);
    const cmd = 'cargo';
    const args = ['run', '--release', '-p', 'j2me-cli', '--', '--scenario', scPath, '--strict'];
    const proc = spawn(cmd, args, {
      cwd: ROOT,
      env: {
        ...process.env,
        PATH: `/Users/foolvan/AntigravityProfiles/1/.rustup/toolchains/stable-aarch64-apple-darwin/bin:${process.env.PATH}`
      }
    });

    let stdout = '';
    let stderr = '';
    proc.stdout.on('data', d => stdout += d.toString());
    proc.stderr.on('data', d => stderr += d.toString());

    proc.on('close', code => {
      if (code === 0 || code === 2) {
        const stem = path.basename(scenarioFile, '.json');
        const gameId = stem.replace(/_init|_gameplay|_longrunning/, '');
        const resPath = path.resolve(ROOT, `tests/results/${gameId}/${stem}/rust_execution.json`);
        let rep = JSON.parse(fs.readFileSync(resPath, 'utf8'));
        resolve(rep);
      } else {
        reject(new Error(`Native execution failed (exit ${code}):\n${stderr}`));
      }
    });
  });
}

async function runNativeAudit(gameName, scenarioFile, runsCount = 5) {
  const runs = [];
  for (let r = 0; r < runsCount; r++) {
    const rep = await runNativeScenarioOnce(scenarioFile);
    runs.push({
      run_index: r + 1,
      wall_ms: +rep.wall_time_ms.toFixed(2),
      ticks_per_sec: +rep.ticks_per_sec.toFixed(1),
      instructions_executed: rep.instructions_executed,
      instructions_per_sec: +(rep.instructions_executed / (rep.wall_time_ms / 1000)).toFixed(0),
      mean_latency_ms: +(rep.wall_time_ms / rep.total_ticks_executed).toFixed(4),
      live_objects: rep.final_heap_objects,
      gc_count: rep.gc_count
    });
  }

  const avgWall = +(runs.reduce((acc, r) => acc + r.wall_ms, 0) / runsCount).toFixed(2);
  const avgTps = +(runs.reduce((acc, r) => acc + r.ticks_per_sec, 0) / runsCount).toFixed(1);
  const avgMean = +(runs.reduce((acc, r) => acc + r.mean_latency_ms, 0) / runsCount).toFixed(4);
  const avgIps = +(runs.reduce((acc, r) => acc + r.instructions_per_sec, 0) / runsCount).toFixed(0);

  return {
    game: gameName,
    runtime: 'Native Rust (AArch64 release)',
    runs_count: runsCount,
    aggregated: {
      ticks_per_sec: avgTps,
      instructions_per_sec: avgIps,
      mean_latency_ms: avgMean,
      vm_fraction_of_16ms_budget_pct: Math.min(100.0, +((avgMean / 16.66667) * 100).toFixed(2))
    },
    individual_runs: runs
  };
}

async function main() {
  console.log('[Performance Audit] Running 5-run statistical benchmark...');
  const results = {
    audit_date: new Date().toISOString(),
    hardware: 'Apple M2 (8 cores), macOS 15.0 arm64',
    methodology: '5 independent runs per workload with 50-tick warm-up for JIT stabilization',
    wasm_audits: [],
    native_audits: []
  };

  // WASM Audits
  console.log('-> Auditing WASM: Bounce Tales...');
  results.wasm_audits.push(await runWasmAudit('Bounce Tales', BOUNCE_JAR, 240, 5));
  console.log('-> Auditing WASM: Doom RPG...');
  results.wasm_audits.push(await runWasmAudit('Doom RPG', DOOM_JAR, 240, 5));
  console.log('-> Auditing WASM: Gravity Defied...');
  results.wasm_audits.push(await runWasmAudit('Gravity Defied', GD_JAR, 200, 5));

  // Native Audits
  console.log('-> Auditing Native: Bounce Tales...');
  results.native_audits.push(await runNativeAudit('Bounce Tales', 'bounce_tales_init.json', 5));
  console.log('-> Auditing Native: Doom RPG...');
  results.native_audits.push(await runNativeAudit('Doom RPG', 'doom_rpg_init.json', 5));
  console.log('-> Auditing Native: Gravity Defied...');
  results.native_audits.push(await runNativeAudit('Gravity Defied', 'gravity_defied_init.json', 5));

  const outPath = path.resolve(__dirname, 'performance_audit.json');
  fs.writeFileSync(outPath, JSON.stringify(results, null, 2));
  console.log(`[Performance Audit Complete] Results written to ${outPath}`);
}

main().catch(e => {
  console.error('[Performance Audit Error]', e);
  process.exit(1);
});
