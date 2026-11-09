use super::green_thread::{GreenThread, ThreadId, ThreadState};
use crate::heap::object::ObjectId;
use crate::vm::frame::Frame;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorState {
    pub owner: Option<ThreadId>,
    pub recursion: usize,
    /// Threads waiting for Object.notify() / Object.notifyAll() or timed timeout: (thread_id, saved_recursion)
    pub wait_set: Vec<(ThreadId, usize)>,
    /// Threads contending to acquire or re-acquire the monitor: (thread_id, pending_recursion)
    pub entry_set: Vec<(ThreadId, usize)>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            owner: None,
            recursion: 0,
            wait_set: Vec::new(),
            entry_set: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    pub threads: Vec<GreenThread>,
    pub current_index: usize,
    pub quantum: usize,
    pub current_time_ms: u64,
    pub sub_ms_remainder: f64,
    /// Exact rational accumulator numerator for 60 Hz integer timing (50 ms / 3 ticks)
    pub rational_remainder_numer: u64,
    pub total_completed_instructions: u64,
    pub total_completed_exceptions_thrown: u64,
    pub total_completed_exceptions_caught: u64,
    pub total_completed_critical_exceptions_caught: u64,
    pub completed_caught_exception_classes: std::collections::HashSet<String>,
    pub total_completed_exceptions_uncaught: u64,
    pub total_completed_auto_healings: u32,
    pub last_auto_healing_signature: Option<String>,
    pub monitors: HashMap<ObjectId, MonitorState>,
    pub strict_mode: bool,
    next_id: u32,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            threads: Vec::new(),
            current_index: 0,
            quantum: 5000, // 5,000 bytecode instructions per slice
            current_time_ms: 0,
            sub_ms_remainder: 0.0,
            rational_remainder_numer: 0,
            total_completed_instructions: 0,
            total_completed_exceptions_thrown: 0,
            total_completed_exceptions_caught: 0,
            total_completed_critical_exceptions_caught: 0,
            completed_caught_exception_classes: std::collections::HashSet::new(),
            total_completed_exceptions_uncaught: 0,
            total_completed_auto_healings: 0,
            last_auto_healing_signature: None,
            monitors: HashMap::new(),
            strict_mode: false,
            next_id: 1,
        }
    }
}

impl Scheduler {
    pub fn new(quantum: usize) -> Self {
        Self {
            quantum,
            ..Default::default()
        }
    }

