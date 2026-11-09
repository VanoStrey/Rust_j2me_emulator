//! Chaos Monkey Agent - Deterministic Smart Input Generator.
//!
//! Generates pseudo-random and patterned input sequences to provoke edge cases,
//! race conditions, input buffer overflows, and state corruption in J2ME games.

use j2me_stdlib::J2meEmulator;
use serde::{Deserialize, Serialize};

/// Fast, deterministic PRNG based on splitmix64 / xorshift64star.
/// Zero external dependencies, 100% reproducible across architectures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicPrng {
    pub state: u64,
}

impl DeterministicPrng {
    pub fn new(seed: u64) -> Self {
        // Ensure state is never 0
        let s = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        let mut prng = Self { state: s };
        // Warm up state
        for _ in 0..8 {
            prng.next_u64();
        }
        prng
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    #[inline]
    pub fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next_u64() as usize % (max - min + 1))
    }

    #[inline]
    pub fn next_i32_choice(&mut self, choices: &[i32]) -> i32 {
        if choices.is_empty() {
            return 0;
        }
        let idx = (self.next_u64() as usize) % choices.len();
        choices[idx]
    }
}

pub const ITU_T_KEYS: [i32; 12] = [
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, // '0'..'9'
    42, 35,                                 // '*', '#'
];

pub const SOFT_KEYS: [i32; 4] = [
    -6,  // MIDP Soft 1
    -7,  // MIDP Soft 2
    -21, // Vendor Soft 1 (Motorola / SE)
    -22, // Vendor Soft 2 (Motorola / SE)
];

pub const GAME_ACTIONS: [i32; 5] = [
    -1, // UP
    -2, // DOWN
    -3, // LEFT
    -4, // RIGHT
    -5, // FIRE
];

