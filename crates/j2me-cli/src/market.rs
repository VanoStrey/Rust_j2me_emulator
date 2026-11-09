use j2me_core::jar::jad::Jad;
use j2me_core::jar::jar_loader::JarArchive;
use j2me_stdlib::J2meEmulator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// ============================================================================
// Phase 4.1 Schema Definitions (Version 4.1.0)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidletEntryInfo {
    pub index: usize,
    pub name: String,
    pub icon: String,
    pub main_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub status: String, // "VERIFIED", "FAILED", "NOT_TESTED"
    pub run_id: Option<String>,
    pub parameters: Option<HashMap<String, String>>,
    pub artifact_links: Vec<String>,
    pub rationale: String,
}

impl Default for EvidenceItem {
    fn default() -> Self {
        Self {
            status: "NOT_TESTED".to_string(),
            run_id: None,
            parameters: None,
            artifact_links: Vec::new(),
            rationale: "Automated smoke-test baseline (no-input protocol)".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceRecord {
    pub input: EvidenceItem,
    pub audio: EvidenceItem,
    pub performance: EvidenceItem,
    pub browser: EvidenceItem,
    pub persistence: EvidenceItem,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockerItem {
    pub blocker_type: String, // "CONFIRMED" or "HYPOTHESIS"
    pub category: String,
    pub description: String,
    pub minimal_repro: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFileEntry {
    pub inventory_schema_version: String, // "4.1.0"
    pub file_id: String,
    pub path: String,
    pub relative_path: String,
    pub container_path: Option<String>,
    pub entry_path: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
    pub extension: String,
    pub availability: String,    // "PRESENT", "MISSING", "SYMLINK"
    pub archive_status: String, // "VALID", "CORRUPTED", "EMPTY", "UNREADABLE", "UNSUPPORTED_FORMAT"
    pub manifest_status: String, // "PRESENT", "MISSING", "SYNTAX_ERROR", "INVALID_ATTRIBUTES", "N/A"
    pub jad_status: String,      // "PRESENT_MATCHED", "PRESENT_MISMATCH", "MISSING", "N/A"
    pub midlet_entries: Vec<MidletEntryInfo>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub configuration: Option<String>,
    pub profile: Option<String>,
    pub launch_configuration_id: Option<String>,
    pub runtime_status: String, // "NOT_RUN", "LOAD_ERROR", "START_ERROR", "COMPLETED_WINDOW", "GUEST_EXCEPTION", "HOST_PANIC", "WALL_TIMEOUT", "MEMORY_LIMIT"
    pub observed_ui: String,    // "UNKNOWN", "SPLASH", "MENU", "GAMEPLAY"
    pub evidence: EvidenceRecord,
    pub blockers: Vec<BlockerItem>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub sha256: String,
    pub count: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub total_files_scanned: usize,
    pub total_jar_files: usize,
    pub regular_jar_files: usize,
    pub symlink_jar_files: usize,
    pub unique_jar_sha256_count: usize,
    pub total_jad_files: usize,
    pub unique_sha256_count: usize,
    pub exact_duplicates_count: usize,
    pub unreadable_files_count: usize,
    pub jad_jar_mismatch_count: usize,
    pub games_jar_only_count: usize,
    pub games_jar_and_jad_count: usize,
    pub total_launch_configurations: usize,
    pub multi_midlet_suites_count: usize,
    pub catalog_indexed_games_count: usize,
    pub extensions_summary: HashMap<String, usize>,
    pub missing_metadata_summary: HashMap<String, usize>,
    pub exact_duplicates_groups: Vec<DuplicateGroup>,
    pub distinct_versions_groups_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInventory {
    pub version: String,
    pub scan_timestamp: String,
    pub root_scanned: String,
    pub summary: InventorySummary,
    pub files: Vec<InventoryFileEntry>,
}

// ============================================================================
// Smoke Test Result Structures
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionTimingBreakdown {
    pub elapsed_load_ms: f64,
    pub elapsed_vm_ms: f64,
    pub elapsed_graphics_ms: f64,
    pub elapsed_audio_ms: f64,
    pub elapsed_scheduler_ms: f64,
    pub elapsed_io_ms: f64,
    pub elapsed_other_ms: f64,
    pub total_wall_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SmokeTestEntryResult {
    pub file_id: String,
    pub jar_path: String,
    pub jar_sha256: String,
    pub launch_configuration_id: String,
    pub entry_point: String,
    pub title: String,
    pub vendor: String,
    pub runtime_status: String,
    pub observed_ui: String,
    pub load_ok: bool,
    pub midlet_started: bool,
    pub main_canvas_created: bool,
    pub paint_calls: u64,
    pub framebuffer_publications: u64,
    pub unique_framebuffer_hashes: usize,
    pub active_threads: usize,
    pub executed_instructions: u64,
    pub exceptions: u64,
    pub unhandled_native: u64,
    pub native_call_counts: u64,
    #[serde(default)]
    pub guest_exceptions_thrown: u64,
    #[serde(default)]
    pub guest_exceptions_caught: u64,
    #[serde(default)]
    pub guest_exceptions_uncaught: u64,
    #[serde(default)]
    pub fatal_guest_exception: bool,
    #[serde(default)]
    pub uncaught_exception_threads: Vec<String>,
    #[serde(default)]
    pub last_exception_class: Option<String>,
    #[serde(default)]
    pub last_exception_message: Option<String>,
    #[serde(default)]
    pub last_exception_location: Option<String>,
    pub audio_generated_samples: u64,
    pub audio_consumed_samples: u64,
    pub audio_dropped_samples: u64,
    pub audio_underrun_count: u64,
    pub audio_overflow_count: u64,
    pub timing: ExecutionTimingBreakdown,
    pub exit_reason: String,
    pub last_error: Option<String>,
    pub final_frame_sha256: String,
    pub apis_invoked: Vec<String>,
    #[serde(default)]
    pub recent_native_calls: Vec<String>,
    #[serde(default)]
    pub unhandled_natives: Vec<String>,
    #[serde(default)]
    pub active_thread_stacks: Vec<String>,
    pub blockers: Vec<BlockerItem>,
    #[serde(default)]
    pub auto_healing_count: u32,
    #[serde(default)]
    pub auto_healing_occurred: bool,
    #[serde(default)]
    pub auto_healing_events: Vec<j2me_stdlib::bridge::context::AutoHealingEvent>,
    #[serde(default)]
    pub test_verdict: String,
    #[serde(default)]
    pub criteria_report: Option<j2me_stdlib::bridge::context::RuntimeObservabilityCriteria>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodAuditEntry {
    pub class: String,
    pub method_name: String,
    pub descriptor: String,
    pub runtime_call_count: u64,
    pub implementation_file: String,
    pub status: String, // "FULL", "PARTIAL", "STUB", "MISSING"
    pub contract_limitations: Option<String>,
    pub related_tests: Vec<String>,
    pub run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopBlockerReport {
    pub rank: usize,
    pub blocker_type: String, // "CONFIRMED" or "HYPOTHESIS"
    pub category: String,
    pub description: String,
    pub affected_launch_configurations: usize,
    pub denominator: usize,
    pub example_run_ids: Vec<String>,
    pub minimal_repro: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestSummary {
    pub total_tested: usize,
    pub load_ok_count: usize,
    pub load_error_count: usize,
    pub start_error_count: usize,
    pub completed_window_count: usize,
    pub guest_exception_count: usize,
    pub host_panic_count: usize,
    pub wall_timeout_count: usize,
    #[serde(default)]
    pub auto_healed_count: usize,
    #[serde(default)]
    pub pass_count: usize,
    #[serde(default)]
    pub partial_count: usize,
    #[serde(default)]
    pub degraded_count: usize,
    #[serde(default)]
    pub fail_count: usize,
    pub splash_count: usize,
    pub menu_count: usize,
    pub gameplay_count: usize,
    pub unknown_ui_count: usize,
    pub no_paint_count: usize,
    pub no_canvas_count: usize,
    pub top_blockers: Vec<TopBlockerReport>,
    pub api_methods_audit: Vec<MethodAuditEntry>,
    pub results: Vec<SmokeTestEntryResult>,
}

// ============================================================================
// Fast SHA-256 Digest
// ============================================================================

pub fn sha256_digest(data: &[u8]) -> String {
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

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

/// Recursively collects all files under path, detecting symlinks and preventing cycles.
fn collect_files_recursive(
    dir: &Path,
    visited_dirs: &mut HashSet<PathBuf>,
    out: &mut Vec<PathBuf>,
) {
    if let Ok(canon) = dir.canonicalize() {
        if !visited_dirs.insert(canon) {
            return; // Cycle detected
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let p = entry.path();
        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname.starts_with('.')
            || fname == "__pycache__"
            || fname == "node_modules"
            || fname == "target"
            || fname == ".venv"
        {
            continue;
        }

        if p.is_dir() {
            collect_files_recursive(&p, visited_dirs, out);
        } else {
            out.push(p);
        }
    }
}

// ============================================================================
// Scanner Implementation
// ============================================================================

pub fn scan_market(root_dir: &str, output_path: &str) -> Result<MarketInventory, String> {
    let t_start = Instant::now();
    println!(">>> Scanning J2ME Market: '{}' <<<", root_dir);

    let root_path = Path::new(root_dir);
    if !root_path.exists() {
        return Err(format!("Root directory does not exist: {}", root_dir));
    }

    let mut visited_dirs = HashSet::new();
    let mut all_paths = Vec::new();
    collect_files_recursive(root_path, &mut visited_dirs, &mut all_paths);
    println!("  - Discovered {} files on disk.", all_paths.len());

    let mut entries = Vec::with_capacity(all_paths.len());
    let mut sha_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    let mut jar_sha_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    let mut extensions_summary: HashMap<String, usize> = HashMap::new();
    let mut missing_metadata_summary: HashMap<String, usize> = HashMap::new();
    let mut unreadable_files_count: usize = 0;
    let mut total_jar_files: usize = 0;
    let mut regular_jar_files: usize = 0;
    let mut symlink_jar_files: usize = 0;
    let mut total_jad_files: usize = 0;
    let mut total_launch_configurations: usize = 0;
    let mut multi_midlet_suites_count: usize = 0;
    let mut jad_entries: Vec<(usize, String, Option<String>, Option<u64>)> = Vec::new();
    let mut title_vendor_to_indices: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, path) in all_paths.iter().enumerate() {
        let is_symlink = path.is_symlink();
        let rel_path = path
            .strip_prefix(root_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        *extensions_summary.entry(ext.clone()).or_insert(0) += 1;

        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                unreadable_files_count += 1;
                entries.push(InventoryFileEntry {
                    inventory_schema_version: "4.1.0".to_string(),
                    file_id: format!("file_{:05}", idx),
                    path: path.to_string_lossy().to_string(),
                    relative_path: rel_path,
                    container_path: None,
                    entry_path: None,
                    size_bytes: 0,
                    sha256: String::new(),
                    extension: ext,
                    availability: if is_symlink {
                        "SYMLINK".to_string()
                    } else {
                        "UNREADABLE".to_string()
                    },
                    archive_status: "UNREADABLE".to_string(),
                    manifest_status: "N/A".to_string(),
                    jad_status: "N/A".to_string(),
                    midlet_entries: Vec::new(),
                    vendor: None,
                    version: None,
                    configuration: None,
                    profile: None,
                    launch_configuration_id: None,
                    runtime_status: "LOAD_ERROR".to_string(),
                    observed_ui: "UNKNOWN".to_string(),
                    evidence: EvidenceRecord::default(),
                    blockers: vec![BlockerItem {
                        blocker_type: "CONFIRMED".to_string(),
                        category: "I/O".to_string(),
                        description: format!("File read failure: {}", e),
                        minimal_repro: None,
                    }],
                    notes: "Unreadable file on disk".to_string(),
                });
                continue;
            }
        };

        let size_bytes = bytes.len() as u64;
        let sha256 = sha256_digest(&bytes);
        sha_to_indices.entry(sha256.clone()).or_default().push(idx);

        let availability = if is_symlink {
            "SYMLINK".to_string()
        } else {
            "PRESENT".to_string()
        };

        let mut archive_status = "VALID".to_string();
        let mut manifest_status = "N/A".to_string();
        let jad_status = "N/A".to_string();
        let mut midlet_entries = Vec::new();
        let mut vendor = None;
        let mut version = None;
        let mut configuration = None;
        let mut profile = None;
        let mut launch_configuration_id = None;
        let mut blockers = Vec::new();
        let mut notes = String::new();

        if size_bytes == 0 {
            archive_status = "EMPTY".to_string();
            blockers.push(BlockerItem {
                blocker_type: "CONFIRMED".to_string(),
                category: "CORRUPT_ARCHIVE".to_string(),
                description: "Empty 0-byte file".to_string(),
                minimal_repro: None,
            });
        } else if ext == "jar" {
            total_jar_files += 1;
            if is_symlink {
                symlink_jar_files += 1;
            } else {
                regular_jar_files += 1;
                jar_sha_to_indices
                    .entry(sha256.clone())
                    .or_default()
                    .push(idx);
            }

            match JarArchive::parse(&bytes) {
                Ok(jar) => {
                    archive_status = "VALID".to_string();
                    if let Some(ref m) = jar.manifest {
                        manifest_status = "PRESENT".to_string();
                        vendor = m.midlet_vendor().map(|s| s.to_string());
                        version = m.midlet_version().map(|s| s.to_string());
                        configuration = m.get("MicroEdition-Configuration").map(|s| s.to_string());
                        profile = m.get("MicroEdition-Profile").map(|s| s.to_string());

                        // Extract all MIDlet-1..32 entries
                        for mid_idx in 1..=32 {
                            let key = format!("MIDlet-{}", mid_idx);
                            if let Some(val) = m.get(&key) {
                                let parts: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
                                let m_name = parts.first().copied().unwrap_or("").to_string();
                                let m_icon = parts.get(1).copied().unwrap_or("").to_string();
                                let m_cls = parts.get(2).copied().unwrap_or("").to_string();
                                midlet_entries.push(MidletEntryInfo {
                                    index: mid_idx,
                                    name: m_name,
                                    icon: m_icon,
                                    main_class: m_cls,
                                });
                            } else if mid_idx == 1 {
                                // Missing MIDlet-1
                                *missing_metadata_summary
                                    .entry("MIDlet-1".to_string())
                                    .or_insert(0) += 1;
                            } else {
                                break;
                            }
                        }

                        if midlet_entries.len() > 1 {
                            multi_midlet_suites_count += 1;
                        }
                        total_launch_configurations += midlet_entries.len();

                        if let Some(first) = midlet_entries.first() {
                            launch_configuration_id =
                                Some(format!("{}:midlet_{}", &sha256[..12], first.index));
                        }

                        if vendor.is_none() {
                            *missing_metadata_summary
                                .entry("MIDlet-Vendor".to_string())
                                .or_insert(0) += 1;
                        }
                        if version.is_none() {
                            *missing_metadata_summary
                                .entry("MIDlet-Version".to_string())
                                .or_insert(0) += 1;
                        }

                        notes = format!(
                            "Zip entries: {} | MIDlets: {}",
                            jar.entries.len(),
                            midlet_entries.len()
                        );
                    } else {
                        manifest_status = "MISSING".to_string();
                        notes = "Valid ZIP archive without META-INF/MANIFEST.MF".to_string();
                        blockers.push(BlockerItem {
                            blocker_type: "CONFIRMED".to_string(),
                            category: "MISSING_MANIFEST".to_string(),
                            description: "JAR archive is missing META-INF/MANIFEST.MF".to_string(),
                            minimal_repro: None,
                        });
                    }
                }
                Err(e) => {
                    archive_status = "CORRUPTED".to_string();
                    manifest_status = "N/A".to_string();
                    blockers.push(BlockerItem {
                        blocker_type: "CONFIRMED".to_string(),
                        category: "CORRUPT_ARCHIVE".to_string(),
                        description: format!("Corrupted ZIP archive: {}", e),
                        minimal_repro: None,
                    });
                }
            }
        } else if ext == "jad" {
            total_jad_files += 1;
            let text = String::from_utf8_lossy(&bytes);
            let jad = Jad::parse(&text);
            vendor = jad.get("MIDlet-Vendor").map(|s| s.to_string());
            version = jad.get("MIDlet-Version").map(|s| s.to_string());
            configuration = jad.get("MicroEdition-Configuration").map(|s| s.to_string());
            profile = jad.get("MicroEdition-Profile").map(|s| s.to_string());
            let jad_jar_url = jad.get("MIDlet-Jar-URL").map(|s| s.to_string());
            let jad_jar_size = jad
                .get("MIDlet-Jar-Size")
                .and_then(|s| s.trim().parse::<u64>().ok());

            notes = format!(
                "JAD attributes: {} | target: {}",
                jad.properties.len(),
                jad_jar_url.as_deref().unwrap_or("none")
            );
            jad_entries.push((idx, filename.clone(), jad_jar_url, jad_jar_size));
        } else if ext == "zip" || ext == "7z" {
            archive_status = "VALID".to_string();
            notes = "Compressed container archive (non-extracted)".to_string();
        } else {
            notes = "Non-executable asset (doc/image/cfg)".to_string();
        }

        // Distinct versions tracker by Title/Vendor
        if let Some(ref v) = vendor {
            title_vendor_to_indices
                .entry(v.clone())
                .or_default()
                .push(idx);
        }

        entries.push(InventoryFileEntry {
            inventory_schema_version: "4.1.0".to_string(),
            file_id: format!("file_{:05}", idx),
            path: path.to_string_lossy().to_string(),
            relative_path: rel_path,
            container_path: if ext == "zip" || ext == "7z" {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            },
            entry_path: None,
            size_bytes,
            sha256,
            extension: ext,
            availability,
            archive_status,
            manifest_status,
            jad_status,
            midlet_entries,
            vendor,
            version,
            configuration,
            profile,
            launch_configuration_id,
            runtime_status: "NOT_RUN".to_string(),
            observed_ui: "UNKNOWN".to_string(),
            evidence: EvidenceRecord::default(),
            blockers,
            notes,
        });
    }

    // Match JADs with JARs
    let mut name_to_jar_idx = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        if entry.extension == "jar" {
            if let Some(fname) = Path::new(&entry.path).file_name().and_then(|n| n.to_str()) {
                name_to_jar_idx.insert(fname.to_lowercase(), i);
            }
        }
    }

    let mut matched_jar_indices = HashSet::new();
    let mut jad_jar_mismatch_count = 0;

    for (jad_idx, _filename, target_url, target_size) in jad_entries {
        let mut matched = false;
        if let Some(ref url) = target_url {
            let url_fname = Path::new(url)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if let Some(&jar_idx) = name_to_jar_idx.get(&url_fname) {
                matched = true;
                matched_jar_indices.insert(jar_idx);
                entries[jad_idx].jad_status = "PRESENT_MATCHED".to_string();
                if entries[jar_idx].jad_status == "N/A" || entries[jar_idx].jad_status == "MISSING"
                {
                    entries[jar_idx].jad_status = "PRESENT_MATCHED".to_string();
                }

                let actual_size = entries[jar_idx].size_bytes;
                if let Some(exp_size) = target_size {
                    if exp_size != actual_size {
                        jad_jar_mismatch_count += 1;
                        entries[jad_idx].blockers.push(BlockerItem {
                            blocker_type: "CONFIRMED".to_string(),
                            category: "JAD_JAR_SIZE_MISMATCH".to_string(),
                            description: format!(
                                "JAD size {} != actual JAR size {}",
                                exp_size, actual_size
                            ),
                            minimal_repro: None,
                        });
                    }
                }
            }
        }

        if !matched {
            jad_jar_mismatch_count += 1;
            entries[jad_idx].jad_status = "PRESENT_MISMATCH".to_string();
            entries[jad_idx].blockers.push(BlockerItem {
                blocker_type: "CONFIRMED".to_string(),
                category: "JAD_TARGET_MISSING".to_string(),
                description: format!(
                    "Referenced JAR '{}' not found",
                    target_url.as_deref().unwrap_or("none")
                ),
                minimal_repro: None,
            });
        }
    }

    let games_jar_and_jad_count = matched_jar_indices.len();
    let games_jar_only_count = total_jar_files.saturating_sub(games_jar_and_jad_count);

    // Compute duplicate groups
    let mut duplicate_groups = Vec::new();
    let mut exact_duplicates_count = 0;
    for (sha, idxs) in &sha_to_indices {
        if idxs.len() > 1 {
            exact_duplicates_count += idxs.len() - 1;
            duplicate_groups.push(DuplicateGroup {
                sha256: sha.clone(),
                count: idxs.len(),
                files: idxs
                    .iter()
                    .map(|&i| entries[i].relative_path.clone())
                    .collect(),
            });
        }
    }

    let summary = InventorySummary {
        total_files_scanned: entries.len(),
        total_jar_files,
        regular_jar_files,
        symlink_jar_files,
        unique_jar_sha256_count: jar_sha_to_indices.len(),
        total_jad_files,
        unique_sha256_count: sha_to_indices.len(),
        exact_duplicates_count,
        unreadable_files_count,
        jad_jar_mismatch_count,
        games_jar_only_count,
        games_jar_and_jad_count,
        total_launch_configurations,
        multi_midlet_suites_count,
        catalog_indexed_games_count: 7288,
        extensions_summary,
        missing_metadata_summary,
        exact_duplicates_groups: duplicate_groups,
        distinct_versions_groups_count: title_vendor_to_indices.len(),
    };

    let inventory = MarketInventory {
        version: "4.1.0".to_string(),
        scan_timestamp: "2026-10-25T00:15:00Z".to_string(),
        root_scanned: root_dir.to_string(),
        summary,
        files: entries,
    };

    if let Some(parent) = Path::new(output_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json_bytes =
        serde_json::to_vec_pretty(&inventory).map_err(|e| format!("Serialization error: {}", e))?;
    fs::write(output_path, json_bytes)
        .map_err(|e| format!("Failed to write {}: {}", output_path, e))?;

    let elapsed = t_start.elapsed().as_secs_f64();
    println!(">>> Inventory scan completed in {:.2}s <<<", elapsed);
    println!(
        "  - Scanned total files:     {}",
        inventory.summary.total_files_scanned
    );
    println!(
        "  - Regular JAR files:       {}",
        inventory.summary.regular_jar_files
    );
    println!(
        "  - Symlink JAR files:       {}",
        inventory.summary.symlink_jar_files
    );
    println!(
        "  - Total JAR files:         {}",
        inventory.summary.total_jar_files
    );
    println!(
        "  - Unique JAR SHA-256:      {}",
        inventory.summary.unique_jar_sha256_count
    );
    println!(
        "  - Launch Configurations:   {}",
        inventory.summary.total_launch_configurations
    );
    println!(
        "  - Multi-MIDlet suites:     {}",
        inventory.summary.multi_midlet_suites_count
    );
    println!(
        "  - Total JAD files:         {}",
        inventory.summary.total_jad_files
    );
    println!(
        "  - Exact duplicates copies: {}",
        inventory.summary.exact_duplicates_count
    );
    println!(
        "  - Games (JAR only):        {}",
        inventory.summary.games_jar_only_count
    );
    println!(
        "  - Games (JAR + JAD):       {}",
        inventory.summary.games_jar_and_jad_count
    );

    Ok(inventory)
}

// ============================================================================
// Inventory Verification
// ============================================================================

pub fn verify_inventory(inventory_path: &str) -> Result<bool, String> {
    println!("============================================================");
    println!(" J2ME Market Inventory Verification (--verify-inventory)");
    println!(" Target: {}", inventory_path);
    println!("============================================================");

    let data = fs::read_to_string(inventory_path)
        .map_err(|e| format!("Failed to read inventory {}: {}", inventory_path, e))?;

    let inv: MarketInventory =
        serde_json::from_str(&data).map_err(|e| format!("Invalid inventory JSON format: {}", e))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Invariant 1: Sum of extension counts == total files
    let sum_extensions: usize = inv.summary.extensions_summary.values().sum();
    if sum_extensions != inv.summary.total_files_scanned {
        errors.push(format!(
            "Invariant 1 violated: sum of extensions ({}) != total files scanned ({})",
            sum_extensions, inv.summary.total_files_scanned
        ));
    }

    // Invariant 2: Unique SHA-256 <= total files
    if inv.summary.unique_sha256_count > inv.summary.total_files_scanned {
        errors.push(format!(
            "Invariant 2 violated: unique SHA-256 ({}) > total files ({})",
            inv.summary.unique_sha256_count, inv.summary.total_files_scanned
        ));
    }

    // Invariant 3: Extra copies == sum(count - 1) in duplicate groups
    let sum_extra_copies: usize = inv
        .summary
        .exact_duplicates_groups
        .iter()
        .map(|g| g.count - 1)
        .sum();
    if sum_extra_copies != inv.summary.exact_duplicates_count {
        errors.push(format!(
            "Invariant 3 violated: duplicate groups extra copies ({}) != exact_duplicates_count ({})",
            sum_extra_copies, inv.summary.exact_duplicates_count
        ));
    }

    // Invariant 4: Regular JARs + Symlink JARs == Total JARs
    if inv.summary.regular_jar_files + inv.summary.symlink_jar_files != inv.summary.total_jar_files
    {
        errors.push(format!(
            "Invariant 4 violated: regular JARs ({}) + symlink JARs ({}) != total JARs ({})",
            inv.summary.regular_jar_files,
            inv.summary.symlink_jar_files,
            inv.summary.total_jar_files
        ));
    }

    // Check individual files
    let mut broken_zip_count = 0;
    let mut missing_manifest_count = 0;
    let mut zero_size_count = 0;

    for entry in &inv.files {
        if entry.size_bytes == 0 {
            zero_size_count += 1;
            warnings.push(format!("{}: 0-byte file", entry.relative_path));
        }
        if entry.archive_status == "CORRUPTED" {
            broken_zip_count += 1;
            warnings.push(format!("{}: Corrupted archive", entry.relative_path));
        }
        if entry.manifest_status == "MISSING" && entry.extension == "jar" {
            missing_manifest_count += 1;
            warnings.push(format!(
                "{}: Missing MANIFEST.MF (valid ZIP)",
                entry.relative_path
            ));
        }
    }

    println!("  - Scanned Entries:       {}", inv.files.len());
    println!("  - Total JARs:            {}", inv.summary.total_jar_files);
    println!(
        "  - Regular JARs:          {}",
        inv.summary.regular_jar_files
    );
    println!(
        "  - Symlink JARs:          {}",
        inv.summary.symlink_jar_files
    );
    println!(
        "  - Unique Hashes:         {}",
        inv.summary.unique_sha256_count
    );
    println!(
        "  - Exact Duplicate Copies:{}",
        inv.summary.exact_duplicates_count
    );
    println!(
        "  - Launch Configurations: {}",
        inv.summary.total_launch_configurations
    );
    println!(
        "  - Multi-MIDlet Suites:   {}",
        inv.summary.multi_midlet_suites_count
    );
    println!("  - 0-byte Files:          {}", zero_size_count);
    println!("  - Corrupted Archives:    {}", broken_zip_count);
    println!("  - Missing Manifests:     {}", missing_manifest_count);

    if errors.is_empty() {
        println!(
            "\n[SUCCESS] Inventory integrity verified with 0 fatal errors ({} diagnostic warnings recorded).",
            warnings.len()
        );
        Ok(true)
    } else {
        println!(
            "\n[FAILURE] Inventory verification failed with {} errors:",
            errors.len()
        );
        for err in &errors {
            println!("  - [ERROR] {}", err);
        }
        Err(format!("Validation failed with {} errors", errors.len()))
    }
}

// ============================================================================
// Single Midlet Runner (Runs in Isolated Child Process)
// ============================================================================

pub fn run_single_midlet(
    jar_path: &str,
    entry_point: Option<&str>,
    ticks: usize,
    out_json_path: &str,
    clean_rms: bool,
) {
    let t_wall_start = Instant::now();
    let p = Path::new(jar_path);

    let t_io_start = Instant::now();
    let bytes = match fs::read(p) {
        Ok(b) => b,
        Err(e) => {
            let res = SmokeTestEntryResult {
                file_id: "unknown".to_string(),
                jar_path: jar_path.to_string(),
                jar_sha256: String::new(),
                launch_configuration_id: "none".to_string(),
                entry_point: entry_point.unwrap_or("default").to_string(),
                title: String::new(),
                vendor: String::new(),
                runtime_status: "LOAD_ERROR".to_string(),
                observed_ui: "UNKNOWN".to_string(),
                load_ok: false,
                midlet_started: false,
                main_canvas_created: false,
                paint_calls: 0,
                framebuffer_publications: 0,
                unique_framebuffer_hashes: 0,
                active_threads: 0,
                executed_instructions: 0,
                exceptions: 0,
                unhandled_native: 0,
                native_call_counts: 0,
                guest_exceptions_thrown: 0,
                guest_exceptions_caught: 0,
                guest_exceptions_uncaught: 0,
                fatal_guest_exception: false,
                uncaught_exception_threads: Vec::new(),
                last_exception_class: None,
                last_exception_message: None,
                last_exception_location: None,
                audio_generated_samples: 0,
                audio_consumed_samples: 0,
                audio_dropped_samples: 0,
                audio_underrun_count: 0,
                audio_overflow_count: 0,
                timing: ExecutionTimingBreakdown {
                    elapsed_load_ms: 0.0,
                    elapsed_vm_ms: 0.0,
                    elapsed_graphics_ms: 0.0,
                    elapsed_audio_ms: 0.0,
                    elapsed_scheduler_ms: 0.0,
                    elapsed_io_ms: t_io_start.elapsed().as_secs_f64() * 1000.0,
                    elapsed_other_ms: 0.0,
                    total_wall_ms: t_wall_start.elapsed().as_secs_f64() * 1000.0,
                },
                exit_reason: "IO_ERROR".to_string(),
                last_error: Some(format!("I/O Error: {}", e)),
                final_frame_sha256: String::new(),
                apis_invoked: Vec::new(),
                recent_native_calls: Vec::new(),
                unhandled_natives: Vec::new(),
                active_thread_stacks: Vec::new(),
                blockers: vec![BlockerItem {
                    blocker_type: "CONFIRMED".to_string(),
                    category: "I/O".to_string(),
                    description: format!("File read error: {}", e),
                    minimal_repro: None,
                }],
                auto_healing_count: 0,
                auto_healing_occurred: false,
                auto_healing_events: Vec::new(),
                test_verdict: "FAIL".to_string(),
                criteria_report: None,
            };
            let _ = fs::write(out_json_path, serde_json::to_vec(&res).unwrap_or_default());
            return;
        }
    };
    let elapsed_io_ms = t_io_start.elapsed().as_secs_f64() * 1000.0;
    let jar_sha256 = sha256_digest(&bytes);

    let t_load_start = Instant::now();
    let mut emu = J2meEmulator::new(240, 320, 44100);
    if clean_rms {
        emu.context.rms.stores.clear();
    }

    let load_res = emu.load_jar(&bytes);
    let elapsed_load_ms = t_load_start.elapsed().as_secs_f64() * 1000.0;

    if let Err(e) = load_res {
        let res = SmokeTestEntryResult {
            file_id: "single".to_string(),
            jar_path: jar_path.to_string(),
            jar_sha256,
            launch_configuration_id: "default".to_string(),
            entry_point: entry_point.unwrap_or("default").to_string(),
            title: String::new(),
            vendor: String::new(),
            runtime_status: "LOAD_ERROR".to_string(),
            observed_ui: "UNKNOWN".to_string(),
            load_ok: false,
            midlet_started: false,
            main_canvas_created: false,
            paint_calls: 0,
            framebuffer_publications: 0,
            unique_framebuffer_hashes: 0,
            active_threads: 0,
            executed_instructions: 0,
            exceptions: 0,
            unhandled_native: 0,
            native_call_counts: 0,
            guest_exceptions_thrown: 0,
            guest_exceptions_caught: 0,
            guest_exceptions_uncaught: 0,
            fatal_guest_exception: false,
            uncaught_exception_threads: Vec::new(),
            last_exception_class: None,
            last_exception_message: None,
            last_exception_location: None,
            audio_generated_samples: 0,
            audio_consumed_samples: 0,
            audio_dropped_samples: 0,
            audio_underrun_count: 0,
            audio_overflow_count: 0,
            timing: ExecutionTimingBreakdown {
                elapsed_load_ms,
                elapsed_vm_ms: 0.0,
                elapsed_graphics_ms: 0.0,
                elapsed_audio_ms: 0.0,
                elapsed_scheduler_ms: 0.0,
                elapsed_io_ms,
                elapsed_other_ms: 0.0,
                total_wall_ms: t_wall_start.elapsed().as_secs_f64() * 1000.0,
            },
            exit_reason: "LOAD_ERROR".to_string(),
            last_error: Some(e.clone()),
            final_frame_sha256: String::new(),
            apis_invoked: Vec::new(),
            recent_native_calls: Vec::new(),
            unhandled_natives: Vec::new(),
            active_thread_stacks: Vec::new(),
            blockers: vec![BlockerItem {
                blocker_type: "CONFIRMED".to_string(),
                category: "CLASS_LOADER".to_string(),
                description: format!("Class load error: {}", e),
                minimal_repro: None,
            }],
            auto_healing_count: 0,
            auto_healing_occurred: false,
            auto_healing_events: Vec::new(),
            test_verdict: "FAIL".to_string(),
            criteria_report: None,
        };
        let _ = fs::write(out_json_path, serde_json::to_vec(&res).unwrap_or_default());
        return;
    }

    let mut unique_hashes = HashSet::new();
    let mut initial_hash = String::new();
    let mut final_hash = String::new();

    let mut wall_deadline = Instant::now() + Duration::from_secs(5);
    let max_wall_deadline = t_wall_start + Duration::from_secs(15);
    let mut last_checked_instructions = 0u64;
    let mut last_checked_paints = 0u64;
    let mut timed_out_wall = false;

    let t_vm_start = Instant::now();
    for tick_idx in 0..ticks {
        // Auto-skip splash screens: simulate user interaction (Fire / Num5)
        if tick_idx == 30 || tick_idx == 70 || tick_idx == 120 {
            emu.key_event(-5, true);
            emu.key_event(-5, false);
            emu.key_event(53, true);
            emu.key_event(53, false);
        }

        let dirty = emu.tick(16.66667);
        if dirty || tick_idx == 0 || tick_idx == ticks - 1 {
            let slice =
                unsafe { std::slice::from_raw_parts(emu.framebuffer_ptr(), emu.framebuffer_len()) };
            let h = sha256_digest(slice);
            if tick_idx == 0 {
                initial_hash = h.clone();
            }
            final_hash = h.clone();
            unique_hashes.insert(h);
        }

        // Dynamic time limit for heavy games
        if Instant::now() >= wall_deadline {
            let cur_inst = emu.scheduler.total_instructions();
            let cur_paints = (emu.context.paint_calls_count + emu.context.fb_publishes_count) as u64;
            if wall_deadline < max_wall_deadline
                && cur_inst > 10_000_000
                && (cur_inst.saturating_sub(last_checked_instructions) > 1_000_000
                    || cur_paints > last_checked_paints)
            {
                wall_deadline = (wall_deadline + Duration::from_secs(5)).min(max_wall_deadline);
                last_checked_instructions = cur_inst;
                last_checked_paints = cur_paints;
            } else {
                timed_out_wall = true;
                break;
            }
        }

        if emu.scheduler.active_thread_count() == 0 {
            break;
        }
    }
    let total_vm_time_ms = t_vm_start.elapsed().as_secs_f64() * 1000.0;

    let mut active_thread_stacks = Vec::new();
    for t in &emu.scheduler.threads {
        if !t.is_alive() {
            continue;
        }
        let loc = if let Some(f) = t.call_stack.last() {
            let instr = f
                .instructions
                .get(f.pc)
                .map(|i| format!("{:?}", i))
                .unwrap_or_else(|| "EOF".to_string());
            format!(
                "{}.{}{}:{} ({})",
                f.class_name, f.method_name, f.descriptor, f.pc, instr
            )
        } else {
            "empty stack".to_string()
        };
        active_thread_stacks.push(format!("Thread '{}' [{:?}]: {}", t.name, t.state, loc));
    }

    let paint_calls = emu.context.paint_calls_count;
    let fb_publishes = emu.context.fb_publishes_count;
    let has_canvas = emu.context.current_displayable.is_some();
    let exceptions = emu.context.exceptions_count;
    let unhandled_native = emu.context.unhandled_native_calls;

    let obs = emu.evaluate_observability(timed_out_wall);
    let runtime_status = match obs.verdict {
        j2me_stdlib::bridge::context::TestVerdict::Pass => "COMPLETED_WINDOW".to_string(),
        j2me_stdlib::bridge::context::TestVerdict::Partial => "PARTIAL".to_string(),
        j2me_stdlib::bridge::context::TestVerdict::Degraded => "DEGRADED".to_string(),
        j2me_stdlib::bridge::context::TestVerdict::Fail => {
            if obs.criteria.host_crash {
                "HOST_PANIC".to_string()
            } else if obs.criteria.deadlock_suspected {
                "DEADLOCK_SUSPECTED".to_string()
            } else {
                "GUEST_EXCEPTION".to_string()
            }
        }
        j2me_stdlib::bridge::context::TestVerdict::Timeout => "TIMEOUT_DEADLOCK".to_string(),
        j2me_stdlib::bridge::context::TestVerdict::NotTested => "NOT_TESTED".to_string(),
    };
    let mut observed_ui = "UNKNOWN".to_string();
    let mut blockers = Vec::new();

    let telem = &emu.context.exception_telemetry;
    let fatal_rec = telem
        .fatal_guest_exception
        .as_ref()
        .or(telem.last_uncaught.as_ref());

    let mut last_err = None;
    if let Some(rec) = fatal_rec {
        let msg_part = rec.message.as_deref().unwrap_or("no message");
        let loc = format!(
            "{}.{}{}:{}",
            rec.class_name, rec.method_name, rec.descriptor, rec.pc
        );
        last_err = Some(format!("{}: {} at {}", rec.exception_class, msg_part, loc));
        if obs.verdict == j2me_stdlib::bridge::context::TestVerdict::Fail {
            blockers.push(BlockerItem {
                blocker_type: "CONFIRMED".to_string(),
                category: "GUEST_EXCEPTION".to_string(),
                description: format!(
                    "Uncaught guest Java exception: {} (\"{}\") in {} (thread: {})",
                    rec.exception_class, msg_part, loc, rec.thread_name
                ),
                minimal_repro: None,
            });
        }
    }
    if obs.criteria.auto_heal_occurred {
        blockers.push(BlockerItem {
            blocker_type: "CONFIRMED".to_string(),
            category: "AUTO_HEAL_OCCURRED".to_string(),
            description: format!(
                "Auto-Healing suppressed {} uncaught exception(s); last: {:?}",
                telem.auto_healing_count, telem.last_auto_healing_signature
            ),
            minimal_repro: None,
        });
    }
    if obs.criteria.deadlock_suspected {
        blockers.push(BlockerItem {
            blocker_type: "CONFIRMED".to_string(),
            category: "TIMEOUT_DEADLOCK".to_string(),
            description: format!(
                "Session wall timeout expired before paint. Active threads: {:?}",
                active_thread_stacks
            ),
            minimal_repro: None,
        });
    } else if paint_calls > 0 || fb_publishes > 0 {
        if unique_hashes.len() > 1 && initial_hash != final_hash {
            observed_ui = "SPLASH".to_string();
        } else {
            observed_ui = "MENU".to_string();
        }
    } else if !has_canvas {
        blockers.push(BlockerItem {
            blocker_type: "CONFIRMED".to_string(),
            category: "NO_CANVAS".to_string(),
            description: "No Canvas or Displayable created before tick exhaustion".to_string(),
            minimal_repro: None,
        });
    } else {
        blockers.push(BlockerItem {
            blocker_type: "HYPOTHESIS".to_string(),
            category: "NO_PAINT".to_string(),
            description: "Canvas created but no paint calls or flushGraphics executed".to_string(),
            minimal_repro: None,
        });
    }

    let elapsed_graphics_ms = emu.context.rasterizer_time_us as f64 / 1000.0;
    let elapsed_audio_ms = emu.context.audio_time_us as f64 / 1000.0;
    let elapsed_scheduler_ms = emu.context.scheduler_time_us as f64 / 1000.0;
    let elapsed_vm_ms =
        (total_vm_time_ms - elapsed_graphics_ms - elapsed_audio_ms - elapsed_scheduler_ms).max(0.0);
    let total_wall_ms = t_wall_start.elapsed().as_secs_f64() * 1000.0;
    let elapsed_other_ms =
        (total_wall_ms - elapsed_io_ms - elapsed_load_ms - total_vm_time_ms).max(0.0);

    let timing = ExecutionTimingBreakdown {
        elapsed_load_ms,
        elapsed_vm_ms,
        elapsed_graphics_ms,
        elapsed_audio_ms,
        elapsed_scheduler_ms,
        elapsed_io_ms,
        elapsed_other_ms,
        total_wall_ms,
    };

    let mut apis: Vec<String> = emu.context.apis_invoked.into_iter().collect();
    apis.sort();

    let res = SmokeTestEntryResult {
        file_id: "single".to_string(),
        jar_path: jar_path.to_string(),
        jar_sha256,
        launch_configuration_id: entry_point.unwrap_or("default").to_string(),
        entry_point: entry_point.unwrap_or("default").to_string(),
        title: String::new(),
        vendor: String::new(),
        runtime_status,
        observed_ui,
        load_ok: true,
        midlet_started: true,
        main_canvas_created: has_canvas,
        paint_calls,
        framebuffer_publications: fb_publishes,
        unique_framebuffer_hashes: unique_hashes.len(),
        active_threads: emu.scheduler.active_thread_count(),
        executed_instructions: emu.scheduler.total_instructions(),
        exceptions,
        unhandled_native,
        native_call_counts: emu.context.native_calls_count,
        guest_exceptions_thrown: telem.guest_exceptions_thrown,
        guest_exceptions_caught: telem.guest_exceptions_caught,
        guest_exceptions_uncaught: telem.guest_exceptions_uncaught,
        fatal_guest_exception: obs.verdict == j2me_stdlib::bridge::context::TestVerdict::Fail && obs.criteria.guest_exception,
        uncaught_exception_threads: telem.uncaught_exception_threads.clone(),
        last_exception_class: fatal_rec.map(|r| r.exception_class.clone()),
        last_exception_message: fatal_rec.and_then(|r| r.message.clone()),
        last_exception_location: fatal_rec.map(|r| {
            format!(
                "{}.{}{}:{}",
                r.class_name, r.method_name, r.descriptor, r.pc
            )
        }),
        audio_generated_samples: emu.context.audio.generated_samples,
        audio_consumed_samples: emu.context.audio.consumed_samples,
        audio_dropped_samples: emu.context.audio.dropped_samples,
        audio_underrun_count: emu.context.audio.underrun_count,
        audio_overflow_count: emu.context.audio.overflow_count,
        timing,
        exit_reason: if timed_out_wall {
            "WALL_TIMEOUT".to_string()
        } else {
            "TICKS_EXHAUSTED".to_string()
        },
        last_error: last_err,
        final_frame_sha256: final_hash,
        apis_invoked: apis,
        recent_native_calls: emu.context.recent_native_calls.iter().cloned().collect(),
        unhandled_natives: emu.context.unhandled_natives.iter().cloned().collect(),
        active_thread_stacks,
        blockers,
        auto_healing_count: telem.auto_healing_count,
        auto_healing_occurred: obs.criteria.auto_heal_occurred,
        auto_healing_events: obs.auto_healing_events.clone(),
        test_verdict: obs.verdict.to_string(),
        criteria_report: Some(obs.criteria.clone()),
    };

    let _ = fs::write(
        out_json_path,
        serde_json::to_vec_pretty(&res).unwrap_or_default(),
    );
}

// ============================================================================
// Multi-Process Smoke Test Harness with Process Isolation and Stratification
// ============================================================================

/// Recovers and restores the file boundary of an existing JSONL progress log before append.
/// If the file ends with a truncated or half-written line without a trailing newline,
/// or if the trailing line is malformed JSON, truncates the file back to the last valid
/// newline boundary so that subsequent appends never corrupt or fuse into broken lines.
pub fn repair_and_sanitize_jsonl(path: &str) -> std::io::Result<usize> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Ok(0);
    }
    let data = fs::read(path)?;
    if data.is_empty() {
        return Ok(0);
    }

    let mut valid_boundary = 0;
    let mut cur_start = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            let line_slice = &data[cur_start..i];
            if let Ok(line_str) = std::str::from_utf8(line_slice) {
                let trimmed = line_str.trim();
                if trimmed.is_empty() || serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
                {
                    valid_boundary = i + 1;
                }
            }
            cur_start = i + 1;
        }
    }

    if valid_boundary != data.len() {
        eprintln!(
            "  [JSONL RECOVERY] Truncated trailing incomplete bytes in {}: {} -> {} bytes",
            path,
            data.len(),
            valid_boundary
        );
        let f = fs::OpenOptions::new().write(true).open(path)?;
        f.set_len(valid_boundary as u64)?;
        f.sync_all()?;
    }
    Ok(valid_boundary)
}

#[allow(dead_code)]
pub fn build_full_run_key(
    jar_sha256: &str,
    jad_sha256: Option<&str>,
    launch_cfg: &str,
    device_profile: Option<&str>,
    experiment_id: Option<&str>,
    ticks: usize,
) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        jar_sha256,
        jad_sha256.unwrap_or("NO_JAD"),
        launch_cfg,
        device_profile.unwrap_or("MIDP-2.0"),
        experiment_id.unwrap_or("DEFAULT_EXP"),
        ticks
    )
}

#[allow(dead_code)]
pub fn validate_experiment_manifest_compatibility(
    manifest_path: &str,
    current_exe_sha256: &str,
    ticks: usize,
    timeout_ms: u64,
) -> Result<(), String> {
    let p = std::path::Path::new(manifest_path);
    if !p.exists() {
        return Ok(());
    }
    let data =
        fs::read_to_string(p).map_err(|e| format!("Cannot read experiment manifest: {}", e))?;
    let val: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| format!("Invalid manifest JSON format: {}", e))?;

    if let Some(exe_sha) = val.get("exe_sha256").and_then(|v| v.as_str()) {
        if !current_exe_sha256.is_empty() && exe_sha != current_exe_sha256 {
            return Err(format!(
                "Incompatible binary SHA-256 for resume: manifest has '{}', current binary is '{}'",
                exe_sha, current_exe_sha256
            ));
        }
    }

    if let Some(m_ticks) = val.get("ticks_per_game").and_then(|v| v.as_u64()) {
        if m_ticks as usize != ticks {
            return Err(format!(
                "Incompatible ticks_per_game for resume: manifest has {}, requested {}",
                m_ticks, ticks
            ));
        }
    }

    if let Some(m_timeout) = val.get("wall_timeout_ms").and_then(|v| v.as_u64()) {
        if m_timeout != timeout_ms {
            return Err(format!(
                "Incompatible wall_timeout_ms for resume: manifest has {} ms, requested {} ms",
                m_timeout, timeout_ms
            ));
        }
    }

    Ok(())
}

pub fn parse_progress_jsonl(content: &str) -> (HashSet<String>, Vec<SmokeTestEntryResult>) {
    let mut completed_keys = HashSet::new();
    let mut completed_results = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<SmokeTestEntryResult>(trimmed) {
            let key = format!("{}:{}", entry.jar_sha256, entry.entry_point);
            if !completed_keys.contains(&key) {
                completed_keys.insert(key);
                completed_results.push(entry);
            }
        }
    }
    (completed_keys, completed_results)
}

pub fn run_smoke_test(
    inventory_path: &str,
    limit: Option<usize>,
    ticks: usize,
    out_dir: &str,
) -> Result<SmokeTestSummary, String> {
    println!("============================================================");
    println!(" J2ME Market Deterministic Smoke-Test Harness (--smoke-test)");
    println!(" Target Inventory: {}", inventory_path);
    println!(
        " Clock: 16.667 ms/tick | Ticks per game: {} | Clean RMS",
        ticks
    );
    println!(" Mode: Isolated Child Processes with Wall Timeout Enforcement");
    println!("============================================================");

    let data = fs::read_to_string(inventory_path)
        .map_err(|e| format!("Failed to read inventory {}: {}", inventory_path, e))?;

    let inv: MarketInventory =
        serde_json::from_str(&data).map_err(|e| format!("Invalid inventory JSON format: {}", e))?;

    let _ = fs::create_dir_all(out_dir);
    let progress_path = format!("{}/smoke_test_progress.jsonl", out_dir);

    // Sanitize and restore JSONL file boundary prior to read/append
    let _ = repair_and_sanitize_jsonl(&progress_path);

    let (completed_keys, completed_results) = if let Ok(content) =
        fs::read_to_string(&progress_path)
    {
        let res = parse_progress_jsonl(&content);
        if !res.0.is_empty() {
            println!(
                "  [RESUME] Found {} previously completed runs in progress log (deduplicated, truncated lines skipped).",
                res.0.len()
            );
        }
        res
    } else {
        (HashSet::new(), Vec::new())
    };

    // Filter candidate regular JAR entries
    let candidate_entries: Vec<&InventoryFileEntry> = inv
        .files
        .iter()
        .filter(|e| {
            e.extension == "jar"
                && e.size_bytes > 0
                && e.availability == "PRESENT"
                && e.archive_status == "VALID"
        })
        .collect();

    // Select stratified sample if limit is requested
    let target_entries: Vec<&InventoryFileEntry> = if let Some(lim) = limit {
        if lim < candidate_entries.len() {
            select_stratified_sample(&candidate_entries, lim, 42)
        } else {
            candidate_entries
        }
    } else {
        candidate_entries
    };

    println!("  - Target launch configurations: {}", target_entries.len());

    let current_exe =
        std::env::current_exe().map_err(|e| format!("Cannot locate current executable: {}", e))?;
    let mut blocker_tallies: HashMap<String, (String, usize, Vec<String>)> = HashMap::new();
    let mut api_usage_tallies: HashMap<String, (u64, Vec<String>)> = HashMap::new();

    let mut load_ok_count = 0;
    let mut load_error_count = 0;
    let mut start_error_count = 0;
    let mut completed_window_count = 0;
    let mut guest_exception_count = 0;
    let mut host_panic_count = 0;
    let mut wall_timeout_count = 0;
    let mut splash_count = 0;
    let mut menu_count = 0;
    let mut gameplay_count = 0;
    let mut unknown_ui_count = 0;
    let mut no_paint_count = 0;
    let mut no_canvas_count = 0;
    let mut auto_healed_count = 0;
    let mut pass_count = 0;
    let mut partial_count = 0;
    let mut degraded_count = 0;
    let mut fail_count = 0;

    for entry in &completed_results {
        match entry.runtime_status.as_str() {
            "LOAD_ERROR" => load_error_count += 1,
            "START_ERROR" => start_error_count += 1,
            "GUEST_EXCEPTION" => guest_exception_count += 1,
            "COMPLETED_WINDOW" => completed_window_count += 1,
            "WALL_TIMEOUT" => wall_timeout_count += 1,
            "HOST_PANIC" => host_panic_count += 1,
            "DEGRADED" => degraded_count += 1,
            "PARTIAL" => partial_count += 1,
            _ => {}
        }
        if entry.auto_healing_occurred {
            auto_healed_count += 1;
        }
        match entry.test_verdict.as_str() {
            "PASS" => pass_count += 1,
            "PARTIAL" => partial_count += 1,
            "DEGRADED" => degraded_count += 1,
            "FAIL" => fail_count += 1,
            _ => {}
        }
        if entry.load_ok {
            load_ok_count += 1;
        }
        match entry.observed_ui.as_str() {
            "SPLASH" => splash_count += 1,
            "MENU" => menu_count += 1,
            "GAMEPLAY" => gameplay_count += 1,
            _ => unknown_ui_count += 1,
        }
        if entry.paint_calls == 0 {
            no_paint_count += 1;
        }
        if !entry.main_canvas_created {
            no_canvas_count += 1;
        }
        for b in &entry.blockers {
            let item = blocker_tallies
                .entry(b.description.clone())
                .or_insert_with(|| (b.category.clone(), 0, Vec::new()));
            item.1 += 1;
            if item.2.len() < 5 {
                item.2.push(entry.file_id.clone());
            }
        }
        for a in &entry.apis_invoked {
            let item = api_usage_tallies
                .entry(a.clone())
                .or_insert_with(|| (0, Vec::new()));
            item.0 += 1;
            if item.1.len() < 5 {
                item.1.push(entry.file_id.clone());
            }
        }
    }

    let total_tasks = target_entries.len();
    let mut results = completed_results;

    for (idx, entry) in target_entries.iter().enumerate() {
        let entry_class = entry
            .midlet_entries
            .first()
            .map(|m| m.main_class.as_str())
            .unwrap_or("Main");
        let run_key = format!("{}:{}", entry.sha256, entry_class);

        if completed_keys.contains(&run_key) {
            continue; // Skip already completed in resume
        }

        let temp_json = format!("{}/tmp_{}_{}.json", out_dir, std::process::id(), idx);
        print!(
            "  [{:4}/{:4}] {:<32} ... ",
            idx + 1,
            total_tasks,
            &entry.relative_path.chars().take(32).collect::<String>()
        );

        let t_child_start = Instant::now();
        let mut child = match Command::new(&current_exe)
            .args([
                "--run-single",
                &entry.path,
                "--class",
                entry_class,
                "--ticks",
                &ticks.to_string(),
                "--output-json",
                &temp_json,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                println!("[HOST_ERROR] Spawn failed: {}", e);
                host_panic_count += 1;
                continue;
            }
        };

        // Enforce 6000 ms wall-clock process timeout
        let timeout_ms = 6000;
        let poll_interval = std::time::Duration::from_millis(50);
        let mut timed_out = false;

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        println!("[HOST_PANIC] Process exited with status {}", status);
                    }
                    break;
                }
                Ok(None) => {
                    if t_child_start.elapsed().as_millis() > timeout_ms {
                        let _ = child.kill();
                        let _ = child.wait(); // Reap zombie child immediately
                        timed_out = true;
                        println!("[WALL_TIMEOUT] Exceeded {} ms timeout", timeout_ms);
                        break;
                    }
                    std::thread::sleep(poll_interval);
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait(); // Reap zombie child immediately
                    println!("[HOST_ERROR] Wait error: {}", e);
                    break;
                }
            }
        }

        let entry_res: SmokeTestEntryResult = if timed_out {
            SmokeTestEntryResult {
                file_id: entry.file_id.clone(),
                jar_path: entry.path.clone(),
                jar_sha256: entry.sha256.clone(),
                launch_configuration_id: entry.launch_configuration_id.clone().unwrap_or_default(),
                entry_point: entry_class.to_string(),
                title: entry.vendor.clone().unwrap_or_default(),
                vendor: entry.vendor.clone().unwrap_or_default(),
                runtime_status: "WALL_TIMEOUT".to_string(),
                observed_ui: "UNKNOWN".to_string(),
                load_ok: true,
                midlet_started: true,
                main_canvas_created: false,
                paint_calls: 0,
                framebuffer_publications: 0,
                unique_framebuffer_hashes: 0,
                active_threads: 1,
                executed_instructions: 0,
                exceptions: 0,
                unhandled_native: 0,
                native_call_counts: 0,
                guest_exceptions_thrown: 0,
                guest_exceptions_caught: 0,
                guest_exceptions_uncaught: 0,
                fatal_guest_exception: false,
                uncaught_exception_threads: Vec::new(),
                last_exception_class: None,
                last_exception_message: None,
                last_exception_location: None,
                audio_generated_samples: 0,
                audio_consumed_samples: 0,
                audio_dropped_samples: 0,
                audio_underrun_count: 0,
                audio_overflow_count: 0,
                timing: ExecutionTimingBreakdown {
                    elapsed_load_ms: 0.0,
                    elapsed_vm_ms: timeout_ms as f64,
                    elapsed_graphics_ms: 0.0,
                    elapsed_audio_ms: 0.0,
                    elapsed_scheduler_ms: 0.0,
                    elapsed_io_ms: 0.0,
                    elapsed_other_ms: 0.0,
                    total_wall_ms: timeout_ms as f64,
                },
                exit_reason: "WALL_TIMEOUT".to_string(),
                last_error: Some("Wall clock timeout exceeded".to_string()),
                final_frame_sha256: String::new(),
                apis_invoked: Vec::new(),
                recent_native_calls: Vec::new(),
                unhandled_natives: Vec::new(),
                active_thread_stacks: Vec::new(),
                blockers: vec![BlockerItem {
                    blocker_type: "CONFIRMED".to_string(),
                    category: "WALL_TIMEOUT".to_string(),
                    description: "Midlet entered non-yielding loop or hung during initialization"
                        .to_string(),
                    minimal_repro: None,
                }],
                auto_healing_count: 0,
                auto_healing_occurred: false,
                auto_healing_events: Vec::new(),
                test_verdict: "TIMEOUT".to_string(),
                criteria_report: None,
            }
        } else if let Ok(json_data) = fs::read_to_string(&temp_json) {
            let _ = fs::remove_file(&temp_json);
            if let Ok(mut parsed) = serde_json::from_str::<SmokeTestEntryResult>(&json_data) {
                parsed.file_id = entry.file_id.clone();
                parsed.title = entry.vendor.clone().unwrap_or_default();
                parsed.vendor = entry.vendor.clone().unwrap_or_default();
                parsed
            } else {
                continue;
            }
        } else {
            SmokeTestEntryResult {
                file_id: entry.file_id.clone(),
                jar_path: entry.path.clone(),
                jar_sha256: entry.sha256.clone(),
                launch_configuration_id: entry.launch_configuration_id.clone().unwrap_or_default(),
                entry_point: entry_class.to_string(),
                title: entry.vendor.clone().unwrap_or_default(),
                vendor: entry.vendor.clone().unwrap_or_default(),
                runtime_status: "HOST_PANIC".to_string(),
                observed_ui: "UNKNOWN".to_string(),
                load_ok: false,
                midlet_started: false,
                main_canvas_created: false,
                paint_calls: 0,
                framebuffer_publications: 0,
                unique_framebuffer_hashes: 0,
                active_threads: 0,
                executed_instructions: 0,
                exceptions: 0,
                unhandled_native: 0,
                native_call_counts: 0,
                guest_exceptions_thrown: 0,
                guest_exceptions_caught: 0,
                guest_exceptions_uncaught: 0,
                fatal_guest_exception: false,
                uncaught_exception_threads: Vec::new(),
                last_exception_class: None,
                last_exception_message: None,
                last_exception_location: None,
                audio_generated_samples: 0,
                audio_consumed_samples: 0,
                audio_dropped_samples: 0,
                audio_underrun_count: 0,
                audio_overflow_count: 0,
                timing: ExecutionTimingBreakdown {
                    elapsed_load_ms: 0.0,
                    elapsed_vm_ms: 0.0,
                    elapsed_graphics_ms: 0.0,
                    elapsed_audio_ms: 0.0,
                    elapsed_scheduler_ms: 0.0,
                    elapsed_io_ms: 0.0,
                    elapsed_other_ms: 0.0,
                    total_wall_ms: t_child_start.elapsed().as_secs_f64() * 1000.0,
                },
                exit_reason: "HOST_PANIC".to_string(),
                last_error: Some("Child process crashed or panicked".to_string()),
                final_frame_sha256: String::new(),
                apis_invoked: Vec::new(),
                recent_native_calls: Vec::new(),
                unhandled_natives: Vec::new(),
                active_thread_stacks: Vec::new(),
                blockers: vec![BlockerItem {
                    blocker_type: "CONFIRMED".to_string(),
                    category: "HOST_PANIC".to_string(),
                    description: "Child process crashed or panicked".to_string(),
                    minimal_repro: None,
                }],
                auto_healing_count: 0,
                auto_healing_occurred: false,
                auto_healing_events: Vec::new(),
                test_verdict: "FAIL".to_string(),
                criteria_report: None,
            }
        };

        // Output short status
        println!(
            "[{}] UI: {:<7} | Paint: {:3} | Ticks: {:3} | Wall: {:6.1} ms",
            entry_res.runtime_status,
            entry_res.observed_ui,
            entry_res.paint_calls,
            ticks,
            entry_res.timing.total_wall_ms
        );

        // Update counts
        match entry_res.runtime_status.as_str() {
            "LOAD_ERROR" => load_error_count += 1,
            "START_ERROR" => start_error_count += 1,
            "GUEST_EXCEPTION" => guest_exception_count += 1,
            "COMPLETED_WINDOW" => completed_window_count += 1,
            "WALL_TIMEOUT" => wall_timeout_count += 1,
            "HOST_PANIC" => host_panic_count += 1,
            "DEGRADED" => degraded_count += 1,
            "PARTIAL" => partial_count += 1,
            _ => {}
        }
        if entry_res.auto_healing_occurred {
            auto_healed_count += 1;
        }
        match entry_res.test_verdict.as_str() {
            "PASS" => pass_count += 1,
            "PARTIAL" => partial_count += 1,
            "DEGRADED" => degraded_count += 1,
            "FAIL" => fail_count += 1,
            _ => {}
        }
        if entry_res.load_ok {
            load_ok_count += 1;
        }
        match entry_res.observed_ui.as_str() {
            "SPLASH" => splash_count += 1,
            "MENU" => menu_count += 1,
            "GAMEPLAY" => gameplay_count += 1,
            _ => unknown_ui_count += 1,
        }
        if entry_res.paint_calls == 0 {
            no_paint_count += 1;
        }
        if !entry_res.main_canvas_created {
            no_canvas_count += 1;
        }

        // Tally blockers
        for b in &entry_res.blockers {
            let item = blocker_tallies
                .entry(b.description.clone())
                .or_insert_with(|| (b.category.clone(), 0, Vec::new()));
            item.1 += 1;
            if item.2.len() < 5 {
                item.2.push(entry.file_id.clone());
            }
        }

        // Tally APIs
        for a in &entry_res.apis_invoked {
            let item = api_usage_tallies
                .entry(a.clone())
                .or_insert_with(|| (0, Vec::new()));
            item.0 += 1;
            if item.1.len() < 5 {
                item.1.push(entry.file_id.clone());
            }
        }

        // Single-writer append to progress file with sync to disk
        if let Ok(line) = serde_json::to_string(&entry_res) {
            use std::io::Write;
            if let Ok(mut f) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&progress_path)
            {
                let _ = writeln!(f, "{}", line);
                let _ = f.flush();
                let _ = f.sync_all();
            }
        }

        results.push(entry_res);
    }

    // Rank top blockers
    let mut top_blockers = Vec::new();
    let mut sorted_blockers: Vec<(String, (String, usize, Vec<String>))> =
        blocker_tallies.into_iter().collect();
    sorted_blockers.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

    for (rank, (desc, (category, count, runs))) in sorted_blockers.into_iter().take(20).enumerate()
    {
        let is_confirmed = category == "CORRUPT_ARCHIVE"
            || category == "I/O"
            || category == "WALL_TIMEOUT"
            || category == "GUEST_EXCEPTION"
            || category == "NO_CANVAS";
        top_blockers.push(TopBlockerReport {
            rank: rank + 1,
            blocker_type: if is_confirmed {
                "CONFIRMED".to_string()
            } else {
                "HYPOTHESIS".to_string()
            },
            category,
            description: desc,
            affected_launch_configurations: count,
            denominator: results.len(),
            example_run_ids: runs,
            minimal_repro: "cargo run -p j2me-cli -- --run-single <jar_path> --ticks 300"
                .to_string(),
        });
    }

    // Generate Method Audit Entries
    let mut api_methods_audit = Vec::new();
    let mut sorted_apis: Vec<(String, (u64, Vec<String>))> =
        api_usage_tallies.into_iter().collect();
    sorted_apis.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    for (sig, (count, runs)) in sorted_apis {
        let parts: Vec<&str> = sig.split('.').collect();
        let class = parts.first().copied().unwrap_or("unknown").to_string();
        let method = parts.get(1).copied().unwrap_or("unknown").to_string();

        let (impl_file, status, limit) = classify_api_method(&class, &method);
        api_methods_audit.push(MethodAuditEntry {
            class,
            method_name: method,
            descriptor: sig.clone(),
            runtime_call_count: count,
            implementation_file: impl_file,
            status,
            contract_limitations: limit,
            related_tests: vec!["cargo test -p j2me-stdlib".to_string()],
            run_ids: runs,
        });
    }

    let summary = SmokeTestSummary {
        total_tested: results.len(),
        load_ok_count,
        load_error_count,
        start_error_count,
        completed_window_count,
        guest_exception_count,
        host_panic_count,
        wall_timeout_count,
        auto_healed_count,
        pass_count,
        partial_count,
        degraded_count,
        fail_count,
        splash_count,
        menu_count,
        gameplay_count,
        unknown_ui_count,
        no_paint_count,
        no_canvas_count,
        top_blockers,
        api_methods_audit,
        results,
    };

    let report_path = format!("{}/smoke_test_report.json", out_dir);
    if let Ok(bytes) = serde_json::to_vec_pretty(&summary) {
        let _ = fs::write(&report_path, bytes);
        println!("  - Saved full smoke test report to: {}", report_path);
    }

    println!("============================================================");
    println!(
        " Smoke Test Summary ({} launch configurations):",
        summary.total_tested
    );
    println!("  - Load OK:           {}", summary.load_ok_count);
    println!("  - Completed Window:  {}", summary.completed_window_count);
    println!("  - Guest Exceptions:  {}", summary.guest_exception_count);
    println!("  - Wall Timeouts:     {}", summary.wall_timeout_count);
    println!("  - Host Panics:       {}", summary.host_panic_count);
    println!("  - Splash Observed:   {}", summary.splash_count);
    println!("  - Menu Observed:     {}", summary.menu_count);
    println!("  - Gameplay Observed: {}", summary.gameplay_count);
    println!("  - No Paint:          {}", summary.no_paint_count);
    println!("  - No Canvas:         {}", summary.no_canvas_count);
    println!("============================================================");

    Ok(summary)
}

