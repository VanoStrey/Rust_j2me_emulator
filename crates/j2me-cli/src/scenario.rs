use j2me_stdlib::bridge::context::{RuntimeObservabilityReport, TestVerdict};
use j2me_stdlib::midp::lcdui::image::Image;
use j2me_stdlib::J2meEmulator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn default_tick_step() -> f64 {
    16.66667
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInputEvent {
    pub tick: usize,
    pub key: i32,
    pub action: String, // "press", "release", "down", "up"
    #[serde(default)]
    pub free_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialScenario {
    pub schema_version: String,
    pub game_id: String,
    pub jar_file: String,
    #[serde(default)]
    pub jar_sha256: Option<String>,
    pub resolution: ScenarioResolution,
    pub scenario_type: String,
    pub description: String,
    pub total_ticks: usize,
    #[serde(default = "default_tick_step")]
    pub tick_step_ms: f64,
    #[serde(default)]
    pub inputs: Vec<ScenarioInputEvent>,
    #[serde(default)]
    pub snapshot_ticks: Vec<usize>,
    #[serde(default)]
    pub expected_behavior: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSnapshotMeta {
    pub tick: usize,
    pub png_file: String,
    pub raw_file: String,
    pub raw_sha256: String,
    pub width: u32,
    pub height: u32,
    pub non_black_pixels: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioExecutionReport {
    pub game_id: String,
    pub scenario_type: String,
    pub scenario_file: String,
    pub jar_file: String,
    pub jar_sha256: String,
    pub resolution: ScenarioResolution,
    pub total_ticks_executed: usize,
    pub wall_time_ms: f64,
    pub ticks_per_sec: f64,
    pub instructions_executed: u64,
    pub snapshots: Vec<ScenarioSnapshotMeta>,
    pub criteria_report: RuntimeObservabilityReport,
    pub peak_heap_objects: usize,
    pub final_heap_objects: usize,
    pub gc_count: usize,
    pub verdict: String,
    pub passed: bool,
}

pub fn sha256_bytes(data: &[u8]) -> String {
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

    let mut out = String::with_capacity(64);
    for val in h {
        out.push_str(&format!("{:08x}", val));
    }
    out
}

fn resolve_jar_path(jar_file: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(jar_file),
        PathBuf::from("web").join(jar_file),
        PathBuf::from("../web").join(jar_file),
        PathBuf::from("../../web").join(jar_file),
    ];
    for p in &candidates {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    None
}

pub fn run_scenario(
    scenario_path: &str,
    out_dir_override: Option<&str>,
    strict_mode: bool,
) -> Result<ScenarioExecutionReport, String> {
    let content = fs::read_to_string(scenario_path)
        .map_err(|e| format!("Failed to read scenario file '{}': {}", scenario_path, e))?;
    let scenario: DifferentialScenario = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON scenario '{}': {}", scenario_path, e))?;

    let jar_path = resolve_jar_path(&scenario.jar_file).ok_or_else(|| {
        format!(
            "JAR file '{}' referenced in scenario not found in workspace",
            scenario.jar_file
        )
    })?;

    let jar_bytes = fs::read(&jar_path)
        .map_err(|e| format!("Failed to read JAR '{}': {}", jar_path.display(), e))?;
    let actual_sha = sha256_bytes(&jar_bytes);
    if let Some(ref exp_sha) = scenario.jar_sha256 {
        if !exp_sha.eq_ignore_ascii_case(&actual_sha) {
            return Err(format!(
                "SHA-256 mismatch for '{}': expected {}, got {}",
                jar_path.display(),
                exp_sha,
                actual_sha
            ));
        }
    }

    let scenario_stem = Path::new(scenario_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("scenario");

    let out_dir = if let Some(base) = out_dir_override {
        PathBuf::from(base)
            .join(&scenario.game_id)
            .join(scenario_stem)
    } else {
        PathBuf::from("tests/results")
            .join(&scenario.game_id)
            .join(scenario_stem)
    };
    fs::create_dir_all(&out_dir).map_err(|e| {
        format!(
            "Failed to create output dir '{}': {}",
            out_dir.display(),
            e
        )
    })?;

    let mut emu = J2meEmulator::new(
        scenario.resolution.width,
        scenario.resolution.height,
        44100,
    );
    emu.set_strict_mode(strict_mode);

    emu.load_jar(&jar_bytes)
        .map_err(|e| format!("Failed to load JAR in emulator: {}", e))?;

    // Pre-organize input events by tick
    let mut input_map: HashMap<usize, Vec<(i32, bool)>> = HashMap::new();
    for ev in &scenario.inputs {
        let is_down = ev.action.starts_with('p')
            || ev.action.contains("down")
            || ev.action.contains("press")
            || ev.action == "1";
        input_map.entry(ev.tick).or_default().push((ev.key, is_down));
    }

    let mut snapshots = Vec::new();
    let mut peak_heap_objects = 0;

    let t0 = Instant::now();
    for tick in 1..=scenario.total_ticks {
        // Dispatch scheduled inputs
        if let Some(events) = input_map.get(&tick) {
            for &(key, pressed) in events {
                emu.key_event(key, pressed);
            }
        }

        emu.tick(scenario.tick_step_ms);

        let live_objs = emu.heap.live_objects();
        if live_objs > peak_heap_objects {
            peak_heap_objects = live_objs;
        }

        if scenario.snapshot_ticks.contains(&tick) {
            let fb = &emu.context.framebuffer;
            let w = emu.context.screen_width as u32;
            let h = emu.context.screen_height as u32;
            let non_black = fb.iter().filter(|&&p| (p & 0x00FFFFFF) != 0).count();

            let png_bytes = Image::encode_png(fb, w, h);
            let png_name = format!("rust_tick_{:04}.png", tick);
            let png_path = out_dir.join(&png_name);
            let _ = fs::write(&png_path, &png_bytes);

            let raw_bytes: Vec<u8> = fb.iter().flat_map(|&p| p.to_le_bytes()).collect();
            let raw_sha256 = sha256_bytes(&raw_bytes);
            let raw_name = format!("rust_tick_{:04}.raw", tick);
            let raw_path = out_dir.join(&raw_name);
            let _ = fs::write(&raw_path, &raw_bytes);

            snapshots.push(ScenarioSnapshotMeta {
                tick,
                png_file: png_name,
                raw_file: raw_name,
                raw_sha256,
                width: w,
                height: h,
                non_black_pixels: non_black,
            });
        }
    }
    let wall_time_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let ticks_per_sec = (scenario.total_ticks as f64) / (wall_time_ms / 1000.0).max(0.001);

    let criteria_report = emu.evaluate_observability(false);
    let passed = criteria_report.verdict == TestVerdict::Pass;
    let verdict_str = format!("{:?}", criteria_report.verdict);

    let report = ScenarioExecutionReport {
        game_id: scenario.game_id.clone(),
        scenario_type: scenario.scenario_type.clone(),
        scenario_file: scenario_path.to_string(),
        jar_file: scenario.jar_file.clone(),
        jar_sha256: actual_sha,
        resolution: scenario.resolution.clone(),
        total_ticks_executed: scenario.total_ticks,
        wall_time_ms,
        ticks_per_sec,
        instructions_executed: emu.scheduler.total_instructions(),
        snapshots,
        criteria_report,
        peak_heap_objects,
        final_heap_objects: emu.heap.live_objects(),
        gc_count: emu.gc_count,
        verdict: verdict_str,
        passed,
    };

    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Failed to serialize report: {}", e))?;
    let report_path = out_dir.join("rust_execution.json");
    fs::write(&report_path, report_json)
        .map_err(|e| format!("Failed to write report '{}': {}", report_path.display(), e))?;

    Ok(report)
}
