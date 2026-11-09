//! Batch Fuzzing Runner for J2ME Market.
//!
//! Provides automated, high-throughput fuzz testing over large catalogs of games
//! with deterministic game sampling, panic isolation, crash recording, and structured reporting.
//! Upgraded to Sentinel 2.0 with multi-dimensional Verdict Matrix.

use super::anomaly_sentinel::{AnomalyKind, AnomalyReport, AnomalySentinel, SentinelVerdict};
use super::chaos_agent::{ChaosAgent, DeterministicPrng};
use super::replay_recorder::{ReplayLog, ReplayRecorder};
use j2me_stdlib::J2meEmulator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGameRecord {
    pub index: usize,
    pub game_id: String,
    pub jar_path: String,
    pub verdict: SentinelVerdict,
    pub ticks_executed: usize,
    pub max_non_black_pixels: usize,
    pub screen_coverage_pct: f64,
    pub unique_scenes: usize,
    pub causal_reactions: usize,
    pub causal_probes: usize,
    pub swallowed_critical_exceptions: u64,
    pub swallowed_classes: Vec<String>,
    pub audio_energy: f64,
    pub silent_audio: bool,
    pub instructions: u64,
    pub events_injected: usize,
    pub patterns_triggered: usize,
    pub elapsed_secs: f64,
    pub ticks_per_sec: f64,
    pub anomaly_kind: Option<String>,
    pub anomaly_detail: Option<String>,
    pub flags: Vec<String>,
    pub replay_file: Option<String>,
    pub screen_width: u32,
    pub screen_height: u32,
    pub vendor_stubs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFuzzSummary {
    pub total_sampled: usize,
    pub full_gameplay_passes: usize,
    pub menu_reached_passes: usize,
    pub splash_loop_suspicious: usize,
    pub silent_exception_failures: usize,
    pub hard_anomalies: usize,
    pub load_errors: usize,
    pub panics: usize,
    pub total_ticks: usize,
    pub total_instructions: u64,
    pub total_elapsed_secs: f64,
    pub records: Vec<BatchGameRecord>,
}

/// Run batch fuzz testing over a directory of ROMs.
pub fn run_batch_fuzzer(
    roms_dir: &str,
    count: usize,
    ticks_per_game: usize,
    seed: u64,
    crashes_dir: &str,
    report_md_path: Option<&str>,
) -> Result<BatchFuzzSummary, String> {
    let dir = Path::new(roms_dir);
    if !dir.is_dir() {
        return Err(format!("ROM directory not found: {:?}", roms_dir));
    }

    println!("============================================================");
    println!(" 🐵 [CHAOS MONKEY BATCH FUZZER 2.0] Initializing Multi-ROM Run");
    println!(" Directory: {}", roms_dir);
    println!(" Requested Games: {} | Ticks per game: {} | Seed: {}", count, ticks_per_game, seed);
    println!(" Crash Replays Target: {}", crashes_dir);
    println!("============================================================");

    // 1. Discover all .jar files
    let mut all_jars: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory {}: {}", roms_dir, e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("jar") {
                    all_jars.push(path);
                }
            }
        }
    }

    if all_jars.is_empty() {
        return Err(format!("No .jar files found in {:?}", roms_dir));
    }

    // Sort to guarantee deterministic order before sampling
    all_jars.sort();
    println!(" [DISCOVERY] Found {} total JAR ROMs in catalog.", all_jars.len());

    // 2. Deterministic pseudo-random selection with seed
    let sample_count = count.min(all_jars.len());
    let mut prng = DeterministicPrng::new(seed);
    let mut indices: Vec<usize> = (0..all_jars.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = (prng.next_u64() % ((i + 1) as u64)) as usize;
        indices.swap(i, j);
    }
    let selected_jars: Vec<PathBuf> = indices
        .into_iter()
        .take(sample_count)
        .map(|idx| all_jars[idx].clone())
        .collect();

    println!(" [SAMPLE] Selected {} games deterministically using seed {}.", selected_jars.len(), seed);
    println!("------------------------------------------------------------");

    let total_start = Instant::now();
    let mut records: Vec<BatchGameRecord> = Vec::new();

    let mut full_gameplay_passes = 0;
    let mut menu_reached_passes = 0;
    let mut splash_loop_suspicious = 0;
    let mut silent_exception_failures = 0;
    let mut hard_anomalies = 0;
    let mut load_errors = 0;
    let mut panics = 0;
    let mut total_ticks = 0;
    let mut total_instructions = 0u64;

    for (idx, jar_path) in selected_jars.iter().enumerate() {
        let game_num = idx + 1;
        let game_id = jar_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("game")
            .to_string();

        let game_seed = seed.wrapping_add((idx as u64).wrapping_mul(7919));

        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_single_fuzz_game(jar_path, game_seed, ticks_per_game, crashes_dir)
        }));

        let record = match res {
            Ok(Ok(mut rec)) => {
                rec.index = game_num;
                rec
            }
            Ok(Err(load_err)) => BatchGameRecord {
                index: game_num,
                game_id: game_id.clone(),
                jar_path: jar_path.to_string_lossy().to_string(),
                verdict: SentinelVerdict::LoadError,
                ticks_executed: 0,
                max_non_black_pixels: 0,
                screen_coverage_pct: 0.0,
                unique_scenes: 0,
                causal_reactions: 0,
                causal_probes: 0,
                swallowed_critical_exceptions: 0,
                swallowed_classes: Vec::new(),
                audio_energy: 0.0,
                silent_audio: false,
                instructions: 0,
                events_injected: 0,
                patterns_triggered: 0,
                elapsed_secs: 0.0,
                ticks_per_sec: 0.0,
                anomaly_kind: Some("LoadError".to_string()),
                anomaly_detail: Some(load_err),
                flags: vec!["LOAD_ERROR".to_string()],
                replay_file: None,
                screen_width: 240,
                screen_height: 320,
                vendor_stubs: Vec::new(),
            },
            Err(panic_payload) => {
                let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Box<Any> panic without text".to_string()
                };

                // Generate a panic crash log
                let crash_path = Path::new(crashes_dir).join(format!("{}_panic_seed_{}.json", game_id, game_seed));
                let panic_log = ReplayLog {
                    game_id: game_id.clone(),
                    jar_path: jar_path.to_string_lossy().to_string(),
                    seed: game_seed,
                    total_ticks: 0,
                    anomaly: Some(AnomalyReport {
                        tick: 0,
                        kind: AnomalyKind::HostCrash {
                            error: format!("Host panic: {}", panic_msg),
                        },
                        total_instructions: 0,
                        active_threads: 0,
                        summary: format!("Host panic: {}", panic_msg),
                    }),
                    events: Vec::new(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&panic_log) {
                    let _ = fs::create_dir_all(crashes_dir);
                    let _ = fs::write(&crash_path, json);
                }

                BatchGameRecord {
                    index: game_num,
                    game_id: game_id.clone(),
                    jar_path: jar_path.to_string_lossy().to_string(),
                    verdict: SentinelVerdict::Panic,
                    ticks_executed: 0,
                    max_non_black_pixels: 0,
                    screen_coverage_pct: 0.0,
                    unique_scenes: 0,
                    causal_reactions: 0,
                    causal_probes: 0,
                    swallowed_critical_exceptions: 0,
                    swallowed_classes: Vec::new(),
                    audio_energy: 0.0,
                    silent_audio: false,
                    instructions: 0,
                    events_injected: 0,
                    patterns_triggered: 0,
                    elapsed_secs: 0.0,
                    ticks_per_sec: 0.0,
                    anomaly_kind: Some("HostPanic".to_string()),
                    anomaly_detail: Some(panic_msg),
                    flags: vec!["HOST_PANIC".to_string()],
                    replay_file: Some(crash_path.to_string_lossy().to_string()),
                    screen_width: 240,
                    screen_height: 320,
                    vendor_stubs: Vec::new(),
                }
            }
        };

        // Accumulate statistics
        match record.verdict {
            SentinelVerdict::FullGameplayPass => full_gameplay_passes += 1,
            SentinelVerdict::MenuReachedPass => menu_reached_passes += 1,
            SentinelVerdict::SplashLoopSuspicious => splash_loop_suspicious += 1,
            SentinelVerdict::SilentExceptionFailure => silent_exception_failures += 1,
            SentinelVerdict::HardAnomaly => hard_anomalies += 1,
            SentinelVerdict::LoadError => load_errors += 1,
            SentinelVerdict::Panic => panics += 1,
        }
        total_ticks += record.ticks_executed;
        total_instructions += record.instructions;

        // Print single line progress for batch run
        let verdict_str = match record.verdict {
            SentinelVerdict::FullGameplayPass => "\x1b[32m[FULL_PASS]\x1b[0m",
            SentinelVerdict::MenuReachedPass => "\x1b[36m[MENU_PASS]\x1b[0m",
            SentinelVerdict::SplashLoopSuspicious => "\x1b[33m[SPLASH_LOOP]\x1b[0m",
            SentinelVerdict::SilentExceptionFailure => "\x1b[35m[SILENT_EXC]\x1b[0m",
            SentinelVerdict::HardAnomaly => "\x1b[31m[HARD_ANOM]\x1b[0m",
            SentinelVerdict::LoadError => "\x1b[34m[LOAD_ERR]\x1b[0m",
            SentinelVerdict::Panic => "\x1b[41m[PANIC]\x1b[0m",
        };

        let detail_str = if let Some(ref anom) = record.anomaly_kind {
            format!(" -> {}", anom)
        } else {
            let flags_str = if record.flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", record.flags.join(", "))
            };
            format!(
                " | scenes: {} | react: {}/{} | exc: {} | {} px ({:.0}%){}",
                record.unique_scenes,
                record.causal_reactions,
                record.causal_probes,
                record.swallowed_critical_exceptions,
                record.max_non_black_pixels,
                record.screen_coverage_pct,
                flags_str
            )
        };

        println!(
            " [{:02}/{:02}] {:12} {:30} (ticks: {:04}){}",
            game_num, sample_count, verdict_str, truncate_str(&record.game_id, 30), record.ticks_executed, detail_str
        );

        records.push(record);
    }

    let total_elapsed = total_start.elapsed().as_secs_f64();

    let summary = BatchFuzzSummary {
        total_sampled: sample_count,
        full_gameplay_passes,
        menu_reached_passes,
        splash_loop_suspicious,
        silent_exception_failures,
        hard_anomalies,
        load_errors,
        panics,
        total_ticks,
        total_instructions,
        total_elapsed_secs: total_elapsed,
        records,
    };

    println!("============================================================");
    println!(" 📊 [CHAOS MONKEY SENTINEL 2.0 BATCH SUMMARY]");
    println!(" Total Games Tested: {}", summary.total_sampled);
    println!("   🌟 FULL_GAMEPLAY_PASS:        {} ({:.1}%)", summary.full_gameplay_passes, (summary.full_gameplay_passes as f64 / summary.total_sampled as f64) * 100.0);
    println!("   🎮 MENU_REACHED_PASS:         {} ({:.1}%)", summary.menu_reached_passes, (summary.menu_reached_passes as f64 / summary.total_sampled as f64) * 100.0);
    println!("   ⚠️  SPLASH_LOOP_SUSPICIOUS:    {} ({:.1}%)", summary.splash_loop_suspicious, (summary.splash_loop_suspicious as f64 / summary.total_sampled as f64) * 100.0);
    println!("   🔇 SILENT_EXCEPTION_FAILURE:  {} ({:.1}%)", summary.silent_exception_failures, (summary.silent_exception_failures as f64 / summary.total_sampled as f64) * 100.0);
    println!("   🚨 HARD_ANOMALIES (Crashes):   {} ({:.1}%)", summary.hard_anomalies, (summary.hard_anomalies as f64 / summary.total_sampled as f64) * 100.0);
    println!("   ℹ️  LOAD ERRORS:               {} ({:.1}%)", summary.load_errors, (summary.load_errors as f64 / summary.total_sampled as f64) * 100.0);
    println!("   💥 HOST PANICS:                {} ({:.1}%)", summary.panics, (summary.panics as f64 / summary.total_sampled as f64) * 100.0);
    println!(" Total Ticks Simulated:  {} ({:.1} avg ticks/s)", summary.total_ticks, summary.total_ticks as f64 / total_elapsed.max(0.001));
    println!(" Total VM Instructions:  {}", summary.total_instructions);
    println!(" Total Time Elapsed:     {:.2}s", total_elapsed);
    println!("============================================================");

    // 4. Generate Markdown report if requested or default
    let md_path = report_md_path.unwrap_or("reports/batch_fuzz_report.md");
    if let Err(e) = write_markdown_report(&summary, md_path) {
        eprintln!("[WARN] Failed to write Markdown report to {}: {}", md_path, e);
    } else {
        println!(" [REPORT] Full Markdown analysis saved to: {}", md_path);
    }

    // Also write JSON summary
    let json_path = "reports/batch_fuzz_report.json";
    if let Ok(json_str) = serde_json::to_string_pretty(&summary) {
        let _ = fs::write(json_path, json_str);
    }

    Ok(summary)
}

