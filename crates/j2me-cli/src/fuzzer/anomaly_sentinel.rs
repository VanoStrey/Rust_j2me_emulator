//! Anomaly Sentinel - Automated Detection of Runtime Anomalies vs Normal Operation.
//!
//! Evaluates emulator frames, thread states, instruction budgets, audio buffers,
//! and heap telemetry to detect:
//! - Deadlocks / Infinite Loops
//! - Black Screen stalls (> 180 ticks)
//! - Visual Freezes (> 300 ticks with SSIM >= 0.9999 despite delivered input)
//! - Audio distortion (NaN, Inf, continuous hard clipping)
//! - Heap leaks (monotonic growth unaffected by GC)

use j2me_core::threads::green_thread::ThreadState;
use j2me_stdlib::J2meEmulator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyKind {
    DeadlockOrInfiniteLoop { details: String },
    BlackScreen { ticks_count: usize },
    VisualFreeze { ssim: f64, frozen_ticks: usize },
    AudioAnomaly { details: String },
    HeapLeak { live_objects: usize, freed_by_gc: usize },
    HostCrash { error: String },
    FatalGuestException { exception_class: String, message: String },
    SilentExceptionLoop { count: u64, classes: Vec<String> },
    SplashLoopStuck { unique_scenes: usize, duration_ticks: usize },
    InputDecoupledIgnored { probes_sent: usize, reactions: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentinelVerdict {
    FullGameplayPass,
    MenuReachedPass,
    SplashLoopSuspicious,
    SilentExceptionFailure,
    HardAnomaly,
    LoadError,
    Panic,
}

impl std::fmt::Display for SentinelVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentinelVerdict::FullGameplayPass => write!(f, "FULL_GAMEPLAY_PASS"),
            SentinelVerdict::MenuReachedPass => write!(f, "MENU_REACHED_PASS"),
            SentinelVerdict::SplashLoopSuspicious => write!(f, "SPLASH_LOOP_SUSPICIOUS"),
            SentinelVerdict::SilentExceptionFailure => write!(f, "SILENT_EXCEPTION_FAILURE"),
            SentinelVerdict::HardAnomaly => write!(f, "HARD_ANOMALY"),
            SentinelVerdict::LoadError => write!(f, "LOAD_ERROR"),
            SentinelVerdict::Panic => write!(f, "PANIC"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub tick: usize,
    pub kind: AnomalyKind,
    pub total_instructions: u64,
    pub active_threads: usize,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct AnomalySentinel {
    pub consecutive_black_screen_ticks: usize,
    pub consecutive_identical_frame_ticks: usize,
    pub last_frame_luma: Vec<f32>,
    pub last_frame_hash: u64,
    pub prev_instructions: u64,
    pub heap_check_interval: usize,
    pub heap_history: Vec<usize>,
    pub audio_consecutive_clipped_samples: usize,
    pub consecutive_deadlock_ticks: usize,
    pub input_delivered_since_freeze_check: bool,

    // Sentinel 2.0 Objective Metrics
    pub unique_scene_hashes: std::collections::HashSet<u64>,
    pub pending_input_probe: Option<(usize, u64)>, // (probe_tick, baseline_frame_hash)
    pub causal_probes_sent: usize,
    pub causal_input_reactions: usize,
    pub total_audio_energy: f64,
    pub max_non_black_pixels: usize,
    pub critical_swallowed_exceptions: u64,
    pub caught_exception_classes: Vec<String>,
}

impl AnomalySentinel {
    pub fn new() -> Self {
        Self {
            consecutive_black_screen_ticks: 0,
            consecutive_identical_frame_ticks: 0,
            last_frame_luma: Vec::new(),
            last_frame_hash: 0,
            prev_instructions: 0,
            heap_check_interval: 50,
            heap_history: Vec::new(),
            audio_consecutive_clipped_samples: 0,
            consecutive_deadlock_ticks: 0,
            input_delivered_since_freeze_check: false,

            unique_scene_hashes: std::collections::HashSet::new(),
            pending_input_probe: None,
            causal_probes_sent: 0,
            causal_input_reactions: 0,
            total_audio_energy: 0.0,
            max_non_black_pixels: 0,
            critical_swallowed_exceptions: 0,
            caught_exception_classes: Vec::new(),
        }
    }

    /// Mark that input was delivered to the emulator this tick.
    pub fn note_input_delivered(&mut self) {
        self.input_delivered_since_freeze_check = true;
    }

    /// Register a causal input probe to check for game reaction.
    pub fn note_input_probe(&mut self, tick: usize, current_hash: u64) {
        self.note_input_delivered();
        if self.pending_input_probe.is_none() {
            self.pending_input_probe = Some((tick, current_hash));
            self.causal_probes_sent += 1;
        }
    }

    /// Check all sentinel invariants after a tick has completed.
    pub fn check_tick(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        audio_chunk: &[f32],
    ) -> Option<AnomalyReport> {
        let current_instructions = emu.scheduler.total_instructions();
        let instructions_delta = current_instructions.saturating_sub(self.prev_instructions);
        self.prev_instructions = current_instructions;

        // 1. Host Crash or Fatal Guest Exception Check
        if emu.context.exception_telemetry.host_panics > 0 {
            return Some(AnomalyReport {
                tick,
                kind: AnomalyKind::HostCrash {
                    error: "Host panic detected in emulator context".to_string(),
                },
                total_instructions: current_instructions,
                active_threads: emu.scheduler.active_thread_count(),
                summary: "Host crash / panic".to_string(),
            });
        }

        if let Some(ref exc) = emu.context.exception_telemetry.fatal_guest_exception {
            return Some(AnomalyReport {
                tick,
                kind: AnomalyKind::FatalGuestException {
                    exception_class: exc.exception_class.clone(),
                    message: exc.message.clone().unwrap_or_default(),
                },
                total_instructions: current_instructions,
                active_threads: emu.scheduler.active_thread_count(),
                summary: format!("Fatal uncaught guest exception: {}", exc.exception_class),
            });
        }

        // 2. Deadlock / Infinite Loop Check
        // Condition A: Thread burned > 5,000,000 instructions in a single tick without sleep/repaint
        if instructions_delta >= 5_000_000 && emu.context.repaints_this_tick == 0 {
            return Some(AnomalyReport {
                tick,
                kind: AnomalyKind::DeadlockOrInfiniteLoop {
                    details: format!(
                        "Single tick executed {} instructions without repainting or sleeping",
                        instructions_delta
                    ),
                },
                total_instructions: current_instructions,
                active_threads: emu.scheduler.active_thread_count(),
                summary: "Infinite loop detected: excessive instruction consumption".to_string(),
            });
        }

        // Condition B: All active threads are permanently blocked on monitors in a lock contention deadlock
        let active_threads = emu.scheduler.active_thread_count();
        if active_threads > 0 && !emu.context.is_app_paused {
            let has_blocked = emu.scheduler.threads.iter().filter(|t| t.is_alive()).any(|t| {
                matches!(t.state, ThreadState::Blocked { .. })
            });
            let all_stuck = emu.scheduler.threads.iter().filter(|t| t.is_alive()).all(|t| {
                matches!(
                    t.state,
                    ThreadState::Blocked { .. }
                        | ThreadState::Waiting {
                            timeout_wake_ms: None,
                            ..
                        }
                )
            });
            if has_blocked && all_stuck && emu.context.ui_queue.is_empty() {
                self.consecutive_deadlock_ticks += 1;
                if self.consecutive_deadlock_ticks >= 180 {
                    return Some(AnomalyReport {
                        tick,
                        kind: AnomalyKind::DeadlockOrInfiniteLoop {
                            details: format!("All {} active threads are deadlocked on monitors across {} ticks", active_threads, self.consecutive_deadlock_ticks),
                        },
                        total_instructions: current_instructions,
                        active_threads,
                        summary: "Deadlock detected: all threads blocked without timeout".to_string(),
                    });
                }
            } else {
                self.consecutive_deadlock_ticks = 0;
            }
        } else {
            self.consecutive_deadlock_ticks = 0;
        }

        // 3. Audio Anomaly Check (NaN / Inf / Prolonged Clipping)
        for &s in audio_chunk {
            self.total_audio_energy += (s * s) as f64;
            if !s.is_finite() {
                return Some(AnomalyReport {
                    tick,
                    kind: AnomalyKind::AudioAnomaly {
                        details: format!("Audio sample contains non-finite float value: {}", s),
                    },
                    total_instructions: current_instructions,
                    active_threads,
                    summary: "Audio anomaly: non-finite sample (NaN or Inf)".to_string(),
                });
            }
            if s.abs() >= 0.999 {
                self.audio_consecutive_clipped_samples += 1;
                if self.audio_consecutive_clipped_samples >= 1500 {
                    return Some(AnomalyReport {
                        tick,
                        kind: AnomalyKind::AudioAnomaly {
                            details: format!(
                                "Continuous audio hard clipping across {} samples",
                                self.audio_consecutive_clipped_samples
                            ),
                        },
                        total_instructions: current_instructions,
                        active_threads,
                        summary: "Audio anomaly: continuous hard clipping / distortion".to_string(),
                    });
                }
            } else {
                self.audio_consecutive_clipped_samples = 0;
            }
        }

        // 4. Black Screen Check (> 180 ticks of 100% black screen with active CPU)
        let non_black_pixels = emu
            .context
            .framebuffer
            .iter()
            .filter(|&&p| (p & 0x00FFFFFF) != 0)
            .count();

        if non_black_pixels > self.max_non_black_pixels {
            self.max_non_black_pixels = non_black_pixels;
        }

        // Scene Diversity Sampling (Sentinel 2.0)
        if tick % 30 == 0 && non_black_pixels > 500 {
            let sh = compute_scene_hash(
                &emu.context.framebuffer,
                emu.context.screen_width as usize,
                emu.context.screen_height as usize,
            );
            self.unique_scene_hashes.insert(sh);
        }

        // Swallowed Exceptions Loop Check (Sentinel 2.0)
        let crit_swallowed = emu.context.exception_telemetry.critical_exceptions_caught;
        self.critical_swallowed_exceptions = crit_swallowed;
        self.caught_exception_classes = emu.context.exception_telemetry.caught_exception_classes.clone();
        if crit_swallowed >= 50 {
            return Some(AnomalyReport {
                tick,
                kind: AnomalyKind::SilentExceptionLoop {
                    count: crit_swallowed,
                    classes: self.caught_exception_classes.clone(),
                },
                total_instructions: current_instructions,
                active_threads,
                summary: format!(
                    "Silent exception loop: {} critical exceptions swallowed in try-catch ({:?})",
                    crit_swallowed, self.caught_exception_classes
                ),
            });
        }

        if non_black_pixels == 0 {
            if instructions_delta > 0 || tick > 30 {
                self.consecutive_black_screen_ticks += 1;
            }
            if self.consecutive_black_screen_ticks > 180 {
                return Some(AnomalyReport {
                    tick,
                    kind: AnomalyKind::BlackScreen {
                        ticks_count: self.consecutive_black_screen_ticks,
                    },
                    total_instructions: current_instructions,
                    active_threads,
                    summary: format!(
                        "Black screen anomaly: 0% colored pixels for {} ticks",
                        self.consecutive_black_screen_ticks
                    ),
                });
            }
        } else {
            self.consecutive_black_screen_ticks = 0;
        }

        // 5. Visual Freeze Check (> 600 ticks with identical frame while input is arriving and CPU is idle)
        let is_actively_loading = instructions_delta > 50_000;
        let current_hash = compute_frame_hash(&emu.context.framebuffer);
        let current_luma = extract_luminance_grid(
            &emu.context.framebuffer,
            emu.context.screen_width as usize,
            emu.context.screen_height as usize,
        );

        // Causal Input Reaction Checking (Sentinel 2.0)
        if let Some((probe_tick, baseline_hash)) = self.pending_input_probe {
            if tick > probe_tick + 3 && tick <= probe_tick + 35 {
                if current_hash != baseline_hash && non_black_pixels > 500 {
                    self.causal_input_reactions += 1;
                    self.pending_input_probe = None;
                }
            } else if tick > probe_tick + 35 {
                self.pending_input_probe = None;
            }
        }

        if self.last_frame_hash != 0 {
            if current_hash == self.last_frame_hash {
                if !is_actively_loading {
                    self.consecutive_identical_frame_ticks += 1;
                }

                if tick > 600
                    && self.consecutive_identical_frame_ticks > 600
                    && self.input_delivered_since_freeze_check
                    && active_threads > 0
                {
                    let ssim = compute_ssim(&self.last_frame_luma, &current_luma);
                    return Some(AnomalyReport {
                        tick,
                        kind: AnomalyKind::VisualFreeze {
                            ssim,
                            frozen_ticks: self.consecutive_identical_frame_ticks,
                        },
                        total_instructions: current_instructions,
                        active_threads,
                        summary: format!(
                            "Visual freeze: SSIM {:.4} across {} ticks with active input",
                            ssim, self.consecutive_identical_frame_ticks
                        ),
                    });
                }
            } else {
                self.consecutive_identical_frame_ticks = 0;
                self.input_delivered_since_freeze_check = false;
            }
        }
        self.last_frame_hash = current_hash;
        self.last_frame_luma = current_luma;

        // 6. Heap Leak Check (Monotonic heap growth unaffected by GC)
        if tick % self.heap_check_interval == 0 {
            let live = emu.heap.live_objects();
            self.heap_history.push(live);
            if self.heap_history.len() > 6 {
                self.heap_history.remove(0);
                let strictly_growing = self.heap_history.windows(2).all(|w| w[1] > w[0]);
                let total_growth = self.heap_history.last().unwrap() - self.heap_history.first().unwrap();
                if strictly_growing && total_growth > 600 {
                    let freed = emu.force_gc();
                    if freed == 0 {
                        return Some(AnomalyReport {
                            tick,
                            kind: AnomalyKind::HeapLeak {
                                live_objects: live,
                                freed_by_gc: freed,
                            },
                            total_instructions: current_instructions,
                            active_threads,
                            summary: format!(
                                "Heap leak: live objects grew by {} slots, GC freed 0",
                                total_growth
                            ),
                        });
                    }
                }
            }
        }

        None
    }

    /// Evaluates final multi-dimensional Sentinel 2.0 verdict across all objective metrics.
    pub fn evaluate_verdict(
        &self,
        emu: &J2meEmulator,
        ticks_executed: usize,
        detected_anomaly: &Option<AnomalyReport>,
    ) -> (SentinelVerdict, Vec<String>) {
        let mut flags = Vec::new();

        if emu.context.audio_api_invoked && self.total_audio_energy < 1e-6 {
            flags.push("FLAG_SILENT_AUDIO".to_string());
        }

        if let Some(ref anom) = detected_anomaly {
            match anom.kind {
                AnomalyKind::SilentExceptionLoop { .. } => {
                    return (SentinelVerdict::SilentExceptionFailure, flags);
                }
                AnomalyKind::SplashLoopStuck { .. } => {
                    return (SentinelVerdict::SplashLoopSuspicious, flags);
                }
                _ => return (SentinelVerdict::HardAnomaly, flags),
            }
        }

        let crit_exc = self.critical_swallowed_exceptions;
        if crit_exc >= 50 {
            flags.push(format!("CRIT_EXC:{}", crit_exc));
            return (SentinelVerdict::SilentExceptionFailure, flags);
        }

        let unique_scenes = self.unique_scene_hashes.len();
        let reactions = self.causal_input_reactions;

        if ticks_executed >= 300 && self.max_non_black_pixels < 500 {
            flags.push("BLACK_SCREEN".to_string());
            return (SentinelVerdict::HardAnomaly, flags);
        }

        if ticks_executed >= 400 && unique_scenes < 3 && reactions == 0 {
            flags.push(format!("SCENES:{}_REACT:0", unique_scenes));
            return (SentinelVerdict::SplashLoopSuspicious, flags);
        }

        if unique_scenes >= 3 && reactions > 0 {
            (SentinelVerdict::FullGameplayPass, flags)
        } else if unique_scenes >= 2 || reactions > 0 {
            (SentinelVerdict::MenuReachedPass, flags)
        } else {
            flags.push(format!("SCENES:{}", unique_scenes));
            (SentinelVerdict::SplashLoopSuspicious, flags)
        }
    }
}

#[inline]
pub fn compute_scene_hash(fb: &[u32], w: usize, h: usize) -> u64 {
    let grid = extract_luminance_grid(fb, w, h);
    let mut h = 0xcbf29ce484222325u64;
    for &lum in &grid {
        let q = ((lum * 15.0).round() as u8).min(15);
        h ^= q as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[inline]
fn compute_frame_hash(fb: &[u32]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &p in fb {
        h ^= p as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Downsample framebuffer to a normalized luminance vector (0.0..1.0)
fn extract_luminance_grid(fb: &[u32], w: usize, h: usize) -> Vec<f32> {
    if fb.is_empty() || w == 0 || h == 0 {
        return Vec::new();
    }
    // Downsample to 32x32 luminance grid for stable, fast SSIM
    let grid_w = 32;
    let grid_h = 32;
    let mut grid = vec![0.0f32; grid_w * grid_h];
    let step_x = (w / grid_w).max(1);
    let step_y = (h / grid_h).max(1);

    for gy in 0..grid_h {
        let sy = (gy * step_y).min(h - 1);
        for gx in 0..grid_w {
            let sx = (gx * step_x).min(w - 1);
            let idx = sy * w + sx;
            let px = fb.get(idx).copied().unwrap_or(0);
            let r = ((px >> 16) & 0xFF) as f32 / 255.0;
            let g = ((px >> 8) & 0xFF) as f32 / 255.0;
            let b = (px & 0xFF) as f32 / 255.0;
            // ITU-R BT.601 luma formula
            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            grid[gy * grid_w + gx] = luma;
        }
    }
    grid
}

/// Compute Structural Similarity Index (SSIM) between two normalized luminance maps.
pub fn compute_ssim(x: &[f32], y: &[f32]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 1.0;
    }
    let n = x.len() as f64;

    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        sum_x += xi as f64;
        sum_y += yi as f64;
    }
    let mu_x = sum_x / n;
    let mu_y = sum_y / n;

    let mut var_x = 0.0f64;
    let mut var_y = 0.0f64;
    let mut covar_xy = 0.0f64;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi as f64 - mu_x;
        let dy = yi as f64 - mu_y;
        var_x += dx * dx;
        var_y += dy * dy;
        covar_xy += dx * dy;
    }
    var_x /= n - 1.0;
    var_y /= n - 1.0;
    covar_xy /= n - 1.0;

    let c1 = 0.0001f64; // (0.01 * 1.0)^2
    let c2 = 0.0009f64; // (0.03 * 1.0)^2

    let numerator = (2.0 * mu_x * mu_y + c1) * (2.0 * covar_xy + c2);
    let denominator = (mu_x * mu_x + mu_y * mu_y + c1) * (var_x + var_y + c2);

    if denominator == 0.0 {
        1.0
    } else {
        (numerator / denominator).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssim_identical_frames_is_one() {
        let frame1 = vec![0.5f32; 1024];
        let frame2 = vec![0.5f32; 1024];
        let ssim = compute_ssim(&frame1, &frame2);
        assert!((ssim - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ssim_different_frames_is_lower() {
        let frame1: Vec<f32> = (0..1024).map(|i| (i % 256) as f32 / 255.0).collect();
        let frame2: Vec<f32> = (0..1024).map(|i| ((i + 128) % 256) as f32 / 255.0).collect();
        let ssim = compute_ssim(&frame1, &frame2);
        assert!(ssim < 0.9);
    }
}
