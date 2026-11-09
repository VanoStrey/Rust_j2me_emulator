use j2me_core::heap::Heap;
use j2me_core::threads::green_thread::{GreenThread, ThreadId};
use j2me_core::vm::class_loader::ClassLoader;
use j2me_core::vm::frame::Frame;
use j2me_core::vm::instruction::Instruction;
use j2me_core::vm::interpreter::{Interpreter, StepResult};
use j2me_stdlib::bridge::context::{hash_framebuffer, TestVerdict};
use j2me_stdlib::J2meEmulator;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_auto_healing_recorded_and_prevents_false_pass() {
    let mut heap = Heap::new();
    let cl = ClassLoader::new();

    // Setup caller and callee frames
    let parent_frame = Frame::new(
        "com/game/GameEngine".to_string(),
        "update".to_string(),
        "()V".to_string(),
        Arc::new(vec![Instruction::Nop, Instruction::Return]),
        4,
        4,
        vec![],
        Arc::new(HashMap::new()),
    );
    let mut thread = GreenThread::new(ThreadId(1), "main".to_string(), parent_frame, None);

    let child_frame = Frame::new(
        "com/game/Physics".to_string(),
        "computeVelocities".to_string(),
        "()I".to_string(),
        Arc::new(vec![Instruction::AThrow]),
        4,
        4,
        vec![],
        Arc::new(HashMap::new()),
    );
    thread.call_stack.push(child_frame);

    let exc_id = heap.alloc_instance("java/lang/NullPointerException");
    let step_res = Interpreter::handle_athrow(&mut thread, &mut heap, &cl, exc_id);

    // Auto-healing continues execution
    assert!(matches!(step_res, StepResult::Continue));
    assert_eq!(thread.auto_healing_count, 1);
    assert_eq!(thread.auto_healing_events.len(), 1);

    // Create emulator and transfer thread
    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;
    emu.context.current_displayable = Some(j2me_core::heap::object::ObjectId(1));
    emu.scheduler.threads.push(thread);

    // Run a tick to sync telemetry
    emu.tick(16.6);

    // Standard mode: verdict MUST NOT be PASS; it must be DEGRADED
    let report = emu.evaluate_observability(false);
    println!("REPORT: {:#?}", report);
    assert_eq!(
        report.verdict,
        TestVerdict::Degraded,
        "Auto-healing MUST degrade the verdict and NEVER masquerade as PASS"
    );
    assert!(report.criteria.auto_heal_occurred);
    assert_eq!(report.auto_healing_events.len(), 1);

    let ev = &report.auto_healing_events[0];
    assert_eq!(ev.exception_class, "java/lang/NullPointerException");
    assert_eq!(ev.class_name, "com/game/Physics");
    assert_eq!(ev.method_name, "computeVelocities");
    assert_eq!(ev.descriptor, "()I");
    assert_eq!(ev.pc, 0);
    assert!(!ev.stack_trace.is_empty());
    assert!(ev.can_continue);

    // Strict mode: verdict MUST also NEVER be PASS
    emu.set_strict_mode(true);
    let strict_report = emu.evaluate_observability(false);
    assert_ne!(
        strict_report.verdict,
        TestVerdict::Pass,
        "Strict mode must never classify auto-healed runs as PASS"
    );
    assert_eq!(strict_report.verdict, TestVerdict::Degraded);
}

#[test]
fn test_framebuffer_published_vs_changed_distinction() {
    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;
    emu.context.current_displayable = Some(j2me_core::heap::object::ObjectId(1));

    // Initial state: buffer is standby/white
    let initial_hash = hash_framebuffer(&emu.context.framebuffer);
    emu.context.last_framebuffer_hash = initial_hash;
    emu.context.framebuffer_changes_count = 0;
    emu.context.fb_publishes_count = 0;

    // Simulate 10 ticks where screen_dirty is marked but pixels do NOT change
    for _ in 0..10 {
        emu.context.screen_dirty = true;
        emu.tick(16.6);
    }

    // 10 publishes occurred
    assert_eq!(emu.context.fb_publishes_count, 10);
    // But 0 visual changes occurred because pixel values remained identical!
    assert_eq!(
        emu.context.framebuffer_changes_count, 0,
        "Identical frames must NOT increment framebuffer_changes_count"
    );

    let report_static = emu.evaluate_observability(false);
    assert!(
        report_static.criteria.framebuffer_published,
        "FRAMEBUFFER_PUBLISHED must be true"
    );
    assert!(
        !report_static.criteria.framebuffer_changed,
        "FRAMEBUFFER_CHANGED must be false when pixel values are identical"
    );
    assert_eq!(
        report_static.verdict,
        TestVerdict::Partial,
        "Static published buffer without visual mutation must be PARTIAL, not PASS"
    );

    // Now mutate pixels on the framebuffer (simulating actual gameplay/drawing)
    emu.context.framebuffer[100] = 0xFF00FF00; // Mutate pixel to green
    emu.context.screen_dirty = true;
    emu.tick(16.6);

    assert_eq!(emu.context.fb_publishes_count, 11);
    assert_eq!(
        emu.context.framebuffer_changes_count, 1,
        "Actual visual pixel mutation MUST increment framebuffer_changes_count"
    );

    let report_mutated = emu.evaluate_observability(false);
    assert!(report_mutated.criteria.framebuffer_changed);
}

