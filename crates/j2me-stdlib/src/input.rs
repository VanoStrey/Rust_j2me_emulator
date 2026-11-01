//! Ordered key input queue with auto-repeat.
//!
//! Host key events arrive asynchronously, while the VM executes each handler as
//! a green thread. Dispatching one thread per event lets bursts interleave and
//! silently drops events once the thread cap is reached. This module keeps a
//! single FIFO queue instead: the emulator spawns at most one handler at a time
//! and pops the next event when that handler is gone, so press/release order is
//! preserved and nothing is lost between frames.

use std::collections::VecDeque;

/// Milliseconds a key stays held before the first `keyRepeated` event.
pub const REPEAT_DELAY_MS: u64 = 400;
/// Milliseconds between subsequent `keyRepeated` events.
pub const REPEAT_PERIOD_MS: u64 = 120;
/// Hard cap on queued events. Only reachable if the VM stops dispatching.
pub const MAX_QUEUE: usize = 256;
/// Upper bound of repeats generated for one key in a single `advance` call.
const MAX_REPEATS_PER_ADVANCE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Pressed,
    Released,
    Repeated,
}

impl KeyAction {
    /// MIDP method invoked on the active `Canvas` for this action.
    pub fn method_name(self) -> &'static str {
        match self {
            KeyAction::Pressed => "keyPressed",
            KeyAction::Released => "keyReleased",
            KeyAction::Repeated => "keyRepeated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: i32,
    pub action: KeyAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeldKey {
    key: i32,
    next_repeat_ms: u64,
}

/// Ordered key event queue plus the set of currently held keys.
#[derive(Debug, Default)]
pub struct KeyInput {
    queue: VecDeque<KeyEvent>,
    held: Vec<HeldKey>,
    dropped: u64,
    coalesced: u64,
}

impl KeyInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a key press. Returns `false` if the key was already held
    /// (duplicate presses are ignored so repeat timing stays stable).
    pub fn press(&mut self, key: i32, now_ms: u64) -> bool {
        if self.is_held(key) {
            return false;
        }
        self.held.push(HeldKey {
            key,
            next_repeat_ms: now_ms.saturating_add(REPEAT_DELAY_MS),
        });
        self.enqueue(KeyEvent {
            key,
            action: KeyAction::Pressed,
        });
        true
    }

    /// Records a key release and cancels its pending repeats.
    ///
    /// The release is queued even if the key was never seen pressed, so a
    /// canvas that missed the press never observes a permanently held key.
    /// Returns whether the key was actually held.
    pub fn release(&mut self, key: i32, _now_ms: u64) -> bool {
        let was_held = if let Some(idx) = self.held.iter().position(|h| h.key == key) {
            self.held.remove(idx);
            true
        } else {
            false
        };
        self.enqueue(KeyEvent {
            key,
            action: KeyAction::Released,
        });
        was_held
    }

    /// Emits `Repeated` events for keys held longer than the repeat delay,
    /// then once per [`REPEAT_PERIOD_MS`]. Repeated events are queued after all
    /// already pending events, so a press is never overtaken by a repeat.
    pub fn advance(&mut self, now_ms: u64) {
        let mut repeated = Vec::new();
        for held in &mut self.held {
            let mut guard = 0;
            while held.next_repeat_ms <= now_ms && guard < MAX_REPEATS_PER_ADVANCE {
                repeated.push(held.key);
                held.next_repeat_ms = held.next_repeat_ms.saturating_add(REPEAT_PERIOD_MS);
                guard += 1;
            }
            // If the emulator was paused past several periods, resynchronise
            // instead of emitting a catch-up burst on later frames.
            if held.next_repeat_ms <= now_ms {
                held.next_repeat_ms = now_ms.saturating_add(REPEAT_PERIOD_MS);
            }
        }
        for key in repeated {
            self.enqueue(KeyEvent {
                key,
                action: KeyAction::Repeated,
            });
        }
    }

    /// Pops the oldest event, preserving strict FIFO order.
    pub fn next_event(&mut self) -> Option<KeyEvent> {
        self.queue.pop_front()
    }

    /// Called when the active `Displayable` changes: pending events belong to
    /// the old canvas and are discarded, and every held key is queued as a
    /// release so the new canvas does not inherit a phantom hold.
    /// Returns the number of discarded events.
    pub fn canvas_changed(&mut self) -> usize {
        let flushed = self.queue.len();
        self.queue.clear();
        let held = std::mem::take(&mut self.held);
        for key in held {
            self.enqueue(KeyEvent {
                key: key.key,
                action: KeyAction::Released,
            });
        }
        flushed
    }

    pub fn is_held(&self, key: i32) -> bool {
        self.held.iter().any(|h| h.key == key)
    }

    pub fn held_count(&self) -> usize {
        self.held.len()
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Events discarded because the queue was full of press/release pairs.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Repeat events discarded to make room for a press/release pair.
    pub fn coalesced(&self) -> u64 {
        self.coalesced
    }

    /// Overflow policy: repeats are redundant, so they are discarded first;
    /// a press/release pair is only dropped if the queue holds nothing but
    /// pairs, and every such drop is counted.
    fn enqueue(&mut self, event: KeyEvent) {
        if self.queue.len() >= MAX_QUEUE {
            if let Some(pos) = self
                .queue
                .iter()
                .position(|e| e.action == KeyAction::Repeated)
            {
                self.queue.remove(pos);
                self.coalesced += 1;
            } else {
                self.dropped += 1;
                return;
            }
        }
        self.queue.push_back(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(input: &mut KeyInput) -> Vec<(i32, KeyAction)> {
        let mut out = Vec::new();
        while let Some(ev) = input.next_event() {
            out.push((ev.key, ev.action));
        }
        out
    }

    #[test]
    fn preserves_order_for_fast_alternating_keys() {
        let mut input = KeyInput::new();
        input.press(1, 0);
        input.release(1, 1);
        input.press(2, 2);
        input.release(2, 3);
        input.press(1, 4);

        assert_eq!(
            drain(&mut input),
            vec![
                (1, KeyAction::Pressed),
                (1, KeyAction::Released),
                (2, KeyAction::Pressed),
                (2, KeyAction::Released),
                (1, KeyAction::Pressed),
            ]
        );
        assert_eq!(input.dropped(), 0);
        assert_eq!(input.pending(), 0);
    }

    #[test]
    fn delivers_more_than_eight_events_without_loss() {
        let mut input = KeyInput::new();
        // 12 keys pressed and released = 24 events, three times the old
        // eight-handler cap.
        for key in 0..12 {
            input.press(key, 0);
            input.release(key, 1);
        }
        let events = drain(&mut input);
        assert_eq!(events.len(), 24);
        for (i, (key, action)) in events.iter().enumerate() {
            assert_eq!(*key, (i / 2) as i32);
            assert_eq!(
                *action,
                if i % 2 == 0 {
                    KeyAction::Pressed
                } else {
                    KeyAction::Released
                }
            );
        }
        assert_eq!(input.dropped(), 0);
    }

    #[test]
    fn repeat_starts_after_delay_and_keeps_period() {
        let mut input = KeyInput::new();
        input.press(5, 1_000);
        assert_eq!(drain(&mut input), vec![(5, KeyAction::Pressed)]);

        // One millisecond short of the delay: no repeat yet.
        input.advance(1_000 + REPEAT_DELAY_MS - 1);
        assert!(input.is_empty());

        input.advance(1_000 + REPEAT_DELAY_MS);
        assert_eq!(drain(&mut input), vec![(5, KeyAction::Repeated)]);

        // Half a period later still silent, full period gives another repeat.
        input.advance(1_000 + REPEAT_DELAY_MS + REPEAT_PERIOD_MS / 2);
        assert!(input.is_empty());
        input.advance(1_000 + REPEAT_DELAY_MS + REPEAT_PERIOD_MS);
        assert_eq!(drain(&mut input), vec![(5, KeyAction::Repeated)]);
    }

    #[test]
    fn release_cancels_pending_repeats() {
        let mut input = KeyInput::new();
        input.press(7, 0);
        input.release(7, 50);
        input.advance(10_000);

        assert_eq!(
            drain(&mut input),
            vec![(7, KeyAction::Pressed), (7, KeyAction::Released)]
        );
        assert!(!input.is_held(7));
    }

    #[test]
    fn release_without_press_still_reports_release() {
        let mut input = KeyInput::new();
        assert!(!input.release(9, 0));
        assert_eq!(drain(&mut input), vec![(9, KeyAction::Released)]);
    }

    #[test]
    fn multiple_held_keys_repeat_independently() {
        let mut input = KeyInput::new();
        input.press(1, 0);
        input.press(2, 100);
        drain(&mut input);

        input.advance(REPEAT_DELAY_MS); // key 1 due, key 2 not yet
        assert_eq!(drain(&mut input), vec![(1, KeyAction::Repeated)]);

        input.advance(100 + REPEAT_DELAY_MS); // key 2 due, key 1 period not elapsed
        assert_eq!(drain(&mut input), vec![(2, KeyAction::Repeated)]);
        assert_eq!(input.held_count(), 2);
    }

    #[test]
    fn duplicate_press_does_not_restart_repeat_or_duplicate_event() {
        let mut input = KeyInput::new();
        assert!(input.press(3, 0));
        assert!(!input.press(3, 10));
        assert_eq!(drain(&mut input), vec![(3, KeyAction::Pressed)]);
        input.advance(REPEAT_DELAY_MS);
        assert_eq!(drain(&mut input), vec![(3, KeyAction::Repeated)]);
    }

    #[test]
    fn canvas_change_flushes_queue_and_releases_held_keys() {
        let mut input = KeyInput::new();
        input.press(1, 0);
        input.press(2, 0);
        drain(&mut input);

        input.press(3, 5); // queued press, never dispatched
        let flushed = input.canvas_changed();
        assert_eq!(flushed, 1);
        assert_eq!(input.held_count(), 0);

        // All three held keys (1, 2 and 3) are released so the new canvas
        // never inherits a phantom hold; the queued press for 3 is gone.
        let events = drain(&mut input);
        assert_eq!(events.len(), 3);
        assert!(events.iter().all(|(_, a)| *a == KeyAction::Released));

        // No repeats survive the switch.
        input.advance(100_000);
        assert!(input.is_empty());
    }

    #[test]
    fn overflow_discards_repeats_before_press_release_pairs() {
        let mut input = KeyInput::new();
        input.press(1, 0);
        drain(&mut input);

        // Fill the queue with repeats.
        for i in 0..MAX_QUEUE {
            input.enqueue(KeyEvent {
                key: 1,
                action: KeyAction::Repeated,
            });
            let _ = i;
        }
        assert_eq!(input.pending(), MAX_QUEUE);

        // A press must still get through by evicting a repeat.
        input.press(2, 0);
        assert_eq!(input.pending(), MAX_QUEUE);
        assert_eq!(input.coalesced(), 1);
        assert_eq!(input.dropped(), 0);

        let events = drain(&mut input);
        assert_eq!(events.last().copied(), Some((2, KeyAction::Pressed)));
    }
}
