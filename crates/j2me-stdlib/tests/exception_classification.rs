use j2me_core::classfile::attributes::ExceptionTableEntry;
use j2me_core::heap::value::JavaValue;
use j2me_core::threads::green_thread::ThreadState;
use j2me_core::vm::frame::Frame;
use j2me_core::vm::instruction::Instruction;
use j2me_stdlib::bridge::native_bridge::NativeBridge;
use j2me_stdlib::J2meEmulator;
use std::collections::HashMap;
use std::sync::Arc;

fn make_test_frame(
    class_name: &str,
    method_name: &str,
    descriptor: &str,
    instructions: Vec<Instruction>,
    max_locals: usize,
    max_stack: usize,
    exception_table: Vec<ExceptionTableEntry>,
) -> Frame {
    let mut pc_map = HashMap::new();
    for i in 0..instructions.len() {
        pc_map.insert(i as u16, i);
    }
    Frame::new(
        class_name.to_string(),
        method_name.to_string(),
        descriptor.to_string(),
        Arc::new(instructions),
        max_locals,
        max_stack,
        exception_table,
        Arc::new(pc_map),
    )
}

#[test]
fn test_exception_caught_in_same_method() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Bytecode instructions:
    // 0: AConstNull
    // 1: AThrow       (throws NullPointerException)
    // 2: Pop          (handler at pc=2 pops the caught exception)
    // 3: Return
    let instructions = vec![
        Instruction::AConstNull,
        Instruction::AThrow,
        Instruction::Pop,
        Instruction::Return,
    ];
    let exception_table = vec![ExceptionTableEntry {
        start_pc: 0,
        end_pc: 2,
        handler_pc: 2,
        catch_type: 0,
        catch_class: None, // catch-all
    }];

    let frame = make_test_frame(
        "TestClass",
        "run",
        "()V",
        instructions,
        1,
        2,
        exception_table,
    );
    emu.scheduler.spawn("main", frame, None);
    emu.is_running = true;

    // Run emulator tick
    emu.tick(50.0);
    assert_eq!(
        emu.scheduler.active_thread_count(),
        0,
        "Thread should complete normally via Return"
    );

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 1, "Thrown count must be 1");
    assert_eq!(telem.guest_exceptions_caught, 1, "Caught count must be 1");
    assert_eq!(
        telem.guest_exceptions_uncaught, 0,
        "Uncaught count must be 0"
    );
    assert!(
        telem.fatal_guest_exception.is_none(),
        "Handled exception must NOT be fatal"
    );
    assert!(
        telem.uncaught_exception_threads.is_empty(),
        "No threads terminated with uncaught exception"
    );
}

#[test]
fn test_exception_caught_in_caller_method() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Frame 1 (Caller):
    // 0: Nop          (placeholder for invocation site)
    // 1: Pop          (handler at pc=1 pops caught exception)
    // 2: Return
    let caller_instructions = vec![Instruction::Nop, Instruction::Pop, Instruction::Return];
    let caller_ex_table = vec![ExceptionTableEntry {
        start_pc: 0,
        end_pc: 1,
        handler_pc: 1,
        catch_type: 0,
        catch_class: None,
    }];
    let caller_frame = make_test_frame(
        "CallerClass",
        "main",
        "()V",
        caller_instructions,
        1,
        2,
        caller_ex_table,
    );

    // Frame 2 (Callee):
    // 0: AConstNull
    // 1: AThrow       (throws NullPointerException, no handler in callee)
    let callee_instructions = vec![Instruction::AConstNull, Instruction::AThrow];
    let callee_frame = make_test_frame(
        "CalleeClass",
        "helper",
        "()V",
        callee_instructions,
        1,
        2,
        Vec::new(),
    );

    // Spawn thread and push caller then callee onto call_stack
    let t_id = emu.scheduler.spawn("main", caller_frame, None);
    emu.scheduler.threads[t_id.0 as usize - 1]
        .call_stack
        .push(callee_frame);
    emu.is_running = true;

    emu.tick(50.0);
    assert_eq!(
        emu.scheduler.active_thread_count(),
        0,
        "Caller catches exception and completes cleanly"
    );

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 1, "Thrown count must be 1");
    assert_eq!(
        telem.guest_exceptions_caught, 1,
        "Caught count must be 1 (unwound to caller)"
    );
    assert_eq!(
        telem.guest_exceptions_uncaught, 0,
        "Uncaught count must be 0"
    );
    assert!(
        telem.fatal_guest_exception.is_none(),
        "Caught in caller must NOT be fatal"
    );
}

#[test]
fn test_uncaught_exception_in_main_thread() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Instructions: throw without handler
    let instructions = vec![Instruction::AConstNull, Instruction::AThrow];
    let frame = make_test_frame(
        "MainGame",
        "startApp",
        "()V",
        instructions,
        1,
        2,
        Vec::new(),
    );
    emu.scheduler.spawn("main", frame, None);
    emu.is_running = true;

    emu.tick(50.0);

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 1, "Thrown count must be 1");
    assert_eq!(telem.guest_exceptions_caught, 0, "Caught count must be 0");
    assert_eq!(
        telem.guest_exceptions_uncaught, 1,
        "Uncaught count must be 1"
    );
    assert!(
        telem.fatal_guest_exception.is_some(),
        "Uncaught exception in main thread MUST be recorded as fatal"
    );
    assert_eq!(telem.uncaught_exception_threads, vec!["main".to_string()]);

    let fatal = telem.fatal_guest_exception.as_ref().unwrap();
    assert_eq!(fatal.exception_class, "java/lang/NullPointerException");
    assert_eq!(fatal.thread_name, "main");
    assert_eq!(fatal.class_name, "MainGame");
}

