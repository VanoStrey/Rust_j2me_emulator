use crate::input::KeyAction;
use j2me_core::heap::object::ObjectId;
use std::collections::{HashMap, VecDeque};

pub const UI_DISPATCHER_THREAD: &str = "ui_dispatcher";
pub const MAX_CALL_SERIALLY: usize = 2048;
pub const MAX_KEY_QUEUE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    CallSerially(ObjectId),
    Paint(ObjectId),
    Key {
        key: i32,
        action: KeyAction,
        displayable: ObjectId,
    },
    ShowNotify(ObjectId),
    HideNotify(ObjectId),
}

#[derive(Debug, Default)]
pub struct UiEventQueue {
    events: VecDeque<UiEvent>,
    paint_pending: Option<ObjectId>,
    // Statistics & Diagnostics
    pub posted: u64,
    pub delivered: u64,
    pub coalesced: u64,
    pub dropped: u64,
    pub coalesced_repaints: u64,
    pub overflow_by_type: HashMap<&'static str, u64>,
}

impl UiEventQueue {
    pub fn new() -> Self {
        let mut overflow_by_type = HashMap::new();
        overflow_by_type.insert("call_serially", 0);
        overflow_by_type.insert("key", 0);
        overflow_by_type.insert("paint", 0);
        overflow_by_type.insert("lifecycle", 0);

        Self {
            events: VecDeque::with_capacity(64),
            paint_pending: None,
            posted: 0,
            delivered: 0,
            coalesced: 0,
            dropped: 0,
            coalesced_repaints: 0,
            overflow_by_type,
        }
    }

    /// Posts a Runnable callback to the serialized UI dispatcher queue.
    /// Strictly guarantees FIFO order without silent drops.
    pub fn post_call_serially(&mut self, runnable_id: ObjectId) -> bool {
        self.posted += 1;
        let cs_count = self
            .events
            .iter()
            .filter(|ev| matches!(ev, UiEvent::CallSerially(_)))
            .count();

        if cs_count >= MAX_CALL_SERIALLY {
            self.dropped += 1;
            *self.overflow_by_type.entry("call_serially").or_default() += 1;
            eprintln!(
                "[UI BACKPRESSURE] CallSerially queue limit {} reached! Backpressure applied.",
                MAX_CALL_SERIALLY
            );
            return false;
        }

        self.events.push_back(UiEvent::CallSerially(runnable_id));
        true
    }

    /// Requests a repaint for the specified displayable.
    /// Coalesces redundant repaints before the previous paint has been dispatched.
    pub fn request_repaint(&mut self, displayable_id: ObjectId) {
        self.posted += 1;
        if self.paint_pending == Some(displayable_id) {
            self.coalesced += 1;
            self.coalesced_repaints += 1;
            return;
        }

        self.events.push_back(UiEvent::Paint(displayable_id));
        self.paint_pending = Some(displayable_id);
    }

    /// Defers a repaint for the specified displayable to the front of the queue,
    /// marking it as pending so subsequent request_repaint calls coalesce into it.
    pub fn defer_paint(&mut self, displayable_id: ObjectId) {
        if self.paint_pending == Some(displayable_id) {
            return;
        }
        self.events.push_front(UiEvent::Paint(displayable_id));
        self.paint_pending = Some(displayable_id);
    }

    /// Posts a keyboard input event with explicit bounded capacity.
    pub fn post_key(&mut self, key: i32, action: KeyAction, displayable: ObjectId) -> bool {
        self.posted += 1;
        let key_count = self
            .events
            .iter()
            .filter(|ev| matches!(ev, UiEvent::Key { .. }))
            .count();

        if key_count >= MAX_KEY_QUEUE {
            self.dropped += 1;
            *self.overflow_by_type.entry("key").or_default() += 1;
            eprintln!(
                "[UI OVERFLOW] Key event queue reached limit {}, dropping input event",
                MAX_KEY_QUEUE
            );
            return false;
        }

        self.events.push_back(UiEvent::Key {
            key,
            action,
            displayable,
        });
        true
    }

    /// Posts ShowNotify with state-based coalescing to avoid duplicate sequential shows.
    pub fn post_show_notify(&mut self, displayable: ObjectId) {
        self.posted += 1;
        if let Some(UiEvent::ShowNotify(last)) = self.events.back() {
            if *last == displayable {
                self.coalesced += 1;
                return;
            }
        }
        self.events.push_back(UiEvent::ShowNotify(displayable));
    }