fn run_single_fuzz_game(
    jar_path: &Path,
    seed: u64,
    target_ticks: usize,
    crashes_dir: &str,
) -> Result<BatchGameRecord, String> {
    let jar_bytes = fs::read(jar_path)
        .map_err(|e| format!("Read error: {}", e))?;

    let game_id = jar_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("game")
        .to_string();

    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Check for optional companion .jad descriptor
    let jad_candidate = jar_path.with_extension("jad");
    if jad_candidate.exists() {
        if let Ok(jad_content) = fs::read_to_string(&jad_candidate) {
            emu.load_jad_str(&jad_content);
        }
    }

    let fname = jar_path.file_name().and_then(|s| s.to_str());
    emu.load_jar_with_filename(&jar_bytes, fname)
        .map_err(|e| format!("MIDlet load failed: {}", e))?;

    let mut agent = ChaosAgent::new(seed);
    let mut sentinel = AnomalySentinel::new();
    let mut recorder = ReplayRecorder::new(game_id.clone(), jar_path.to_string_lossy().to_string(), seed);

    let t_start = Instant::now();
    let mut detected_anomaly: Option<AnomalyReport> = None;
    let mut audio_buf = vec![0.0f32; 2048];
    let mut saved_replay_path: Option<String> = None;
    let mut max_non_black = 0usize;
    let mut executed_ticks = 0usize;

    for tick in 1..=target_ticks {
        executed_ticks = tick;
        agent.step_with_sentinel(tick, &mut emu, &mut recorder, &mut sentinel);

        emu.tick(16.66667);

        // Track frame rendering coverage
        let non_black = emu
            .context
            .framebuffer
            .iter()
            .filter(|&&p| (p & 0x00FFFFFF) != 0)
            .count();
        if non_black > max_non_black {
            max_non_black = non_black;
        }

        let num_samples = emu.read_audio_samples(&mut audio_buf);
        let audio_slice = &audio_buf[..num_samples];

        if let Some(anomaly) = sentinel.check_tick(tick, &mut emu, audio_slice) {
            // Save replay
            let _ = fs::create_dir_all(crashes_dir);
            let filename = format!("{}_seed_{}.json", game_id, seed);
            let path = Path::new(crashes_dir).join(filename);

            let log = ReplayLog {
                game_id: game_id.clone(),
                jar_path: jar_path.to_string_lossy().to_string(),
                seed,
                total_ticks: tick,
                anomaly: Some(anomaly.clone()),
                events: recorder.events.clone(),
            };

            if let Ok(json) = serde_json::to_string_pretty(&log) {
                let _ = fs::write(&path, json);
                saved_replay_path = Some(path.to_string_lossy().to_string());
            }

            detected_anomaly = Some(anomaly);
            break;
        }

        let has_pending_timers = emu.context.timers.active_tasks_count() > 0;
        let has_active_ui = emu.context.current_displayable.is_some();
        let has_pending_ui = !emu.context.ui_queue.is_empty();
        let is_destroyed = !emu.is_running;
        if is_destroyed || (emu.scheduler.active_thread_count() == 0 && !has_pending_timers && !has_active_ui && !has_pending_ui && tick > 10) {
            // Natural termination
            break;
        }
    }

    let elapsed = t_start.elapsed().as_secs_f64();
    let coverage_pct = (max_non_black as f64 / 76800.0) * 100.0;
    let tps = executed_ticks as f64 / elapsed.max(0.001);

    let (verdict, flags) = sentinel.evaluate_verdict(&emu, executed_ticks, &detected_anomaly);
    let silent_audio = flags.iter().any(|f| f == "FLAG_SILENT_AUDIO");

    let anom_kind = detected_anomaly.as_ref().map(|a| format!("{:?}", a.kind));
    let anom_detail = detected_anomaly.as_ref().map(|a| a.summary.clone());

    let vendor_stubs: Vec<String> = emu
        .context
        .unhandled_natives
        .iter()
        .filter(|m| {
            let s = m.to_lowercase();
            s.contains("nokia") || s.contains("siemens") || s.contains("motorola") || s.contains("samsung") || s.contains("sony") || s.contains("vodafone")
        })
        .cloned()
        .collect();

    Ok(BatchGameRecord {
        index: 0,
        game_id,
        jar_path: jar_path.to_string_lossy().to_string(),
        verdict,
        ticks_executed: executed_ticks,
        max_non_black_pixels: max_non_black,
        screen_coverage_pct: coverage_pct,
        unique_scenes: sentinel.unique_scene_hashes.len(),
        causal_reactions: sentinel.causal_input_reactions,
        causal_probes: sentinel.causal_probes_sent,
        swallowed_critical_exceptions: sentinel.critical_swallowed_exceptions,
        swallowed_classes: sentinel.caught_exception_classes,
        audio_energy: sentinel.total_audio_energy,
        silent_audio,
        instructions: emu.scheduler.total_instructions(),
        events_injected: agent.total_events_sent,
        patterns_triggered: agent.patterns_triggered,
        elapsed_secs: elapsed,
        ticks_per_sec: tps,
        anomaly_kind: anom_kind,
        anomaly_detail: anom_detail,
        flags,
        replay_file: saved_replay_path,
        screen_width: emu.context.screen_width,
        screen_height: emu.context.screen_height,
        vendor_stubs,
    })
}