    pub fn allocate_thread_id(&mut self) -> ThreadId {
        let id = ThreadId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn spawn(
        &mut self,
        name: &str,
        initial_frame: Frame,
        java_obj: Option<ObjectId>,
    ) -> ThreadId {
        if std::env::var_os("J2ME_DEBUG_THREADS").is_some() {
            eprintln!("[THREAD] spawn '{}' t={}ms", name, self.current_time_ms);
        }
        let id = self.allocate_thread_id();
        let sync_monitor = initial_frame.sync_monitor;
        let mut thread = GreenThread::new(id, name.to_string(), initial_frame, java_obj);
        thread.strict_mode = self.strict_mode;
        self.threads.push(thread);
        if let Some(m) = sync_monitor {
            if !self.try_acquire_monitor(id, m) {
                self.block_on_monitor(id, m, 1);
            }
        }
        id
    }

    /// Advance virtual time using an exact integer rational model for standard 60 Hz mode (50 ms / 3 ticks).
    /// Returns the exact whole milliseconds advanced in this tick.
    pub fn tick_time_fixed_60hz(&mut self) -> u64 {
        // 1000 ms per 60 frames = 50 / 3 ms per tick
        self.rational_remainder_numer += 50;
        let whole_ms = self.rational_remainder_numer / 3;
        self.rational_remainder_numer %= 3;
        self.advance_virtual_time(whole_ms);
        whole_ms
    }

    /// Advance virtual time from host delta (in milliseconds).
    /// Safely handles NaN, infinite, negative, and excessive deltas with sub-millisecond accumulation.
    pub fn tick_time_rational(&mut self, delta_ms: f64) -> u64 {
        if delta_ms.is_nan() || delta_ms.is_infinite() || delta_ms <= 0.0 {
            return 0;
        }
        // Clamp excessive jumps (> 1000 ms) to avoid runaway scheduling
        let clamped = delta_ms.min(1000.0);
        self.sub_ms_remainder += clamped;
        let whole_ms = self.sub_ms_remainder.floor() as u64;
        if whole_ms > 0 {
            self.sub_ms_remainder -= whole_ms as f64;
            self.advance_virtual_time(whole_ms);
        }
        whole_ms
    }

    /// Legacy integer tick_time interface for backwards compatibility
    pub fn tick_time(&mut self, delta_ms: u64) {
        self.advance_virtual_time(delta_ms);
    }

    fn advance_virtual_time(&mut self, whole_ms: u64) {
        if whole_ms == 0 {
            return;
        }
        self.current_time_ms += whole_ms;

        // 1. Wake sleeping threads whose deadline has elapsed
        for t in &mut self.threads {
            if let ThreadState::Sleeping { wake_time_ms } = t.state {
                if self.current_time_ms >= wake_time_ms {
                    t.state = ThreadState::Ready;
                }
            }
        }

        // 2. Collect timed-waiting threads whose timeout deadline has elapsed OR exceeded 200k instructions
        let mut timed_out = Vec::new();
        let total_inst = self.total_instructions();
        for t in &self.threads {
            if let ThreadState::Waiting {
                monitor: Some(mon),
                timeout_wake_ms: Some(wake_ms),
                saved_recursion,
            } = t.state
            {
                let is_time_elapsed = self.current_time_ms >= wake_ms;
                let is_inst_elapsed = total_inst.saturating_sub(t.wait_start_instructions) > 200_000;
                if is_time_elapsed || is_inst_elapsed {
                    timed_out.push((t.id, mon, saved_recursion));
                }
            }
        }

        // 3. For each timed-out thread, remove from wait_set and attempt monitor re-acquisition
        for (tid, mon_id, saved_rec) in timed_out {
            if let Some(monitor) = self.monitors.get_mut(&mon_id) {
                monitor.wait_set.retain(|(t, _)| *t != tid);
                if monitor.owner.is_none() {
                    // Monitor is currently unowned: acquire immediately!
                    monitor.owner = Some(tid);
                    monitor.recursion = saved_rec;
                    if let Some(t) = self.threads.iter_mut().find(|t| t.id == tid) {
                        t.state = ThreadState::Ready;
                    }
                } else {
                    // Monitor is occupied: move to entry_set as Blocked!
                    if !monitor.entry_set.iter().any(|(t, _)| *t == tid) {
                        monitor.entry_set.push((tid, saved_rec));
                    }
                    if let Some(t) = self.threads.iter_mut().find(|t| t.id == tid) {
                        t.state = ThreadState::Blocked {
                            monitor: mon_id,
                            pending_recursion: saved_rec,
                        };
                    }
                }
            }
        }
    }

    /// Deadlock Breaker:
    /// If no threads are in Ready state, but threads exist in Waiting, Blocked, or Sleeping:
    /// 1. If any thread is Sleeping, fast-forward virtual time to the earliest wake_time_ms.
    /// 2. If all threads are Waiting (e.g. Object.wait without notify) or Blocked (deadlock cycle):
    ///    Spuriously wake the waiting thread back to Ready (Java JLS §17.8.1).
    pub fn break_deadlocks(&mut self) -> bool {
        if self.threads.is_empty() || self.threads.iter().any(|t| t.state == ThreadState::Ready) {
            return false;
        }

        // If any thread is Sleeping, it is voluntarily waiting for virtual time to advance across frames; not a deadlock
        if self.threads.iter().any(|t| matches!(t.state, ThreadState::Sleeping { .. })) {
            return false;
        }

        // 1. Spuriously wake a waiting thread (Java JLS §17.8.1 spurious wakeup contract)
        for t in &mut self.threads {
            if let ThreadState::Waiting {
                monitor,
                saved_recursion,
                ..
            } = t.state
            {
                let tid = t.id;
                if let Some(mon_id) = monitor {
                    if let Some(mon) = self.monitors.get_mut(&mon_id) {
                        mon.wait_set.retain(|(id, _)| *id != tid);
                        if mon.owner.is_none() {
                            mon.owner = Some(tid);
                            mon.recursion = saved_recursion;
                            t.state = ThreadState::Ready;
                            return true;
                        } else {
                            if !mon.entry_set.iter().any(|(id, _)| *id != tid) {
                                mon.entry_set.push((tid, saved_recursion));
                            }
                            t.state = ThreadState::Blocked {
                                monitor: mon_id,
                                pending_recursion: saved_recursion,
                            };
                        }
                    } else {
                        t.state = ThreadState::Ready;
                        return true;
                    }
                } else {
                    t.state = ThreadState::Ready;
                    return true;
                }
            }
        }

        // 3. If all threads are Blocked (deadlock monitor cycle), grant monitor to first blocked thread
        for t in &mut self.threads {
            if let ThreadState::Blocked {
                monitor,
                pending_recursion,
            } = t.state
            {
                let tid = t.id;
                if let Some(mon) = self.monitors.get_mut(&monitor) {
                    mon.entry_set.retain(|(id, _)| *id != tid);
                    mon.owner = Some(tid);
                    mon.recursion = pending_recursion;
                    t.state = ThreadState::Ready;
                    return true;
                }
            }
        }

        false
    }

    pub fn next_ready_thread_index(&mut self) -> Option<usize> {
        if self.threads.is_empty() {
            return None;
        }

        let total = self.threads.len();
        let mut any_ready = false;
        let mut min_yields = usize::MAX;

        for t in &self.threads {
            if t.state == ThreadState::Ready {
                any_ready = true;
                if t.yield_count_in_frame < min_yields {
                    min_yields = t.yield_count_in_frame;
                }
            }
        }

        if !any_ready || min_yields >= 2 {
            return None;
        }

        for i in 0..total {
            let idx = (self.current_index + i) % total;
            if self.threads[idx].state == ThreadState::Ready
                && self.threads[idx].yield_count_in_frame == min_yields
            {
                self.current_index = (idx + 1) % total;
                return Some(idx);
            }
        }
        None
    }

    pub fn current_thread_mut(&mut self, index: usize) -> Option<&mut GreenThread> {
        self.threads.get_mut(index)
    }

    pub fn is_monitor_owner(&self, thread_id: ThreadId, obj_id: ObjectId) -> bool {
        self.monitors
            .get(&obj_id)
            .and_then(|m| m.owner)
            .map(|owner| owner == thread_id)
            .unwrap_or(false)
    }

    pub fn get_monitor_owner(&self, obj_id: ObjectId) -> Option<ThreadId> {
        self.monitors.get(&obj_id).and_then(|m| m.owner)
    }

    /// Attempt to acquire a monitor for thread_id with recursion = 1.
    /// If monitor is unowned, acquires with recursion = 1.
    /// If already owned by thread_id, increments recursion.
    /// If owned by another thread, returns false without modifying ownership.
    pub fn try_acquire_monitor(&mut self, thread_id: ThreadId, obj_id: ObjectId) -> bool {
        self.try_acquire_monitor_with_recursion(thread_id, obj_id, 1)
    }

    pub fn try_acquire_monitor_with_recursion(
        &mut self,
        thread_id: ThreadId,
        obj_id: ObjectId,
        recursion: usize,
    ) -> bool {
        let monitor = self.monitors.entry(obj_id).or_default();
        match monitor.owner {
            None => {
                monitor.owner = Some(thread_id);
                monitor.recursion = recursion.max(1);
                true
            }
            Some(owner) if owner == thread_id => {
                monitor.recursion += recursion.max(1);
                true
            }
            Some(_) => false,
        }
    }

    /// Enqueue thread into entry_set and set state to Blocked.
    pub fn block_on_monitor(&mut self, thread_id: ThreadId, obj_id: ObjectId, recursion: usize) {
        let monitor = self.monitors.entry(obj_id).or_default();
        if !monitor.entry_set.iter().any(|(t, _)| *t == thread_id) {
            monitor.entry_set.push((thread_id, recursion.max(1)));
        }
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) {
            t.state = ThreadState::Blocked {
                monitor: obj_id,
                pending_recursion: recursion.max(1),
            };
        }
    }