    /// Posts HideNotify with state-based coalescing.
    pub fn post_hide_notify(&mut self, displayable: ObjectId) {
        self.posted += 1;
        if let Some(UiEvent::HideNotify(last)) = self.events.back() {
            if *last == displayable {
                self.coalesced += 1;
                return;
            }
        }
        self.events.push_back(UiEvent::HideNotify(displayable));
    }

    /// Flushes events when the current active Canvas changes.
    pub fn canvas_changed(&mut self, old_canvas: Option<ObjectId>, new_canvas: Option<ObjectId>) {
        if let Some(old) = old_canvas {
            // Drop pending repaint and key events for old canvas
            let before = self.events.len();
            self.events.retain(|ev| match ev {
                UiEvent::Paint(id) if *id == old => false,
                UiEvent::Key { displayable, .. } if *displayable == old => false,
                _ => true,
            });
            let removed = (before - self.events.len()) as u64;
            self.coalesced += removed;
            if self.paint_pending == Some(old) {
                self.paint_pending = None;
            }
            self.post_hide_notify(old);
        }
        if let Some(new) = new_canvas {
            self.post_show_notify(new);
        }
    }

    /// Cancels any pending paint events for the given displayable (e.g. when serviced synchronously).
    pub fn cancel_repaints_for(&mut self, displayable_id: ObjectId) {
        let before = self.events.len();
        self.events
            .retain(|ev| !matches!(ev, UiEvent::Paint(id) if *id == displayable_id));
        let removed = (before - self.events.len()) as u64;
        self.coalesced += removed;
        if self.paint_pending == Some(displayable_id) {
            self.paint_pending = None;
        }
    }

    /// Cancels any pending showNotify events for the given displayable.
    pub fn cancel_show_notify_for(&mut self, displayable_id: ObjectId) {
        self.events
            .retain(|ev| !matches!(ev, UiEvent::ShowNotify(id) if *id == displayable_id));
    }

    /// Cancels any pending hideNotify events for the given displayable.
    pub fn cancel_hide_notify_for(&mut self, displayable_id: ObjectId) {
        self.events
            .retain(|ev| !matches!(ev, UiEvent::HideNotify(id) if *id == displayable_id));
    }

    /// Checks if a showNotify event is pending for the given displayable.
    pub fn has_pending_show_notify(&self, displayable_id: ObjectId) -> bool {
        self.events
            .iter()
            .any(|ev| matches!(ev, UiEvent::ShowNotify(id) if *id == displayable_id))
    }