pub const CONFLICT_PAIRS: [(i32, i32); 4] = [
    (-1, -2),   // UP + DOWN
    (-3, -4),   // LEFT + RIGHT
    (-6, -7),   // SOFT1 + SOFT2
    (-21, -22), // VENDOR SOFT1 + SOFT2
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuzzPattern {
    Idle,
    SingleTap,
    RapidFire,
    HoldKey,
    Conflict,
    LifecycleShock,
}

#[derive(Debug, Clone)]
pub struct HeldKeyInfo {
    pub key: i32,
    pub release_tick: usize,
}

#[derive(Debug, Clone)]
pub struct LifecycleShockInfo {
    pub resume_tick: usize,
}

#[derive(Debug, Clone)]
pub struct ChaosAgent {
    pub prng: DeterministicPrng,
    pub active_holds: Vec<HeldKeyInfo>,
    pub active_shock: Option<LifecycleShockInfo>,
    pub last_pattern: FuzzPattern,
    pub total_events_sent: usize,
    pub patterns_triggered: usize,
    pub next_probe_tick: usize,
}

impl ChaosAgent {
    pub fn new(seed: u64) -> Self {
        Self {
            prng: DeterministicPrng::new(seed),
            active_holds: Vec::new(),
            active_shock: None,
            last_pattern: FuzzPattern::Idle,
            total_events_sent: 0,
            patterns_triggered: 0,
            next_probe_tick: 30,
        }
    }

    /// Step with Causal Input Probing correlated with AnomalySentinel 2.0.
    pub fn step_with_sentinel(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
        sentinel: &mut super::anomaly_sentinel::AnomalySentinel,
    ) {
        // 0. Periodic Causal Input Probe
        if tick >= self.next_probe_tick && self.active_shock.is_none() {
            self.next_probe_tick = tick + self.prng.next_range(35, 65);
            let probe_keys: &[i32] = if tick < 120 {
                &[-5, 53, -6, -21, -1, -2]
            } else {
                &[-5, 53, -6, -7, -21, -22, -1, -2]
            };
            let key = self.prng.next_i32_choice(probe_keys);
            let cur_hash = super::anomaly_sentinel::compute_scene_hash(
                &emu.context.framebuffer,
                emu.context.screen_width as usize,
                emu.context.screen_height as usize,
            );
            emu.key_event(key, true);
            recorder.record_key(tick, key, true);
            self.total_events_sent += 1;
            sentinel.note_input_probe(tick, cur_hash);
            self.active_holds.push(HeldKeyInfo {
                key,
                release_tick: tick + self.prng.next_range(3, 5),
            });
            self.patterns_triggered += 1;
            return;
        }

        let prev = self.total_events_sent;
        self.step(tick, emu, recorder);
        if self.total_events_sent > prev {
            sentinel.note_input_delivered();
        }
    }

    /// Primary step executed on every tick before or during VM execution.
    pub fn step(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        // 1. Check pending key releases for Hold patterns
        let mut still_held = Vec::new();
        for held in self.active_holds.drain(..) {
            if tick >= held.release_tick {
                emu.key_event(held.key, false);
                recorder.record_key(tick, held.key, false);
                self.total_events_sent += 1;
            } else {
                still_held.push(held);
            }
        }
        self.active_holds = still_held;

        // 2. Check pending Lifecycle Shock resume
        if let Some(shock) = self.active_shock.take() {
            if tick >= shock.resume_tick {
                emu.resume_app();
                recorder.record_resume(tick);
                self.patterns_triggered += 1;
            } else {
                self.active_shock = Some(shock);
                return; // App is paused, skip generating input
            }
        }

        // 3. Roll probability of injecting new chaos pattern this tick
        // 25% chance of input activity per tick
        let roll = self.prng.next_f64();
        if roll > 0.35 {
            return;
        }

        let pattern_roll = self.prng.next_f64();
        if pattern_roll < 0.30 {
            // Pattern A: Single Tap / Navigation
            self.execute_single_tap(tick, emu, recorder);
        } else if pattern_roll < 0.55 {
            // Pattern B: Rapid-fire jitter (10-20 presses/releases in single tick)
            self.execute_rapid_fire(tick, emu, recorder);
        } else if pattern_roll < 0.75 {
            // Pattern C: Hold button for 100-300 ticks
            self.execute_hold(tick, emu, recorder);
        } else if pattern_roll < 0.90 {
            // Pattern D: Conflict (simultaneous opposite keys)
            self.execute_conflict(tick, emu, recorder);
        } else if tick > 150 {
            // Pattern E: Lifecycle Shock (pauseApp -> wait 15-30 ticks -> startApp)
            self.execute_lifecycle_shock(tick, emu, recorder);
        }
    }

    fn pick_random_key(&mut self, tick: usize) -> i32 {
        let pool_choice = self.prng.next_range(0, 2);
        match pool_choice {
            0 => self.prng.next_i32_choice(&GAME_ACTIONS),
            1 => {
                // In early startup (< 120 ticks), avoid SOFT2 (-7, -22) which acts as Exit/Quit on MIDP menus
                if tick < 120 {
                    const STARTUP_SOFT_KEYS: [i32; 2] = [-6, -21];
                    self.prng.next_i32_choice(&STARTUP_SOFT_KEYS)
                } else {
                    self.prng.next_i32_choice(&SOFT_KEYS)
                }
            }
            _ => self.prng.next_i32_choice(&ITU_T_KEYS),
        }
    }

    fn execute_single_tap(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        let key = self.pick_random_key(tick);
        let duration = self.prng.next_range(3, 6);
        emu.key_event(key, true);
        recorder.record_key(tick, key, true);
        self.total_events_sent += 1;
        self.active_holds.push(HeldKeyInfo {
            key,
            release_tick: tick + duration,
        });
        self.patterns_triggered += 1;
        self.last_pattern = FuzzPattern::SingleTap;
    }

    fn execute_rapid_fire(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        let count = self.prng.next_range(10, 20);
        let key = self.pick_random_key(tick);
        for _ in 0..count {
            emu.key_event(key, true);
            recorder.record_key(tick, key, true);
            emu.key_event(key, false);
            recorder.record_key(tick, key, false);
            self.total_events_sent += 2;
        }
        self.patterns_triggered += 1;
        self.last_pattern = FuzzPattern::RapidFire;
    }

    fn execute_hold(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        let key = self.pick_random_key(tick);
        let duration = self.prng.next_range(100, 300);
        emu.key_event(key, true);
        recorder.record_key(tick, key, true);
        self.total_events_sent += 1;
        self.active_holds.push(HeldKeyInfo {
            key,
            release_tick: tick + duration,
        });
        self.patterns_triggered += 1;
        self.last_pattern = FuzzPattern::HoldKey;
    }

    fn execute_conflict(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        let max_pair = if tick < 120 { 1 } else { CONFLICT_PAIRS.len() - 1 };
        let pair_idx = self.prng.next_range(0, max_pair);
        let (k1, k2) = CONFLICT_PAIRS[pair_idx];
        let duration = self.prng.next_range(20, 80);

        emu.key_event(k1, true);
        recorder.record_key(tick, k1, true);
        emu.key_event(k2, true);
        recorder.record_key(tick, k2, true);
        self.total_events_sent += 2;

        self.active_holds.push(HeldKeyInfo {
            key: k1,
            release_tick: tick + duration,
        });
        self.active_holds.push(HeldKeyInfo {
            key: k2,
            release_tick: tick + duration,
        });
        self.patterns_triggered += 1;
        self.last_pattern = FuzzPattern::Conflict;
    }

    fn execute_lifecycle_shock(
        &mut self,
        tick: usize,
        emu: &mut J2meEmulator,
        recorder: &mut super::replay_recorder::ReplayRecorder,
    ) {
        if self.active_shock.is_some() {
            return;
        }
        let pause_duration = self.prng.next_range(15, 30);
        emu.pause_app();
        recorder.record_pause(tick);
        self.active_shock = Some(LifecycleShockInfo {
            resume_tick: tick + pause_duration,
        });
        self.patterns_triggered += 1;
        self.last_pattern = FuzzPattern::LifecycleShock;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_prng_reproducibility() {
        let mut prng1 = DeterministicPrng::new(12345);
        let mut prng2 = DeterministicPrng::new(12345);

        for _ in 0..100 {
            assert_eq!(prng1.next_u64(), prng2.next_u64());
            assert_eq!(prng1.next_range(10, 50), prng2.next_range(10, 50));
        }
    }

    #[test]
    fn test_conflict_pairs_are_valid_scan_codes() {
        for &(k1, k2) in &CONFLICT_PAIRS {
            assert_ne!(k1, k2);
        }
    }
}
