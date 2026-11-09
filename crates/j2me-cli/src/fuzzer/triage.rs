//! Grey Zone Inspector - Automated Triage and Ranking for Ambiguous J2ME ROMs.
//!
//! Evaluates sampled games against the Uncertainty Model to surface edge-case titles
//! that require manual human review, eliminating obvious passes and instant crashes.

use super::anomaly_sentinel::SentinelVerdict;
use super::batch::{run_batch_fuzzer, BatchGameRecord};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyCandidate {
    pub rank: usize,
    pub game_id: String,
    pub jar_path: String,
    pub screen_res: String,
    pub verdict: SentinelVerdict,
    pub uncertainty_score: f64,
    pub reasons: Vec<String>,
    pub flags: Vec<String>,
    pub unique_scenes: usize,
    pub causal_reactions: usize,
    pub causal_probes: usize,
    pub causal_ratio_pct: f64,
    pub swallowed_critical_exceptions: u64,
    pub silent_audio: bool,
    pub vendor_stubs: Vec<String>,
    pub instructions: u64,
    pub quick_run_cmd: String,
}

/// Compute the Uncertainty Score in range [0.0, 1.0] along with explanatory signals.
pub fn compute_uncertainty_score(record: &BatchGameRecord) -> (f64, Vec<String>) {
    let mut score = 0.0f64;
    let mut reasons = Vec::new();

    // Signal 1 (+0.35): Low causal reaction to input (< 20%) while display is active
    if record.causal_probes > 0 {
        let ratio = record.causal_reactions as f64 / record.causal_probes as f64;
        if ratio < 0.20 && record.max_non_black_pixels > 0 {
            score += 0.35;
            reasons.push(format!(
                "Low causal input reaction ({:.1}% < 20% across {} probes)",
                ratio * 100.0,
                record.causal_probes
            ));
        }
    } else if record.ticks_executed > 50 && record.max_non_black_pixels > 0 && record.unique_scenes <= 1 {
        score += 0.35;
        reasons.push("Zero input responsiveness detected with active display".to_string());
    }

    // Signal 2 (+0.25): Audio playback attempted but energy is 0.0 (silent audio)
    if record.silent_audio || record.flags.iter().any(|f| f == "FLAG_SILENT_AUDIO") {
        score += 0.25;
        reasons.push("FLAG_SILENT_AUDIO (Audio playback attempted but energy = 0.0)".to_string());
    }

    // Signal 3 (+0.20): Swallowed critical exceptions between 10 and 45
    if record.swallowed_critical_exceptions >= 10 && record.swallowed_critical_exceptions <= 45 {
        score += 0.20;
        reasons.push(format!(
            "Borderline swallowed exceptions: {} (threshold 50)",
            record.swallowed_critical_exceptions
        ));
    }

    // Signal 4 (+0.20): Exactly 2 unique visual scenes (passed splash, potential menu stall)
    if record.unique_scenes == 2 {
        score += 0.20;
        reasons.push("Exactly 2 visual scenes (potential title/menu stall)".to_string());
    }

    // Signal 5 (+0.15): Vendor API stub calls detected (Nokia / Siemens / Motorola / Sony)
    if !record.vendor_stubs.is_empty() {
        score += 0.15;
        let stubs_preview = if record.vendor_stubs.len() <= 3 {
            record.vendor_stubs.join(", ")
        } else {
            format!("{} and {} more", record.vendor_stubs[..2].join(", "), record.vendor_stubs.len() - 2)
        };
        reasons.push(format!("Vendor API stubs called: {}", stubs_preview));
    }

    // Cap score at 1.0
    let clamped_score = (score.min(1.0) * 100.0).round() / 100.0;
    (clamped_score, reasons)
}