    pub fn pop_next(&mut self) -> Option<UiEvent> {
        let ev = self.events.pop_front();
        if let Some(ref e) = ev {
            self.delivered += 1;
            if let UiEvent::Paint(id) = e {
                if self.paint_pending == Some(*id) {
                    self.paint_pending = None;
                }
            }
        }
        ev
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.paint_pending = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintains_fifo_order() {
        let mut queue = UiEventQueue::new();
        let r1 = ObjectId(10);
        let r2 = ObjectId(20);
        let d = ObjectId(30);

        assert!(queue.post_call_serially(r1));
        queue.request_repaint(d);
        assert!(queue.post_call_serially(r2));

        assert_eq!(queue.pop_next(), Some(UiEvent::CallSerially(r1)));
        assert_eq!(queue.pop_next(), Some(UiEvent::Paint(d)));
        assert_eq!(queue.pop_next(), Some(UiEvent::CallSerially(r2)));
        assert_eq!(queue.pop_next(), None);
        assert_eq!(queue.delivered, 3);
    }

    #[test]
    fn coalesces_multiple_repaints_before_paint_executed() {
        let mut queue = UiEventQueue::new();
        let d = ObjectId(1);

        queue.request_repaint(d);
        queue.request_repaint(d);
        queue.request_repaint(d);

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.coalesced_repaints, 2);
        assert_eq!(queue.coalesced, 2);

        // Once popped, a new repaint request can be queued
        assert_eq!(queue.pop_next(), Some(UiEvent::Paint(d)));
        queue.request_repaint(d);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop_next(), Some(UiEvent::Paint(d)));
    }

    #[test]
    fn call_serially_delivers_128_plus_events_without_silent_drop() {
        let mut queue = UiEventQueue::new();
        for i in 1..=256 {
            assert!(queue.post_call_serially(ObjectId(i)));
        }

        assert_eq!(queue.len(), 256);
        assert_eq!(queue.dropped, 0);

        for i in 1..=256 {
            assert_eq!(queue.pop_next(), Some(UiEvent::CallSerially(ObjectId(i))));
        }
        assert_eq!(queue.delivered, 256);
        assert_eq!(queue.pop_next(), None);
    }

    #[test]
    fn key_queue_bounded_capacity_and_diagnostics() {
        let mut queue = UiEventQueue::new();
        let d = ObjectId(100);

        for i in 0..MAX_KEY_QUEUE {
            assert!(queue.post_key(i as i32, KeyAction::Pressed, d));
        }

        // Exceeding capacity should return false and track dropped count
        assert!(!queue.post_key(999, KeyAction::Pressed, d));
        assert_eq!(queue.dropped, 1);
        assert_eq!(queue.overflow_by_type["key"], 1);
    }

    #[test]
    fn call_serially_boundary_2048_and_2049_backpressure() {
        let mut queue = UiEventQueue::new();
        // 2048 events succeed
        for i in 1..=MAX_CALL_SERIALLY {
            assert!(queue.post_call_serially(ObjectId(i as u32)));
        }
        assert_eq!(queue.len(), MAX_CALL_SERIALLY);
        assert_eq!(queue.dropped, 0);

        // 2049th event exceeds capacity and is rejected with backpressure (no silent drop)
        assert!(!queue.post_call_serially(ObjectId(2049)));
        assert_eq!(queue.len(), MAX_CALL_SERIALLY);
        assert_eq!(queue.dropped, 1);
        assert_eq!(queue.overflow_by_type["call_serially"], 1);

        // Popping 1 event frees a slot
        assert_eq!(queue.pop_next(), Some(UiEvent::CallSerially(ObjectId(1))));
        assert_eq!(queue.len(), MAX_CALL_SERIALLY - 1);

        // Now post succeeds again
        assert!(queue.post_call_serially(ObjectId(2050)));
        assert_eq!(queue.len(), MAX_CALL_SERIALLY);
    }

    #[test]
    fn key_queue_boundary_128_and_129_and_causal_order() {
        let mut queue = UiEventQueue::new();
        let disp = ObjectId(50);

        // 128 events accepted
        for i in 1..=MAX_KEY_QUEUE {
            assert!(queue.post_key(i as i32, KeyAction::Pressed, disp));
        }
        assert_eq!(queue.len(), MAX_KEY_QUEUE);
        assert_eq!(queue.dropped, 0);

        // 129th event rejected (backpressure)
        assert!(!queue.post_key(129, KeyAction::Pressed, disp));
        assert_eq!(queue.len(), MAX_KEY_QUEUE);
        assert_eq!(queue.dropped, 1);
        assert_eq!(queue.overflow_by_type["key"], 1);

        // Free up and test causal ordering press -> release
        queue.clear();
        assert!(queue.post_key(-5, KeyAction::Pressed, disp));
        assert!(queue.post_key(-5, KeyAction::Released, disp));
        assert!(queue.post_key(-6, KeyAction::Pressed, disp));
        assert!(queue.post_key(-6, KeyAction::Released, disp));

        assert_eq!(
            queue.pop_next(),
            Some(UiEvent::Key {
                key: -5,
                action: KeyAction::Pressed,
                displayable: disp
            })
        );
        assert_eq!(
            queue.pop_next(),
            Some(UiEvent::Key {
                key: -5,
                action: KeyAction::Released,
                displayable: disp
            })
        );
        assert_eq!(
            queue.pop_next(),
            Some(UiEvent::Key {
                key: -6,
                action: KeyAction::Pressed,
                displayable: disp
            })
        );
        assert_eq!(
            queue.pop_next(),
            Some(UiEvent::Key {
                key: -6,
                action: KeyAction::Released,
                displayable: disp
            })
        );
        assert_eq!(queue.pop_next(), None);
    }

    #[test]
    fn canvas_change_discards_pending_events_for_old_canvas() {
        let mut queue = UiEventQueue::new();
        let old_c = ObjectId(10);
        let new_c = ObjectId(20);

        assert!(queue.post_key(-1, KeyAction::Pressed, old_c));
        queue.request_repaint(old_c);
        assert!(queue.post_call_serially(ObjectId(99))); // should survive canvas change

        queue.canvas_changed(Some(old_c), Some(new_c));

        // old_c key and paint should be discarded; showNotify(new_c) and callSerially remain
        assert_eq!(queue.pop_next(), Some(UiEvent::CallSerially(ObjectId(99))));
        assert_eq!(queue.pop_next(), Some(UiEvent::HideNotify(old_c)));
        assert_eq!(queue.pop_next(), Some(UiEvent::ShowNotify(new_c)));
        assert_eq!(queue.pop_next(), None);
    }
}
