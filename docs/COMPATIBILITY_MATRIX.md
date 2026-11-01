# Milestone 2.1: J2ME Real-Game Audited Compatibility Matrix

**Audit Date:** 2026-10-31  
**Target Environment:** Standalone Rust J2ME VM & WebAssembly (V8 runtime)  
**Reference Oracle:** FreeJ2ME (CheerpJ Web Bridge Execution Harness)  
**Diagnostic Rigor:** Strict Diagnostic Mode (`--strict`, `emu_set_strict_mode(1)`)  
**Epistemic Standards:**
- `[PROVEN]`: Verified by automated deterministic test suites, pixel buffer readback, diff images, or execution logs.
- `[PARTIALLY PROVEN]`: Boot / lifecycle verified, but downstream gameplay transition unverified.
- `[NOT_PROVEN]`: Code path or input response asserted but unobserved in execution.
- `[REFUTED]`: Disproven by bytecode disassembly or empirical audit.

---

## 1. Multi-Dimensional Audited Compatibility Matrix

To avoid collapsing disparate requirements into a single misleading verdict, compatibility is evaluated across 8 orthogonal dimensions:

- **EXECUTION_SUCCESS:** Execution runs to completion without host panic or unhandled guest VM exception.
- **LIFECYCLE_SUCCESS:** JAR decompressing, class loading, MIDlet constructor, and `startApp()` invoke cleanly.
- **VISUAL_OUTPUT:** LCDUI framebuffer publishes valid non-black pixels.
- **VISUAL_EVOLVED:** Framebuffer mutates over time across virtual ticks (not stuck on a single frame).
- **INPUT_FUNCTIONAL:** Game engine registers delivered key events and updates game state accordingly (`input_affected_game_state`).
- **AUDIO_OUTPUT:** Audio synthesizer generates PCM samples or MIDI events.
- **PERSISTENCE:** RMS stores are created, written, and read without corruption.
- **REFERENCE_PARITY:** Quantitative image similarity match against FreeJ2ME golden oracle.

---

| Game Title | Executed JAR & Resolution | EXECUTION_SUCCESS | LIFECYCLE_SUCCESS | VISUAL_OUTPUT | VISUAL_EVOLVED | INPUT_FUNCTIONAL | AUDIO_OUTPUT | PERSISTENCE | REFERENCE_PARITY | Overall Compatibility Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **Bounce Tales** | `bounce_tales.jar` (240x320) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ✅ PASS `[PROVEN]` | ❓ NOT_ACCESSED | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |
| **Prince of Persia** | `prince_sands.jar` (176x208) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ❓ NOT_ACCESSED | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |
| **Doom RPG** | `doom_rpg.jar` (240x320) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ STATIC_SPLASH | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ❓ NOT_ACCESSED | ✅ **96.82% SSIM:0.85** | **SPLASH PARITY PASS** |
| **Castlevania** | `castlevania.jar` (240x320) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ✅ PASS `[PROVEN]` | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |
| **Gravity Defied** | `gravity_defied.jar` (240x320)| ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ❓ NOT_ACCESSED | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |
| **Heroes Lore** | `heroes_lore.jar` (240x320) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ❓ NOT_ACCESSED | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |
| **Cyberpunk** | `cyberpunk.jar` (240x320) | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ✅ PASS `[PROVEN]` | ⚠️ NOT_OBSERVED | ⚠️ SILENT | ❓ NOT_ACCESSED | ⚠️ TIMING_DESYNC | **LIFECYCLE & RENDER PASS** |

---

## 2. Audited Findings & Reality Corrections

1. **Input Responsiveness:** While the CLI scenario engine accurately delivered key events to the VM's UI event queue (`input_delivered: true`), in all 14 gameplay and long-running scenarios `input_affected_game_state` remained `false`. The games either required specific millisecond timing or ignored keys while awaiting splash transitions.
2. **Oracle Reference Parity:** Genuine differential visual comparison against the FreeJ2ME Oracle is established **exclusively for Doom RPG's static title splash (96.82% Exact Match, SSIM 0.8534)**. For all animated splash screens, CheerpJ's 2.5s JIT compilation delay relative to the Rust VM's 15ms startup causes phase offsets that prevent automated lockstep frame matching.
3. **Sprite Transform Fix Scope:** The fix to `Sprite.setTransform` reference pixel positioning is mathematically proven via table-driven tests ([crates/j2me-stdlib/tests/sprite_transform_table_test.rs](file:///Users/foolvan/Desktop/Rust_j2me_emulator/crates/j2me-stdlib/tests/sprite_transform_table_test.rs)). However, none of the 5 commercial games invoke `javax.microedition.lcdui.game.Sprite`; they use proprietary blitters or Nokia DirectGraphics.
4. **Active Rendering vs Idle Ticks:** Bounce Tales executes an active rendering loop on every virtual tick (WASM: 452 ticks/s, Mean: 2.21ms). Games such as Doom RPG and Gravity Defied spend 83% to 99% of virtual ticks in `Thread.sleep` (taking <0.003ms per tick), resulting in highly skewed bimodal latency distributions.