    /// Release monitor held by thread_id.
    /// Decrements recursion; if 0, clears ownership and grants monitor to the first contender in entry_set.
    /// Returns true on success, false if thread does not own monitor.
    pub fn release_monitor(&mut self, thread_id: ThreadId, obj_id: ObjectId) -> bool {
        let monitor = match self.monitors.get_mut(&obj_id) {
            Some(m) => m,
            None => return false,
        };
        if monitor.owner != Some(thread_id) {
            return false;
        }

        monitor.recursion -= 1;
        if monitor.recursion == 0 {
            monitor.owner = None;
            // If contenders are waiting in entry_set, transfer ownership to the first contender
            if !monitor.entry_set.is_empty() {
                let (next_tid, pending_rec) = monitor.entry_set.remove(0);
                monitor.owner = Some(next_tid);
                monitor.recursion = pending_rec;
                if let Some(t) = self.threads.iter_mut().find(|t| t.id == next_tid) {
                    t.state = ThreadState::Ready;
                }
            } else if monitor.wait_set.is_empty() {
                // Both wait_set and entry_set are empty, remove monitor entry to prevent memory leaks
                self.monitors.remove(&obj_id);
            }
        }
        true
    }

    /// Object.wait(): atomically releases monitor and enqueues thread to wait_set.
    /// Returns Err(()) if thread does not own the monitor (triggers IllegalMonitorStateException).
    pub fn wait_monitor(
        &mut self,
        thread_id: ThreadId,
        obj_id: ObjectId,
        timeout_wake_ms: Option<u64>,
    ) -> Result<(), ()> {
        let total_inst = self.total_instructions();
        let monitor = match self.monitors.get_mut(&obj_id) {
            Some(m) => m,
            None => return Err(()),
        };
        if monitor.owner != Some(thread_id) {
            return Err(());
        }

        let saved_recursion = monitor.recursion;
        monitor.owner = None;
        monitor.recursion = 0;

        // Wake next contender in entry_set if one is waiting
        if !monitor.entry_set.is_empty() {
            let (next_tid, pending_rec) = monitor.entry_set.remove(0);
            monitor.owner = Some(next_tid);
            monitor.recursion = pending_rec;
            if let Some(t) = self.threads.iter_mut().find(|t| t.id == next_tid) {
                t.state = ThreadState::Ready;
            }
        }

        // Atomically place into wait_set
        monitor.wait_set.push((thread_id, saved_recursion));
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) {
            t.wait_start_instructions = total_inst;
            t.state = ThreadState::Waiting {
                monitor: Some(obj_id),
                timeout_wake_ms,
                saved_recursion,
            };
        }