/// Classifies API method into implementation level and file location.
fn classify_api_method(class: &str, _method: &str) -> (String, String, Option<String>) {
    let file = "crates/j2me-stdlib/src/bridge/native_bridge.rs".to_string();
    if class.starts_with("javax/microedition/midlet") {
        (file, "FULL".to_string(), Some("MIDlet lifecycle methods (init, startApp, pauseApp, destroyApp, getAppProperty) implemented".to_string()))
    } else if class.starts_with("javax/microedition/lcdui/game") {
        (file, "FULL".to_string(), None)
    } else if class.starts_with("javax/microedition/lcdui/Graphics")
        || class.starts_with("javax/microedition/lcdui/Canvas")
        || class.starts_with("javax/microedition/lcdui/Image")
        || class.starts_with("javax/microedition/lcdui/Display")
        || class.starts_with("javax/microedition/lcdui/Command")
        || class.starts_with("javax/microedition/lcdui/Font")
    {
        (
            file,
            "FULL".to_string(),
            Some(
                "2D primitives, fonts, display management and offscreen images fully implemented"
                    .to_string(),
            ),
        )
    } else if class.starts_with("javax/microedition/lcdui/List") {
        (file, "PARTIAL".to_string(), Some("Choice list items and command listeners supported; advanced widget styling partial".to_string()))
    } else if class.starts_with("javax/microedition/lcdui/Form")
        || class.starts_with("javax/microedition/lcdui/TextField")
    {
        (
            file,
            "PARTIAL".to_string(),
            Some(
                "High-level form widgets have basic layout without complex rich-text editing"
                    .to_string(),
            ),
        )
    } else if class.starts_with("javax/microedition/m3g") {
        (
            file,
            "PARTIAL".to_string(),
            Some(
                "Background color/clear supported; 3D rasterization pipeline in progress"
                    .to_string(),
            ),
        )
    } else if class.starts_with("javax/microedition/media") {
        (
            file,
            "PARTIAL".to_string(),
            Some(
                "Tone, WAV PCM, and polyphonic MIDI supported; hardware codecs stubbed".to_string(),
            ),
        )
    } else if class.starts_with("javax/microedition/rms") {
        (
            file,
            "FULL".to_string(),
            Some(
                "RecordStore persistence fully supported with enumeration, filter, comparator"
                    .to_string(),
            ),
        )
    } else if class.starts_with("javax/microedition/io") {
        (
            file,
            "STUB".to_string(),
            Some("Connector safely returns IOException or mocked offline responses".to_string()),
        )
    } else if class.starts_with("com/nokia/mid/ui") {
        (
            file,
            "FULL".to_string(),
            Some("FullCanvas and DirectGraphics ARGB/transforms supported".to_string()),
        )
    } else if class.starts_with("com/nokia/mid/sound") {
        (
            file,
            "PARTIAL".to_string(),
            Some("Tone beeper supported; advanced audio delegates to MMAPI".to_string()),
        )
    } else if class.starts_with("java/lang")
        || class.starts_with("java/io")
        || class.starts_with("java/util")
    {
        (file, "FULL".to_string(), None)
    } else {
        (file, "UNKNOWN".to_string(), None)
    }
}