fn write_markdown_report(summary: &BatchFuzzSummary, file_path: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut md = String::new();
    md.push_str("# 🐵 Chaos Monkey Sentinel 2.0: Batch Fuzzing Report\n\n");
    md.push_str(&format!("* **Generated At:** {}\n", chrono_timestamp()));
    md.push_str(&format!("* **Total Games Tested:** {}\n", summary.total_sampled));
    md.push_str(&format!("* **Total Emulated Ticks:** {}\n", summary.total_ticks));
    md.push_str(&format!("* **Total VM Bytecode Instructions:** {}\n", summary.total_instructions));
    md.push_str(&format!("* **Elapsed Run Time:** {:.2}s\n\n", summary.total_elapsed_secs));

    md.push_str("## 1. Multi-Dimensional Verdict Matrix\n\n");
    md.push_str("| Verdict Classification | Count | Percentage | Description |\n");
    md.push_str("|---|---|---|---|\n");
    md.push_str(&format!("| **🌟 FULL_GAMEPLAY_PASS** | **{}** | **{:.1}%** | $\\ge 3$ visual phases, proven causal input response, clean runtime |\n", summary.full_gameplay_passes, (summary.full_gameplay_passes as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **🎮 MENU_REACHED_PASS** | **{}** | **{:.1}%** | $\\ge 2$ visual phases or reactive menu, awaiting longer navigation |\n", summary.menu_reached_passes, (summary.menu_reached_passes as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **⚠️ SPLASH_LOOP_SUSPICIOUS** | **{}** | **{:.1}%** | *False Negative Risk:* $< 3$ scenes & zero input reaction (stuck on splash) |\n", summary.splash_loop_suspicious, (summary.splash_loop_suspicious as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **🔇 SILENT_EXCEPTION_FAILURE** | **{}** | **{:.1}%** | *False Negative Risk:* $> 50$ critical exceptions swallowed in `try-catch` |\n", summary.silent_exception_failures, (summary.silent_exception_failures as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **🚨 HARD_ANOMALIES** | **{}** | **{:.1}%** | Deadlock, freeze, black screen, uncaught crash |\n", summary.hard_anomalies, (summary.hard_anomalies as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **ℹ️ LOAD ERRORS** | **{}** | **{:.1}%** | Corrupted JAR or bad manifest |\n", summary.load_errors, (summary.load_errors as f64 / summary.total_sampled as f64) * 100.0));
    md.push_str(&format!("| **💥 HOST PANICS** | **{}** | **{:.1}%** | Isolated host emulator panic |\n\n", summary.panics, (summary.panics as f64 / summary.total_sampled as f64) * 100.0));

    md.push_str("## 2. Replay Instructions for Captured Anomalies\n\n");
    md.push_str("To deterministically reproduce any detected crash or anomaly, run:\n");
    md.push_str("```bash\n");
    md.push_str("cargo run --release -p j2me-cli -- replay --file <path_to_replay_json>\n");
    md.push_str("```\n\n");

    md.push_str("## 3. Detailed Results Table\n\n");
    md.push_str("| # | Game ID | Verdict | Ticks | Scenes | Causal React | Swallowed Exc | Audio | Max Px | Cov | TPS | Instructions | Flags / Notes | Replay |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");

    for r in &summary.records {
        let v_icon = match r.verdict {
            SentinelVerdict::FullGameplayPass => "🌟 FULL_PASS",
            SentinelVerdict::MenuReachedPass => "🎮 MENU_PASS",
            SentinelVerdict::SplashLoopSuspicious => "⚠️ SPLASH_LOOP",
            SentinelVerdict::SilentExceptionFailure => "🔇 SILENT_EXC",
            SentinelVerdict::HardAnomaly => "🚨 HARD_ANOM",
            SentinelVerdict::LoadError => "ℹ️ LOAD_ERR",
            SentinelVerdict::Panic => "💥 PANIC",
        };

        let audio_str = if r.silent_audio {
            "🔇 SILENT"
        } else if r.audio_energy > 0.0 {
            "🔊 ACTIVE"
        } else {
            "⚪ NONE"
        };

        let notes = if !r.flags.is_empty() {
            r.flags.join(", ")
        } else if let Some(ref detail) = r.anomaly_detail {
            detail.clone()
        } else {
            "-".to_string()
        };

        let replay = r.replay_file.as_deref().unwrap_or("-");

        md.push_str(&format!(
            "| {} | `{}` | {} | {} | {} | {}/{} | {} | {} | {} | {:.1}% | {:.0} | {} | {} | `{}` |\n",
            r.index,
            r.game_id,
            v_icon,
            r.ticks_executed,
            r.unique_scenes,
            r.causal_reactions,
            r.causal_probes,
            r.swallowed_critical_exceptions,
            audio_str,
            r.max_non_black_pixels,
            r.screen_coverage_pct,
            r.ticks_per_sec,
            r.instructions,
            notes,
            replay
        ));
    }

    fs::write(file_path, md)
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn chrono_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    format!("Unix Epoch +{}s", duration.as_secs())
}