#[test]
fn test_input_delivered_vs_input_affected_game_state_distinction() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    let initial_report = emu.evaluate_observability(false);
    assert!(!initial_report.criteria.input_delivered);
    assert!(!initial_report.criteria.input_affected_game_state);

    // Deliver input event (FIRE key pressed)
    emu.key_event(-5, true);

    assert_eq!(emu.context.input_delivered_count, 1);
    let after_delivery = emu.evaluate_observability(false);
    assert!(after_delivery.criteria.input_delivered);
    assert!(
        !after_delivery.criteria.input_affected_game_state,
        "Input delivery alone does NOT imply game state reacted"
    );

    // Game queries key states via MIDP 2.0 GameCanvas.getKeyStates()
    let states = emu.context.get_and_clear_key_states();
    assert_ne!(states, 0, "FIRE key bit must be latched in key states");
    emu.context.input_affected_game_state = true;

    let after_consumed = emu.evaluate_observability(false);
    assert!(after_consumed.criteria.input_affected_game_state);
}

#[test]
fn test_guest_exception_preserves_full_stack_trace_and_bytecode_pc() {
    let mut heap = Heap::new();
    let cl = ClassLoader::new();

    let root_frame = Frame::new(
        "com/game/App".to_string(),
        "startApp".to_string(),
        "()V".to_string(),
        Arc::new(vec![Instruction::AThrow]),
        2,
        2,
        vec![],
        Arc::new(HashMap::new()),
    );
    let mut thread = GreenThread::new(ThreadId(1), "main".to_string(), root_frame, None);

    let exc_id = heap.alloc_instance("java/lang/IllegalArgumentException");
    let step_res = Interpreter::handle_athrow(&mut thread, &mut heap, &cl, exc_id);

    // When stack depth == 1, uncaught athrow returns UnhandledException
    assert!(matches!(step_res, StepResult::UnhandledException(_)));

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.scheduler.threads.push(thread);
    emu.record_uncaught_exception(0, exc_id);

    let report = emu.evaluate_observability(false);
    assert_eq!(report.verdict, TestVerdict::Fail);
    assert!(report.criteria.guest_exception);

    let exc = report.last_uncaught_exception.expect("Must preserve uncaught exception record");
    assert_eq!(exc.exception_class, "java/lang/IllegalArgumentException");
    assert_eq!(exc.class_name, "com/game/App");
    assert_eq!(exc.method_name, "startApp");
    assert_eq!(exc.descriptor, "()V");
    assert_eq!(exc.pc, 0);
    assert!(!exc.stack_trace.is_empty());
}

#[test]
fn test_sprite_set_transform_preserves_reference_pixel_position() {
    use j2me_stdlib::midp::lcdui::game::sprite::{Sprite, TRANS_MIRROR, TRANS_NONE};
    use j2me_stdlib::midp::lcdui::image::Image;

    let img = Image::new_mutable(32, 32);
    let mut spr = Sprite::new(img, 16, 16);
    spr.set_position(100, 100);
    spr.define_reference_pixel(4, 8);

    // Initial reference pixel in display coordinates:
    // With TRANS_NONE, display ref pixel is at (100 + 4, 100 + 8) = (104, 108)
    // Now change transform to TRANS_MIRROR (2)
    // Under TRANS_MIRROR, dx = 16 - 1 - 4 = 11, dy = 8.
    // The sprite (x, y) must update to (ref_disp_x - dx, ref_disp_y - dy) = (104 - 11, 108 - 8) = (93, 100)
    spr.set_transform(TRANS_MIRROR);
    assert_eq!(spr.x, 93, "Sprite x must update so reference pixel remains at 104");
    assert_eq!(spr.y, 100, "Sprite y must update so reference pixel remains at 108");

    // Switching back to TRANS_NONE should restore (x, y) back to (100, 100)
    spr.set_transform(TRANS_NONE);
    assert_eq!(spr.x, 100, "Sprite x must restore to 100");
    assert_eq!(spr.y, 100, "Sprite y must restore to 100");
}

#[test]
fn test_strict_mode_prevents_auto_healing_and_returns_unhandled_exception() {
    let mut heap = Heap::new();
    let cl = ClassLoader::new();

    let parent_frame = Frame::new(
        "com/game/Engine".to_string(),
        "run".to_string(),
        "()V".to_string(),
        Arc::new(vec![Instruction::Nop, Instruction::Return]),
        4,
        4,
        vec![],
        Arc::new(HashMap::new()),
    );
    let mut thread = GreenThread::new(ThreadId(1), "main".to_string(), parent_frame, None);
    thread.strict_mode = true; // Strict mode enabled

    let child_frame = Frame::new(
        "com/game/Physics".to_string(),
        "step".to_string(),
        "()I".to_string(),
        Arc::new(vec![Instruction::AThrow]),
        4,
        4,
        vec![], // No catch block
        Arc::new(HashMap::new()),
    );
    thread.call_stack.push(child_frame);

    let exc_id = heap.alloc_instance("java/lang/NullPointerException");
    let step_res = Interpreter::handle_athrow(&mut thread, &mut heap, &cl, exc_id);

    // In strict mode, Auto-Healing MUST NOT intervene: it must return UnhandledException
    assert!(
        matches!(step_res, StepResult::UnhandledException(_)),
        "Strict mode must return UnhandledException instead of auto-healing"
    );
    assert_eq!(thread.auto_healing_count, 0, "Auto-healing count must remain 0 in strict mode");
}

