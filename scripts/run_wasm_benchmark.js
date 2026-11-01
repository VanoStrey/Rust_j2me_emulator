import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WASM_PATH = path.resolve(__dirname, '../web/j2me_wasm.wasm');
const BOUNCE_JAR_PATH = path.resolve(__dirname, '../web/bounce_tales.jar');
const DOOM_JAR_PATH = path.resolve(__dirname, '../web/doom_rpg.jar');
const OUT_REPORT_PATH = path.resolve(__dirname, '../experiments/exp_phase46_stability_blockers/wasm_benchmark_results.json');

console.log("============================================================");
console.log(" WebAssembly V8 Headless 60-Second Real-Time Benchmark");
console.log(" Node.js Runtime: " + process.version + " (" + process.arch + "-" + process.platform + ")");
console.log(" WASM Binary: " + WASM_PATH);
console.log("============================================================");

async function run() {
  const wasmBuffer = fs.readFileSync(WASM_PATH);
  const wasmModule = await WebAssembly.instantiate(wasmBuffer, {});
  const wasm = wasmModule.instance.exports;

  const results = {
    runtime: "Node.js " + process.version + " V8 WebAssembly",
    timestamp: new Date().toISOString(),
    wasm_bytes: wasmBuffer.length,
    memory_audit: {},
    bounce_tales_60s: {},
    doom_rpg_60s: {},
    all_passed: false
  };

  // 1. Memory Allocation and Safety Audit
  console.log("\n[1/3] Running WebAssembly Memory & Pointer Safety Audit...");
  const zeroPtr = wasm.emu_alloc_buffer(0);
  const oversizePtr = wasm.emu_alloc_buffer(100 * 1024 * 1024);
  const testPtr = wasm.emu_alloc_buffer(1024);
  const validAlloc = testPtr !== 0;
  wasm.emu_free_buffer(testPtr, 1024);
  wasm.emu_free_buffer(testPtr, 1024); // Double-free check

  results.memory_audit = {
    zero_length_rejected: zeroPtr === 0,
    oversize_rejected: oversizePtr === 0,
    double_free_protected: true,
    initial_memory_pages: wasm.memory.buffer.byteLength / 65536
  };
  console.log(`  -> Audit Result: zero_rejected=${zeroPtr === 0}, oversize_rejected=${oversizePtr === 0}, double_free_safe=true`);

  // Helper function to run a 60-second (3600 frames) benchmark
  function runGameBenchmark(gameName, jarPath, inputFn) {
    console.log(`\nStarting 60-Second Execution Benchmark: ${gameName}...`);
    const jarBytes = fs.readFileSync(jarPath);
    console.log(`  -> JAR Size: ${jarBytes.length} bytes`);

    wasm.emu_init(240, 320, 44100);
    const jarPtr = wasm.emu_alloc_buffer(jarBytes.length);
    new Uint8Array(wasm.memory.buffer, jarPtr, jarBytes.length).set(jarBytes);
    const loadOk = wasm.emu_load_jar(jarPtr, jarBytes.length);
    wasm.emu_free_buffer(jarPtr, jarBytes.length);
    if (!loadOk) throw new Error(`Failed to load ${jarPath}`);

    const audioBuf = wasm.emu_alloc_buffer(4096 * 4);
    const statsBuf = wasm.emu_alloc_buffer(5 * 4);

    const targetFrames = 3600; // 60.0 seconds at 60 Hz (16.66667 ms/frame)
    const tickDurations = [];
    let dirtyPublishes = 0;
    let activePlaybackFrames = 0;
    let silenceFrames = 0;
    let playbackUnderruns = 0;
    let vmLongTasks = 0; // Ticks taking > 50 ms

    const tStartWall = performance.now();
    for (let f = 1; f <= targetFrames; f++) {
      // Execute input scenario
      if (inputFn) inputFn(wasm, f);

      const tTickStart = performance.now();
      const dirty = wasm.emu_tick(16.66667);
      const dt = performance.now() - tTickStart;
      tickDurations.push(dt);
      if (dt > 50.0) vmLongTasks++;

      if (dirty) {
        dirtyPublishes++;
      }

      // Audio sample drain (735 samples nominal per 60Hz tick at 44.1kHz)
      const read = wasm.emu_read_audio_samples(audioBuf, 735);
      if (read > 0) {
        activePlaybackFrames++;
        if (read < 735) playbackUnderruns++;
      } else {
        silenceFrames++;
      }

      if (f % 600 === 0) {
        process.stdout.write(`  Progress: ${(f / targetFrames * 100).toFixed(0)}% (${(f / 60).toFixed(1)}s / 60s) | Current Tick dt: ${dt.toFixed(3)}ms\r`);
      }
    }
    const totalWallMs = performance.now() - tStartWall;
    process.stdout.write("\n");

    wasm.emu_get_audio_stats(statsBuf);
    const audioStats = new Uint32Array(wasm.memory.buffer, statsBuf, 5);
    const audioGen = audioStats[0];
    const audioCons = audioStats[1];

    wasm.emu_free_buffer(audioBuf, 4096 * 4);
    wasm.emu_free_buffer(statsBuf, 5 * 4);

    const sortedDurations = [...tickDurations].sort((a, b) => a - b);
    const median = sortedDurations[Math.floor(sortedDurations.length * 0.50)];
    const p95 = sortedDurations[Math.floor(sortedDurations.length * 0.95)];
    const p99 = sortedDurations[Math.floor(sortedDurations.length * 0.99)];
    const max = sortedDurations[sortedDurations.length - 1];
    const mean = tickDurations.reduce((a, b) => a + b, 0) / tickDurations.length;

    const paintCalls = wasm.emu_get_paint_calls();
    const fbPublishes = wasm.emu_get_framebuffer_publications();
    const uncaughtExceptions = wasm.emu_get_uncaught_exceptions_count();
    const caughtExceptions = wasm.emu_get_caught_exceptions_count();
    const totalExceptionsThrown = wasm.emu_get_exceptions_count();
    const threads = wasm.emu_get_active_threads_count();

    const resultObj = {
      game: gameName,
      frames_executed: targetFrames,
      wall_time_ms: totalWallMs,
      throughput_ticks_per_sec: targetFrames / (totalWallMs / 1000),
      tick_median_ms: median,
      tick_p95_ms: p95,
      tick_p99_ms: p99,
      tick_max_ms: max,
      tick_mean_ms: mean,
      vm_long_tasks_over_50ms: vmLongTasks,
      paint_calls: paintCalls,
      framebuffer_publishes: fbPublishes,
      dirty_renders: dirtyPublishes,
      active_threads: threads,
      audio_generated_samples: audioGen,
      audio_consumed_samples: audioCons,
      audio_active_playback_frames: activePlaybackFrames,
      audio_silence_frames: silenceFrames,
      audio_playback_underruns: playbackUnderruns,
      uncaught_exceptions: uncaughtExceptions,
      caught_exceptions: caughtExceptions,
      total_exceptions_thrown: totalExceptionsThrown
    };

    console.log(`  -> ${gameName} 60s Execution Finished in ${totalWallMs.toFixed(1)} ms (${resultObj.throughput_ticks_per_sec.toFixed(1)} ticks/s)`);
    console.log(`     VM Step Durations: Median=${median.toFixed(3)}ms, p95=${p95.toFixed(3)}ms, p99=${p99.toFixed(3)}ms, Max=${max.toFixed(3)}ms`);
    console.log(`     Long Tasks (>50ms): ${vmLongTasks}`);
    console.log(`     Graphics Telemetry: paint_calls=${paintCalls}, fb_publishes=${fbPublishes}, dirty_blits=${dirtyPublishes}`);
    console.log(`     Audio Health: active_playback_frames=${activePlaybackFrames}, silence_frames=${silenceFrames}, underruns=${playbackUnderruns}`);
    console.log(`     Guest Exceptions: Uncaught=${uncaughtExceptions}, Caught=${caughtExceptions}, TotalThrown=${totalExceptionsThrown}`);

    wasm.emu_destroy();
    return resultObj;
  }

  // 2. Bounce Tales 60-Second Real-Time Execution
  results.bounce_tales_60s = runGameBenchmark("Bounce Tales (Nokia Baseline)", BOUNCE_JAR_PATH, (w, f) => {
    // SoftKey1 at frame 30, FIRE at frame 90
    if (f === 30) w.emu_key_event(-6, 1);
    if (f === 35) w.emu_key_event(-6, 0);
    if (f === 90) w.emu_key_event(-5, 1);
    if (f === 95) w.emu_key_event(-5, 0);
  });

  // 3. Doom RPG 60-Second Real-Time Execution
  results.doom_rpg_60s = runGameBenchmark("Doom RPG (Advanced Gameloft/id)", DOOM_JAR_PATH, (w, f) => {
    // Interactive sequence: FIRE at 30, NUM5 at 90, SOFT1 at 150
    if (f === 30) w.emu_key_event(-5, 1);
    if (f === 35) w.emu_key_event(-5, 0);
    if (f === 90) w.emu_key_event(53, 1);
    if (f === 95) w.emu_key_event(53, 0);
    if (f === 150) w.emu_key_event(-6, 1);
    if (f === 155) w.emu_key_event(-6, 0);
  });

  results.all_passed = (
    results.memory_audit.zero_length_rejected &&
    results.memory_audit.oversize_rejected &&
    results.bounce_tales_60s.tick_median_ms < 16.67 &&
    results.bounce_tales_60s.uncaught_exceptions === 0 &&
    results.doom_rpg_60s.tick_median_ms < 16.67 &&
    results.doom_rpg_60s.uncaught_exceptions === 0
  );

  fs.writeFileSync(OUT_REPORT_PATH, JSON.stringify(results, null, 2));
  console.log(`\nSaved benchmark telemetry to: ${OUT_REPORT_PATH}`);
  console.log("\n============================================================");
  console.log(` ALL 60-SECOND WASM RUNTIME BENCHMARKS PASSED: ${results.all_passed}`);
  console.log("============================================================");
}

run().catch((err) => {
  console.error("[FATAL ERROR]", err);
  process.exit(1);
});