#[test]
fn test_uncaught_exception_in_worker_thread_main_remains_active() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Main thread has a frame that sleeps/waits (keeps alive)
    let main_instructions = vec![Instruction::Nop, Instruction::Nop, Instruction::Return];
    let main_frame = make_test_frame(
        "MainGame",
        "run",
        "()V",
        main_instructions,
        1,
        2,
        Vec::new(),
    );
    let main_tid = emu.scheduler.spawn("main", main_frame, None);
    emu.scheduler.threads[main_tid.0 as usize - 1].state = ThreadState::Sleeping {
        wake_time_ms: 100_000,
    };

    // Worker thread throws an uncaught exception
    let worker_instructions = vec![Instruction::AConstNull, Instruction::AThrow];
    let worker_frame = make_test_frame(
        "WorkerTask",
        "run",
        "()V",
        worker_instructions,
        1,
        2,
        Vec::new(),
    );
    let _worker_tid = emu.scheduler.spawn("Worker-Thread", worker_frame, None);

    emu.is_running = true;
    emu.tick(50.0);

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 1);
    assert_eq!(telem.guest_exceptions_caught, 0);
    assert_eq!(telem.guest_exceptions_uncaught, 1);
    assert_eq!(
        telem.uncaught_exception_threads,
        vec!["Worker-Thread".to_string()]
    );

    // Main thread is still alive, so worker thread death does not mark fatal_guest_exception!
    assert!(
        telem.fatal_guest_exception.is_none(),
        "Uncaught exception in non-main worker thread while main is alive must NOT be fatal to entire VM"
    );
    // Worker thread terminated and was cleaned up, main thread is still alive!
    assert_eq!(emu.scheduler.active_thread_count(), 1);
    assert_eq!(emu.scheduler.threads[0].name, "main");
    assert_eq!(
        emu.scheduler.threads[0].state,
        ThreadState::Sleeping {
            wake_time_ms: 100_000
        }
    );
}

#[test]
fn test_multiple_exceptions_some_caught_some_uncaught() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // 0: AConstNull
    // 1: AThrow      (caught by handler at 2)
    // 2: Pop
    // 3: AConstNull
    // 4: AThrow      (caught by handler at 5)
    // 5: Pop
    // 6: AConstNull
    // 7: AThrow      (uncaught!)
    let instructions = vec![
        Instruction::AConstNull,
        Instruction::AThrow,
        Instruction::Pop,
        Instruction::AConstNull,
        Instruction::AThrow,
        Instruction::Pop,
        Instruction::AConstNull,
        Instruction::AThrow,
    ];
    let exception_table = vec![
        ExceptionTableEntry {
            start_pc: 0,
            end_pc: 2,
            handler_pc: 2,
            catch_type: 0,
            catch_class: None,
        },
        ExceptionTableEntry {
            start_pc: 3,
            end_pc: 5,
            handler_pc: 5,
            catch_type: 0,
            catch_class: None,
        },
    ];

    let frame = make_test_frame(
        "MultiExcClass",
        "run",
        "()V",
        instructions,
        1,
        2,
        exception_table,
    );
    emu.scheduler.spawn("main", frame, None);
    emu.is_running = true;

    emu.tick(50.0);

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 3, "Total thrown should be 3");
    assert_eq!(telem.guest_exceptions_caught, 2, "Caught count should be 2");
    assert_eq!(
        telem.guest_exceptions_uncaught, 1,
        "Uncaught count should be 1"
    );
    assert!(
        telem.fatal_guest_exception.is_some(),
        "3rd uncaught exception in main thread must be fatal"
    );
}

#[test]
fn test_resource_lookup_exception_handled_by_game_code() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Simulate game code looking up a resource:
    // 1. calls Class.getResourceAsStream("nonexistent.png") -> returns null
    // 2. game code checks or attempts to use it, throws/catches NullPointerException
    // 3. game falls back to procedural graphics and finishes frame
    let class_obj = emu.heap.alloc_instance("java/lang/Class");
    let name_str = emu.heap.alloc_string("com/game/Main");
    emu.heap
        .set_field(class_obj, "name", JavaValue::Object(Some(name_str)));

    let path_str = emu.heap.alloc_string("missing_sprite.png");
    let res = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(path_str))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    );
    assert_eq!(
        res.unwrap(),
        Some(JavaValue::NULL),
        "Missing resource must return null without crashing"
    );

    // Game executes catch block in bytecode
    let instructions = vec![
        Instruction::AConstNull,
        Instruction::AThrow, // throws handled exception
        Instruction::Pop,    // catch block recovers
        Instruction::Return, // game proceeds normally
    ];
    let exception_table = vec![ExceptionTableEntry {
        start_pc: 0,
        end_pc: 2,
        handler_pc: 2,
        catch_type: 0,
        catch_class: None,
    }];
    let frame = make_test_frame(
        "com/game/Main",
        "loadAssets",
        "()V",
        instructions,
        1,
        2,
        exception_table,
    );
    emu.scheduler.spawn("main", frame, None);
    emu.is_running = true;

    emu.tick(50.0);
    assert_eq!(
        emu.scheduler.active_thread_count(),
        0,
        "Game thread completes cleanly after handled exception"
    );

    let telem = &emu.context.exception_telemetry;
    assert_eq!(telem.guest_exceptions_thrown, 1);
    assert_eq!(telem.guest_exceptions_caught, 1);
    assert_eq!(telem.guest_exceptions_uncaught, 0);
    assert!(telem.fatal_guest_exception.is_none());
}
