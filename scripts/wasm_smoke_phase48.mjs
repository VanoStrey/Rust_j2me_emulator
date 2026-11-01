import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WASM_PATH = path.resolve(__dirname, '../web/j2me_wasm.wasm');
const BOUNCE_JAR = path.resolve(__dirname, '../web/bounce_tales.jar');
const POP_JAR = '/Users/foolvan/Desktop/j2me_market/roms/raw/prince_sands.jar';

async function testGame(wasmInstance, gameName, jarPath) {
  const wasm = wasmInstance.exports;
  console.log(`[WASM Test] Starting 10s smoke test for ${gameName}...`);
  const jarBytes = fs.readFileSync(jarPath);

  wasm.emu_init(240, 320, 44100);
  const jarPtr = wasm.emu_alloc_buffer(jarBytes.length);
  new Uint8Array(wasm.memory.buffer, jarPtr, jarBytes.length).set(jarBytes);

  const loadOk = wasm.emu_load_jar(jarPtr, jarBytes.length);
  wasm.emu_free_buffer(jarPtr, jarBytes.length);

  if (loadOk === 0) {
    return { game: gameName, status: "LOAD_FAILED", uncaught_exceptions: 0, host_panics: 1 };
  }

  // Run 600 ticks = 10s @ 60 FPS
  for (let tick = 0; tick < 600; tick++) {
    const res = wasm.emu_tick(16.66667);
    if (res < 0) {
      return { game: gameName, status: "CRASH", uncaught_exceptions: 0, host_panics: 1 };
    }
  }

  const uncaught = wasm.emu_get_uncaught_exceptions_count();
  const fbPubs = wasm.emu_get_framebuffer_publications();
  const paintCalls = wasm.emu_get_paint_calls();
  const activeThreads = wasm.emu_get_active_threads_count();

  console.log(`  -> ${gameName}: FB Publishes=${fbPubs}, Paint=${paintCalls}, ActiveThreads=${activeThreads}, Uncaught=${uncaught}`);

  wasm.emu_destroy();

  return {
    game: gameName,
    status: uncaught === 0 ? "PASSED" : "EXCEPTION",
    uncaught_exceptions: uncaught,
    host_panics: 0,
    fb_pubs: fbPubs,
    paint_calls: paintCalls
  };
}

async function main() {
  const wasmBuffer = fs.readFileSync(WASM_PATH);
  const wasmModule = await WebAssembly.instantiate(wasmBuffer, {});

  const btRes = await testGame(wasmModule.instance, "Bounce Tales", BOUNCE_JAR);
  const popRes = await testGame(wasmModule.instance, "Prince of Persia", POP_JAR);

  const totalUncaught = btRes.uncaught_exceptions + popRes.uncaught_exceptions;
  const totalPanics = btRes.host_panics + popRes.host_panics;
  const allPassed = btRes.status === "PASSED" && popRes.status === "PASSED";

  console.log("\n============================================================");
  console.log("   WASM 10-SECOND SMOKE TEST RESULTS");
  console.log("============================================================");
  console.log(`Status:              ${allPassed ? "PASSED" : "FAILED"}`);
  console.log(`Uncaught Exceptions: ${totalUncaught}`);
  console.log(`Host Panics:         ${totalPanics}`);
  console.log("============================================================\n");
}

main().catch(err => {
  console.error("FATAL WASM TEST ERROR:", err);
  process.exit(1);
});