/// Run automated triage session to produce top N Grey Zone candidates.
pub fn run_triage_manual(
    roms_dir: &str,
    batch_size: usize,
    select_count: usize,
    ticks: usize,
    seed: u64,
    crashes_dir: &str,
    output_path: &str,
) -> Result<Vec<UncertaintyCandidate>, String> {
    println!("============================================================");
    println!(" 🔍 [GREY ZONE INSPECTOR] Starting Manual Review Triage");
    println!(" ROMs Directory: {}", roms_dir);
    println!(" Batch Size: {} | Select Count: {} | Ticks: {}", batch_size, select_count, ticks);
    println!(" Output Report: {}", output_path);
    println!("============================================================");

    // 1. Run batch fuzzer over the requested batch size
    let summary = run_batch_fuzzer(
        roms_dir,
        batch_size,
        ticks,
        seed,
        crashes_dir,
        None, // Do not write default batch md
    )?;

    println!("\n [TRIAGE] Evaluating Uncertainty Model over {} records...", summary.records.len());

    // 2. Score and filter candidates
    let mut candidates: Vec<UncertaintyCandidate> = Vec::new();

    for record in &summary.records {
        // Filter out instant load errors / tick-0 crashes
        if record.verdict == SentinelVerdict::LoadError || record.ticks_executed == 0 {
            continue;
        }

        let (score, reasons) = compute_uncertainty_score(record);

        // Filter out trivial passes (score == 0.0)
        if score <= 0.0 {
            continue;
        }

        let ratio_pct = if record.causal_probes > 0 {
            (record.causal_reactions as f64 / record.causal_probes as f64) * 100.0
        } else {
            0.0
        };

        let quick_cmd = format!("cargo run --release -p j2me-cli -- \"{}\"", record.jar_path);

        candidates.push(UncertaintyCandidate {
            rank: 0,
            game_id: record.game_id.clone(),
            jar_path: record.jar_path.clone(),
            screen_res: format!("{}x{}", record.screen_width, record.screen_height),
            verdict: record.verdict,
            uncertainty_score: score,
            reasons,
            flags: record.flags.clone(),
            unique_scenes: record.unique_scenes,
            causal_reactions: record.causal_reactions,
            causal_probes: record.causal_probes,
            causal_ratio_pct: ratio_pct,
            swallowed_critical_exceptions: record.swallowed_critical_exceptions,
            silent_audio: record.silent_audio,
            vendor_stubs: record.vendor_stubs.clone(),
            instructions: record.instructions,
            quick_run_cmd: quick_cmd,
        });
    }

    // 3. Sort candidates descending by uncertainty score, then by swallowed exceptions
    candidates.sort_by(|a, b| {
        b.uncertainty_score
            .partial_cmp(&a.uncertainty_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.swallowed_critical_exceptions.cmp(&a.swallowed_critical_exceptions))
            .then_with(|| a.causal_reactions.cmp(&b.causal_reactions))
    });

    // 4. Truncate to top `select_count` and assign ranks
    candidates.truncate(select_count);
    for (i, c) in candidates.iter_mut().enumerate() {
        c.rank = i + 1;
    }

    println!(" [TRIAGE] Identified {} Grey Zone candidates for human audit.", candidates.len());

    // 5. Generate Markdown Report
    write_triage_report(&candidates, roms_dir, batch_size, ticks, output_path)?;
    println!(" [TRIAGE] Saved review queue to: {}", output_path);

    Ok(candidates)
}

fn write_triage_report(
    candidates: &[UncertaintyCandidate],
    roms_dir: &str,
    batch_size: usize,
    ticks: usize,
    file_path: &str,
) -> Result<(), String> {
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dir: {}", e))?;
    }

    let mut md = String::new();
    md.push_str("# 🔍 Grey Zone Inspector: Manual Review Queue\n\n");
    md.push_str(&format!("* **Catalog Analyzed:** `{}`\n", roms_dir));
    md.push_str(&format!("* **Total Batch Sampled:** {} games\n", batch_size));
    md.push_str(&format!("* **Ticks per Session:** {}\n", ticks));
    md.push_str(&format!("* **Selected Candidates:** {} titles\n\n", candidates.len()));

    md.push_str("## 🎯 Executive Review Dashboard\n\n");
    md.push_str("| Rank | Game Name | Uncertainty Score | Primary Suspicions / Signals | Resolution | Quick CLI Launch |\n");
    md.push_str("|:---:|:---|:---:|:---|:---:|:---|\n");

    for c in candidates {
        let reasons_summary = if c.reasons.is_empty() {
            "Ambiguous lifecycle".to_string()
        } else {
            c.reasons.join("; ")
        };

        md.push_str(&format!(
            "| **{}** | `{}` | `⚠️ {:.2}` | {} | {} | `{}` |\n",
            c.rank, c.game_id, c.uncertainty_score, reasons_summary, c.screen_res, c.quick_run_cmd
        ));
    }
    md.push_str("\n---\n\n");

    md.push_str("## 🔬 Detailed Diagnostics & Action Matrix\n\n");

    for c in candidates {
        md.push_str(&format!("### Rank {}: `{}` (Score: `{:.2}`)\n\n", c.rank, c.game_id, c.uncertainty_score));
        md.push_str(&format!("* **ROM Path:** `{}`\n", c.jar_path));
        md.push_str(&format!("* **Verdict:** `{}` | **Flags:** `{:?}`\n", c.verdict, c.flags));
        md.push_str(&format!("* **Resolution:** `{}`\n", c.screen_res));
        md.push_str(&format!("* **VM Instructions Executed:** {}\n", c.instructions));
        md.push_str(&format!(
            "* **Telemetry:** Unique Scenes: **{}** | Causal Reactions: **{}/{}** ({:.1}%) | Swallowed Exceptions: **{}** | Silent Audio: **{}**\n",
            c.unique_scenes, c.causal_reactions, c.causal_probes, c.causal_ratio_pct, c.swallowed_critical_exceptions, c.silent_audio
        ));

        if !c.vendor_stubs.is_empty() {
            md.push_str(&format!("* **Vendor Stubs Triggered:** `{}`\n", c.vendor_stubs.join(", ")));
        }

        md.push_str("\n**Primary Suspicions:**\n");
        for r in &c.reasons {
            md.push_str(&format!("- ⚠️ {}\n", r));
        }

        md.push_str("\n**Reproduction & Audit Commands:**\n");
        md.push_str("```bash\n");
        md.push_str(&format!("# 1. Run game interactively in CLI with frame dumping:\n{}\n\n", c.quick_run_cmd));
        md.push_str(&format!(
            "# 2. Re-run Sentinel 2.0 with Chaos Monkey:\ncargo run --release -p j2me-cli -- fuzz --jar \"{}\" --ticks 1000\n",
            c.jar_path
        ));
        md.push_str("```\n\n");
        md.push_str("---\n\n");
    }

    fs::write(file_path, md).map_err(|e| format!("Failed to write triage report to {}: {}", file_path, e))?;
    Ok(())
}