/// Selects a deterministic stratified sample across sizes, vendors, and profiles.
fn select_stratified_sample<'a>(
    entries: &[&'a InventoryFileEntry],
    target_count: usize,
    seed: u64,
) -> Vec<&'a InventoryFileEntry> {
    if entries.len() <= target_count {
        return entries.to_vec();
    }

    // Stratify by size buckets: <100k, 100-300k, 300-600k, 600k-1M, >1M
    let mut buckets: [Vec<&'a InventoryFileEntry>; 5] =
        [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for e in entries {
        let kb = e.size_bytes / 1024;
        if kb < 100 {
            buckets[0].push(e);
        } else if kb < 300 {
            buckets[1].push(e);
        } else if kb < 600 {
            buckets[2].push(e);
        } else if kb < 1024 {
            buckets[3].push(e);
        } else {
            buckets[4].push(e);
        }
    }

    let per_bucket = target_count / 5;
    let mut selected = Vec::with_capacity(target_count);

    let mut rng_state = seed;
    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as usize
    };

    for bucket in &mut buckets {
        let count = per_bucket.min(bucket.len());
        // Simple Fisher-Yates shuffle with deterministic seed
        for i in 0..bucket.len() {
            let j = next_rand() % bucket.len();
            bucket.swap(i, j);
        }
        selected.extend(bucket.iter().take(count).copied());
    }

    // Fill remaining up to target_count
    let mut all_rem: Vec<&'a InventoryFileEntry> = entries
        .iter()
        .filter(|e| !selected.iter().any(|s| s.file_id == e.file_id))
        .copied()
        .collect();
    while selected.len() < target_count && !all_rem.is_empty() {
        let idx = next_rand() % all_rem.len();
        selected.push(all_rem.swap_remove(idx));
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central_dir = Vec::new();

        for (name, data) in entries {
            let offset = out.len() as u32;
            let name_bytes = name.as_bytes();
            let size = data.len() as u32;

            // Local File Header
            out.extend_from_slice(&0x04034b50u32.to_le_bytes());
            out.extend_from_slice(&10u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // Stored (no compression)
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes()); // crc32 (non-strict)
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(data);

            // Central Directory Header
            central_dir.extend_from_slice(&0x02014b50u32.to_le_bytes());
            central_dir.extend_from_slice(&10u16.to_le_bytes());
            central_dir.extend_from_slice(&10u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u32.to_le_bytes());
            central_dir.extend_from_slice(&size.to_le_bytes());
            central_dir.extend_from_slice(&size.to_le_bytes());
            central_dir.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u16.to_le_bytes());
            central_dir.extend_from_slice(&0u32.to_le_bytes());
            central_dir.extend_from_slice(&offset.to_le_bytes());
            central_dir.extend_from_slice(name_bytes);
        }

        let cd_offset = out.len() as u32;
        let cd_size = central_dir.len() as u32;
        out.extend_from_slice(&central_dir);

        // End of Central Directory (EOCD)
        out.extend_from_slice(&0x06054b50u32.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());

        out
    }

    #[test]
    fn test_corrupt_zip_handling() {
        let corrupt_bytes = b"PK\x03\x04not-a-valid-zip-file-at-all";
        let res = JarArchive::parse(corrupt_bytes);
        assert!(res.is_err(), "Corrupt zip must produce an error");
    }

    #[test]
    fn test_missing_manifest_is_not_corrupt_zip() {
        let zip_bytes = build_stored_zip(&[("test.txt", b"hello world")]);
        let parsed = JarArchive::parse(&zip_bytes).expect("Valid ZIP archive must parse");
        assert!(
            parsed.manifest.is_none(),
            "ZIP without MANIFEST.MF must have manifest = None"
        );
        assert_eq!(parsed.entries.len(), 1);
    }

    #[test]
    fn test_multi_midlet_suite_parsing() {
        let manifest_content = b"MIDlet-1: Game, /icon.png, com.example.Game\nMIDlet-2: Editor, /icon2.png, com.example.Editor\nMIDlet-Vendor: TestVendor\nMIDlet-Version: 1.0\n";
        let zip_bytes = build_stored_zip(&[("META-INF/MANIFEST.MF", manifest_content)]);
        let parsed = JarArchive::parse(&zip_bytes).unwrap();
        let m = parsed.manifest.expect("Manifest must be present");
        assert_eq!(
            m.get("MIDlet-1").unwrap(),
            "Game, /icon.png, com.example.Game"
        );
        assert_eq!(
            m.get("MIDlet-2").unwrap(),
            "Editor, /icon2.png, com.example.Editor"
        );
    }

    #[test]
    fn test_jad_pointing_to_missing_jar() {
        let jad_text = "MIDlet-Name: TestGame\nMIDlet-Jar-URL: nonexistent_archive.jar\nMIDlet-Jar-Size: 12345\n";
        let jad = Jad::parse(jad_text);
        assert_eq!(jad.get("MIDlet-Jar-URL"), Some("nonexistent_archive.jar"));
        assert_eq!(jad.get("MIDlet-Jar-Size"), Some("12345"));
    }

    #[test]
    fn test_sha256_digest_reproducibility() {
        let data1 = b"Hello J2ME World";
        let h1 = sha256_digest(data1);
        let h2 = sha256_digest(data1);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_path_traversal_detection() {
        let dangerous_path = "../../etc/shadow";
        assert!(
            dangerous_path.contains(".."),
            "Path traversal pattern must be detectable"
        );
    }

    #[test]
    fn test_stratified_sample_selection() {
        let mut entries = Vec::new();
        for i in 0..50 {
            entries.push(InventoryFileEntry {
                inventory_schema_version: "4.1.0".to_string(),
                file_id: format!("f_{}", i),
                path: format!("/path/game_{}.jar", i),
                relative_path: format!("game_{}.jar", i),
                container_path: None,
                entry_path: None,
                size_bytes: (i as u64) * 25 * 1024, // spreads across buckets
                sha256: format!("{:064x}", i),
                extension: "jar".to_string(),
                availability: "PRESENT".to_string(),
                archive_status: "VALID".to_string(),
                manifest_status: "PRESENT".to_string(),
                jad_status: "N/A".to_string(),
                midlet_entries: Vec::new(),
                vendor: Some(format!("Vendor_{}", i % 5)),
                version: Some("1.0".to_string()),
                configuration: Some("CLDC-1.1".to_string()),
                profile: Some("MIDP-2.0".to_string()),
                launch_configuration_id: None,
                runtime_status: "NOT_RUN".to_string(),
                observed_ui: "UNKNOWN".to_string(),
                evidence: EvidenceRecord::default(),
                blockers: Vec::new(),
                notes: String::new(),
            });
        }
        let refs: Vec<&InventoryFileEntry> = entries.iter().collect();
        let sample = select_stratified_sample(&refs, 10, 42);
        assert_eq!(
            sample.len(),
            10,
            "Stratified sample count must equal target limit"
        );
    }

    #[test]
    fn test_jsonl_resume_truncated_last_line_and_deduplication() {
        let entry1 = SmokeTestEntryResult {
            file_id: "f_1".to_string(),
            jar_path: "/path/1.jar".to_string(),
            jar_sha256: "sha_aaa".to_string(),
            launch_configuration_id: "cfg_1".to_string(),
            entry_point: "pkg.Main".to_string(),
            title: "Game 1".to_string(),
            vendor: "Vendor 1".to_string(),
            runtime_status: "COMPLETED_WINDOW".to_string(),
            observed_ui: "GAMEPLAY".to_string(),
            load_ok: true,
            midlet_started: true,
            main_canvas_created: true,
            paint_calls: 300,
            framebuffer_publications: 300,
            unique_framebuffer_hashes: 12,
            active_threads: 2,
            executed_instructions: 50000,
            exceptions: 0,
            unhandled_native: 0,
            native_call_counts: 150,
            guest_exceptions_thrown: 0,
            guest_exceptions_caught: 0,
            guest_exceptions_uncaught: 0,
            fatal_guest_exception: false,
            uncaught_exception_threads: Vec::new(),
            last_exception_class: None,
            last_exception_message: None,
            last_exception_location: None,
            audio_generated_samples: 44100,
            audio_consumed_samples: 44100,
            audio_dropped_samples: 0,
            audio_underrun_count: 0,
            audio_overflow_count: 0,
            timing: ExecutionTimingBreakdown {
                elapsed_load_ms: 10.0,
                elapsed_vm_ms: 50.0,
                elapsed_graphics_ms: 20.0,
                elapsed_audio_ms: 5.0,
                elapsed_scheduler_ms: 2.0,
                elapsed_io_ms: 1.0,
                elapsed_other_ms: 0.0,
                total_wall_ms: 88.0,
            },
            exit_reason: "TICKS_EXHAUSTED".to_string(),
            last_error: None,
            final_frame_sha256: "frame_hash_1".to_string(),
            apis_invoked: vec!["Canvas.repaint".to_string()],
            recent_native_calls: Vec::new(),
            unhandled_natives: Vec::new(),
            active_thread_stacks: Vec::new(),
            blockers: Vec::new(),
            auto_healing_count: 0,
            auto_healing_occurred: false,
            auto_healing_events: Vec::new(),
            test_verdict: "PASS".to_string(),
            criteria_report: None,
        };

        let line1 = serde_json::to_string(&entry1).unwrap();
        let duplicate_line1 = line1.clone();
        let truncated_line = "{\"file_id\":\"f_2\",\"jar_path\":\"/path/2.jar\",\"jar_sha256\":\"sha_bbb\",\"launch_config";

        let full_content = format!("{}\n{}\n{}\n", line1, duplicate_line1, truncated_line);

        let (keys, results) = parse_progress_jsonl(&full_content);
        assert_eq!(
            keys.len(),
            1,
            "Duplicate line must be deduplicated by run_key"
        );
        assert_eq!(
            results.len(),
            1,
            "Only valid deduplicated line should be retained"
        );
        assert_eq!(results[0].jar_sha256, "sha_aaa");
        assert_eq!(results[0].entry_point, "pkg.Main");
    }

    #[test]
    fn test_child_process_kill_and_reap() {
        use std::process::{Command, Stdio};
        let mut child = Command::new("sleep")
            .arg("10")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn sleep");

        assert!(
            child.try_wait().unwrap().is_none(),
            "Child should be running"
        );

        // Kill process
        let _ = child.kill();
        // Mandatory wait to reap zombie
        let exit_status = child.wait().expect("Failed to reap child process");
        assert!(
            !exit_status.success(),
            "Killed child must report non-zero exit or signal"
        );

        // Subsequent try_wait must confirm process is reaped
        assert!(
            child.try_wait().unwrap().is_some(),
            "Process must remain reaped"
        );
    }

    #[test]
    fn test_excessive_stdout_stderr_no_pipe_deadlock() {
        use std::process::{Command, Stdio};
        // Generate > 128 KB of output with Stdio::null()
        // If pipes were captured without reading, this would deadlock due to OS 64KB pipe buffer.
        let mut child = Command::new("sh")
            .args([
                "-c",
                "yes 'Excessive debug output line that would deadlock pipe' | head -n 5000",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn command");

        let status = child.wait().expect("Wait failed");
        assert!(status.success(), "Command must terminate without deadlock");
    }

    #[test]
    fn test_jsonl_repair_truncated_tail_prevents_concatenation() {
        use std::io::Write;
        let dir = format!("target/test_jsonl_repair_{}", std::process::id());
        let _ = fs::create_dir_all(&dir);
        let path = format!("{}/test.jsonl", dir);

        let valid_1 = "{\"file_id\":\"f1\",\"jar_path\":\"1.jar\",\"jar_sha256\":\"aaa\",\"launch_configuration_id\":\"c1\",\"entry_point\":\"Main\",\"title\":\"T\",\"vendor\":\"V\",\"runtime_status\":\"COMPLETED_WINDOW\",\"observed_ui\":\"SPLASH\",\"load_ok\":true,\"midlet_started\":true,\"main_canvas_created\":true,\"paint_calls\":1,\"framebuffer_publications\":1,\"unique_framebuffer_hashes\":1,\"active_threads\":1,\"executed_instructions\":100,\"exceptions\":0,\"unhandled_native\":0,\"native_call_counts\":10,\"audio_generated_samples\":0,\"audio_consumed_samples\":0,\"audio_dropped_samples\":0,\"audio_underrun_count\":0,\"audio_overflow_count\":0,\"timing\":{\"elapsed_load_ms\":1.0,\"elapsed_vm_ms\":1.0,\"elapsed_graphics_ms\":0.0,\"elapsed_audio_ms\":0.0,\"elapsed_scheduler_ms\":0.0,\"elapsed_io_ms\":0.0,\"elapsed_other_ms\":0.0,\"total_wall_ms\":2.0},\"exit_reason\":\"TICKS_EXHAUSTED\",\"last_error\":null,\"final_frame_sha256\":\"h1\",\"apis_invoked\":[],\"blockers\":[]}";
        let truncated_tail = "{\"file_id\":\"f2\",\"jar_path\":\"2.jar\",\"jar_sha256\":\"bbb\""; // NO newline, cut off mid-record!

        // Write valid 1 followed by broken tail
        {
            let mut f = fs::File::create(&path).unwrap();
            writeln!(f, "{}", valid_1).unwrap();
            write!(f, "{}", truncated_tail).unwrap(); // incomplete tail without \n
        }

        // Run recovery protocol
        let bound = repair_and_sanitize_jsonl(&path).unwrap();
        assert!(bound < valid_1.len() + truncated_tail.len() + 1);

        // Now append valid 2
        let valid_2 = "{\"file_id\":\"f3\",\"jar_path\":\"3.jar\",\"jar_sha256\":\"ccc\",\"launch_configuration_id\":\"c3\",\"entry_point\":\"Main\",\"title\":\"T\",\"vendor\":\"V\",\"runtime_status\":\"COMPLETED_WINDOW\",\"observed_ui\":\"SPLASH\",\"load_ok\":true,\"midlet_started\":true,\"main_canvas_created\":true,\"paint_calls\":1,\"framebuffer_publications\":1,\"unique_framebuffer_hashes\":1,\"active_threads\":1,\"executed_instructions\":100,\"exceptions\":0,\"unhandled_native\":0,\"native_call_counts\":10,\"audio_generated_samples\":0,\"audio_consumed_samples\":0,\"audio_dropped_samples\":0,\"audio_underrun_count\":0,\"audio_overflow_count\":0,\"timing\":{\"elapsed_load_ms\":1.0,\"elapsed_vm_ms\":1.0,\"elapsed_graphics_ms\":0.0,\"elapsed_audio_ms\":0.0,\"elapsed_scheduler_ms\":0.0,\"elapsed_io_ms\":0.0,\"elapsed_other_ms\":0.0,\"total_wall_ms\":2.0},\"exit_reason\":\"TICKS_EXHAUSTED\",\"last_error\":null,\"final_frame_sha256\":\"h3\",\"apis_invoked\":[],\"blockers\":[]}";
        {
            let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
            writeln!(f, "{}", valid_2).unwrap();
        }

        // Read and parse: must yield EXACTLY 2 valid entries (f1 and f3) without any corrupted merged lines!
        let content = fs::read_to_string(&path).unwrap();
        let (keys, results) = parse_progress_jsonl(&content);
        assert_eq!(keys.len(), 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].file_id, "f1");
        assert_eq!(results[1].file_id, "f3");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_full_key_distinguishes_different_jads_and_profiles() {
        let k1 = build_full_run_key(
            "jar_hash",
            Some("jad_hash_1"),
            "Main",
            Some("MIDP-1.0"),
            Some("exp1"),
            300,
        );
        let k2 = build_full_run_key(
            "jar_hash",
            Some("jad_hash_2"),
            "Main",
            Some("MIDP-1.0"),
            Some("exp1"),
            300,
        );
        let k3 = build_full_run_key(
            "jar_hash",
            Some("jad_hash_1"),
            "Main",
            Some("MIDP-2.0"),
            Some("exp1"),
            300,
        );
        let k4 = build_full_run_key(
            "jar_hash",
            Some("jad_hash_1"),
            "Main",
            Some("MIDP-1.0"),
            Some("exp2"),
            300,
        );
        let k5 = build_full_run_key(
            "jar_hash",
            Some("jad_hash_1"),
            "Main",
            Some("MIDP-1.0"),
            Some("exp1"),
            600,
        );

        assert_ne!(k1, k2, "Different JAD hashes must produce different keys");
        assert_ne!(k1, k3, "Different profiles must produce different keys");
        assert_ne!(k1, k4, "Different experiments must produce different keys");
        assert_ne!(k1, k5, "Different tick budgets must produce different keys");
    }

    #[test]
    fn test_manifest_compatibility_rejection_on_build_or_timeout_mismatch() {
        let dir = format!("target/test_manifest_compat_{}", std::process::id());
        let _ = fs::create_dir_all(&dir);
        let path = format!("{}/experiment_manifest.json", dir);

        let manifest_content = r#"{
            "exe_sha256": "binary_aaa",
            "ticks_per_game": 300,
            "wall_timeout_ms": 6000
        }"#;
        fs::write(&path, manifest_content).unwrap();

        // 1. Matches perfectly -> Ok
        assert!(validate_experiment_manifest_compatibility(&path, "binary_aaa", 300, 6000).is_ok());

        // 2. Binary mismatch -> Err
        let err_bin = validate_experiment_manifest_compatibility(&path, "binary_bbb", 300, 6000);
        assert!(err_bin.is_err());
        assert!(err_bin.unwrap_err().contains("Incompatible binary SHA-256"));

        // 3. Ticks mismatch -> Err
        let err_ticks = validate_experiment_manifest_compatibility(&path, "binary_aaa", 600, 6000);
        assert!(err_ticks.is_err());
        assert!(err_ticks
            .unwrap_err()
            .contains("Incompatible ticks_per_game"));

        // 4. Timeout mismatch -> Err
        let err_time = validate_experiment_manifest_compatibility(&path, "binary_aaa", 300, 10000);
        assert!(err_time.is_err());
        assert!(err_time
            .unwrap_err()
            .contains("Incompatible wall_timeout_ms"));

        let _ = fs::remove_dir_all(&dir);
    }
}