        Ok(())
    }

    /// Object.notify(): moves one waiting thread from wait_set to entry_set.
    /// Does NOT grant the monitor immediately, since caller still owns it.
    /// Returns Err(()) if thread does not own monitor (triggers IllegalMonitorStateException).
    pub fn notify_monitor(&mut self, thread_id: ThreadId, obj_id: ObjectId) -> Result<bool, ()> {
        let monitor = match self.monitors.get_mut(&obj_id) {
            Some(m) => m,
            None => return Err(()),
        };
        if monitor.owner != Some(thread_id) {
            return Err(());
        }

        if !monitor.wait_set.is_empty() {
            let (w_tid, saved_rec) = monitor.wait_set.remove(0);
            // Move to entry_set: thread is Blocked waiting to re-acquire the monitor!
            monitor.entry_set.push((w_tid, saved_rec));
            if let Some(t) = self.threads.iter_mut().find(|t| t.id == w_tid) {
                t.state = ThreadState::Blocked {
                    monitor: obj_id,
                    pending_recursion: saved_rec,
                };
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Object.notifyAll(): moves ALL waiting threads from wait_set to entry_set.
    /// Returns Err(()) if thread does not own monitor (triggers IllegalMonitorStateException).
    pub fn notify_all_monitor(
        &mut self,
        thread_id: ThreadId,
        obj_id: ObjectId,
    ) -> Result<usize, ()> {
        let monitor = match self.monitors.get_mut(&obj_id) {
            Some(m) => m,
            None => return Err(()),
        };
        if monitor.owner != Some(thread_id) {
            return Err(());
        }

        let count = monitor.wait_set.len();
        while !monitor.wait_set.is_empty() {
            let (w_tid, saved_rec) = monitor.wait_set.remove(0);
            monitor.entry_set.push((w_tid, saved_rec));
            if let Some(t) = self.threads.iter_mut().find(|t| t.id == w_tid) {
                t.state = ThreadState::Blocked {
                    monitor: obj_id,
                    pending_recursion: saved_rec,
                };
            }
        }
        Ok(count)
    }

    /// Wakes any threads waiting on the target object (e.g. following Thread.join or thread termination).
    pub fn wake_threads_waiting_for(&mut self, target_obj: ObjectId) {
        if let Some(monitor) = self.monitors.get_mut(&target_obj) {
            while !monitor.wait_set.is_empty() {
                let (w_tid, saved_rec) = monitor.wait_set.remove(0);
                if monitor.owner.is_none() {
                    monitor.owner = Some(w_tid);
                    monitor.recursion = saved_rec;
                    if let Some(t) = self.threads.iter_mut().find(|t| t.id == w_tid) {
                        t.state = ThreadState::Ready;
                    }
                } else {
                    monitor.entry_set.push((w_tid, saved_rec));
                    if let Some(t) = self.threads.iter_mut().find(|t| t.id == w_tid) {
                        t.state = ThreadState::Blocked {
                            monitor: target_obj,
                            pending_recursion: saved_rec,
                        };
                    }
                }
            }
        }
    }

    pub fn clean_terminated(&mut self) {
        if std::env::var_os("J2ME_DEBUG_THREADS").is_some() {
            for t in self
                .threads
                .iter()
                .filter(|t| matches!(t.state, ThreadState::Terminated))
            {
                eprintln!("[THREAD] dead '{}' t={}ms", t.name, self.current_time_ms);
            }
        }
        for t in self.threads.iter().filter(|t| !t.is_alive()) {
            self.total_completed_instructions += t.instructions_executed;
            self.total_completed_exceptions_thrown += t.exceptions_thrown;
            self.total_completed_exceptions_caught += t.exceptions_caught;
            self.total_completed_critical_exceptions_caught += t.critical_exceptions_caught;
            for c in &t.caught_exception_classes {
                self.completed_caught_exception_classes.insert(c.clone());
            }
            self.total_completed_exceptions_uncaught += t.exceptions_uncaught;
            self.total_completed_auto_healings += t.auto_healing_count;
            if t.last_auto_healing_signature.is_some() {
                self.last_auto_healing_signature = t.last_auto_healing_signature.clone();
            }
        }

        let live_ids: std::collections::HashSet<ThreadId> = self
            .threads
            .iter()
            .filter(|t| t.is_alive())
            .map(|t| t.id)
            .collect();

        // Clean up terminated threads from monitors to avoid leaks or hung locks
        for monitor in self.monitors.values_mut() {
            monitor.wait_set.retain(|(tid, _)| live_ids.contains(tid));
            monitor.entry_set.retain(|(tid, _)| live_ids.contains(tid));
            if let Some(owner) = monitor.owner {
                if !live_ids.contains(&owner) {
                    monitor.owner = None;
                    monitor.recursion = 0;
                    if !monitor.entry_set.is_empty() {
                        let (next_tid, rec) = monitor.entry_set.remove(0);
                        monitor.owner = Some(next_tid);
                        monitor.recursion = rec;
                        if let Some(t) = self.threads.iter_mut().find(|t| t.id == next_tid) {
                            t.state = ThreadState::Ready;
                        }
                    }
                }
            }
        }

        self.threads.retain(|t| t.is_alive());
        if self.current_index >= self.threads.len() && !self.threads.is_empty() {
            self.current_index = 0;
        }
    }

    pub fn total_instructions(&self) -> u64 {
        self.total_completed_instructions
            + self
                .threads
                .iter()
                .map(|t| t.instructions_executed)
                .sum::<u64>()
    }

    pub fn total_exceptions_thrown(&self) -> u64 {
        self.total_completed_exceptions_thrown
            + self
                .threads
                .iter()
                .map(|t| t.exceptions_thrown)
                .sum::<u64>()
    }

    pub fn total_exceptions_caught(&self) -> u64 {
        self.total_completed_exceptions_caught
            + self
                .threads
                .iter()
                .map(|t| t.exceptions_caught)
                .sum::<u64>()
    }

    pub fn total_critical_exceptions_caught(&self) -> u64 {
        self.total_completed_critical_exceptions_caught
            + self
                .threads
                .iter()
                .map(|t| t.critical_exceptions_caught)
                .sum::<u64>()
    }

    pub fn unique_caught_exception_classes(&self) -> Vec<String> {
        let mut set = self.completed_caught_exception_classes.clone();
        for t in &self.threads {
            for c in &t.caught_exception_classes {
                set.insert(c.clone());
            }
        }
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }

    pub fn total_exceptions_uncaught(&self) -> u64 {
        self.total_completed_exceptions_uncaught
            + self
                .threads
                .iter()
                .map(|t| t.exceptions_uncaught)
                .sum::<u64>()
    }

    pub fn total_auto_healings(&self) -> u32 {
        self.total_completed_auto_healings
            + self
                .threads
                .iter()
                .map(|t| t.auto_healing_count)
                .sum::<u32>()
    }

    pub fn last_auto_healing_signature(&self) -> Option<String> {
        for t in self.threads.iter().rev() {
            if let Some(ref sig) = t.last_auto_healing_signature {
                return Some(sig.clone());
            }
        }
        self.last_auto_healing_signature.clone()
    }

    pub fn active_thread_count(&self) -> usize {
        self.threads.iter().filter(|t| t.is_alive()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::instruction::Instruction;
    use std::sync::Arc;

    fn make_dummy_frame() -> Frame {
        Frame::new(
            "Test".to_string(),
            "run".to_string(),
            "()V".to_string(),
            Arc::new(vec![Instruction::Nop, Instruction::Return]),
            1,
            1,
            Vec::new(),
            Arc::new(HashMap::new()),
        )
    }

    #[test]
    fn test_rational_clock_60_and_3600_and_216000_ticks_drift_free() {
        let mut scheduler = Scheduler::default();
        let frame_delta = 1000.0 / 60.0; // 16.666666666666668 ms

        // After 60 ticks (1 second), exactly 1000 ms must have passed
        for _ in 0..60 {
            scheduler.tick_time_rational(frame_delta);
        }
        assert_eq!(scheduler.current_time_ms, 1000);

        // After 3600 ticks (60 seconds = 1 minute), exactly 60,000 ms must have passed
        for _ in 60..3600 {
            scheduler.tick_time_rational(frame_delta);
        }
        assert_eq!(scheduler.current_time_ms, 60_000);

        // After 216,000 ticks (1 hour of continuous simulation), exactly 3,600,000 ms must have passed
        for _ in 3600..216_000 {
            scheduler.tick_time_rational(frame_delta);
        }
        assert_eq!(scheduler.current_time_ms, 3_600_000);
    }

    #[test]
    fn test_rational_clock_handles_nan_inf_negative_and_excessive_jumps() {
        let mut scheduler = Scheduler::default();

        assert_eq!(scheduler.tick_time_rational(f64::NAN), 0);
        assert_eq!(scheduler.tick_time_rational(f64::INFINITY), 0);
        assert_eq!(scheduler.tick_time_rational(f64::NEG_INFINITY), 0);
        assert_eq!(scheduler.tick_time_rational(-16.666), 0);
        assert_eq!(scheduler.current_time_ms, 0);

        // Large jump (>1000 ms) is clamped to 1000 ms
        let adv = scheduler.tick_time_rational(50000.0);
        assert_eq!(adv, 1000);
        assert_eq!(scheduler.current_time_ms, 1000);
    }

    #[test]
    fn test_sleep_deadlines_boundary_16_and_17() {
        let mut scheduler = Scheduler::default();
        let frame_delta = 1000.0 / 60.0;

        scheduler.spawn("t1", make_dummy_frame(), None);
        scheduler.spawn("t2", make_dummy_frame(), None);

        scheduler.threads[0].state = ThreadState::Sleeping { wake_time_ms: 16 };
        scheduler.threads[1].state = ThreadState::Sleeping { wake_time_ms: 17 };

        let advanced = scheduler.tick_time_rational(frame_delta);
        assert_eq!(advanced, 16);
        assert_eq!(scheduler.current_time_ms, 16);
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
        assert!(matches!(
            scheduler.threads[1].state,
            ThreadState::Sleeping { wake_time_ms: 17 }
        ));

        let advanced2 = scheduler.tick_time_rational(frame_delta);
        assert_eq!(advanced2, 17);
        assert_eq!(scheduler.current_time_ms, 33);
        assert_eq!(scheduler.threads[1].state, ThreadState::Ready);
    }

    #[test]
    fn test_identical_deadlines_awaken_together() {
        let mut scheduler = Scheduler::default();
        scheduler.spawn("t1", make_dummy_frame(), None);
        scheduler.spawn("t2", make_dummy_frame(), None);

        scheduler.threads[0].state = ThreadState::Sleeping { wake_time_ms: 50 };
        scheduler.threads[1].state = ThreadState::Sleeping { wake_time_ms: 50 };

        scheduler.tick_time_rational(30.0);
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { .. }
        ));
        assert!(matches!(
            scheduler.threads[1].state,
            ThreadState::Sleeping { .. }
        ));

        scheduler.tick_time_rational(30.0);
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
        assert_eq!(scheduler.threads[1].state, ThreadState::Ready);
    }

    #[test]
    fn test_monitor_ownership_and_illegal_monitor_state() {
        let mut scheduler = Scheduler::default();
        let t1 = scheduler.spawn("t1", make_dummy_frame(), None);
        let t2 = scheduler.spawn("t2", make_dummy_frame(), None);
        let lock_obj = ObjectId(100);

        // Calling wait, notify, or release without holding the monitor returns Err(()) / false
        assert_eq!(scheduler.wait_monitor(t1, lock_obj, None), Err(()));
        assert_eq!(scheduler.notify_monitor(t1, lock_obj), Err(()));
        assert_eq!(scheduler.notify_all_monitor(t1, lock_obj), Err(()));
        assert!(!scheduler.release_monitor(t1, lock_obj));

        // t1 acquires monitor
        assert!(scheduler.try_acquire_monitor(t1, lock_obj));
        assert!(scheduler.is_monitor_owner(t1, lock_obj));

        // t2 cannot wait or notify on t1's monitor
        assert_eq!(scheduler.wait_monitor(t2, lock_obj, None), Err(()));
        assert_eq!(scheduler.notify_monitor(t2, lock_obj), Err(()));
        assert!(!scheduler.release_monitor(t2, lock_obj));
    }

    #[test]
    fn test_monitor_wait_and_notify_moves_to_entry_set_without_premature_execution() {
        let mut scheduler = Scheduler::default();
        let t1 = scheduler.spawn("t1", make_dummy_frame(), None);
        let t2 = scheduler.spawn("t2", make_dummy_frame(), None);
        let lock_obj = ObjectId(100);

        // 1. t2 acquires monitor with recursion 2
        assert!(scheduler.try_acquire_monitor(t2, lock_obj));
        assert!(scheduler.try_acquire_monitor(t2, lock_obj));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().recursion, 2);

        // 2. t2 calls wait(): monitor is released, t2 enters wait_set with saved_recursion = 2
        assert_eq!(scheduler.wait_monitor(t2, lock_obj, None), Ok(()));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, None);
        assert_eq!(
            scheduler.threads[1].state,
            ThreadState::Waiting {
                monitor: Some(lock_obj),
                timeout_wake_ms: None,
                saved_recursion: 2,
            }
        );

        // 3. t1 acquires the released monitor
        assert!(scheduler.try_acquire_monitor(t1, lock_obj));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, Some(t1));

        // 4. t1 calls notify(): t2 is moved from wait_set to entry_set as Blocked!
        // CRITICAL CONTRACT: t2 is NOT Ready! It cannot execute bytecode until t1 releases monitor.
        assert_eq!(scheduler.notify_monitor(t1, lock_obj), Ok(true));
        assert_eq!(
            scheduler.threads[1].state,
            ThreadState::Blocked {
                monitor: lock_obj,
                pending_recursion: 2,
            }
        );
        assert_eq!(scheduler.next_ready_thread_index(), Some(0)); // Only t1 is ready!

        // 5. t1 releases monitor -> ownership transfers to t2, restoring its saved recursion of 2!
        assert!(scheduler.release_monitor(t1, lock_obj));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, Some(t2));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().recursion, 2);
        assert_eq!(scheduler.threads[1].state, ThreadState::Ready);
    }

    #[test]
    fn test_sleep_does_not_release_monitor_and_is_not_awakened_by_notify() {
        let mut scheduler = Scheduler::default();
        let t1 = scheduler.spawn("t1", make_dummy_frame(), None);
        let t2 = scheduler.spawn("t2", make_dummy_frame(), None);
        let lock_obj = ObjectId(100);

        // t1 acquires lock and goes to sleep for 100 ms
        assert!(scheduler.try_acquire_monitor(t1, lock_obj));
        scheduler.threads[0].state = ThreadState::Sleeping { wake_time_ms: 100 };

        // Monitor remains owned by t1 while sleeping!
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, Some(t1));

        // t2 cannot acquire lock while t1 sleeps
        assert!(!scheduler.try_acquire_monitor(t2, lock_obj));

        // Notify on monitor does not wake t1 from sleep
        let _ = scheduler.notify_monitor(t1, lock_obj);
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { .. }
        ));
    }
}
