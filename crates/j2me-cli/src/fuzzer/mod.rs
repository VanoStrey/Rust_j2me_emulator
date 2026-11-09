//! Chaos Monkey Sentinel - Autonomous AI Testing and Fuzzing Framework for J2ME.

pub mod anomaly_sentinel;
pub mod batch;
pub mod chaos_agent;
pub mod replay_recorder;
pub mod triage;

pub use batch::run_batch_fuzzer;
pub use triage::run_triage_manual;

use anomaly_sentinel::{AnomalyReport, AnomalySentinel};
use chaos_agent::ChaosAgent;
use j2me_stdlib::J2meEmulator;
use replay_recorder::{FuzzAction, ReplayRecorder};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FuzzSummary {
    pub game_id: String,
    pub seed: u64,
    pub ticks_executed: usize,
    pub events_injected: usize,
    pub patterns_triggered: usize,
    pub instructions_total: u64,
    pub anomaly: Option<AnomalyReport>,
    pub replay_path: Option<PathBuf>,
}

pub fn run_fuzzer(
    jar_path: &str,
    ticks: usize,
    seed: u64,
    dump_dir: &str,
) -> Result<FuzzSummary, String> {
    println!("------------------------------------------------------------");
    println!(" [CHAOS MONKEY FUZZER] Initializing Sentinel Fuzzing Session");
    println!(" JAR: {}", jar_path);
    println!(" Seed: {} | Max Ticks: {}", seed, ticks);
    println!("------------------------------------------------------------");

    let jar_bytes = fs::read(jar_path)
        .map_err(|e| format!("Failed to read JAR from {}: {}", jar_path, e))?;

    let game_id = Path::new(jar_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("game")
        .to_string();

    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Check for optional .jad descriptor alongside .jar
    let jad_candidate = Path::new(jar_path).with_extension("jad");
    if jad_candidate.exists() {
        if let Ok(jad_content) = fs::read_to_string(&jad_candidate) {
            emu.load_jad_str(&jad_content);
            println!(" [FUZZER] Loaded companion JAD descriptor: {:?}", jad_candidate);
        }
    }

    let fname = Path::new(jar_path).file_name().and_then(|s| s.to_str());
    let load_res = emu.load_jar_with_filename(&jar_bytes, fname)?;
    println!(" [FUZZER] MIDlet '{}' loaded successfully.", load_res);

    let mut agent = ChaosAgent::new(seed);
    let mut sentinel = AnomalySentinel::new();
    let mut recorder = ReplayRecorder::new(game_id.clone(), jar_path.to_string(), seed);

    let t_start = Instant::now();
    let mut detected_anomaly: Option<AnomalyReport> = None;
    let mut audio_buf = vec![0.0f32; 2048];
    let mut saved_replay_path: Option<PathBuf> = None;

    let dump_path = Path::new(dump_dir);
    if !dump_path.exists() {
        let _ = fs::create_dir_all(dump_path);
    }

    for tick in 1..=ticks {
        agent.step_with_sentinel(tick, &mut emu, &mut recorder, &mut sentinel);

        emu.tick(16.66667);

        // Fetch audio chunk for monitoring
        let num_samples = emu.read_audio_samples(&mut audio_buf);
        let audio_slice = &audio_buf[..num_samples];

        // Sentinel invariant checks
        if let Some(anomaly) = sentinel.check_tick(tick, &mut emu, audio_slice) {
            eprintln!("\n🚨 [ANOMALY DETECTED] at tick {}: {}", tick, anomaly.summary);
            let path = recorder
                .save_crash_report(anomaly.clone(), tick)
                .map_err(|e| format!("Failed to save crash log: {}", e))?;
            println!(" [FUZZER] Saved deterministic replay scenario: {:?}", path);
            saved_replay_path = Some(path);
            detected_anomaly = Some(anomaly);
            break;
        }

        if tick % 200 == 0 || tick == ticks {
            let non_black = emu
                .context
                .framebuffer
                .iter()
                .filter(|&&p| (p & 0x00FFFFFF) != 0)
                .count();
            println!(
                " [FUZZER] Tick {:04}/{} | Events: {} | Patterns: {} | Non-black: {}/76800",
                tick, ticks, agent.total_events_sent, agent.patterns_triggered, non_black
            );
        }

        let has_pending_timers = emu.context.timers.active_tasks_count() > 0;
        let has_active_ui = emu.context.current_displayable.is_some();
        let has_pending_ui = !emu.context.ui_queue.is_empty();
        let is_destroyed = !emu.is_running;
        if is_destroyed || (emu.scheduler.active_thread_count() == 0 && !has_pending_timers && !has_active_ui && !has_pending_ui && tick > 10) {
            eprintln!(" [FUZZER] All threads terminated naturally at tick {}.", tick);
            break;
        }
    }

    let elapsed = t_start.elapsed().as_secs_f64();
    println!("------------------------------------------------------------");
    println!(" [CHAOS MONKEY SUMMARY]");
    println!(" Game: {}", game_id);
    println!(" Elapsed Time: {:.2}s ({:.1} ticks/s)", elapsed, ticks as f64 / elapsed.max(0.001));
    println!(" Total Injected Events: {}", agent.total_events_sent);
    println!(" Chaos Patterns Triggered: {}", agent.patterns_triggered);
    println!(" Total Instructions Executed: {}", emu.scheduler.total_instructions());
    let (verdict, flags) = sentinel.evaluate_verdict(&emu, ticks, &detected_anomaly);
    println!(" Sentinel 2.0 Verdict: {} (Flags: {:?})", verdict, flags);
    println!(" Unique Visual Scenes: {}", sentinel.unique_scene_hashes.len());
    println!(" Causal Input Reactions: {}/{}", sentinel.causal_input_reactions, sentinel.causal_probes_sent);
    println!(" Swallowed Critical Exceptions: {}", sentinel.critical_swallowed_exceptions);
    if let Some(ref anom) = detected_anomaly {
        println!(" Verdict: ANOMALY_FOUND ({:?})", anom.kind);
        if let Some(ref p) = saved_replay_path {
            println!(" Replay File: {:?}", p);
        }
    } else {
        println!(" Verdict: CLEAN (Zero crashes or deadlocks encountered)");
    }
    println!("------------------------------------------------------------");

    Ok(FuzzSummary {
        game_id,
        seed,
        ticks_executed: ticks,
        events_injected: agent.total_events_sent,
        patterns_triggered: agent.patterns_triggered,
        instructions_total: emu.scheduler.total_instructions(),
        anomaly: detected_anomaly,
        replay_path: saved_replay_path,
    })
}

pub fn run_replay(replay_path: &str, custom_ticks: Option<usize>) -> Result<(), String> {
    println!("============================================================");
    println!(" [REPLAY RUNNER] Reproducing Bug Scenario from JSON");
    println!(" File: {}", replay_path);
    println!("============================================================");

    let log = ReplayRecorder::load_from_file(Path::new(replay_path))?;
    println!(" Replay target game: {}", log.game_id);
    println!(" Original Seed: {}", log.seed);
    println!(" Target JAR: {}", log.jar_path);
    if let Some(ref anom) = log.anomaly {
        println!(" Original Anomaly: {} (Tick {})", anom.summary, anom.tick);
    }

    let jar_bytes = fs::read(&log.jar_path)
        .map_err(|e| format!("Failed to read JAR {}: {}", log.jar_path, e))?;

    let mut emu = J2meEmulator::new(240, 320, 44100);

    let jad_candidate = Path::new(&log.jar_path).with_extension("jad");
    if jad_candidate.exists() {
        if let Ok(jad_content) = fs::read_to_string(&jad_candidate) {
            emu.load_jad_str(&jad_content);
        }
    }

    let fname = Path::new(&log.jar_path).file_name().and_then(|s| s.to_str());
    emu.load_jar_with_filename(&jar_bytes, fname)?;

    // Map replay events by tick
    let mut event_map: std::collections::HashMap<usize, Vec<FuzzAction>> = std::collections::HashMap::new();
    for ev in log.events {
        event_map.entry(ev.tick).or_default().push(ev.action);
    }

    let total_ticks = custom_ticks.unwrap_or(log.total_ticks);
    println!(" Executing replay across {} ticks...", total_ticks);

    let mut sentinel = AnomalySentinel::new();
    let mut audio_buf = vec![0.0f32; 2048];

    for tick in 1..=total_ticks {
        if let Some(actions) = event_map.get(&tick) {
            for act in actions {
                match act {
                    FuzzAction::Key { key, pressed } => {
                        emu.key_event(*key, *pressed);
                        sentinel.note_input_delivered();
                    }
                    FuzzAction::PauseApp => emu.pause_app(),
                    FuzzAction::ResumeApp => emu.resume_app(),
                }
            }
        }

        emu.tick(16.66667);

        let n = emu.read_audio_samples(&mut audio_buf);
        if let Some(anomaly) = sentinel.check_tick(tick, &mut emu, &audio_buf[..n]) {
            println!(" [REPLAY REPRODUCED] Anomaly at tick {}: {}", tick, anomaly.summary);
            if let Some(exc) = &emu.context.exception_telemetry.last_uncaught {
                println!("   Exception: {} in {}.{}{}:{}", exc.exception_class, exc.class_name, exc.method_name, exc.descriptor, exc.pc);
                if let Some(msg) = &exc.message {
                    println!("   Message: {}", msg);
                }
                for frame in &exc.stack_trace {
                    println!("     at {}", frame);
                }
            }
            for t in emu.scheduler.threads.iter().filter(|t| t.is_alive()) {
                println!("   Thread '{}' id={:?} state={:?} top={:?}", t.name, t.id, t.state, t.current_frame().map(|f| format!("{}.{}:{}", f.class_name, f.method_name, f.pc)));
            }
            for (m_id, mon) in &emu.scheduler.monitors {
                println!("   Monitor {:?} owner={:?} wait_set={:?} entry_set={:?}", m_id, mon.owner, mon.wait_set, mon.entry_set);
            }
            return Ok(());
        }

        if tick % 200 == 0 || tick == total_ticks {
            let non_black = emu
                .context
                .framebuffer
                .iter()
                .filter(|&&p| (p & 0x00FFFFFF) != 0)
                .count();
            println!(
                " [REPLAY] Tick {:04}/{} | Non-black: {}/76800",
                tick, total_ticks, non_black
            );
        }
    }

    println!(" [REPLAY COMPLETE] Finished all {} ticks.", total_ticks);
    Ok(())
}
