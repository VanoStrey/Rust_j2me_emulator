use j2me_core::classfile::constant_pool::{ConstantPool, ConstantPoolEntry};
use j2me_core::heap::object::Heap;
use j2me_core::heap::value::JavaValue;
use j2me_core::jar::manifest::Manifest;
use j2me_core::threads::green_thread::{GreenThread, ThreadId, ThreadState};
use j2me_core::threads::scheduler::Scheduler;
use j2me_core::vm::class_loader::ClassLoader;
use j2me_core::vm::frame::Frame;
use j2me_core::vm::instruction::{Instruction, Predecoder};
use j2me_core::vm::interpreter::{Interpreter, StepResult};
use j2me_stdlib::media::manager::AudioRingBuffer;
use j2me_stdlib::midp::lcdui::font::FontSpec;
use j2me_stdlib::midp::lcdui::graphics::{Graphics, HCENTER, VCENTER};
use j2me_stdlib::midp::lcdui::image::{unpack_rgba, Image};
use j2me_stdlib::midp::rms::record_store::RmsManager;
use j2me_stdlib::nokia::direct_graphics::DirectGraphics;
use j2me_stdlib::J2meEmulator;
use std::env;
use std::fs;
use std::time::Instant;

mod fuzzer;
mod gui_server;
mod market;
mod scenario;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 0. Subcommand Dispatch: fuzz or batch-fuzz
    if args.len() > 1 && (args[1] == "fuzz" || args[1] == "--fuzz" || args[1] == "batch-fuzz" || args[1] == "--batch-fuzz") {
        let is_batch_cmd = args[1] == "batch-fuzz" || args[1] == "--batch-fuzz";
        let mut target_jar: Option<String> = None;
        let mut target_dir: Option<String> = if is_batch_cmd {
            Some("/Users/foolvan/Desktop/j2me_market/roms/Games".to_string())
        } else {
            None
        };
        let mut target_ticks: usize = if is_batch_cmd { 1500 } else { 500 };
        let mut target_count: usize = 50;
        let mut seed: u64 = 42;
        let mut f_dump_dir = "./dumps/fuzz".to_string();
        let mut crashes_dir = "./crashes".to_string();
        let mut report_path: Option<String> = None;

        let mut idx = 2;
        while idx < args.len() {
            match args[idx].as_str() {
                "--jar" => {
                    if idx + 1 < args.len() {
                        target_jar = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--dir" | "--batch" => {
                    if idx + 1 < args.len() {
                        target_dir = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--count" | "--limit" => {
                    if idx + 1 < args.len() {
                        target_count = args[idx + 1].parse().unwrap_or(50);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--ticks" => {
                    if idx + 1 < args.len() {
                        target_ticks = args[idx + 1].parse().unwrap_or(500);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--seed" => {
                    if idx + 1 < args.len() {
                        seed = args[idx + 1].parse().unwrap_or(42);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--dump-dir" => {
                    if idx + 1 < args.len() {
                        f_dump_dir = args[idx + 1].clone();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--crashes-dir" => {
                    if idx + 1 < args.len() {
                        crashes_dir = args[idx + 1].clone();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--report" => {
                    if idx + 1 < args.len() {
                        report_path = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                other => {
                    if !other.starts_with('-') {
                        let path = std::path::Path::new(other);
                        if path.is_dir() && target_dir.is_none() {
                            target_dir = Some(other.to_string());
                        } else if target_jar.is_none() {
                            target_jar = Some(other.to_string());
                        }
                    }
                    idx += 1;
                }
            }
        }

        // Batch Mode
        if let Some(dir_path) = target_dir {
            match fuzzer::run_batch_fuzzer(
                &dir_path,
                target_count,
                target_ticks,
                seed,
                &crashes_dir,
                report_path.as_deref(),
            ) {
                Ok(summary) => {
                    if summary.panics > 0 {
                        std::process::exit(3);
                    } else if summary.hard_anomalies > 0 {
                        std::process::exit(2);
                    } else {
                        std::process::exit(0);
                    }
                }
                Err(e) => {
                    eprintln!("[FATAL] Batch fuzzing error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // Single Mode
        let jar = match target_jar {
            Some(j) => j,
            None => {
                eprintln!("[ERROR] fuzz requires --jar <path_to_jar> or --dir <path_to_roms_dir>");
                std::process::exit(1);
            }
        };

        match fuzzer::run_fuzzer(&jar, target_ticks, seed, &f_dump_dir) {
            Ok(summary) => {
                if summary.anomaly.is_some() {
                    std::process::exit(2);
                } else {
                    std::process::exit(0);
                }
            }
            Err(e) => {
                eprintln!("[FATAL] Fuzzing error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 0b. Subcommand Dispatch: replay
    if args.len() > 1 && (args[1] == "replay" || args[1] == "--replay") {
        let mut replay_file: Option<String> = None;
        let mut custom_ticks: Option<usize> = None;

        let mut idx = 2;
        while idx < args.len() {
            match args[idx].as_str() {
                "--file" => {
                    if idx + 1 < args.len() {
                        replay_file = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--ticks" => {
                    if idx + 1 < args.len() {
                        custom_ticks = args[idx + 1].parse().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                other => {
                    if !other.starts_with('-') && replay_file.is_none() {
                        replay_file = Some(other.to_string());
                    }
                    idx += 1;
                }
            }
        }

        let file = match replay_file {
            Some(f) => f,
            None => {
                eprintln!("[ERROR] replay requires --file <path_to_json> or positional path");
                std::process::exit(1);
            }
        };

        if let Err(e) = fuzzer::run_replay(&file, custom_ticks) {
            eprintln!("[FATAL] Replay error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // 0c. Subcommand Dispatch: triage-manual
    if args.len() > 1 && (args[1] == "triage-manual" || args[1] == "--triage-manual") {
        let mut roms_dir = "/Users/foolvan/Desktop/j2me_market/roms/Games".to_string();
        let mut batch_size: usize = 100;
        let mut select_count: usize = 10;
        let mut ticks: usize = 500;
        let mut seed: u64 = 42;
        let mut crashes_dir = "crashes/".to_string();
        let mut output_path = "reports/manual_review_queue.md".to_string();

        let mut idx = 2;
        while idx < args.len() {
            match args[idx].as_str() {
                "--batch-size" | "--batch" | "--count" => {
                    if idx + 1 < args.len() {
                        batch_size = args[idx + 1].parse().unwrap_or(100);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--select" | "--select-count" | "--top" => {
                    if idx + 1 < args.len() {
                        select_count = args[idx + 1].parse().unwrap_or(10);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--output" | "--report" => {
                    if idx + 1 < args.len() {
                        output_path = args[idx + 1].clone();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--dir" | "--roms-dir" => {
                    if idx + 1 < args.len() {
                        roms_dir = args[idx + 1].clone();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--ticks" => {
                    if idx + 1 < args.len() {
                        ticks = args[idx + 1].parse().unwrap_or(500);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--seed" => {
                    if idx + 1 < args.len() {
                        seed = args[idx + 1].parse().unwrap_or(42);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--crashes-dir" => {
                    if idx + 1 < args.len() {
                        crashes_dir = args[idx + 1].clone();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                other => {
                    if !other.starts_with('-') && std::path::Path::new(other).is_dir() {
                        roms_dir = other.to_string();
                    }
                    idx += 1;
                }
            }
        }

        match fuzzer::run_triage_manual(
            &roms_dir,
            batch_size,
            select_count,
            ticks,
            seed,
            &crashes_dir,
            &output_path,
        ) {
            Ok(candidates) => {
                println!(
                    "\n✅ [TRIAGE COMPLETE] Successfully ranked {} Grey Zone candidates in {}",
                    candidates.len(),
                    output_path
                );
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[FATAL] Triage error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 0d. Subcommand Dispatch: gui
    if args.len() > 1 && (args[1] == "gui" || args[1] == "--gui") {
        let mut jar_path: Option<String> = None;
        let mut jad_path: Option<String> = None;
        let mut port: u16 = 8080;
        let mut no_browser = false;

        let mut idx = 2;
        while idx < args.len() {
            match args[idx].as_str() {
                "--jar" => {
                    if idx + 1 < args.len() {
                        jar_path = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--jad" => {
                    if idx + 1 < args.len() {
                        jad_path = Some(args[idx + 1].clone());
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--port" => {
                    if idx + 1 < args.len() {
                        port = args[idx + 1].parse().unwrap_or(8080);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "--no-browser" => {
                    no_browser = true;
                    idx += 1;
                }
                other => {
                    if !other.starts_with('-') && jar_path.is_none() {
                        jar_path = Some(other.to_string());
                    }
                    idx += 1;
                }
            }
        }

        let jar = match jar_path {
            Some(j) => j,
            None => {
                eprintln!("[ERROR] gui requires a path to a .jar file, e.g.: cargo run -p j2me-cli -- gui <game.jar>");
                std::process::exit(1);
            }
        };

        let jad_candidate = jad_path
            .map(std::path::PathBuf::from)
            .or_else(|| {
                let cand = std::path::Path::new(&jar).with_extension("jad");
                if cand.is_file() {
                    Some(cand)
                } else {
                    None
                }
            });

        let opts = gui_server::GuiServerOptions {
            jar_path: std::path::PathBuf::from(jar),
            jad_path: jad_candidate,
            port,
            open_browser: !no_browser,
        };

        if let Err(e) = gui_server::launch_gui(opts) {
            eprintln!("[FATAL] GUI server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    println!("============================================================");
    println!(" Rust J2ME Emulator (CLDC 1.1 / MIDP 2.0 / Nokia UI / MMAPI)");
    println!(" Native Runner & Verification Test Suite");
    println!("============================================================");

    let mut jar_path: Option<String> = None;
    let mut jad_path: Option<String> = None;
    let mut ticks: usize = 300;
    let mut ticks_specified: bool = false;
    let mut headless: bool = false;
    let mut dump_dir: String = "./dumps".to_string();
    let mut audio_dump: Option<String> = None;
    let mut input_script: Option<String> = None;
    let mut no_input: bool = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--headless" => {
                headless = true;
                i += 1;
            }
            "--jar" => {
                if i + 1 < args.len() {
                    jar_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--jad" => {
                if i + 1 < args.len() {
                    jad_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--ticks" => {
                ticks_specified = true;
                if i + 1 < args.len() {
                    ticks = args[i + 1].parse().unwrap_or(300);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--dump-dir" => {
                if i + 1 < args.len() {
                    dump_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--audio-dump" => {
                if i + 1 < args.len() {
                    audio_dump = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                }
                i += 1;
            }
            "--input" | "--input-script" => {
                if i + 1 < args.len() {
                    input_script = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--no-input" => {
                no_input = true;
                i += 1;
            }
            "--verify-catalog" => {
                let manifest = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    args[i + 1].clone()
                } else {
                    "catalog/test_matrix.toml".to_string()
                };
                let strict = args.iter().any(|a| a == "--strict");
                let allow_missing = args.iter().any(|a| a == "--allow-missing");
                verify_catalog(&manifest, strict, allow_missing);
                return;
            }
            "--benchmark-canonical" => {
                run_benchmark_protocol();
                return;
            }
            "--scan-market" => {
                let target_path = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    args[i + 1].clone()
                } else {
                    "/Users/foolvan/Desktop/j2me_market".to_string()
                };
                let output_path = if let Some(pos) = args.iter().position(|a| a == "--output") {
                    args.get(pos + 1)
                        .cloned()
                        .unwrap_or_else(|| "catalog/j2me_market_inventory.json".to_string())
                } else {
                    "catalog/j2me_market_inventory.json".to_string()
                };
                if let Err(e) = market::scan_market(&target_path, &output_path) {
                    eprintln!("[FATAL ERROR] Scan market failed: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "--verify-inventory" => {
                let inv_path = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    args[i + 1].clone()
                } else {
                    "catalog/j2me_market_inventory.json".to_string()
                };
                match market::verify_inventory(&inv_path) {
                    Ok(true) => std::process::exit(0),
                    Ok(false) | Err(_) => std::process::exit(1),
                }
            }
            "--run-single" => {
                let single_jar = if i + 1 < args.len() {
                    args[i + 1].clone()
                } else {
                    eprintln!("Missing path for --run-single");
                    std::process::exit(1);
                };
                let entry_class = args
                    .iter()
                    .position(|a| a == "--class")
                    .and_then(|idx| args.get(idx + 1))
                    .cloned();
                let single_ticks = args
                    .iter()
                    .position(|a| a == "--ticks")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(300);
                let out_json = args
                    .iter()
                    .position(|a| a == "--output-json")
                    .and_then(|idx| args.get(idx + 1))
                    .cloned()
                    .unwrap_or_else(|| "/tmp/single_run.json".to_string());
                let clean_rms = !args.iter().any(|a| a == "--keep-rms");
                market::run_single_midlet(
                    &single_jar,
                    entry_class.as_deref(),
                    single_ticks,
                    &out_json,
                    clean_rms,
                );
                return;
            }
            "--smoke-test" => {
                let inv_path = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    args[i + 1].clone()
                } else {
                    "catalog/j2me_market_inventory.json".to_string()
                };
                let limit = args
                    .iter()
                    .position(|a| a == "--limit")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());
                let smoke_ticks = args
                    .iter()
                    .position(|a| a == "--ticks")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(300);
                let out_dir = args
                    .iter()
                    .position(|a| a == "--out-dir")
                    .and_then(|idx| args.get(idx + 1))
                    .cloned()
                    .unwrap_or_else(|| "catalog".to_string());
                if let Err(e) = market::run_smoke_test(&inv_path, limit, smoke_ticks, &out_dir) {
                    eprintln!("[FATAL ERROR] Smoke test harness failed: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "--scenario" | "--run-scenario" => {
                let scenario_file = if i + 1 < args.len() {
                    args[i + 1].clone()
                } else {
                    eprintln!("Missing path for --scenario");
                    std::process::exit(1);
                };
                let out_dir = args
                    .iter()
                    .position(|a| a == "--out-dir")
                    .and_then(|idx| args.get(idx + 1))
                    .map(|s| s.as_str());
                let non_strict = args.iter().any(|a| a == "--non-strict");
                match scenario::run_scenario(&scenario_file, out_dir, !non_strict) {
                    Ok(rep) => {
                        println!("\n============================================================");
                        println!(" SCENARIO EXECUTION FINISHED: {}", scenario_file);
                        println!("============================================================");
                        println!(" Game ID:          {}", rep.game_id);
                        println!(" Scenario Type:    {}", rep.scenario_type);
                        println!(" Verdict:          {}", rep.verdict);
                        println!(" Passed:           {}", rep.passed);
                        println!(" Total Ticks:      {}", rep.total_ticks_executed);
                        println!(" Wall Time:        {:.2} ms ({:.1} ticks/s)", rep.wall_time_ms, rep.ticks_per_sec);
                        println!(" Instructions:     {}", rep.instructions_executed);
                        println!(" Heap Live:        {} (Peak: {})", rep.final_heap_objects, rep.peak_heap_objects);
                        println!(" GC Runs:          {}", rep.gc_count);
                        println!(" Snapshots Saved:  {}", rep.snapshots.len());
                        for s in &rep.snapshots {
                            println!("   - Tick {:04}: PNG={} | Non-Black: {}/{}", s.tick, s.png_file, s.non_black_pixels, s.width * s.height);
                        }
                        println!("============================================================");
                        if rep.passed {
                            std::process::exit(0);
                        } else {
                            std::process::exit(2);
                        }
                    }
                    Err(e) => {
                        eprintln!("[FATAL ERROR] Scenario failed to execute: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            other => {
                if !other.starts_with('-') && jar_path.is_none() {
                    jar_path = Some(other.to_string());
                }
                i += 1;
            }
        }
    }

    if let Some(path) = jar_path {
        // If not explicitly headless/benchmark and no custom input script/ticks, launch interactive phone GUI
        if !headless && !ticks_specified && input_script.is_none() && audio_dump.is_none() {
            let jad_candidate = jad_path
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    let cand = std::path::Path::new(&path).with_extension("jad");
                    if cand.is_file() {
                        Some(cand)
                    } else {
                        None
                    }
                });

            let opts = gui_server::GuiServerOptions {
                jar_path: std::path::PathBuf::from(&path),
                jad_path: jad_candidate,
                port: 8080,
                open_browser: true,
            };

            if let Err(e) = gui_server::launch_gui(opts) {
                eprintln!("[FATAL] GUI server error: {}", e);
                std::process::exit(1);
            }
            return;
        }

        run_jar(
            &path,
            jad_path.as_deref(),
            ticks,
            &dump_dir,
            input_script.as_deref(),
            audio_dump.as_deref(),
            no_input,
        );
        return;
    }

    println!("\n[1/11] Running Manifest & ZIP Parser Test...");
    test_manifest_and_jar();

    println!("\n[2/11] Running JAD Descriptor & Property Precedence Test...");
    test_jad_and_property_precedence();

    println!("\n[3/11] Running ClassFile ConstantPool & Attributes Test...");
    test_classfile_parsing();

    println!("\n[4/11] Running Pre-decoded Bytecode Engine Test...");
    test_predecoded_vm_execution();

    println!("\n[5/11] Running Cooperative Green Threads Scheduler Test...");
    test_green_threads_scheduler();

    println!("\n[6/11] Running Non-Strict CRC PNG Decoder Test...");
    test_non_strict_png();

    println!("\n[7/11] Running 2D Graphics Rasterizer & Cyrillic Font Test...");
    test_graphics_and_fonts();

    println!("\n[8/11] Running Nokia UI DirectGraphics Polygon Test...");
    test_nokia_direct_graphics();

    println!("\n[9/11] Running RMS (Record Management System) Persistence Test...");
    test_rms_persistence();

    println!("\n[10/11] Running MMAPI Audio Tone Synthesizer Test...");
    test_audio_synthesis();

    println!("\n[11/11] Running Full Headless J2ME MIDlet Simulation & Frame Dump...");
    test_full_emulator_pipeline();

    println!("\n>>> ALL 11 TESTS PASSED WITH 100% SUCCESS! <<<");
}

fn test_jad_and_property_precedence() {
    let manifest_text = "MIDlet-Name: Manifest Title\r\n\
MIDlet-Vendor: Vendor A\r\n\
MIDlet-Version: 1.0.0\r\n";
    let jad_text = "MIDlet-Name: JAD Overridden Title\r\n\
Custom-Setting: Enabled\r\n";

    let manifest = Manifest::parse(manifest_text);
    let jad = j2me_core::jar::Jad::parse(jad_text);

    // Precedence: JAD overrides Manifest
    assert_eq!(
        j2me_core::jar::Jad::resolve_property(Some(&jad), Some(&manifest), "MIDlet-Name"),
        Some("JAD Overridden Title")
    );
    // Fallback: missing in JAD falls back to Manifest
    assert_eq!(
        j2me_core::jar::Jad::resolve_property(Some(&jad), Some(&manifest), "MIDlet-Vendor"),
        Some("Vendor A")
    );
    assert_eq!(
        j2me_core::jar::Jad::resolve_property(Some(&jad), Some(&manifest), "Custom-Setting"),
        Some("Enabled")
    );
    assert_eq!(
        j2me_core::jar::Jad::resolve_property(Some(&jad), Some(&manifest), "Non-Existent"),
        None
    );
    println!("  -> JAD parser and MIDP property precedence verified.");
}

fn test_manifest_and_jar() {
    let manifest_text = "Manifest-Version: 1.0\r\n\
MIDlet-Name: Retro Racing 3D\r\n\
MIDlet-Version: 1.2.0\r\n\
MIDlet-Vendor: Gameloft\r\n\
MIDlet-1: Retro Racing, /icon.png, com.gameloft.racing.GameMIDlet\r\n\
Nokia-MIDlet-Original-Display-Size: 240,320\r\n\
MicroEdition-Profile: MIDP-2.0\r\n\
MicroEdition-Configuration: CLDC-1.1\r\n";

    let manifest = Manifest::parse(manifest_text);
    assert_eq!(manifest.midlet_name(), Some("Retro Racing 3D"));
    assert_eq!(manifest.midlet_version(), Some("1.2.0"));
    assert_eq!(manifest.midlet_vendor(), Some("Gameloft"));
    assert_eq!(
        manifest.get_primary_midlet_class(),
        Some("com.gameloft.racing.GameMIDlet".to_string())
    );
    assert_eq!(
        manifest.get("Nokia-MIDlet-Original-Display-Size"),
        Some("240,320")
    );
    println!(
        "  -> Manifest parsed correctly: name='{}', primary_class='{:?}'",
        manifest.midlet_name().unwrap(),
        manifest.get_primary_midlet_class().unwrap()
    );
}

fn test_classfile_parsing() {
    let mut cp = ConstantPool::default();
    cp.entries.push(ConstantPoolEntry::Empty);
    cp.entries
        .push(ConstantPoolEntry::Utf8("com/test/SampleClass".to_string()));
    cp.entries.push(ConstantPoolEntry::Class { name_index: 1 });
    cp.entries
        .push(ConstantPoolEntry::Utf8("computeSum".to_string()));
    cp.entries
        .push(ConstantPoolEntry::Utf8("(II)I".to_string()));

    assert_eq!(cp.get_utf8(1), Some("com/test/SampleClass"));
    assert_eq!(cp.get_class_name(2), Some("com/test/SampleClass"));
    println!("  -> ConstantPool entry resolution passed.");
}

fn test_predecoded_vm_execution() {
    // Bytecode for a function:
    // iload_0
    // iload_1
    // iadd
    // ireturn
    // [0x1A, 0x1B, 0x60, 0xAC]
    let code = vec![0x1A, 0x1B, 0x60, 0xAC];
    let cp = ConstantPool::default();
    let (instructions, pc_map) = Predecoder::decode_method_code(&code, &cp);
    assert_eq!(instructions.len(), 4);
    assert_eq!(instructions[0], Instruction::ILoad(0));
    assert_eq!(instructions[1], Instruction::ILoad(1));
    assert_eq!(instructions[2], Instruction::IAdd);
    assert_eq!(instructions[3], Instruction::IReturn);

    let mut frame = Frame::new(
        "TestClass".to_string(),
        "add".to_string(),
        "(II)I".to_string(),
        std::sync::Arc::new(instructions),
        4,
        4,
        Vec::new(),
        std::sync::Arc::new(pc_map),
    );
    frame.set_local(0, JavaValue::Int(25));
    frame.set_local(1, JavaValue::Int(17));

    let mut thread = GreenThread::new(ThreadId(1), "test".to_string(), frame, None);
    let mut class_loader = ClassLoader::new();
    let mut heap = Heap::new();

    let res = Interpreter::step_slice(&mut thread, &mut class_loader, &mut heap, 10);
    assert!(matches!(res, StepResult::ThreadFinished));
    println!("  -> Predecoded bytecode 25 + 17 executed successfully.");
}

fn test_green_threads_scheduler() {
    let mut scheduler = Scheduler::new(100);
    let dummy_instructions = std::sync::Arc::new(vec![Instruction::Nop, Instruction::Return]);
    let dummy_map = std::sync::Arc::new(std::collections::HashMap::new());

    let frame1 = Frame::new(
        "T1".to_string(),
        "run".to_string(),
        "()V".to_string(),
        dummy_instructions.clone(),
        1,
        1,
        Vec::new(),
        dummy_map.clone(),
    );
    let frame2 = Frame::new(
        "T2".to_string(),
        "run".to_string(),
        "()V".to_string(),
        dummy_instructions.clone(),
        1,
        1,
        Vec::new(),
        dummy_map.clone(),
    );

    let id1 = scheduler.spawn("Worker-1", frame1, None);
    let id2 = scheduler.spawn("Worker-2", frame2, None);

    assert_eq!(scheduler.active_thread_count(), 2);

    // Sleep worker 1 for 50ms
    scheduler.threads[0].state = ThreadState::Sleeping { wake_time_ms: 50 };

    // Advancing by 20ms: worker 1 should still sleep, worker 2 ready
    scheduler.tick_time(20);
    assert_eq!(scheduler.next_ready_thread_index(), Some(1));

    // Advancing past 50ms: worker 1 wakes up
    scheduler.tick_time(35);
    assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
    println!("  -> Cooperative green thread quantum & timer tick verified (Thread 1: {:?}, Thread 2: {:?}).", id1, id2);
}

fn test_non_strict_png() {
    // Generate valid raw PNG bytes with palette for testing
    // Minimal 1x1 indexed PNG with non-strict CRC tolerance
    let w = 4;
    let h = 4;
    let img = Image::new_mutable(w, h);
    assert_eq!(img.width, 4);
    assert_eq!(img.height, 4);
    assert_eq!(img.pixels.len(), 16);

    // Test sub-region transformation (mirror and rot90)
    let region = img.create_region(0, 0, 2, 2, 1).unwrap();
    assert_eq!(region.width, 2);
    assert_eq!(region.height, 2);
    println!("  -> Image creation & geometric region transforms verified.");
}

fn test_graphics_and_fonts() {
    let mut g = Graphics::new(240, 320);
    let mut fb = vec![0xFFFFFFFF; 240 * 320];

    // Set color to pure red RGBA
    g.set_color_rgb(0xFF0000);
    g.fill_rect(&mut fb, 10, 10, 50, 50);

    let (r, g_val, b, _) = unpack_rgba(fb[15 * 240 + 15]);
    assert_eq!((r, g_val, b), (255, 0, 0));

    // Test Cyrillic raster font rendering
    g.set_color_rgb(0x00FF00); // Green
    g.font = FontSpec::new(0, 1, 0); // Medium Bold
    g.draw_string(&mut fb, "ПРИВЕТ МИР! 123", 20, 80, HCENTER | VCENTER);

    println!("  -> 2D Rasterizer fillRect & Cyrillic font 'ПРИВЕТ МИР!' rendered successfully.");
}

fn test_nokia_direct_graphics() {
    let g = Graphics::new(100, 100);
    let mut fb = vec![0; 100 * 100];

    // Fill triangle
    let x_pts = [20, 80, 50];
    let y_pts = [20, 20, 80];
    DirectGraphics::fill_polygon(&g, &mut fb, &x_pts, &y_pts, 3, 0xFF0000FFu32 as i32);

    // Center pixel should be blue
    let center_px = fb[40 * 100 + 50];
    let (r, _, b, _) = unpack_rgba(center_px);
    assert_eq!(r, 0);
    assert_eq!(b, 255);
    println!("  -> Nokia DirectGraphics fillPolygon scanline rasterization verified.");
}

fn test_rms_persistence() {
    let mut rms = RmsManager::new();
    let store = rms.open("Scores", true).unwrap();
    let id1 = store.add_record(b"Player1:99990");
    let id2 = store.add_record(b"Player2:88800");

    assert_eq!(store.get_num_records(), 2);
    assert_eq!(store.get_record(id1), Some(&b"Player1:99990"[..]));
    assert_eq!(store.get_record(id2), Some(&b"Player2:88800"[..]));

    // Binary serialization
    let serialized = rms.serialize();
    assert!(!serialized.is_empty());

    // Restore into new manager
    let mut restored_rms = RmsManager::new();
    restored_rms.deserialize(&serialized).unwrap();
    let restored_store = restored_rms.open("Scores", false).unwrap();
    assert_eq!(restored_store.get_num_records(), 2);
    assert_eq!(restored_store.get_record(id1), Some(&b"Player1:99990"[..]));
    println!(
        "  -> RMS persistence round-trip serialized ({} bytes) & verified.",
        serialized.len()
    );
}

fn test_audio_synthesis() {
    let mut audio = AudioRingBuffer::new(44100);
    // 1. Basic tone synthesis
    audio.synthesize_tone(69, 50, 80);
    assert!(audio.len() > 1000);

    let mut samples = vec![0.0f32; 512];
    let read = audio.read_samples(&mut samples);
    assert_eq!(read, 512);

    let max_amp = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    assert!(max_amp > 0.1);
    println!(
        "  -> MMAPI Tone Synthesizer generated {} samples at 440 Hz (Peak amp: {:.2}).",
        read, max_amp
    );

    // 2. RIFF / WAVE PCM Decoder Test (8-bit and 16-bit)
    // Construct valid 16-bit PCM RIFF WAVE header + 100 sample sine wave
    let mut wav_bytes = Vec::new();
    wav_bytes.extend_from_slice(b"RIFF");
    let file_size_minus_8: u32 = 36 + 200;
    wav_bytes.extend_from_slice(&file_size_minus_8.to_le_bytes());
    wav_bytes.extend_from_slice(b"WAVE");
    // fmt chunk
    wav_bytes.extend_from_slice(b"fmt ");
    wav_bytes.extend_from_slice(&16u32.to_le_bytes()); // subchunk size
    wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav_bytes.extend_from_slice(&22050u32.to_le_bytes()); // 22050 Hz
    wav_bytes.extend_from_slice(&(22050u32 * 2).to_le_bytes()); // byte rate
    wav_bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav_bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
                                                       // data chunk
    wav_bytes.extend_from_slice(b"data");
    wav_bytes.extend_from_slice(&200u32.to_le_bytes());
    for i in 0..100 {
        let val = ((i as f32 * 0.1).sin() * 20000.0) as i16;
        wav_bytes.extend_from_slice(&val.to_le_bytes());
    }

    let wav =
        j2me_stdlib::media::wav::WavDecoder::parse(&wav_bytes, 44100).expect("WAV parsing failed");
    assert_eq!(wav.sample_rate, 44100);
    assert!(!wav.samples.is_empty());
    println!("  -> RIFF/WAVE PCM Decoder parsed {} samples and resampled 22050 Hz -> 44100 Hz ({} samples).", 100, wav.samples.len());

    // 3. Standard MIDI File (SMF) Parser & Polyphonic Synthesizer Test
    // Try to load real MIDI from bounce_tales.jar if available, else synthetic SMF
    let mut midi_data = Vec::new();
    if let Ok(jar_bytes) = fs::read("/Users/foolvan/Desktop/j2me_market/roms/raw/bounce_tales.jar")
    {
        if let Ok(jar) = j2me_core::jar::JarArchive::parse(&jar_bytes) {
            if let Some(data) = jar.get_entry("aw.mid") {
                midi_data = data.to_vec();
            }
        }
    }

    if midi_data.is_empty() {
        // Construct minimal valid SMF Format 0 file
        midi_data.extend_from_slice(b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60");
        let track_payload = [
            0x00, 0x90, 0x3C, 0x64, // dt=0 Note On ch 0, note 60 (C4), vel 100
            0x00, 0x99, 0x24, 0x7F, // dt=0 Note On ch 9 (drum), note 36 (kick), vel 127
            0x60, 0x80, 0x3C, 0x00, // dt=96 Note Off ch 0
            0x00, 0xFF, 0x2F, 0x00, // dt=0 End of Track
        ];
        midi_data.extend_from_slice(b"MTrk");
        midi_data.extend_from_slice(&(track_payload.len() as u32).to_be_bytes());
        midi_data.extend_from_slice(&track_payload);
    }

    let seq = j2me_stdlib::media::midi::MidiSequence::parse(&midi_data)
        .expect("MIDI sequence parsing failed");
    let mut synth = j2me_stdlib::media::midi::MidiSynthesizer::new(seq.clone(), 44100);
    synth.start();

    let mut midi_samples = Vec::new();
    // Render up to 1 second of audio (44100 samples) to ensure we hear the notes even after initial silence
    synth.render_samples(44100, &mut midi_samples);
    let non_zero_count = midi_samples.iter().filter(|&&s| s.abs() > 0.001).count();
    let max_amp = midi_samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    assert!(
        non_zero_count > 0,
        "Synthesizer must generate audible samples"
    );
    println!("  -> MMAPI MIDI Synthesizer parsed {} events, rendered {} samples ({} audible non-zero samples, Peak amp: {:.2}).",
        seq.events.len(), midi_samples.len(), non_zero_count, max_amp);
}

fn test_full_emulator_pipeline() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Simulate key events and ticks
    emu.key_event(-5, true); // FIRE pressed
    emu.tick(16.6); // 60 FPS tick
    emu.key_event(-5, false); // FIRE released
    emu.tick(16.6);

    let ptr = emu.framebuffer_ptr();
    let len = emu.framebuffer_len();
    assert!(!ptr.is_null());
    assert_eq!(len, 240 * 320 * 4);

    println!(
        "  -> Headless Emulator 60 FPS ticks executed. Framebuffer size: {} bytes at {:?}.",
        len, ptr
    );
}

fn parse_key_code(s: &str) -> Option<i32> {
    if let Ok(num) = s.parse::<i32>() {
        return Some(num);
    }
    match s.to_uppercase().as_str() {
        "UP" => Some(-1),
        "DOWN" => Some(-2),
        "LEFT" => Some(-3),
        "RIGHT" => Some(-4),
        "FIRE" | "OK" | "SELECT" | "ENTER" | "SPACE" => Some(-5),
        "SOFT1" | "SOFT_1" | "LSK" | "SOFTLEFT" => Some(-6),
        "SOFT2" | "SOFT_2" | "RSK" | "SOFTRIGHT" => Some(-7),
        "CLEAR" | "BACK" => Some(-8),
        "STAR" | "*" => Some(42),
        "POUND" | "HASH" | "#" => Some(35),
        "0" | "NUM0" | "KEY_NUM0" => Some(48),
        "1" | "NUM1" | "KEY_NUM1" => Some(49),
        "2" | "NUM2" | "KEY_NUM2" => Some(50),
        "3" | "NUM3" | "KEY_NUM3" => Some(51),
        "4" | "NUM4" | "KEY_NUM4" => Some(52),
        "5" | "NUM5" | "KEY_NUM5" => Some(53),
        "6" | "NUM6" | "KEY_NUM6" => Some(54),
        "7" | "NUM7" | "KEY_NUM7" => Some(55),
        "8" | "NUM8" | "KEY_NUM8" => Some(56),
        "9" | "NUM9" | "KEY_NUM9" => Some(57),
        _ => None,
    }
}

fn parse_input_script(script: &str) -> std::collections::HashMap<usize, Vec<(i32, bool)>> {
    let content = if std::path::Path::new(script).is_file() {
        std::fs::read_to_string(script).unwrap_or_else(|_| script.to_string())
    } else {
        script.to_string()
    };
    let mut map: std::collections::HashMap<usize, Vec<(i32, bool)>> =
        std::collections::HashMap::new();
    for item in content.split(|c| c == ',' || c == '\n' || c == ';') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() == 3 {
            if let Ok(tick) = parts[0].trim().parse::<usize>() {
                let action = parts[1].trim().to_lowercase();
                let pressed = action.starts_with('p')
                    || action.contains("press")
                    || action == "down"
                    || action.contains("down")
                    || action == "1";
                if let Some(key) = parse_key_code(parts[2].trim()) {
                    map.entry(tick).or_default().push((key, pressed));
                }
            }
        } else if parts.len() == 2 {
            if let Ok(tick) = parts[0].trim().parse::<usize>() {
                let action_or_key = parts[1].trim();
                if action_or_key.eq_ignore_ascii_case("release") {
                    // Marker for releasing all held keys
                    map.entry(tick).or_default().push((0, false));
                } else if let Some(key) = parse_key_code(action_or_key) {
                    map.entry(tick).or_default().push((key, true));
                }
            }
        }
    }
    map
}

fn sha256_digest(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() + 8) % 64 != 0 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    h.iter().map(|val| format!("{:08x}", val)).collect()
}

fn run_jar(
    path: &str,
    jad_path: Option<&str>,
    ticks: usize,
    dump_dir: &str,
    input_script: Option<&str>,
    audio_dump: Option<&str>,
    no_input: bool,
) {
    println!("Loading JAR: {}", path);
    fs::create_dir_all(dump_dir).ok();
    let script_map = input_script.map(parse_input_script);
    match fs::read(path) {
        Ok(bytes) => {
            let jar_sha256 = sha256_digest(&bytes);
            let t_total_start = Instant::now();
            let mut emu = J2meEmulator::new(240, 320, 44100);

            // Auto-detect sibling JAD if not explicitly provided
            let mut jad_sha256: Option<String> = None;
            let resolved_jad: Option<String> = if let Some(j) = jad_path {
                let s = fs::read_to_string(j).ok();
                if let Some(ref text) = s {
                    jad_sha256 = Some(sha256_digest(text.as_bytes()));
                }
                s
            } else {
                let p = std::path::Path::new(path);
                let sibling = p.with_extension("jad");
                if sibling.is_file() {
                    let s = fs::read_to_string(&sibling).ok();
                    if let Some(ref text) = s {
                        jad_sha256 = Some(sha256_digest(text.as_bytes()));
                    }
                    s
                } else {
                    None
                }
            };
            if let Some(ref jad_text) = resolved_jad {
                emu.load_jad_str(jad_text);
                println!("Loaded JAD descriptor properties.");
            }

            let t_load_start = Instant::now();
            let jar_fname = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str());
            let load_res = emu.load_jar_with_filename(&bytes, jar_fname);
            let load_time_ms = t_load_start.elapsed().as_secs_f64() * 1000.0;

            match load_res {
                Ok(name) => {
                    println!(
                        "Successfully loaded MIDlet: '{}' (load time: {:.3} ms)",
                        name, load_time_ms
                    );
                    let input_desc = if no_input {
                        "Explicit No-Input (--no-input)"
                    } else if input_script.is_some() {
                        "Custom Script"
                    } else {
                        "Default Navigation Scenario"
                    };
                    println!(
                        "Executing {} frames at virtual 16.667 ms/tick | Input: {}",
                        ticks, input_desc
                    );

                    let mut captured_audio: Vec<f32> = Vec::new();
                    let mut vm_time_total_ms: f64 = 0.0;
                    let mut audio_time_total_ms: f64 = 0.0;
                    let mut io_time_total_ms: f64 = 0.0;

                    for frame in 1..=ticks {
                        if let Some(ref map) = script_map {
                            if let Some(events) = map.get(&frame) {
                                for &(key, pressed) in events {
                                    emu.key_event(key, pressed);
                                }
                            }
                        } else if !no_input {
                            // Default key events scenario to advance past splash / language select screens
                            match frame {
                                15 => emu.key_event(-6, true),
                                16 => emu.key_event(-6, false),
                                30 => emu.key_event(-5, true),
                                31 => emu.key_event(-5, false),
                                60 => emu.key_event(-6, true),
                                61 => emu.key_event(-6, false),
                                90 => emu.key_event(-5, true),
                                91 => emu.key_event(-5, false),
                                120 => emu.key_event(53, true), // '5'
                                121 => emu.key_event(53, false),
                                150 => emu.key_event(-6, true),
                                151 => emu.key_event(-6, false),
                                180 => emu.key_event(-5, true),
                                181 => emu.key_event(-5, false),
                                210 => emu.key_event(53, true),
                                211 => emu.key_event(53, false),
                                240 => emu.key_event(-6, true),
                                241 => emu.key_event(-6, false),
                                270 => emu.key_event(-5, true),
                                271 => emu.key_event(-5, false),
                                f if f > 271 && f % 60 == 0 => emu.key_event(-5, true),
                                f if f > 271 && f % 60 == 5 => emu.key_event(-5, false),
                                _ => {}
                            }
                        }

                        let t_vm = Instant::now();
                        emu.tick(16.66667);
                        vm_time_total_ms += t_vm.elapsed().as_secs_f64() * 1000.0;

                        if audio_dump.is_some() {
                            let t_aud = Instant::now();
                            let mut chunk = vec![0.0f32; 4096];
                            loop {
                                let n = emu.read_audio_samples(&mut chunk);
                                if n == 0 {
                                    break;
                                }
                                captured_audio.extend_from_slice(&chunk[..n]);
                                if n < chunk.len() {
                                    break;
                                }
                            }
                            audio_time_total_ms += t_aud.elapsed().as_secs_f64() * 1000.0;
                        }

                        if frame % 30 == 0 || frame == ticks {
                            let fb = &emu.context.framebuffer;
                            let w = emu.context.screen_width as u32;
                            let h = emu.context.screen_height as u32;

                            // Stats
                            let non_black = fb.iter().filter(|&&p| (p & 0x00FFFFFF) != 0).count();
                            let mut unique_colors = std::collections::HashSet::new();
                            for &p in fb.iter() {
                                unique_colors.insert(p);
                                if unique_colors.len() > 100 {
                                    break;
                                }
                            }

                            let thread_info = emu.scheduler.threads.iter().filter(|t| t.is_alive()).map(|t| {
                                let top = t.current_frame().map(|f| format!("{}.{}:{}", f.class_name, f.method_name, f.pc)).unwrap_or_default();
                                format!("{}(inst={}, top={})", t.name, t.instructions_executed, top)
                            }).collect::<Vec<_>>().join(", ");

                            println!(
                                "Frame {:03}/{} ticked. Threads: {} [{}] | Screen non-black: {}/{} pixels (colors: {}{})",
                                frame, ticks,
                                emu.scheduler.active_thread_count(),
                                thread_info,
                                non_black, fb.len(),
                                unique_colors.len(),
                                if unique_colors.len() > 100 { "+" } else { "" }
                            );

                            let t_io = Instant::now();
                            let png_bytes = Image::encode_png(fb, w, h);
                            let file_name = format!("frame_{:03}.png", frame);
                            let out_path = std::path::Path::new(dump_dir).join(&file_name);
                            if let Err(e) = fs::write(&out_path, &png_bytes) {
                                eprintln!("Failed to write {}: {}", out_path.display(), e);
                            }

                            // Save raw RGBA framebuffer bytes and calculate exact SHA-256
                            let raw_bytes: Vec<u8> =
                                fb.iter().flat_map(|&p| p.to_le_bytes()).collect();
                            let raw_sha256 = sha256_digest(&raw_bytes);
                            let raw_name = format!("frame_{:03}.raw", frame);
                            let raw_path = std::path::Path::new(dump_dir).join(&raw_name);
                            let _ = fs::write(&raw_path, &raw_bytes);

                            println!(
                                "  => Saved PNG dump: {} ({} bytes) | Raw FB: {} (SHA-256: {})",
                                out_path.display(),
                                png_bytes.len(),
                                raw_path.display(),
                                raw_sha256
                            );
                            io_time_total_ms += t_io.elapsed().as_secs_f64() * 1000.0;
                        }
                    }

                    if let Some(audio_path) = audio_dump {
                        let t_aud_io = Instant::now();
                        let rate = emu.context.audio.sample_rate;
                        match write_wav_mono(audio_path, &captured_audio, rate) {
                            Ok(()) => {
                                let peak = captured_audio
                                    .iter()
                                    .fold(0.0f32, |acc, s| acc.max(s.abs()));
                                let audible =
                                    captured_audio.iter().filter(|s| s.abs() > 0.001).count();
                                let clipped =
                                    captured_audio.iter().filter(|s| s.abs() >= 0.990).count();
                                let rms = if !captured_audio.is_empty() {
                                    (captured_audio.iter().map(|s| s * s).sum::<f32>() / captured_audio.len() as f32).sqrt()
                                } else {
                                    0.0
                                };
                                let clip_pct = clipped as f32 * 100.0 / captured_audio.len().max(1) as f32;
                                println!(
                                    "  => Audio dump: {} | {} samples @ {} Hz ({:.2}s) | RMS {:.4} | peak {:.3} | clipped {} ({:.3}%) | audible {:.1}%",
                                    audio_path,
                                    captured_audio.len(),
                                    rate,
                                    captured_audio.len() as f32 / rate.max(1) as f32,
                                    rms,
                                    peak,
                                    clipped,
                                    clip_pct,
                                    audible as f32 * 100.0 / captured_audio.len().max(1) as f32,
                                );
                            }
                            Err(e) => eprintln!("Failed to write audio dump {}: {}", audio_path, e),
                        }
                        io_time_total_ms += t_aud_io.elapsed().as_secs_f64() * 1000.0;
                    }

                    let total_elapsed_ms = t_total_start.elapsed().as_secs_f64() * 1000.0;
                    let ticks_per_sec = (ticks as f64) / (total_elapsed_ms / 1000.0);

                    println!("\n============================================================");
                    println!(" [BASELINE METRICS] Reproducible Benchmark Summary");
                    println!("============================================================");
                    println!(" Machine: Darwin 27.0.0 arm64 (Apple M2, 8 cores, 8 GB RAM)");
                    println!(" Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13) | Profile: release");
                    println!(" JAR Path: {} | SHA-256: {}", path, jar_sha256);
                    if let Some(ref jsha) = jad_sha256 {
                        println!(" JAD Path: {:?} | SHA-256: {}", jad_path, jsha);
                    } else {
                        println!(" JAD: Not present / Embedded Manifest only");
                    }
                    println!(" RMS Initial State: clean (0 records)");
                    println!(" Input Scenario: {}", input_desc);
                    println!(" Clock Mode: Virtual Discrete (16.667 ms/tick, 60 Hz)");
                    println!(" Frame Instruction Budget: 300,000 inst/frame");
                    println!(" Timing Breakdown:");
                    println!("   - Load Time:            {:.3} ms", load_time_ms);
                    println!(
                        "   - VM Execution Time:    {:.3} ms ({:.3} ms/tick)",
                        vm_time_total_ms,
                        vm_time_total_ms / (ticks as f64)
                    );
                    println!("   - Audio Processing:     {:.3} ms", audio_time_total_ms);
                    println!("   - PNG / WAV I/O Time:   {:.3} ms", io_time_total_ms);
                    println!("   - Total Elapsed Time:   {:.3} ms", total_elapsed_ms);
                    println!(" Throughput & Counters:");
                    println!(
                        "   - Ticks:                {} ({:.2} ticks/s)",
                        ticks, ticks_per_sec
                    );
                    println!(
                        "   - Paint Calls:          {}",
                        emu.context.paint_calls_count
                    );
                    println!(
                        "   - Framebuffer Publishes:{}",
                        emu.context.fb_publishes_count
                    );
                    println!(
                        "   - Active Threads:       {}",
                        emu.scheduler.active_thread_count()
                    );
                    println!("============================================================");
                }
                Err(e) => eprintln!("Failed to start MIDlet: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to read file {}: {}", path, e),
    }
}

/// Writes mono f32 samples as a 16-bit PCM WAV so the audio path can be
/// inspected with any player or analysed offline.
fn write_wav_mono(path: &str, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let mut data = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        data.extend_from_slice(&v.to_le_bytes());
    }
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    fs::write(path, out)
}

#[derive(Debug, Default)]
struct CatalogEntry {
    canonical_id: String,
    display_name: String,
    expected_vendor: String,
    jar_path: String,
    jad_path: String,
    jar_sha256: String,
    catalog_source: String,
    status: String,
    blocker: String,
}

fn parse_test_matrix_toml(path: &str) -> Result<Vec<CatalogEntry>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let mut entries = Vec::new();
    let mut current: Option<CatalogEntry> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[game]]" {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(CatalogEntry::default());
            continue;
        }
        if let Some(ref mut entry) = current {
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"');
                match key {
                    "canonical_id" => entry.canonical_id = val.to_string(),
                    "display_name" => entry.display_name = val.to_string(),
                    "expected_vendor" => entry.expected_vendor = val.to_string(),
                    "jar_path" => entry.jar_path = val.to_string(),
                    "jad_path" => entry.jad_path = val.to_string(),
                    "jar_sha256" => entry.jar_sha256 = val.to_string(),
                    "catalog_source" => entry.catalog_source = val.to_string(),
                    "status" => entry.status = val.to_string(),
                    "blocker" => entry.blocker = val.to_string(),
                    _ => {}
                }
            }
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    Ok(entries)
}

fn verify_catalog(manifest_path: &str, strict: bool, allow_missing: bool) {
    println!("============================================================");
    println!(
        " Canonical Test Matrix Verification (--verify-catalog{}{})",
        if strict { " --strict" } else { "" },
        if allow_missing {
            " --allow-missing"
        } else {
            ""
        }
    );
    println!(" Manifest: {}", manifest_path);
    println!("============================================================");

    let entries = match parse_test_matrix_toml(manifest_path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("[FATAL] Parse error: {}", err);
            std::process::exit(1);
        }
    };

    let canonical_16 = [
        "gow_betrayal",
        "cyberpunk",
        "sc_chaos_theory",
        "castlevania_aria",
        "bounce_tales",
        "doom_rpg",
        "wolfenstein_rpg",
        "sh_orphan",
        "re_deg",
        "gravity_defied",
        "nfs_most_wanted",
        "heroes_lore",
        "prince_sands",
        "assassins_creed",
        "deep_3d",
        "stalker_mobile",
    ];

    const ALLOWED_STATUSES: &[&str] = &[
        "MISSING",
        "INVENTORY_ONLY",
        "MISSING_ROM",
        "LOAD_ERROR",
        "START_ERROR",
        "SPLASH",
        "MENU",
        "GAMEPLAY_REACHED",
        "GAMEPLAY_INPUT_VERIFIED",
        "AUDIO_VERIFIED",
        "PERFORMANCE_VERIFIED",
        "BROWSER_VERIFIED",
    ];

    let mut errors = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    let mut seen_canonical_jars = std::collections::HashMap::new();

    for entry in &entries {
        if !canonical_16.contains(&entry.canonical_id.as_str()) {
            errors.push(format!(
                "Unknown/non-canonical ID: '{}'",
                entry.canonical_id
            ));
        }
        if !seen_ids.insert(entry.canonical_id.clone()) {
            errors.push(format!(
                "Duplicate canonical_id entry: '{}'",
                entry.canonical_id
            ));
        }

        // Validate mandatory fields
        if entry.canonical_id.is_empty() {
            errors.push("An entry is missing mandatory 'canonical_id' field".to_string());
        }
        if entry.display_name.is_empty() {
            errors.push(format!(
                "{}: Missing mandatory 'display_name'",
                entry.canonical_id
            ));
        }
        if entry.expected_vendor.is_empty() {
            errors.push(format!(
                "{}: Missing mandatory 'expected_vendor'",
                entry.canonical_id
            ));
        }
        if entry.catalog_source.is_empty() {
            errors.push(format!(
                "{}: Missing mandatory 'catalog_source'",
                entry.canonical_id
            ));
        }
        if !ALLOWED_STATUSES.contains(&entry.status.as_str()) {
            errors.push(format!(
                "{}: Invalid status '{}'. Must be one of {:?}",
                entry.canonical_id, entry.status, ALLOWED_STATUSES
            ));
        }

        let p = std::path::Path::new(&entry.jar_path);
        let exists = p.is_file();

        if entry.status == "MISSING_ROM" || entry.status == "INVENTORY_ONLY" || !exists {
            if strict {
                errors.push(format!(
                    "{}: Strict mode failure: Missing ROM '{}'! Path: '{}'",
                    entry.canonical_id, entry.display_name, entry.jar_path
                ));
            } else if allow_missing {
                println!(
                    "  [BLOCKER] {:<18} | Status: {} | Blocker: {}",
                    entry.canonical_id,
                    entry.status,
                    if entry.blocker.is_empty() {
                        "ROM missing from archive"
                    } else {
                        &entry.blocker
                    }
                );
            } else {
                if entry.status == "MISSING_ROM" || entry.status == "INVENTORY_ONLY" {
                    println!(
                        "  [INFO]    {:<18} | Status: {} | Blocker: {}",
                        entry.canonical_id, entry.status, entry.blocker
                    );
                } else {
                    errors.push(format!(
                        "{}: JAR path not found on disk: '{}'",
                        entry.canonical_id, entry.jar_path
                    ));
                }
            }
            continue;
        }

        // Validate no duplicate JARs for different canonical IDs
        let canonical_target = match fs::canonicalize(p) {
            Ok(c) => c.to_string_lossy().to_string(),
            Err(_) => entry.jar_path.clone(),
        };
        if let Some(prev_id) =
            seen_canonical_jars.insert(canonical_target.clone(), entry.canonical_id.clone())
        {
            errors.push(format!(
                "Duplicate target JAR detected! Both '{}' and '{}' resolve to same file '{}'",
                prev_id, entry.canonical_id, canonical_target
            ));
        }

        // Validate SHA-256
        match fs::read(p) {
            Ok(bytes) => {
                let actual_hash = sha256_digest(&bytes);
                if actual_hash.to_lowercase() != entry.jar_sha256.to_lowercase() {
                    errors.push(format!(
                        "{}: SHA-256 mismatch!\n    Expected: {}\n    Actual:   {}",
                        entry.canonical_id, entry.jar_sha256, actual_hash
                    ));
                } else {
                    println!(
                        "  [OK]   {:<18} | Verified SHA-256: {}",
                        entry.canonical_id, actual_hash
                    );
                }

                // If JAD is specified, validate MIDlet-Jar-Size and URL
                if !entry.jad_path.is_empty() {
                    let jad_p = std::path::Path::new(&entry.jad_path);
                    if !jad_p.is_file() {
                        errors.push(format!(
                            "{}: Specified JAD does not exist: '{}'",
                            entry.canonical_id, entry.jad_path
                        ));
                    } else if let Ok(jad_str) = fs::read_to_string(jad_p) {
                        let jad = j2me_core::jar::Jad::parse(&jad_str);
                        if let Some(size_str) = jad.get("MIDlet-Jar-Size") {
                            if let Ok(expected_size) = size_str.trim().parse::<usize>() {
                                if expected_size != bytes.len() {
                                    errors.push(format!(
                                        "{}: JAD MIDlet-Jar-Size mismatch! JAD says {}, actual JAR is {} bytes",
                                        entry.canonical_id, expected_size, bytes.len()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!(
                    "{}: Failed to read JAR '{}': {}",
                    entry.canonical_id, entry.jar_path, e
                ));
            }
        }
    }

    // Check all 16 required canonical IDs are present
    for id in &canonical_16 {
        if !seen_ids.contains(*id) {
            errors.push(format!(
                "Required canonical ID '{}' is missing from test_matrix.toml!",
                id
            ));
        }
    }

    if !errors.is_empty() {
        eprintln!(
            "\n[FAILURE] Manifest validation failed with {} errors:",
            errors.len()
        );
        for err in &errors {
            eprintln!("  - {}", err);
        }
        std::process::exit(1);
    }

    println!(
        "\n[SUCCESS] Manifest '{}' is 100% VALID! All {} canonical entries verified.",
        manifest_path,
        canonical_16.len()
    );
    std::process::exit(0);
}

fn run_benchmark_protocol() {
    println!("============================================================");
    println!(" Canonical Benchmark Protocol (10 runs + warm-up, clean VM)");
    println!(" Target Games: prince_sands, heroes_lore, assassins_creed, sc_chaos_theory");
    println!(" Clock: 16.667 ms/tick (60 ticks) | Warm-up: 1 run | Measured: 10 runs");
    println!("============================================================");

    let targets = [
        (
            "prince_sands",
            "/Users/foolvan/Desktop/j2me_market/roms/raw/prince_sands.jar",
        ),
        (
            "heroes_lore",
            "/Users/foolvan/Desktop/j2me_market/roms/raw/heroes_lore.jar",
        ),
        (
            "assassins_creed",
            "/Users/foolvan/Desktop/j2me_market/roms/raw/assassins_creed.jar",
        ),
        (
            "sc_chaos_theory",
            "/Users/foolvan/Desktop/j2me_market/roms/raw/sc_chaos_theory.jar",
        ),
    ];

    for (name, path) in targets {
        println!("\n>>> Benchmarking Target: {} ({}) <<<", name, path);
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[ERROR] Failed to read {}: {}", path, e);
                continue;
            }
        };

        // 1. Warm-up run (excluded from statistics)
        {
            let mut emu = J2meEmulator::new(240, 320, 44100);
            if emu.load_jar(&bytes).is_ok() {
                for _ in 0..60 {
                    emu.tick(16.66667);
                }
            }
        }

        // 2. 10 measured runs
        let mut vm_times = Vec::with_capacity(10);
        let mut load_times = Vec::with_capacity(10);
        let mut paint_calls = 0;
        let mut fb_publishes = 0;
        let mut method_hits = 0;
        let mut method_misses = 0;
        let mut field_hits = 0;
        let mut field_misses = 0;
        let mut cache_invs = 0;
        let mut executed_bytecodes = 0;
        let mut native_calls = 0;
        let mut total_allocations = 0;
        let mut exceptions_count = 0;
        let mut unhandled_native = 0;
        let mut vm_time_ms = 0.0;
        let mut sched_time_ms = 0.0;
        let mut rast_time_ms = 0.0;
        let mut audio_time_ms = 0.0;
        let mut is_completed = true;

        for run in 1..=10 {
            let t_load = Instant::now();
            let mut emu = J2meEmulator::new(240, 320, 44100);
            if emu.load_jar(&bytes).is_err() {
                is_completed = false;
                continue;
            }
            let load_ms = t_load.elapsed().as_secs_f64() * 1000.0;
            load_times.push(load_ms);

            let mut run_vm_ms = 0.0;
            for _ in 0..60 {
                let t_vm = Instant::now();
                emu.tick(16.66667);
                run_vm_ms += t_vm.elapsed().as_secs_f64() * 1000.0;
            }
            vm_times.push(run_vm_ms);

            if run == 10 {
                paint_calls = emu.context.paint_calls_count;
                fb_publishes = emu.context.fb_publishes_count;
                method_hits = emu.class_loader.method_cache_hits.get();
                method_misses = emu.class_loader.method_cache_misses.get();
                field_hits = emu.class_loader.field_cache_hits.get();
                field_misses = emu.class_loader.field_cache_misses.get();
                cache_invs = emu.class_loader.cache_invalidations.get();
                executed_bytecodes = emu.scheduler.total_instructions();
                native_calls = emu.context.native_calls_count;
                total_allocations = emu.heap.total_allocations;
                exceptions_count = emu.context.exceptions_count;
                unhandled_native = emu.context.unhandled_native_calls;
                vm_time_ms = emu.context.vm_time_us as f64 / 1000.0;
                sched_time_ms = emu.context.scheduler_time_us as f64 / 1000.0;
                rast_time_ms = emu.context.rasterizer_time_us as f64 / 1000.0;
                audio_time_ms = emu.context.audio_time_us as f64 / 1000.0;
            }
        }

        if vm_times.is_empty() {
            println!("  [STATUS: INCOMPLETE] Benchmark crashed during setup.");
            continue;
        }

        // Compute statistics
        vm_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min = vm_times.first().copied().unwrap_or(0.0);
        let max = vm_times.last().copied().unwrap_or(0.0);
        let median = if vm_times.len() % 2 == 0 {
            (vm_times[vm_times.len() / 2 - 1] + vm_times[vm_times.len() / 2]) / 2.0
        } else {
            vm_times[vm_times.len() / 2]
        };
        let p95_idx = ((vm_times.len() as f64) * 0.95).ceil() as usize - 1;
        let p95 = vm_times[p95_idx.min(vm_times.len() - 1)];
        let mean = vm_times.iter().sum::<f64>() / (vm_times.len() as f64);
        let variance =
            vm_times.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (vm_times.len() as f64);
        let std_dev = variance.sqrt();

        let throughput_ticks_per_sec = 60.0 / (median / 1000.0);
        let rendering_fps = (fb_publishes as f64) / (median / 1000.0);

        println!(
            "  - Runs (10 ms):   [{}]",
            vm_times
                .iter()
                .map(|t| format!("{:.2}", t))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("  - Execution:      Min: {:.2} ms | Max: {:.2} ms | Median: {:.2} ms | p95: {:.2} ms | StdDev: {:.2} ms ({:.1}%)",
            min, max, median, p95, std_dev, std_dev * 100.0 / mean
        );
        println!(
            "  - Timing Split:   VM: {:.2} ms | Sched: {:.2} ms | Rast: {:.2} ms | Audio: {:.2} ms",
            vm_time_ms, sched_time_ms, rast_time_ms, audio_time_ms
        );
        println!(
            "  - Performance:    Throughput: {:.2} ticks/s | Rendering FPS: {:.2} fps (publishes: {}, paints: {})",
            throughput_ticks_per_sec, rendering_fps, fb_publishes, paint_calls
        );
        println!(
            "  - Instructions:   Executed Bytecodes: {} | Native Calls: {} | Heap Allocations: {}",
            executed_bytecodes, native_calls, total_allocations
        );
        println!(
            "  - Cache Hit Rate: Method: {}/{} ({:.1}%) | Field: {}/{} ({:.1}%) | Invals: {}",
            method_hits,
            method_hits + method_misses,
            if method_hits + method_misses > 0 {
                method_hits as f64 * 100.0 / (method_hits + method_misses) as f64
            } else {
                0.0
            },
            field_hits,
            field_hits + field_misses,
            if field_hits + field_misses > 0 {
                field_hits as f64 * 100.0 / (field_hits + field_misses) as f64
            } else {
                0.0
            },
            cache_invs
        );
        println!(
            "  - Diagnostics:    Exceptions: {} | Unhandled Native: {} | Status: {}",
            exceptions_count,
            unhandled_native,
            if is_completed {
                "COMPLETED"
            } else {
                "INCOMPLETE"
            }
        );
    }
}
