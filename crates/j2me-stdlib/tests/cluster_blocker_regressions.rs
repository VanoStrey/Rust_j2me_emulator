use j2me_core::heap::object::{ArrayData, Heap, ObjectData};
use j2me_core::heap::value::JavaValue;
use j2me_core::threads::green_thread::{GreenThread, ThreadId};
use j2me_core::threads::scheduler::Scheduler;
use j2me_core::vm::class_loader::ClassLoader;
use j2me_core::vm::frame::Frame;
use j2me_stdlib::bridge::context::{GraphicsTarget, VmContext};
use j2me_stdlib::bridge::native_bridge::NativeBridge;
use j2me_stdlib::midp::lcdui::image::Image;
use j2me_stdlib::nokia::DirectGraphics;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_cluster1_record_enumeration_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // 1. Open RecordStore and add 2 records
    let store_name = "ClusterTestStore";
    let name_id = heap.alloc_string(store_name);
    let store_ref = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "openRecordStore",
        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
        None,
        &[JavaValue::Object(Some(name_id)), JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    let rec1_data = heap.alloc_array(ArrayData::Byte(vec![10, 20, 30]));
    let rec1_id = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "addRecord",
        "([BII)I",
        store_ref,
        &[
            JavaValue::Object(Some(rec1_data)),
            JavaValue::Int(0),
            JavaValue::Int(3),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(rec1_id, 1);

    let rec2_data = heap.alloc_array(ArrayData::Byte(vec![40, 50]));
    let rec2_id = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "addRecord",
        "([BII)I",
        store_ref,
        &[
            JavaValue::Object(Some(rec2_data)),
            JavaValue::Int(0),
            JavaValue::Int(2),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(rec2_id, 2);

    // 2. Enumerate records
    let enum_ref = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "enumerateRecords",
        "(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;",
        store_ref,
        &[JavaValue::NULL, JavaValue::NULL, JavaValue::Int(0)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    // numRecords should be 2
    let num = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "numRecords",
        "()I",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(num, 2);

    // hasNextElement is true (1), hasPreviousElement is false (0)
    let has_next = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "hasNextElement",
        "()Z",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(has_next, 1);

    let has_prev = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "hasPreviousElement",
        "()Z",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(has_prev, 0);

    // previousRecordId at index 0 must throw InvalidRecordIDException
    let prev_err = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "previousRecordId",
        "()I",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(prev_err.is_err());
    assert!(prev_err
        .unwrap_err()
        .contains("javax/microedition/rms/InvalidRecordIDException"));

    // Advance 1: nextRecordId() -> 1
    let id1 = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "nextRecordId",
        "()I",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(id1, 1);

    // Advance 2: nextRecord() -> returns byte[]
    let rec2_bytes_obj = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "nextRecord",
        "()[B",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(rec2_bytes_obj) {
        assert_eq!(b, &[40, 50]);
    } else {
        panic!("Expected byte array from nextRecord()");
    }

    // Now at end of records: nextRecordId must throw InvalidRecordIDException (NOT NoSuchElementException!)
    let end_err = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "nextRecordId",
        "()I",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(end_err.is_err());
    let err_str = end_err.unwrap_err();
    assert!(
        err_str.contains("javax/microedition/rms/InvalidRecordIDException"),
        "Must throw InvalidRecordIDException, got: {}",
        err_str
    );
    assert!(
        !err_str.contains("NoSuchElementException"),
        "Must NOT throw NoSuchElementException!"
    );

    // Test moving backward: previousRecordId() -> 2
    let prev_id = NativeBridge::dispatch(
        "javax/microedition/rms/RecordEnumeration",
        "previousRecordId",
        "()I",
        enum_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(prev_id, 2);
}

#[test]
fn test_cluster2_class_reflections_and_stream_exceptions() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let initial_frame = Frame::new(
        "TestClass".to_string(),
        "testMethod".to_string(),
        "()V".to_string(),
        Arc::new(vec![]),
        10,
        10,
        vec![],
        Arc::new(HashMap::new()),
    );
    scheduler.threads.push(GreenThread::new(
        ThreadId(0),
        "test_main".to_string(),
        initial_frame,
        None,
    ));
    let mut context = VmContext::new(240, 320, 44100);

    // 1. Class.forName
    let class_name_str = heap.alloc_string("com.test.GameService");
    let cls_obj = NativeBridge::dispatch(
        "java/lang/Class",
        "forName",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        None,
        &[JavaValue::Object(Some(class_name_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    // Verify instance class is "java/lang/Class"
    if let Some(ObjectData::Instance(inst)) = heap.get(cls_obj) {
        assert_eq!(inst.class_name, "java/lang/Class");
    } else {
        panic!("Class.forName must allocate an instance of java/lang/Class");
    }

    // 2. Class.newInstance on Class object returns an instance of target class
    let instance_obj = NativeBridge::dispatch(
        "java/lang/Class",
        "newInstance",
        "()Ljava/lang/Object;",
        Some(cls_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    if let Some(ObjectData::Instance(inst)) = heap.get(instance_obj) {
        assert_eq!(inst.class_name, "com/test/GameService");
    } else {
        panic!("Class.newInstance must allocate an instance of the target class");
    }

    // 3. DataInputStream on null inner stream must throw NullPointerException
    let dis_obj = heap.alloc_instance("java/io/DataInputStream");
    // "in" field is None (null)
    let read_err = NativeBridge::dispatch(
        "java/io/DataInputStream",
        "readByte",
        "()B",
        Some(dis_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(read_err.is_err());
    let err_str = read_err.unwrap_err();
    assert!(
        err_str.contains("EXC:java/lang/NullPointerException"),
        "Must format as Java exception EXC:java/lang/NullPointerException, got: {}",
        err_str
    );

    // 4. System.getProperty unknown key returns null
    let prop_key = heap.alloc_string("unknown.system.property");
    let prop_res = NativeBridge::dispatch(
        "java/lang/System",
        "getProperty",
        "(Ljava/lang/String;)Ljava/lang/String;",
        None,
        &[JavaValue::Object(Some(prop_key))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(prop_res, JavaValue::NULL);
}

#[test]
fn test_cluster3_sprite_and_layer_contracts() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Create an Image in context
    let img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
    let test_img = Image::new_mutable_with_color(64, 48, 0xFF0000FF);
    context.images.insert(img_id, test_img);

    // Create Sprite with Image and frame size 16x16
    let sprite_id = heap.alloc_instance("javax/microedition/lcdui/game/Sprite");
    let _ = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "<init>",
        "(Ljavax/microedition/lcdui/Image;II)V",
        Some(sprite_id),
        &[
            JavaValue::Object(Some(img_id)),
            JavaValue::Int(16),
            JavaValue::Int(16),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Verify getWidth and getHeight are strictly positive
    let w = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "getWidth",
        "()I",
        Some(sprite_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(w, 16);

    let h = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "getHeight",
        "()I",
        Some(sprite_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(h, 16);

    // Position and move
    let _ = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "setPosition",
        "(II)V",
        Some(sprite_id),
        &[JavaValue::Int(10), JavaValue::Int(20)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );

    let _ = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "move",
        "(II)V",
        Some(sprite_id),
        &[JavaValue::Int(5), JavaValue::Int(-3)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );

    let x = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "getX",
        "()I",
        Some(sprite_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(x, 15);

    let y = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "getY",
        "()I",
        Some(sprite_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(y, 17);

    // Frame count: 64/16 * 48/16 = 4 * 3 = 12 frames
    let frames = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "getRawFrameCount",
        "()I",
        Some(sprite_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(frames, 12);
}

#[test]
fn test_phase47_lcdui_canvas_array_reflection_contracts() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(176, 208, 44100);

    // 1. canvas.isDoubleBuffered() == true on standard Canvas and FullCanvas
    let canvas_id = heap.alloc_instance("javax/microedition/lcdui/Canvas");
    let is_db = NativeBridge::dispatch(
        "javax/microedition/lcdui/Canvas",
        "isDoubleBuffered",
        "()Z",
        Some(canvas_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(is_db, 1);

    let full_canvas_id = heap.alloc_instance("com/nokia/mid/ui/FullCanvas");
    let is_db_full = NativeBridge::dispatch(
        "com/nokia/mid/ui/FullCanvas",
        "isDoubleBuffered",
        "()Z",
        Some(full_canvas_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(is_db_full, 1);

    // 2. Class.forName array descriptors
    // a. Class.forName("[B").getName().equals("[B")
    let b_str_id = heap.alloc_string("[B");
    let b_cls = NativeBridge::dispatch(
        "java/lang/Class",
        "forName",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        None,
        &[JavaValue::Object(Some(b_str_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    let b_name_id = NativeBridge::dispatch(
        "java/lang/Class",
        "getName",
        "()Ljava/lang/String;",
        Some(b_cls),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();
    assert_eq!(heap.get_string_utf8(b_name_id).unwrap(), "[B");

    // b. Class.forName("[Ljava.lang.String;").getName().equals("[Ljava.lang.String;")
    let str_arr_id = heap.alloc_string("[Ljava.lang.String;");
    let str_arr_cls = NativeBridge::dispatch(
        "java/lang/Class",
        "forName",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        None,
        &[JavaValue::Object(Some(str_arr_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    let str_arr_name_id = NativeBridge::dispatch(
        "java/lang/Class",
        "getName",
        "()Ljava/lang/String;",
        Some(str_arr_cls),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();
    assert_eq!(
        heap.get_string_utf8(str_arr_name_id).unwrap(),
        "[Ljava.lang.String;"
    );

    // 3. Subclass of FullCanvas via Class.forName(...).newInstance()
    let fc_sub_name = heap.alloc_string("com.nokia.mid.ui.FullCanvas");
    let fc_cls = NativeBridge::dispatch(
        "java/lang/Class",
        "forName",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        None,
        &[JavaValue::Object(Some(fc_sub_name))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    let fc_inst = NativeBridge::dispatch(
        "java/lang/Class",
        "newInstance",
        "()Ljava/lang/Object;",
        Some(fc_cls),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    // Verify getWidth(), getHeight(), isDoubleBuffered()
    let w = NativeBridge::dispatch(
        "com/nokia/mid/ui/FullCanvas",
        "getWidth",
        "()I",
        Some(fc_inst),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(w, 176);

    let h = NativeBridge::dispatch(
        "com/nokia/mid/ui/FullCanvas",
        "getHeight",
        "()I",
        Some(fc_inst),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(h, 208);

    // Verify Display.getDisplay(midlet)
    let midlet_id = heap.alloc_instance("javax/microedition/midlet/MIDlet");
    let disp_id = NativeBridge::dispatch(
        "javax/microedition/lcdui/Display",
        "getDisplay",
        "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
        None,
        &[JavaValue::Object(Some(midlet_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    if let Some(ObjectData::Instance(inst)) = heap.get(disp_id) {
        assert_eq!(inst.class_name, "javax/microedition/lcdui/Display");
    } else {
        panic!("Display.getDisplay must return a valid Display instance");
    }
}

#[test]
fn test_phase48_acc_synchronized_method_monitors_contract() {
    use j2me_core::classfile::attributes::ExceptionTableEntry;
    use j2me_core::classfile::class_file::{ACC_PUBLIC, ACC_STATIC, ACC_SYNCHRONIZED};
    use j2me_core::vm::class_loader::PredecodedMethod;
    use j2me_core::vm::instruction::Instruction;
    use j2me_stdlib::J2meEmulator;

    // 1. Test synchronized instance method: wait(10) + notifyAll() succeeds without bytecode monitorenter
    {
        let mut emu = J2meEmulator::new(240, 320, 44100);
        emu.is_running = true;

        // Predecode synchronized method TestSync.syncMethod:()V
        // Bytecode: aload_0; ldc2_w 10L; invokevirtual wait(J)V; aload_0; invokevirtual notifyAll()V; return
        let sync_method = PredecodedMethod {
            class_name: "TestSync".to_string(),
            name: "syncMethod".to_string(),
            descriptor: "()V".to_string(),
            access_flags: ACC_PUBLIC | ACC_SYNCHRONIZED,
            max_locals: 2,
            max_stack: 4,
            instructions: Arc::new(vec![
                Instruction::ALoad(0),
                Instruction::LdcLong(10),
                Instruction::InvokeVirtual {
                    class_name: "java/lang/Object".to_string(),
                    method_name: "wait".to_string(),
                    descriptor: "(J)V".to_string(),
                },
                Instruction::ALoad(0),
                Instruction::InvokeVirtual {
                    class_name: "java/lang/Object".to_string(),
                    method_name: "notifyAll".to_string(),
                    descriptor: "()V".to_string(),
                },
                Instruction::Return,
            ]),
            pc_map: Arc::new(HashMap::new()),
            exception_table: Vec::new(),
        };
        emu.class_loader.predecoded_methods.insert(
            (
                "TestSync".to_string(),
                "syncMethod".to_string(),
                "()V".to_string(),
            ),
            Arc::new(sync_method),
        );

        let target_obj = emu.heap.alloc_instance("TestSync");

        // Caller frame: invokevirtual TestSync.syncMethod()V; return
        let mut caller_pc_map = HashMap::new();
        caller_pc_map.insert(0, 0);
        caller_pc_map.insert(1, 1);
        let caller_frame = Frame::new(
            "Main".to_string(),
            "run".to_string(),
            "()V".to_string(),
            Arc::new(vec![
                Instruction::ALoad(0),
                Instruction::InvokeVirtual {
                    class_name: "TestSync".to_string(),
                    method_name: "syncMethod".to_string(),
                    descriptor: "()V".to_string(),
                },
                Instruction::Return,
            ]),
            2,
            4,
            Vec::new(),
            Arc::new(caller_pc_map),
        );

        let _tid = emu.scheduler.spawn("TestThread", caller_frame, None);
        emu.scheduler.threads[0].call_stack[0].set_local(0, JavaValue::Object(Some(target_obj)));

        // Run slice: syncMethod enters, locks target_obj, waits 10ms, wakes, calls notifyAll, returns
        // Monitor is cleanly released on exit!
        for _ in 0..5 {
            emu.tick(50.0);
        }

        // Verify thread completed without uncaught IllegalMonitorStateException
        assert_eq!(
            emu.context.exception_telemetry.guest_exceptions_uncaught, 0,
            "Synchronized wait must not throw uncaught exceptions"
        );
        assert_eq!(
            emu.scheduler.get_monitor_owner(target_obj),
            None,
            "Monitor must be fully released after synchronized method returns"
        );
    }

    // 2. Test unsynchronized method: wait(10) throws IllegalMonitorStateException!
    {
        let mut emu = J2meEmulator::new(240, 320, 44100);
        emu.is_running = true;

        // Predecode unsynchronized method TestSync.unsyncMethod:()V
        let unsync_method = PredecodedMethod {
            class_name: "TestSync".to_string(),
            name: "unsyncMethod".to_string(),
            descriptor: "()V".to_string(),
            access_flags: ACC_PUBLIC, // NOT SYNCHRONIZED
            max_locals: 2,
            max_stack: 4,
            instructions: Arc::new(vec![
                Instruction::ALoad(0),
                Instruction::LdcLong(10),
                Instruction::InvokeVirtual {
                    class_name: "java/lang/Object".to_string(),
                    method_name: "wait".to_string(),
                    descriptor: "(J)V".to_string(),
                },
                Instruction::Return,
            ]),
            pc_map: Arc::new(HashMap::new()),
            exception_table: Vec::new(),
        };
        emu.class_loader.predecoded_methods.insert(
            (
                "TestSync".to_string(),
                "unsyncMethod".to_string(),
                "()V".to_string(),
            ),
            Arc::new(unsync_method),
        );

        let target_obj = emu.heap.alloc_instance("TestSync");

        let mut caller_pc_map = HashMap::new();
        caller_pc_map.insert(0, 0);
        caller_pc_map.insert(1, 1);
        let caller_frame = Frame::new(
            "Main".to_string(),
            "run".to_string(),
            "()V".to_string(),
            Arc::new(vec![
                Instruction::ALoad(0),
                Instruction::InvokeVirtual {
                    class_name: "TestSync".to_string(),
                    method_name: "unsyncMethod".to_string(),
                    descriptor: "()V".to_string(),
                },
                Instruction::Return,
            ]),
            2,
            4,
            Vec::new(),
            Arc::new(caller_pc_map),
        );

        emu.scheduler.spawn("TestThread", caller_frame, None);
        emu.scheduler.threads[0].call_stack[0].set_local(0, JavaValue::Object(Some(target_obj)));

        for _ in 0..5 {
            emu.tick(50.0);
        }

        // Must throw IllegalMonitorStateException
        assert!(
            emu.context.exception_telemetry.guest_exceptions_uncaught > 0,
            "Unsynchronized wait must throw IllegalMonitorStateException"
        );
        let last_exc = emu
            .context
            .exception_telemetry
            .last_uncaught
            .as_ref()
            .unwrap();
        assert_eq!(
            last_exc.exception_class,
            "java/lang/IllegalMonitorStateException"
        );
    }

    // 3. Test static synchronized method locks Class object
    {
        let mut emu = J2meEmulator::new(240, 320, 44100);
        emu.is_running = true;

        let static_sync_method = PredecodedMethod {
            class_name: "TestSync".to_string(),
            name: "staticSync".to_string(),
            descriptor: "()V".to_string(),
            access_flags: ACC_PUBLIC | ACC_STATIC | ACC_SYNCHRONIZED,
            max_locals: 1,
            max_stack: 2,
            instructions: Arc::new(vec![Instruction::Return]),
            pc_map: Arc::new(HashMap::new()),
            exception_table: Vec::new(),
        };
        emu.class_loader.predecoded_methods.insert(
            (
                "TestSync".to_string(),
                "staticSync".to_string(),
                "()V".to_string(),
            ),
            Arc::new(static_sync_method),
        );

        let caller_frame = Frame::new(
            "Main".to_string(),
            "run".to_string(),
            "()V".to_string(),
            Arc::new(vec![
                Instruction::InvokeStatic {
                    class_name: "TestSync".to_string(),
                    method_name: "staticSync".to_string(),
                    descriptor: "()V".to_string(),
                },
                Instruction::Return,
            ]),
            1,
            2,
            Vec::new(),
            Arc::new(HashMap::new()),
        );

        emu.scheduler.spawn("StaticTest", caller_frame, None);
        for _ in 0..3 {
            emu.tick(50.0);
        }

        let cls_obj = emu
            .class_loader
            .get_or_create_class_object(&mut emu.heap, "TestSync");
        assert_eq!(
            emu.scheduler.get_monitor_owner(cls_obj),
            None,
            "Static synchronized method must release Class monitor on exit"
        );
    }

    // 4. Test exception unwinding releases monitor
    {
        let mut emu = J2meEmulator::new(240, 320, 44100);
        emu.is_running = true;

        let sync_throw_method = PredecodedMethod {
            class_name: "TestSync".to_string(),
            name: "syncThrow".to_string(),
            descriptor: "()V".to_string(),
            access_flags: ACC_PUBLIC | ACC_SYNCHRONIZED,
            max_locals: 2,
            max_stack: 4,
            instructions: Arc::new(vec![
                Instruction::AConstNull,
                Instruction::AThrow, // throws NPE
            ]),
            pc_map: Arc::new(HashMap::new()),
            exception_table: Vec::new(),
        };
        emu.class_loader.predecoded_methods.insert(
            (
                "TestSync".to_string(),
                "syncThrow".to_string(),
                "()V".to_string(),
            ),
            Arc::new(sync_throw_method),
        );

        let target_obj = emu.heap.alloc_instance("TestSync");

        let mut pc_map = HashMap::new();
        pc_map.insert(0, 0);
        pc_map.insert(1, 1);
        pc_map.insert(2, 2);
        pc_map.insert(3, 3);
        let caller_frame = Frame::new(
            "Main".to_string(),
            "run".to_string(),
            "()V".to_string(),
            Arc::new(vec![
                Instruction::ALoad(0),
                Instruction::InvokeVirtual {
                    class_name: "TestSync".to_string(),
                    method_name: "syncThrow".to_string(),
                    descriptor: "()V".to_string(),
                },
                Instruction::Pop, // handler at pc=2
                Instruction::Return,
            ]),
            2,
            4,
            vec![ExceptionTableEntry {
                start_pc: 0,
                end_pc: 2,
                handler_pc: 2,
                catch_type: 0,
                catch_class: None,
            }],
            Arc::new(pc_map),
        );

        emu.scheduler.spawn("UnwindTest", caller_frame, None);
        emu.scheduler.threads[0].call_stack[0].set_local(0, JavaValue::Object(Some(target_obj)));

        for _ in 0..5 {
            emu.tick(50.0);
        }

        // Exception was caught by caller, syncThrow frame was unwound -> monitor must be released!
        assert_eq!(
            emu.scheduler.get_monitor_owner(target_obj),
            None,
            "Exception unwinding must release monitor of synchronized method"
        );
        assert_eq!(emu.context.exception_telemetry.guest_exceptions_caught, 1);
    }
}

#[test]
fn test_phase49_integer_hex_and_gauge_contracts() {
    use j2me_core::heap::value::JavaValue;
    use j2me_stdlib::bridge::NativeBridge;
    use j2me_stdlib::J2meEmulator;

    let mut emu = J2meEmulator::new(240, 320, 44100);

    // 1. Integer.toHexString
    let hex_res = NativeBridge::dispatch(
        "java/lang/Integer",
        "toHexString",
        "(I)Ljava/lang/String;",
        None,
        &[JavaValue::Int(255)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .expect("Integer.toHexString must succeed");
    let hex_id = hex_res.unwrap().as_object().unwrap().unwrap();
    assert_eq!(emu.heap.get_string_utf8(hex_id).unwrap(), "ff");

    // Negative int to hex (unsigned 32-bit: -1 -> ffffffff)
    let neg_res = NativeBridge::dispatch(
        "java/lang/Integer",
        "toHexString",
        "(I)Ljava/lang/String;",
        None,
        &[JavaValue::Int(-1)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .expect("Integer.toHexString(-1) must succeed");
    let neg_id = neg_res.unwrap().as_object().unwrap().unwrap();
    assert_eq!(emu.heap.get_string_utf8(neg_id).unwrap(), "ffffffff");

    // 2. Gauge contracts
    let gauge_id = emu.heap.alloc_instance("javax/microedition/lcdui/Gauge");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/Gauge",
        "<init>",
        "(Ljava/lang/String;ZII)V",
        Some(gauge_id),
        &[
            JavaValue::Object(None),
            JavaValue::Int(0),
            JavaValue::Int(50),
            JavaValue::Int(10),
        ],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .expect("Gauge.<init> must succeed");

    let max_v = NativeBridge::dispatch(
        "javax/microedition/lcdui/Gauge",
        "getMaxValue",
        "()I",
        Some(gauge_id),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .expect("Gauge.getMaxValue must succeed");
    assert_eq!(max_v.unwrap().as_int().unwrap(), 50);

    let cur_v = NativeBridge::dispatch(
        "javax/microedition/lcdui/Gauge",
        "getValue",
        "()I",
        Some(gauge_id),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .expect("Gauge.getValue must succeed");
    assert_eq!(cur_v.unwrap().as_int().unwrap(), 10);
}

#[test]
fn test_phase491_stack_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let stack_id = heap.alloc_instance("java/util/Stack");
    NativeBridge::dispatch(
        "java/util/Stack",
        "<init>",
        "()V",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .expect("Stack.<init> must succeed");

    // Initially empty
    let is_empty = NativeBridge::dispatch(
        "java/util/Stack",
        "empty",
        "()Z",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(is_empty, 1);

    // Push item 1
    let item1 = heap.alloc_string("Hello");
    let push_ret = NativeBridge::dispatch(
        "java/util/Stack",
        "push",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        Some(stack_id),
        &[JavaValue::Object(Some(item1))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert_eq!(push_ret, Some(item1));

    // Push item 2
    let item2 = heap.alloc_string("World");
    NativeBridge::dispatch(
        "java/util/Stack",
        "push",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        Some(stack_id),
        &[JavaValue::Object(Some(item2))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Peek should return item2
    let peek_ret = NativeBridge::dispatch(
        "java/util/Stack",
        "peek",
        "()Ljava/lang/Object;",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert_eq!(peek_ret, Some(item2));

    // Pop item 2
    let pop1 = NativeBridge::dispatch(
        "java/util/Stack",
        "pop",
        "()Ljava/lang/Object;",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert_eq!(pop1, Some(item2));

    // Pop item 1
    let pop2 = NativeBridge::dispatch(
        "java/util/Stack",
        "pop",
        "()Ljava/lang/Object;",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert_eq!(pop2, Some(item1));

    // Pop empty should throw EmptyStackException
    let pop_empty = NativeBridge::dispatch(
        "java/util/Stack",
        "pop",
        "()Ljava/lang/Object;",
        Some(stack_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(
        pop_empty.is_err(),
        "Popping empty stack must throw exception"
    );
}

#[test]
fn test_phase491_input_stream_reader_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Allocate ByteArrayInputStream with "ABCDEF"
    let raw_bytes: Vec<i8> = vec![
        b'A' as i8, b'B' as i8, b'C' as i8, b'D' as i8, b'E' as i8, b'F' as i8,
    ];
    let arr_id = heap.alloc_array(ArrayData::Byte(raw_bytes));
    let bais_id = heap.alloc_instance("java/io/ByteArrayInputStream");
    heap.set_field(bais_id, "buf", JavaValue::Object(Some(arr_id)));
    heap.set_field(bais_id, "count", JavaValue::Int(6));
    heap.set_field(bais_id, "pos", JavaValue::Int(0));

    // Allocate InputStreamReader
    let reader_id = heap.alloc_instance("java/io/InputStreamReader");
    NativeBridge::dispatch(
        "java/io/InputStreamReader",
        "<init>",
        "(Ljava/io/InputStream;)V",
        Some(reader_id),
        &[JavaValue::Object(Some(bais_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .expect("InputStreamReader.<init> must succeed");

    // Single char read: 'A' (65)
    let c1 = NativeBridge::dispatch(
        "java/io/InputStreamReader",
        "read",
        "()I",
        Some(reader_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(c1, b'A' as i32);

    // Read into char buffer of length 4: should read 'B', 'C', 'D' with max_len 3
    let cbuf = heap.alloc_array(ArrayData::Char(vec![0; 4]));
    let n_read = NativeBridge::dispatch(
        "java/io/InputStreamReader",
        "read",
        "([CII)I",
        Some(reader_id),
        &[
            JavaValue::Object(Some(cbuf)),
            JavaValue::Int(1),
            JavaValue::Int(3),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(n_read, 3);

    if let Some(ObjectData::Array(ArrayData::Char(chars))) = heap.get(cbuf) {
        assert_eq!(chars[1], b'B' as u16);
        assert_eq!(chars[2], b'C' as u16);
        assert_eq!(chars[3], b'D' as u16);
    } else {
        panic!("Char array missing");
    }
}

#[test]
fn test_phase491_m3g_array_return_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let str_id = heap.alloc_string("/model.m3g");

    // Call Loader.load(Ljava/lang/String;)[Ljavax/microedition/m3g/Object3D;
    let res = NativeBridge::dispatch(
        "javax/microedition/m3g/Loader",
        "load",
        "(Ljava/lang/String;)[Ljavax/microedition/m3g/Object3D;",
        None,
        &[JavaValue::Object(Some(str_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .expect("Loader.load must succeed")
    .expect("Must return array");

    let arr_id = res
        .as_object()
        .unwrap()
        .expect("Array reference must not be null");
    if let Some(ObjectData::Array(ArrayData::Reference(elements))) = heap.get(arr_id) {
        assert!(
            !elements.is_empty(),
            "M3G Loader.load must return non-empty array of roots"
        );
        let root_id = elements[0].expect("Element 0 of roots array must not be null");
        if let Some(ObjectData::Instance(inst)) = heap.get(root_id) {
            assert_eq!(
                inst.class_name, "javax/microedition/m3g/World",
                "Root 0 must be a World instance"
            );
            assert!(cl.is_instance_of(&inst.class_name, "javax/microedition/m3g/World"));
            assert!(cl.is_instance_of(&inst.class_name, "javax/microedition/m3g/Group"));
            assert!(cl.is_instance_of(&inst.class_name, "javax/microedition/m3g/Node"));
            assert!(cl.is_instance_of(&inst.class_name, "javax/microedition/m3g/Transformable"));
            assert!(cl.is_instance_of(&inst.class_name, "javax/microedition/m3g/Object3D"));
        } else {
            panic!("Root 0 must be an instance");
        }
    } else {
        panic!("Expected ArrayData::Reference for M3G Loader.load");
    }
}

#[test]
fn test_phase50_record_store_delete_and_size_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let store_name = "Phase50RS";
    let name_id = heap.alloc_string(store_name);
    let store_ref = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "openRecordStore",
        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
        None,
        &[JavaValue::Object(Some(name_id)), JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    let rec_data = heap.alloc_array(ArrayData::Byte(vec![1, 2, 3, 4]));
    let rec_id = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "addRecord",
        "([BII)I",
        store_ref,
        &[
            JavaValue::Object(Some(rec_data)),
            JavaValue::Int(0),
            JavaValue::Int(4),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(rec_id, 1);

    // Test getSizeAvailable()I
    let size_avail = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "getSizeAvailable",
        "()I",
        store_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert!(
        size_avail >= 1024 * 1024,
        "Available size should be at least 1MB"
    );

    // Test getSize()I
    let size = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "getSize",
        "()I",
        store_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert!(size >= 4, "Store size must account for record bytes");

    // Test deleteRecord(I)V
    let del_res = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "deleteRecord",
        "(I)V",
        store_ref,
        &[JavaValue::Int(rec_id)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(del_res.is_ok(), "deleteRecord should succeed");
}

#[test]
fn test_phase50_string_intern_and_region_matches_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let str_id = heap.alloc_string("HeroesLore");

    // Test String.intern() returns the string reference
    let interned = NativeBridge::dispatch(
        "java/lang/String",
        "intern",
        "()Ljava/lang/String;",
        Some(str_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert_eq!(interned, Some(str_id));

    // Test String.regionMatches(ZILjava/lang/String;II)Z
    let other_id = heap.alloc_string("lore_online");
    let match_res = NativeBridge::dispatch(
        "java/lang/String",
        "regionMatches",
        "(ZILjava/lang/String;II)Z",
        Some(str_id),
        &[
            JavaValue::Int(1), // ignoreCase = true
            JavaValue::Int(6), // toffset = 6 ("Lore")
            JavaValue::Object(Some(other_id)),
            JavaValue::Int(0), // ooffset = 0 ("lore")
            JavaValue::Int(4), // len = 4
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(
        match_res, 1,
        "regionMatches ignoreCase should match 'Lore' with 'lore'"
    );
}

#[test]
fn test_phase50_string_buffer_set_char_at_contract() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let sb_id = heap.alloc_instance("java/lang/StringBuffer");
    let init_str = heap.alloc_string("DoomRPG");
    heap.set_field(sb_id, "str", JavaValue::Object(Some(init_str)));

    // setCharAt(4, 'X') -> "DoomXPG"
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "setCharAt",
        "(IC)V",
        Some(sb_id),
        &[JavaValue::Int(4), JavaValue::Int('X' as i32)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let res_str = heap
        .get_field(sb_id, "str")
        .as_object()
        .ok()
        .flatten()
        .and_then(|id| heap.get_string_utf8(id))
        .unwrap();
    assert_eq!(res_str, "DoomXPG");
}

#[test]
fn test_phase50_jad_and_property_precedence_contract() {
    use j2me_core::jar::jad::Jad;
    use j2me_core::jar::manifest::Manifest;

    let jad_str = "MIDlet-Name: Heroes Lore\nHO-Demo: PAID\nCustom-Key: jad_val\n";
    let manifest_str =
        "MIDlet-Name: Heroes Lore Uncut\nCustom-Key: manifest_val\nOnly-In-Manifest: true\n";

    let jad = Jad::parse(jad_str);
    let manifest = Manifest::parse(manifest_str);

    // 1. JAD overrides Manifest
    assert_eq!(
        Jad::resolve_property(Some(&jad), Some(&manifest), "Custom-Key"),
        Some("jad_val")
    );
    assert_eq!(
        Jad::resolve_property(Some(&jad), Some(&manifest), "MIDlet-Name"),
        Some("Heroes Lore")
    );

    // 2. Manifest fallback when absent in JAD
    assert_eq!(
        Jad::resolve_property(Some(&jad), Some(&manifest), "Only-In-Manifest"),
        Some("true")
    );

    // 3. Case insensitivity and trimming
    assert_eq!(
        Jad::resolve_property(Some(&jad), Some(&manifest), "  ho-demo  "),
        Some("PAID")
    );
}

#[test]
fn test_phase51_asymmetric_4x4_sprite_transforms_contract() {
    use j2me_stdlib::midp::lcdui::graphics::Graphics;
    use j2me_stdlib::midp::lcdui::image::Image;
    use j2me_stdlib::nokia::direct_graphics::*;

    const RED: u32 = 0xFFFF0000;
    const GREEN: u32 = 0xFF00FF00;
    const BLUE: u32 = 0xFF0000FF;
    const WHITE: u32 = 0xFFFFFFFF;
    const TRANS: u32 = 0x00000000;

    // Asymmetric 4x4 image:
    // (0,0) = RED,   (3,0) = GREEN
    // (0,3) = BLUE,  (3,3) = WHITE
    let mut pixels_4x4 = vec![TRANS; 16];
    pixels_4x4[0 * 4 + 0] = RED;
    pixels_4x4[0 * 4 + 3] = GREEN;
    pixels_4x4[3 * 4 + 0] = BLUE;
    pixels_4x4[3 * 4 + 3] = WHITE;

    let img = Image::new_immutable(4, 4, pixels_4x4);

    let render_transform = |transform: i32| -> Vec<u32> {
        let mut dest = vec![TRANS; 16];
        let mut g = Graphics::new(4, 4);
        g.set_clip(0, 0, 4, 4);
        g.draw_region(&mut dest, &img, 0, 0, 4, 4, transform, 0, 0, 0);
        dest
    };

    // 1. TRANS_NONE (0): (0,0)=RED, (3,0)=GREEN, (0,3)=BLUE, (3,3)=WHITE
    let t0 = render_transform(0);
    assert_eq!(t0[0 * 4 + 0], RED);
    assert_eq!(t0[0 * 4 + 3], GREEN);
    assert_eq!(t0[3 * 4 + 0], BLUE);
    assert_eq!(t0[3 * 4 + 3], WHITE);

    // 2. TRANS_MIRROR_ROT180 (1) = FLIP_Y: (0,0)=BLUE, (3,0)=WHITE, (0,3)=RED, (3,3)=GREEN
    let t1 = render_transform(1);
    assert_eq!(t1[0 * 4 + 0], BLUE, "FLIP_Y top-left must be BLUE");
    assert_eq!(t1[0 * 4 + 3], WHITE, "FLIP_Y top-right must be WHITE");
    assert_eq!(t1[3 * 4 + 0], RED, "FLIP_Y bottom-left must be RED");
    assert_eq!(t1[3 * 4 + 3], GREEN, "FLIP_Y bottom-right must be GREEN");

    // 3. TRANS_MIRROR (2) = FLIP_X: (0,0)=GREEN, (3,0)=RED, (0,3)=WHITE, (3,3)=BLUE
    let t2 = render_transform(2);
    assert_eq!(t2[0 * 4 + 0], GREEN, "FLIP_X top-left must be GREEN");
    assert_eq!(t2[0 * 4 + 3], RED, "FLIP_X top-right must be RED");
    assert_eq!(t2[3 * 4 + 0], WHITE, "FLIP_X bottom-left must be WHITE");
    assert_eq!(t2[3 * 4 + 3], BLUE, "FLIP_X bottom-right must be BLUE");

    // 4. TRANS_ROT180 (3) = FLIP_X + FLIP_Y: (0,0)=WHITE, (3,0)=BLUE, (0,3)=GREEN, (3,3)=RED
    let t3 = render_transform(3);
    assert_eq!(t3[0 * 4 + 0], WHITE);
    assert_eq!(t3[0 * 4 + 3], BLUE);
    assert_eq!(t3[3 * 4 + 0], GREEN);
    assert_eq!(t3[3 * 4 + 3], RED);

    // 5. TRANS_MIRROR_ROT270 (4) = Transpose: (row, col)
    let t4 = render_transform(4);
    assert_eq!(t4[0 * 4 + 0], RED);
    assert_eq!(t4[0 * 4 + 3], BLUE);
    assert_eq!(t4[3 * 4 + 0], GREEN);
    assert_eq!(t4[3 * 4 + 3], WHITE);

    // 6. TRANS_ROT90 (5) = 90 deg CW
    let t5 = render_transform(5);
    assert_eq!(t5[0 * 4 + 0], BLUE);
    assert_eq!(t5[0 * 4 + 3], RED);
    assert_eq!(t5[3 * 4 + 0], WHITE);
    assert_eq!(t5[3 * 4 + 3], GREEN);

    // 7. TRANS_ROT270 (6) = 270 deg CW
    let t6 = render_transform(6);
    assert_eq!(t6[0 * 4 + 0], GREEN);
    assert_eq!(t6[0 * 4 + 3], WHITE);
    assert_eq!(t6[3 * 4 + 0], RED);
    assert_eq!(t6[3 * 4 + 3], BLUE);

    // 8. TRANS_MIRROR_ROT90 (7) = Anti-transpose
    let t7 = render_transform(7);
    assert_eq!(t7[0 * 4 + 0], WHITE);
    assert_eq!(t7[0 * 4 + 3], GREEN);
    assert_eq!(t7[3 * 4 + 0], BLUE);
    assert_eq!(t7[3 * 4 + 3], RED);

    // --- DirectGraphics manipulation tests ---
    // FLIP_HORIZONTAL: (0,0) -> (3,0)
    let (c1, r1, _, _) = transform_coords(0, 0, 4, 4, FLIP_HORIZONTAL);
    assert_eq!((c1, r1), (3, 0));

    // FLIP_VERTICAL: (0,0) -> (0,3)
    let (c2, r2, _, _) = transform_coords(0, 0, 4, 4, FLIP_VERTICAL);
    assert_eq!((c2, r2), (0, 3));

    // ROTATE_180: (0,0) -> (3,3)
    let (c3, r3, _, _) = transform_coords(0, 0, 4, 4, ROTATE_180);
    assert_eq!((c3, r3), (3, 3));

    // FLIP_VERTICAL | ROTATE_180: (0,0) -> (3,0) (horizontal mirror)
    let (c4, r4, _, _) = transform_coords(0, 0, 4, 4, FLIP_VERTICAL | ROTATE_180);
    assert_eq!((c4, r4), (3, 0));

    // FLIP_HORIZONTAL | ROTATE_180: (0,0) -> (0,3) (vertical mirror)
    let (c5, r5, _, _) = transform_coords(0, 0, 4, 4, FLIP_HORIZONTAL | ROTATE_180);
    assert_eq!((c5, r5), (0, 3));
}

#[test]
fn test_cyberpunk_audio_and_menu_trace() {
    let jar_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("web/cyberpunk.jar");

    if !jar_path.exists() {
        println!("cyberpunk.jar not found, skipping");
        return;
    }

    let jar_bytes = std::fs::read(&jar_path).expect("read jar");
    let mut emu = j2me_stdlib::emulator::J2meEmulator::new(176, 208, 44100);
    emu.load_jar(&jar_bytes).expect("load cyberpunk jar");

    let mut total_samples = 0usize;
    let mut audio_buf = vec![0.0f32; 1024];

    for frame in 0..2200 {
        // Press FIRE / KEY_NUM5 to dismiss title and enter menu
        if frame % 30 == 0 {
            emu.key_event(-5, true);
        }
        if frame % 30 == 5 {
            emu.key_event(-5, false);
        }

        emu.tick(16.6);
        let read = emu.read_audio_samples(&mut audio_buf);
        total_samples += read;
    }

    println!(
        "Cyberpunk trace: players count = {}, active players = {}, total_samples = {}, inst_count = {}, threads: {:?}",
        emu.context.players.len(),
        emu.context
            .players
            .values()
            .filter(|p| p.state == j2me_stdlib::media::manager::STATE_STARTED)
            .count(),
        total_samples,
        emu.scheduler
            .threads
            .iter()
            .map(|t| t.instructions_executed)
            .sum::<u64>(),
        emu.scheduler
            .threads
            .iter()
            .map(|t| (&t.name, format!("{:?}", t.state)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_phase51_cyberpunk_datainputstream_midi_audio_playback() {
    let jar_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("web/cyberpunk.jar");

    if !jar_path.exists() {
        println!("cyberpunk.jar not found, skipping");
        return;
    }

    let jar_bytes = std::fs::read(&jar_path).expect("read jar");
    let jar = j2me_core::jar::JarArchive::parse(&jar_bytes).expect("parse jar");
    let midi_bytes = jar.get_entry("menu.mid").expect("menu.mid entry").to_vec();
    assert!(midi_bytes.starts_with(b"MThd"), "valid MIDI file header");

    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(5000);
    let mut context = VmContext::new(176, 208, 44100);

    // 1. Create ByteArrayInputStream
    let byte_arr = heap.alloc_array(ArrayData::Byte(
        midi_bytes.iter().map(|&b| b as i8).collect(),
    ));
    let bais = heap.alloc_instance("java/io/ByteArrayInputStream");
    heap.set_field(bais, "buf", JavaValue::Object(Some(byte_arr)));
    heap.set_field(bais, "count", JavaValue::Int(midi_bytes.len() as i32));
    heap.set_field(bais, "pos", JavaValue::Int(0));

    // 2. Wrap in DataInputStream (like CyberPunk h.a(String) does)
    let dis = heap.alloc_instance("java/io/DataInputStream");
    NativeBridge::dispatch(
        "java/io/DataInputStream",
        "<init>",
        "(Ljava/io/InputStream;)V",
        Some(dis),
        &[JavaValue::Object(Some(bais))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // 3. Call Manager.createPlayer(InputStream, "audio/midi")
    let mime_id = heap.alloc_string("audio/midi");
    let player_val = NativeBridge::dispatch(
        "javax/microedition/media/Manager",
        "createPlayer",
        "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
        None,
        &[JavaValue::Object(Some(dis)), JavaValue::Object(Some(mime_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();

    let player_id = player_val.as_object().unwrap().unwrap();
    let player = context.players.get_mut(&player_id).expect("player registered");
    assert!(player.midi_synth.is_some(), "midi_synth must be successfully initialized from DataInputStream");

    // 4. Start Player
    NativeBridge::dispatch(
        "javax/microedition/media/Player",
        "start",
        "()V",
        Some(player_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // 5. Render audio samples into ring buffer
    let player = context.players.get_mut(&player_id).unwrap();
    player.render_audio_into_buffer(1024, &mut context.audio);

    let mut buf = vec![0.0f32; 1024];
    let count = context.audio.read_samples(&mut buf);
    assert!(count > 0, "audio samples must be generated from MIDI");
    let peak = buf[..count].iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    assert!(peak > 0.0, "synthesizer must produce audible sound waves, peak = {peak}");
    println!("Phase 5.1 MIDI test passed: rendered {} samples with peak amplitude {:.4}", count, peak);
}

#[test]
fn test_phase51_layer_manager_reverse_z_order_and_view_window() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(64, 64, 44100);

    // 1. Create two 8x8 images: red (bg) and green (fg)
    let red_img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
    let mut red_img = Image::new_mutable(8, 8);
    red_img.pixels.fill(0xFFFF0000); // Red
    context.images.insert(red_img_id, red_img);

    let green_img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
    let mut green_img = Image::new_mutable(8, 8);
    green_img.pixels.fill(0xFF00FF00); // Green
    context.images.insert(green_img_id, green_img);

    // 2. Create Sprite 0 (red) and Sprite 1 (green)
    let sprite0 = heap.alloc_instance("javax/microedition/lcdui/game/Sprite");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "<init>",
        "(Ljavax/microedition/lcdui/Image;)V",
        Some(sprite0),
        &[JavaValue::Object(Some(red_img_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let sprite1 = heap.alloc_instance("javax/microedition/lcdui/game/Sprite");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "<init>",
        "(Ljavax/microedition/lcdui/Image;)V",
        Some(sprite1),
        &[JavaValue::Object(Some(green_img_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // 3. Create LayerManager
    let lm_id = heap.alloc_instance("javax/microedition/lcdui/game/LayerManager");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "<init>",
        "()V",
        Some(lm_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // In MIDP: index 0 is topmost, index size-1 is bottommost.
    // In our LayerManager: paint renders in reverse order (size - 1 down to 0),
    // so index 0 renders LAST (on top of index 1)!
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "append",
        "(Ljavax/microedition/lcdui/game/Layer;)V",
        Some(lm_id),
        &[JavaValue::Object(Some(sprite0))], // index 0 (topmost)
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "append",
        "(Ljavax/microedition/lcdui/game/Layer;)V",
        Some(lm_id),
        &[JavaValue::Object(Some(sprite1))], // index 1 (bottommost)
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let sz = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "getSize",
        "()I",
        Some(lm_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(sz, 2);

    // Set view window
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "setViewWindow",
        "(IIII)V",
        Some(lm_id),
        &[
            JavaValue::Int(0),
            JavaValue::Int(0),
            JavaValue::Int(8),
            JavaValue::Int(8),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Create Graphics and paint
    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
    context.get_or_create_graphics(g_id, GraphicsTarget::Display);

    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "paint",
        "(Ljavax/microedition/lcdui/Graphics;II)V",
        Some(lm_id),
        &[
            JavaValue::Object(Some(g_id)),
            JavaValue::Int(0),
            JavaValue::Int(0),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Pixel at (0, 0) should be Red (sprite0) because index 0 renders last (topmost) over index 1!
    let top_left = context.framebuffer[0];
    assert_eq!(top_left, 0xFFFF0000, "LayerManager: layer at index 0 must render on top in reverse Z-order");

    // Test remove
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "remove",
        "(Ljavax/microedition/lcdui/game/Layer;)V",
        Some(lm_id),
        &[JavaValue::Object(Some(sprite0))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let sz_after = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/LayerManager",
        "getSize",
        "()I",
        Some(lm_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(sz_after, 1);
}

#[test]
fn test_phase51_lcdui_list_and_command_action() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let title_id = heap.alloc_string("Main Menu");
    let list_id = heap.alloc_instance("javax/microedition/lcdui/List");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/List",
        "<init>",
        "(Ljava/lang/String;I)V",
        Some(list_id),
        &[JavaValue::Object(Some(title_id)), JavaValue::Int(3)], // IMPLICIT
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Append 3 items
    for item in &["Start Game", "Options", "Exit"] {
        let sid = heap.alloc_string(item);
        NativeBridge::dispatch(
            "javax/microedition/lcdui/List",
            "append",
            "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
            Some(list_id),
            &[JavaValue::Object(Some(sid)), JavaValue::Object(None)],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        )
        .unwrap();
    }

    let sz = NativeBridge::dispatch(
        "javax/microedition/lcdui/List",
        "size",
        "()I",
        Some(list_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(sz, 3);

    // Get string of item 1
    let item1_str_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/List",
        "getString",
        "(I)Ljava/lang/String;",
        Some(list_id),
        &[JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    let item1_str = heap.get_string_utf8(item1_str_val.as_object().unwrap().unwrap()).unwrap();
    assert_eq!(item1_str, "Options");

    // Set selected index
    NativeBridge::dispatch(
        "javax/microedition/lcdui/List",
        "setSelectedIndex",
        "(IZ)V",
        Some(list_id),
        &[JavaValue::Int(2), JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let sel = NativeBridge::dispatch(
        "javax/microedition/lcdui/List",
        "getSelectedIndex",
        "()I",
        Some(list_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(sel, 2);
}

#[test]
fn test_phase51_nokia_direct_graphics_draw_pixels_byte_formats() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(16, 16, 44100);

    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
    context.get_or_create_graphics(g_id, GraphicsTarget::Display);

    let dg_val = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectUtils",
        "getDirectGraphics",
        "(Ljavax/microedition/lcdui/Graphics;)Lcom/nokia/mid/ui/DirectGraphics;",
        None,
        &[JavaValue::Object(Some(g_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    let dg_id = dg_val.as_object().unwrap().unwrap();

    // 1. TYPE_BYTE_1_GRAY: 8x8 image = 8 bytes.
    // Row 0: 0b10101010 = 0xAA.
    let mut pixels_1gray = vec![0i8; 8];
    pixels_1gray[0] = 0xAA_u8 as i8; // 1, 0, 1, 0, 1, 0, 1, 0
    let pix_arr = heap.alloc_array(ArrayData::Byte(pixels_1gray.clone()));

    NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([B[BIIIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(pix_arr)),
            JavaValue::Object(None), // no mask
            JavaValue::Int(0),       // offset
            JavaValue::Int(8),       // scanlength (8 pixels)
            JavaValue::Int(0),       // x
            JavaValue::Int(0),       // y
            JavaValue::Int(8),       // width
            JavaValue::Int(8),       // height
            JavaValue::Int(0),       // manipulation
            JavaValue::Int(DirectGraphics::TYPE_BYTE_1_GRAY),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Pixel (0, 0) should be white (bit 7 = 1 -> 255), pixel (1, 0) should be black (bit 6 = 0 -> 0)
    assert_eq!(context.framebuffer[0], 0xFFFFFFFF);
    assert_eq!(context.framebuffer[1], 0xFF000000);

    // 2. Test transparency mask with 1_GRAY:
    // Mask with bit 7 = 0 (transparent) and bit 6 = 1 (opaque)
    let mut mask_bytes = vec![0i8; 8];
    mask_bytes[0] = 0x40_u8 as i8; // 0, 1, 0, 0, 0, 0, 0, 0
    let mask_arr = heap.alloc_array(ArrayData::Byte(mask_bytes));

    // Reset framebuffer to red
    context.framebuffer.fill(0xFFFF0000);

    NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([B[BIIIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(pix_arr)),
            JavaValue::Object(Some(mask_arr)), // with mask
            JavaValue::Int(0),
            JavaValue::Int(8),
            JavaValue::Int(0),
            JavaValue::Int(0),
            JavaValue::Int(8),
            JavaValue::Int(8),
            JavaValue::Int(0),
            JavaValue::Int(DirectGraphics::TYPE_BYTE_1_GRAY),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Pixel (0, 0): masked out -> remains red (0xFFFF0000)
    // Pixel (1, 0): mask bit is 1 -> drawn as black (0xFF000000)
    assert_eq!(context.framebuffer[0], 0xFFFF0000, "Pixel (0,0) must remain background because mask bit is 0");
    assert_eq!(context.framebuffer[1], 0xFF000000, "Pixel (1,0) must be black because mask bit is 1");
}

#[test]
fn test_sprite_ref_pixel_and_sequence() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // 1. Create Image 32x32 (4 frames of 16x16)
    let raw_pixels = vec![0xFFFFFFFFu32; 32 * 32];
    let img = Image::new_immutable(32, 32, raw_pixels);
    let img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
    context.images.insert(img_id, img);

    // 2. Create Sprite
    let spr_id = heap.alloc_instance("javax/microedition/lcdui/game/Sprite");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "<init>",
        "(Ljavax/microedition/lcdui/Image;II)V",
        Some(spr_id),
        &[JavaValue::Object(Some(img_id)), JavaValue::Int(16), JavaValue::Int(16)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // 3. Define reference pixel (4, 8)
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "defineReferencePixel",
        "(II)V",
        Some(spr_id),
        &[JavaValue::Int(4), JavaValue::Int(8)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // TRANS_NONE (0): dx = 4, dy = 8
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/Sprite",
        "setRefPixelPosition",
        "(II)V",
        Some(spr_id),
        &[JavaValue::Int(100), JavaValue::Int(100)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let x = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getX", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    let y = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getY", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(x, 96, "TRANS_NONE: x = 100 - 4 = 96");
    assert_eq!(y, 92, "TRANS_NONE: y = 100 - 8 = 92");

    let rx = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getRefPixelX", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    let ry = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getRefPixelY", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(rx, 100);
    assert_eq!(ry, 100);

    // TRANS_MIRROR (2): dx = 16 - 1 - 4 = 11, dy = 8
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setTransform", "(I)V", Some(spr_id), &[JavaValue::Int(2)], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setRefPixelPosition", "(II)V", Some(spr_id), &[JavaValue::Int(100), JavaValue::Int(100)], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    let x_m = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getX", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    let y_m = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getY", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(x_m, 89, "TRANS_MIRROR: x = 100 - 11 = 89");
    assert_eq!(y_m, 92, "TRANS_MIRROR: y = 100 - 8 = 92");

    // TRANS_ROT90 (5): dx = 16 - 1 - 8 = 7, dy = 4
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setTransform", "(I)V", Some(spr_id), &[JavaValue::Int(5)], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setRefPixelPosition", "(II)V", Some(spr_id), &[JavaValue::Int(100), JavaValue::Int(100)], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    let x_r = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getX", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    let y_r = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getY", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(x_r, 93, "TRANS_ROT90: x = 100 - 7 = 93");
    assert_eq!(y_r, 96, "TRANS_ROT90: y = 100 - 4 = 96");

    // 4. Test frame sequence [2, 0, 1]
    let seq_arr = heap.alloc_array(ArrayData::Int(vec![2, 0, 1]));
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setFrameSequence", "([I)V", Some(spr_id), &[JavaValue::Object(Some(seq_arr))], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();

    let seq_len = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getFrameSequenceLength", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(seq_len, 3);

    let f0 = NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getFrame", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap();
    assert_eq!(f0, 0, "Initial sequence index must be 0");
    assert_eq!(heap.get_field(spr_id, "frame").as_int().unwrap(), 2, "Real frame must be 2");

    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "nextFrame", "()V", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    assert_eq!(NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getFrame", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap(), 1);
    assert_eq!(heap.get_field(spr_id, "frame").as_int().unwrap(), 0);

    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "nextFrame", "()V", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    assert_eq!(NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getFrame", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap(), 2);
    assert_eq!(heap.get_field(spr_id, "frame").as_int().unwrap(), 1);

    // Re-setting sequence resets sequence index to 0
    NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "setFrameSequence", "([I)V", Some(spr_id), &[JavaValue::Object(Some(seq_arr))], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap();
    assert_eq!(NativeBridge::dispatch("javax/microedition/lcdui/game/Sprite", "getFrame", "()I", Some(spr_id), &[], &mut heap, &mut cl, &mut scheduler, &mut context, 0).unwrap().unwrap().as_int().unwrap(), 0);
}

#[test]
fn test_tiled_layer_grid_and_animated_tiles() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Create 32x32 image with 4 distinct tiles (Tile 1: Red, Tile 2: Green, Tile 3: Blue, Tile 4: Yellow)
    let mut pixels = vec![0u32; 32 * 32];
    for y in 0..32 {
        for x in 0..32 {
            let color = match (x < 16, y < 16) {
                (true, true) => 0xFFFF0000,   // Tile 1 (top-left): Red
                (false, true) => 0xFF00FF00,  // Tile 2 (top-right): Green
                (true, false) => 0xFF0000FF,  // Tile 3 (bottom-left): Blue
                (false, false) => 0xFFFFFF00, // Tile 4 (bottom-right): Yellow
            };
            pixels[y * 32 + x] = color;
        }
    }
    let img = Image::new_immutable(32, 32, pixels);
    let img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
    context.images.insert(img_id, img);

    // Create TiledLayer(4, 4, img, 16, 16)
    let tl_id = heap.alloc_instance("javax/microedition/lcdui/game/TiledLayer");
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "<init>",
        "(IILjavax/microedition/lcdui/Image;II)V",
        Some(tl_id),
        &[
            JavaValue::Int(4),
            JavaValue::Int(4),
            JavaValue::Object(Some(img_id)),
            JavaValue::Int(16),
            JavaValue::Int(16),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // setCell(1, 2, 3) and verify getCell
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "setCell",
        "(III)V",
        Some(tl_id),
        &[JavaValue::Int(1), JavaValue::Int(2), JavaValue::Int(3)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let cell_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "getCell",
        "(II)I",
        Some(tl_id),
        &[JavaValue::Int(1), JavaValue::Int(2)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(cell_val, 3);

    // Create animated tile initially bound to static tile 2 (Green)
    let anim_idx = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "createAnimatedTile",
        "(I)I",
        Some(tl_id),
        &[JavaValue::Int(2)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(anim_idx, -1);

    // Place animated tile at (0, 0)
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "setCell",
        "(III)V",
        Some(tl_id),
        &[JavaValue::Int(0), JavaValue::Int(0), JavaValue::Int(-1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Change animated tile -1 to point to static tile 4 (Yellow)
    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "setAnimatedTile",
        "(II)V",
        Some(tl_id),
        &[JavaValue::Int(-1), JavaValue::Int(4)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let anim_cur = NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "getAnimatedTile",
        "(I)I",
        Some(tl_id),
        &[JavaValue::Int(-1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(anim_cur, 4);

    // Paint onto Graphics display
    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
    context.get_or_create_graphics(g_id, GraphicsTarget::Display);

    NativeBridge::dispatch(
        "javax/microedition/lcdui/game/TiledLayer",
        "paint",
        "(Ljavax/microedition/lcdui/Graphics;)V",
        Some(tl_id),
        &[JavaValue::Object(Some(g_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Pixel at (0, 0) must be Yellow (0xFFFFFF00) from Tile 4
    assert_eq!(context.framebuffer[0], 0xFFFFFF00, "Animated tile must render tile 4 (Yellow)");
}

#[test]
fn test_string_buffer_get_chars() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Create StringBuffer("Hello World")
    let sb_id = heap.alloc_instance("java/lang/StringBuffer");
    let init_str = heap.alloc_string("Hello World");
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "<init>",
        "(Ljava/lang/String;)V",
        Some(sb_id),
        &[JavaValue::Object(Some(init_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Create char[10] array
    let dst_arr = heap.alloc_array(ArrayData::Char(vec![0u16; 10]));

    // getChars(6, 11, dst, 2) -> "World" into dst[2..7]
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "getChars",
        "(II[CI)V",
        Some(sb_id),
        &[
            JavaValue::Int(6),
            JavaValue::Int(11),
            JavaValue::Object(Some(dst_arr)),
            JavaValue::Int(2),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let chars = match heap.get(dst_arr).unwrap() {
        ObjectData::Array(ArrayData::Char(arr)) => arr.clone(),
        _ => panic!("Expected char array"),
    };
    let sub: String = chars[2..7].iter().filter_map(|&c| char::from_u32(c as u32)).collect();
    assert_eq!(sub, "World");

    // Test Out of Bounds
    let err = NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "getChars",
        "(II[CI)V",
        Some(sb_id),
        &[
            JavaValue::Int(6),
            JavaValue::Int(12), // beyond length 11
            JavaValue::Object(Some(dst_arr)),
            JavaValue::Int(2),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(err.is_err(), "Must throw StringIndexOutOfBoundsException");
}

#[test]
fn test_date_init_and_get_time() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    scheduler.current_time_ms = 1792941234567;
    let mut context = VmContext::new(240, 320, 44100);

    // Test Date.<init>()
    let d1 = heap.alloc_instance("java/util/Date");
    NativeBridge::dispatch(
        "java/util/Date",
        "<init>",
        "()V",
        Some(d1),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let t1 = NativeBridge::dispatch(
        "java/util/Date",
        "getTime",
        "()J",
        Some(d1),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_long()
    .unwrap();
    assert_eq!(t1, 1792941234567i64);

    // Test Date.<init>(1234567890L)
    let d2 = heap.alloc_instance("java/util/Date");
    NativeBridge::dispatch(
        "java/util/Date",
        "<init>",
        "(J)V",
        Some(d2),
        &[JavaValue::Long(1234567890i64)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let t2 = NativeBridge::dispatch(
        "java/util/Date",
        "getTime",
        "()J",
        Some(d2),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_long()
    .unwrap();
    assert_eq!(t2, 1234567890i64);
}

#[test]
fn test_player_get_controls_and_volume_control() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let player_id = heap.alloc_instance("javax/microedition/media/Player");
    let (_pid, player) = context.alloc_player("audio/midi", vec![1, 2, 3]);
    context.players.insert(player_id, player);

    // Call Player.getControls()
    let controls_arr_val = NativeBridge::dispatch(
        "javax/microedition/media/Player",
        "getControls",
        "()[Ljavax/microedition/media/Control;",
        Some(player_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();

    let arr_id = controls_arr_val.as_object().unwrap().unwrap();
    let arr_data = match heap.get(arr_id).unwrap() {
        ObjectData::Array(ArrayData::Reference(r)) => r.clone(),
        _ => panic!("Expected reference array"),
    };
    assert!(!arr_data.is_empty(), "Controls array must not be empty");
    let vc_id = arr_data[0].unwrap();

    // Call VolumeControl.setLevel(75)
    let lvl = NativeBridge::dispatch(
        "javax/microedition/media/control/VolumeControl",
        "setLevel",
        "(I)I",
        Some(vc_id),
        &[JavaValue::Int(75)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(lvl, 75);

    // Call VolumeControl.getLevel()
    let get_lvl = NativeBridge::dispatch(
        "javax/microedition/media/control/VolumeControl",
        "getLevel",
        "()I",
        Some(vc_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(get_lvl, 75);

    // Clamping setLevel(150) -> 100
    let clamped = NativeBridge::dispatch(
        "javax/microedition/media/control/VolumeControl",
        "setLevel",
        "(I)I",
        Some(vc_id),
        &[JavaValue::Int(150)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(clamped, 100);

    // setMute(true)
    NativeBridge::dispatch(
        "javax/microedition/media/control/VolumeControl",
        "setMute",
        "(Z)V",
        Some(vc_id),
        &[JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let is_muted = NativeBridge::dispatch(
        "javax/microedition/media/control/VolumeControl",
        "isMuted",
        "()Z",
        Some(vc_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(is_muted, 1);
}

#[test]
fn test_nokia_direct_graphics_native_pixel_format() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let dg_id = heap.alloc_instance("com/nokia/mid/ui/DirectGraphics");
    let format = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "getNativePixelFormat",
        "()I",
        Some(dg_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    assert_eq!(format, 4444);
}

#[test]
fn test_m3g_world_add_and_get_child() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let world_id = heap.alloc_instance("javax/microedition/m3g/World");
    let node_id = heap.alloc_instance("javax/microedition/m3g/Node");

    // Add child
    NativeBridge::dispatch(
        "javax/microedition/m3g/World",
        "addChild",
        "(Ljavax/microedition/m3g/Node;)V",
        Some(world_id),
        &[JavaValue::Object(Some(node_id))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    // Check getChildCount() == 1
    let count = NativeBridge::dispatch(
        "javax/microedition/m3g/World",
        "getChildCount",
        "()I",
        Some(world_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(count, 1);

    // Check getChild(0) == node_id
    let child = NativeBridge::dispatch(
        "javax/microedition/m3g/World",
        "getChild",
        "(I)Ljavax/microedition/m3g/Node;",
        Some(world_id),
        &[JavaValue::Int(0)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();
    assert_eq!(child, node_id);

    // Fallback on empty World
    let empty_world = heap.alloc_instance("javax/microedition/m3g/World");
    let fallback_child = NativeBridge::dispatch(
        "javax/microedition/m3g/World",
        "getChild",
        "(I)Ljavax/microedition/m3g/Node;",
        Some(empty_world),
        &[JavaValue::Int(0)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();
    assert!(fallback_child.is_some(), "Fallback child must not be null");
}

#[test]
fn test_message_digest_md5_and_sha1() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // MD5 test
    let algo_str = heap.alloc_string("MD5");
    let md_id = NativeBridge::dispatch(
        "java/security/MessageDigest",
        "getInstance",
        "(Ljava/lang/String;)Ljava/security/MessageDigest;",
        None,
        &[JavaValue::Object(Some(algo_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    let bytes = b"abc";
    let input_arr = heap.alloc_array(ArrayData::Byte(bytes.iter().map(|&b| b as i8).collect()));
    NativeBridge::dispatch(
        "java/security/MessageDigest",
        "update",
        "([B)V",
        Some(md_id),
        &[JavaValue::Object(Some(input_arr))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let digest_val = NativeBridge::dispatch(
        "java/security/MessageDigest",
        "digest",
        "()[B",
        Some(md_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();

    let digest_arr_id = digest_val.as_object().unwrap().unwrap();
    let digest_bytes = match heap.get(digest_arr_id).unwrap() {
        ObjectData::Array(ArrayData::Byte(b)) => b.iter().map(|&x| x as u8).collect::<Vec<_>>(),
        _ => panic!("Expected byte array"),
    };

    assert_eq!(digest_bytes.len(), 16);
    assert_eq!(
        digest_bytes,
        vec![
            0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0,
            0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1, 0x7f, 0x72
        ]
    );

    // SHA-1 test
    let sha_str = heap.alloc_string("SHA-1");
    let sha_md_id = NativeBridge::dispatch(
        "java/security/MessageDigest",
        "getInstance",
        "(Ljava/lang/String;)Ljava/security/MessageDigest;",
        None,
        &[JavaValue::Object(Some(sha_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap()
    .unwrap();

    NativeBridge::dispatch(
        "java/security/MessageDigest",
        "update",
        "([B)V",
        Some(sha_md_id),
        &[JavaValue::Object(Some(input_arr))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let sha_digest_val = NativeBridge::dispatch(
        "java/security/MessageDigest",
        "digest",
        "()[B",
        Some(sha_md_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();

    let sha_digest_arr_id = sha_digest_val.as_object().unwrap().unwrap();
    let sha_digest_bytes = match heap.get(sha_digest_arr_id).unwrap() {
        ObjectData::Array(ArrayData::Byte(b)) => b.iter().map(|&x| x as u8).collect::<Vec<_>>(),
        _ => panic!("Expected byte array"),
    };
    assert_eq!(sha_digest_bytes.len(), 20);
    assert_eq!(
        sha_digest_bytes,
        vec![
            0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e,
            0x25, 0x71, 0x78, 0x50, 0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d
        ]
    );
}

#[test]
fn test_record_store_invalid_id_throws_and_set_record_autocreates() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let store_name = "ClusterCTestStore";
    let name_id = heap.alloc_string(store_name);
    let store_ref = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "openRecordStore",
        "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
        None,
        &[JavaValue::Object(Some(name_id)), JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    // 1. getRecord(99) on empty store must throw InvalidRecordIDException
    let get_res = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "getRecord",
        "(I)[B",
        store_ref,
        &[JavaValue::Int(99)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(get_res.is_err());
    let err = get_res.unwrap_err();
    assert!(
        err.contains("javax/microedition/rms/InvalidRecordIDException"),
        "Expected InvalidRecordIDException, got: {}",
        err
    );

    // 2. setRecord(1, data, 0, len) on new store autocreates record 1
    let data_arr = heap.alloc_array(ArrayData::Byte(vec![1, 2, 3, 4, 5]));
    let set_res = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "setRecord",
        "(I[BII)V",
        store_ref,
        &[
            JavaValue::Int(1),
            JavaValue::Object(Some(data_arr)),
            JavaValue::Int(0),
            JavaValue::Int(5),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(set_res.is_ok(), "setRecord should succeed and autocreate");

    // 3. getRecord(1) should now return [1, 2, 3, 4, 5]
    let rec_val = NativeBridge::dispatch(
        "javax/microedition/rms/RecordStore",
        "getRecord",
        "(I)[B",
        store_ref,
        &[JavaValue::Int(1)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();

    let rec_id = rec_val.as_object().unwrap().unwrap();
    let bytes = match heap.get(rec_id).unwrap() {
        ObjectData::Array(ArrayData::Byte(b)) => b.iter().map(|&x| x as u8).collect::<Vec<_>>(),
        _ => panic!("Expected byte array"),
    };
    assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_graphics_draw_image_null_safe_and_clamp_dimensions() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // 1. createImage(0, 0) should clamp dimensions to >= 1x1
    let img_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "createImage",
        "(II)Ljavax/microedition/lcdui/Image;",
        None,
        &[JavaValue::Int(0), JavaValue::Int(0)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    let img_ref = img_val.as_object().unwrap();

    let w_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "getWidth",
        "()I",
        img_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    let h_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "getHeight",
        "()I",
        img_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    assert!(w_val >= 1, "Width should be clamped to >= 1");
    assert!(h_val >= 1, "Height should be clamped to >= 1");

    // 2. Graphics.drawImage with img == null (0) should be a no-op
    let g_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "getGraphics",
        "()Ljavax/microedition/lcdui/Graphics;",
        img_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    let g_ref = g_val.as_object().unwrap();

    let draw_res = NativeBridge::dispatch(
        "javax/microedition/lcdui/Graphics",
        "drawImage",
        "(Ljavax/microedition/lcdui/Image;III)V",
        g_ref,
        &[
            JavaValue::NULL,
            JavaValue::Int(10),
            JavaValue::Int(10),
            JavaValue::Int(20),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(draw_res.is_ok(), "drawImage with null image must be no-op");
}

#[test]
fn test_resource_lookup_case_insensitive_and_slash_normalization() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Mock a 1x1 PNG at "res/test.png"
    let png_1x1: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let mut jar = j2me_core::jar::JarArchive {
        manifest: None,
        entries: HashMap::new(),
    };
    jar.entries.insert("res/test.png".to_string(), png_1x1);
    context.jar = Some(jar);

    // 1. Class.getResourceAsStream with "//Res/Test.PNG"
    let path_str = heap.alloc_string("//Res/Test.PNG");
    let class_obj = heap.alloc_instance("java/lang/Class");
    let cname = heap.alloc_string("com/test/Game");
    heap.set_field(class_obj, "name", JavaValue::Object(Some(cname)));

    let stream_val = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(path_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    assert!(
        stream_val != JavaValue::NULL,
        "Class.getResourceAsStream should find resource via case-insensitive slash-normalized path"
    );

    // 2. Image.createImage(String) with "//Res/Test.PNG"
    let img_val = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "createImage",
        "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
        None,
        &[JavaValue::Object(Some(path_str))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap();
    assert!(
        img_val != JavaValue::NULL,
        "Image.createImage should resolve case-insensitive normalized path"
    );
}

#[test]
fn test_system_exception_hierarchy_matching() {
    let cl = ClassLoader::new();

    // 1. InvalidRecordIDException is subclass of RecordStoreException, Exception, Throwable
    assert!(cl.is_instance_of(
        "javax/microedition/rms/InvalidRecordIDException",
        "javax/microedition/rms/RecordStoreException"
    ));
    assert!(cl.is_instance_of(
        "javax/microedition/rms/InvalidRecordIDException",
        "java/lang/Exception"
    ));
    assert!(cl.is_instance_of(
        "javax/microedition/rms/InvalidRecordIDException",
        "java/lang/Throwable"
    ));

    // 2. EOFException is subclass of IOException and Exception
    assert!(cl.is_instance_of(
        "java/io/EOFException",
        "java/io/IOException"
    ));
    assert!(cl.is_instance_of(
        "java/io/EOFException",
        "java/lang/Exception"
    ));

    // 3. NoSuchElementException is subclass of RuntimeException and Exception
    assert!(cl.is_instance_of(
        "java/util/NoSuchElementException",
        "java/lang/RuntimeException"
    ));
    assert!(cl.is_instance_of(
        "java/util/NoSuchElementException",
        "java/lang/Exception"
    ));
}

#[test]
fn test_audio_no_clipping_and_pcm_resampling() {
    // 1. Build a 5-note polyphonic MIDI chord with maximum velocity 127
    let track = vec![
        0x00, 0x90, 0x3C, 0x7F, // Note On C4 (60) vel 127
        0x00, 0x90, 0x40, 0x7F, // Note On E4 (64) vel 127
        0x00, 0x90, 0x43, 0x7F, // Note On G4 (67) vel 127
        0x00, 0x90, 0x47, 0x7F, // Note On B4 (71) vel 127
        0x00, 0x90, 0x48, 0x7F, // Note On C5 (72) vel 127
        0x60, 0x80, 0x3C, 0x00, // Note Off C4
        0x00, 0x80, 0x40, 0x00, // Note Off E4
        0x00, 0x80, 0x43, 0x00, // Note Off G4
        0x00, 0x80, 0x47, 0x00, // Note Off B4
        0x00, 0x80, 0x48, 0x00, // Note Off C5
        0x00, 0xFF, 0x2F, 0x00, // End of track
    ];

    let mut smf = Vec::new();
    smf.extend_from_slice(b"MThd");
    smf.extend_from_slice(&6u32.to_be_bytes());
    smf.extend_from_slice(&0u16.to_be_bytes()); // format 0
    smf.extend_from_slice(&1u16.to_be_bytes()); // 1 track
    smf.extend_from_slice(&96u16.to_be_bytes()); // 96 division
    smf.extend_from_slice(b"MTrk");
    smf.extend_from_slice(&(track.len() as u32).to_be_bytes());
    smf.extend_from_slice(&track);

    let seq = j2me_stdlib::media::midi::MidiSequence::parse(&smf).expect("valid SMF");
    let mut synth = j2me_stdlib::media::midi::MidiSynthesizer::new(seq, 44100);
    synth.volume = 100;
    synth.start();

    let mut samples = Vec::new();
    synth.render_samples(44100, &mut samples);
    assert_eq!(samples.len(), 44100);

    // Verify no NaN/Inf, all strictly within [-1.0, 1.0], and 0% clipping (sample.abs() < 0.99)
    let mut clipped_count = 0;
    let mut peak = 0.0f32;
    for &s in &samples {
        assert!(!s.is_nan(), "Sample must not be NaN");
        assert!(!s.is_infinite(), "Sample must not be Infinite");
        assert!(s >= -1.0 && s <= 1.0, "Sample out of bounds: {}", s);
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
        if abs >= 0.99 {
            clipped_count += 1;
        }
    }
    assert!(peak > 0.05, "Chord should produce audible sound, got peak {}", peak);
    assert_eq!(clipped_count, 0, "Clipped samples must be 0, got {}", clipped_count);

    // 2. Test 8-bit unsigned WAV decoding
    let mut wav_8bit_pcm = Vec::new();
    let num_8bit_frames = 400;
    for i in 0..num_8bit_frames {
        let val = 128.0 + 100.0 * (i as f32 * 0.1).sin();
        wav_8bit_pcm.push(val.round() as u8);
    }
    let mut wav_8bit = Vec::new();
    wav_8bit.extend_from_slice(b"RIFF");
    wav_8bit.extend_from_slice(&(36 + wav_8bit_pcm.len() as u32).to_le_bytes());
    wav_8bit.extend_from_slice(b"WAVE");
    wav_8bit.extend_from_slice(b"fmt ");
    wav_8bit.extend_from_slice(&16u32.to_le_bytes());
    wav_8bit.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav_8bit.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav_8bit.extend_from_slice(&22050u32.to_le_bytes()); // 22050 Hz
    wav_8bit.extend_from_slice(&22050u32.to_le_bytes()); // byte rate
    wav_8bit.extend_from_slice(&1u16.to_le_bytes()); // block align
    wav_8bit.extend_from_slice(&8u16.to_le_bytes()); // 8 bits
    wav_8bit.extend_from_slice(b"data");
    wav_8bit.extend_from_slice(&(wav_8bit_pcm.len() as u32).to_le_bytes());
    wav_8bit.extend_from_slice(&wav_8bit_pcm);

    let dec_8bit = j2me_stdlib::media::wav::WavDecoder::parse(&wav_8bit, 44100).expect("parse 8-bit WAV");
    assert!(!dec_8bit.samples.is_empty());
    for &s in &dec_8bit.samples {
        assert!(!s.is_nan() && !s.is_infinite());
        assert!(s >= -1.0 && s <= 1.0);
    }

    // 3. Test 16-bit signed WAV decoding
    let mut wav_16bit_pcm = Vec::new();
    let num_16bit_frames = 400;
    for i in 0..num_16bit_frames {
        let val = (25000.0 * (i as f32 * 0.1).sin()) as i16;
        wav_16bit_pcm.extend_from_slice(&val.to_le_bytes());
    }
    let mut wav_16bit = Vec::new();
    wav_16bit.extend_from_slice(b"RIFF");
    wav_16bit.extend_from_slice(&(36 + wav_16bit_pcm.len() as u32).to_le_bytes());
    wav_16bit.extend_from_slice(b"WAVE");
    wav_16bit.extend_from_slice(b"fmt ");
    wav_16bit.extend_from_slice(&16u32.to_le_bytes());
    wav_16bit.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav_16bit.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav_16bit.extend_from_slice(&44100u32.to_le_bytes()); // 44100 Hz
    wav_16bit.extend_from_slice(&88200u32.to_le_bytes()); // byte rate
    wav_16bit.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav_16bit.extend_from_slice(&16u16.to_le_bytes()); // 16 bits
    wav_16bit.extend_from_slice(b"data");
    wav_16bit.extend_from_slice(&(wav_16bit_pcm.len() as u32).to_le_bytes());
    wav_16bit.extend_from_slice(&wav_16bit_pcm);

    let dec_16bit = j2me_stdlib::media::wav::WavDecoder::parse(&wav_16bit, 44100).expect("parse 16-bit WAV");
    assert!(!dec_16bit.samples.is_empty());
    for &s in &dec_16bit.samples {
        assert!(!s.is_nan() && !s.is_infinite());
        assert!(s >= -1.0 && s <= 1.0);
    }
}

#[test]
fn test_draw_region_rotated_anchor_dimensions() {
    use j2me_stdlib::midp::lcdui::graphics::{Graphics, BOTTOM, RIGHT};
    use j2me_stdlib::midp::lcdui::game::sprite::TRANS_ROT90;
    use j2me_stdlib::midp::lcdui::image::Image;

    let mut fb = vec![0u32; 200 * 200];
    let g = Graphics::new(200, 200);

    // Image with width = 10, height = 20 filled with red (0xFFFF0000)
    let src_w = 10;
    let src_h = 20;
    let pixels = vec![0xFFFF0000; (src_w * src_h) as usize];
    let img = Image::new_immutable(src_w, src_h, pixels);

    // drawRegion at (100, 100) with TRANS_ROT90 and RIGHT | BOTTOM
    // Rotated dimensions should be w=20, h=10.
    // Anchors RIGHT | BOTTOM mean draw_x = 100 - 20 = 80, draw_y = 100 - 10 = 90.
    // So rendered bounding box is x in [80..100), y in [90..100).
    g.draw_region(&mut fb, &img, 0, 0, src_w, src_h, TRANS_ROT90, 100, 100, RIGHT | BOTTOM);

    // Check pixel at (80, 90) - top-left of rotated image: MUST BE RED
    assert_eq!(fb[(90 * 200 + 80) as usize], 0xFFFF0000, "Pixel at (80, 90) must be drawn");
    // Check pixel at (99, 99) - bottom-right of rotated image: MUST BE RED
    assert_eq!(fb[(99 * 200 + 99) as usize], 0xFFFF0000, "Pixel at (99, 99) must be drawn");

    // Check that (79, 90) is NOT drawn (it's outside)
    assert_eq!(fb[(90 * 200 + 79) as usize], 0, "Pixel at (79, 90) must be untouched");
    // Check that (100, 90) is NOT drawn (it's outside, since 80..100 is exclusive of 100)
    assert_eq!(fb[(90 * 200 + 100) as usize], 0, "Pixel at (100, 90) must be untouched");
    // Check that (85, 89) is NOT drawn
    assert_eq!(fb[(89 * 200 + 85) as usize], 0, "Pixel at (85, 89) must be untouched");
    // Check that (85, 100) is NOT drawn
    assert_eq!(fb[(100 * 200 + 85) as usize], 0, "Pixel at (85, 100) must be untouched");

    // If anchor had used unrotated w=10, h=20, draw_x would be 90, draw_y would be 80.
    // Verify (85, 85) is NOT drawn
    assert_eq!(fb[(85 * 200 + 85) as usize], 0, "Unrotated anchor bounding box must not be used");
}

#[test]
fn test_font_vcenter_anchor_and_substring_metrics() {
    use j2me_stdlib::midp::lcdui::font::FontSpec;
    use j2me_stdlib::midp::lcdui::graphics::{Graphics, HCENTER, VCENTER};

    let font = FontSpec::default();
    let text = "Hello FreeJ2ME";
    let sub = "Free";

    // Test substring width
    let sub_w = font.substring_width(text, 6, 4);
    let direct_w = font.string_width(sub);
    assert_eq!(sub_w, direct_w, "substringWidth must equal stringWidth of substring");
    assert!(sub_w > 0);

    // Test drawString with HCENTER | VCENTER doesn't panic and renders centered
    let mut fb = vec![0u32; 100 * 100];
    let g = Graphics::new(100, 100);
    g.draw_string(&mut fb, text, 50, 50, HCENTER | VCENTER);

    // Count drawn non-zero pixels
    let non_zero = fb.iter().filter(|&&p| p != 0).count();
    assert!(non_zero > 0, "Drawn string with HCENTER | VCENTER must produce non-zero pixels");
}

#[test]
fn test_lifecycle_shownotify_called_before_first_paint() {
    use j2me_core::classfile::class_file::ACC_PUBLIC;
    use j2me_core::vm::class_loader::PredecodedMethod;
    use j2me_core::vm::instruction::Instruction;
    use j2me_stdlib::J2meEmulator;

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;

    // Define custom canvas class: MyCanvas extends javax/microedition/lcdui/Canvas
    // showNotify sets a static field flag to 42
    let show_method = PredecodedMethod {
        class_name: "MyCanvas".to_string(),
        name: "showNotify".to_string(),
        descriptor: "()V".to_string(),
        access_flags: ACC_PUBLIC,
        max_locals: 1,
        max_stack: 2,
        instructions: Arc::new(vec![
            Instruction::BiPush(42),
            Instruction::PutStatic {
                class_name: "MyCanvas".to_string(),
                field_name: "shown".to_string(),
                descriptor: "I".to_string(),
            },
            Instruction::Return,
        ]),
        pc_map: Arc::new(HashMap::new()),
        exception_table: Vec::new(),
    };

    // paint divides by MyCanvas.shown: if shown == 0, throws ArithmeticException!
    let paint_method = PredecodedMethod {
        class_name: "MyCanvas".to_string(),
        name: "paint".to_string(),
        descriptor: "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
        access_flags: ACC_PUBLIC,
        max_locals: 2,
        max_stack: 2,
        instructions: Arc::new(vec![
            Instruction::BiPush(100),
            Instruction::GetStatic {
                class_name: "MyCanvas".to_string(),
                field_name: "shown".to_string(),
                descriptor: "I".to_string(),
            },
            Instruction::IDiv,
            Instruction::Pop,
            Instruction::Return,
        ]),
        pc_map: Arc::new(HashMap::new()),
        exception_table: Vec::new(),
    };

    emu.class_loader.predecoded_methods.insert(
        (
            "MyCanvas".to_string(),
            "showNotify".to_string(),
            "()V".to_string(),
        ),
        Arc::new(show_method),
    );
    emu.class_loader.predecoded_methods.insert(
        (
            "MyCanvas".to_string(),
            "paint".to_string(),
            "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
        ),
        Arc::new(paint_method),
    );

    let canvas_obj = emu.heap.alloc_instance("MyCanvas");

    // Thread executes:
    // 1. Display.getDisplay(null).setCurrent(canvas_obj)
    // 2. canvas_obj.serviceRepaints()
    let caller_frame = Frame::new(
        "Main".to_string(),
        "run".to_string(),
        "()V".to_string(),
        Arc::new(vec![
            Instruction::AConstNull,
            Instruction::InvokeStatic {
                class_name: "javax/microedition/lcdui/Display".to_string(),
                method_name: "getDisplay".to_string(),
                descriptor: "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;".to_string(),
            },
            Instruction::ALoad(0),
            Instruction::InvokeVirtual {
                class_name: "javax/microedition/lcdui/Display".to_string(),
                method_name: "setCurrent".to_string(),
                descriptor: "(Ljavax/microedition/lcdui/Displayable;)V".to_string(),
            },
            Instruction::ALoad(0),
            Instruction::InvokeVirtual {
                class_name: "MyCanvas".to_string(),
                method_name: "serviceRepaints".to_string(),
                descriptor: "()V".to_string(),
            },
            Instruction::Return,
        ]),
        2,
        4,
        Vec::new(),
        Arc::new(HashMap::new()),
    );

    let _tid = emu.scheduler.spawn("MainThread", caller_frame, None);
    emu.scheduler.threads[0].call_stack[0].set_local(0, JavaValue::Object(Some(canvas_obj)));

    for _ in 0..10 {
        emu.tick(16.0);
    }

    let shown_val = emu.class_loader.get_static("MyCanvas", "shown", "I");
    assert_eq!(
        shown_val,
        JavaValue::Int(42),
        "showNotify must have been executed before paint()"
    );
}

#[test]
fn test_class_new_instance_invokes_default_constructor() {
    use j2me_core::classfile::class_file::ACC_PUBLIC;
    use j2me_core::vm::class_loader::PredecodedMethod;
    use j2me_core::vm::instruction::Instruction;
    use j2me_stdlib::J2meEmulator;

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;

    // Define DynamicTarget.<init>()V that sets field "initialized" to 999
    let init_method = PredecodedMethod {
        class_name: "DynamicTarget".to_string(),
        name: "<init>".to_string(),
        descriptor: "()V".to_string(),
        access_flags: ACC_PUBLIC,
        max_locals: 1,
        max_stack: 2,
        instructions: Arc::new(vec![
            Instruction::ALoad(0),
            Instruction::SiPush(999),
            Instruction::PutField {
                class_name: "DynamicTarget".to_string(),
                field_name: "initialized".to_string(),
                descriptor: "I".to_string(),
            },
            Instruction::Return,
        ]),
        pc_map: Arc::new(HashMap::new()),
        exception_table: Vec::new(),
    };

    emu.class_loader.predecoded_methods.insert(
        (
            "DynamicTarget".to_string(),
            "<init>".to_string(),
            "()V".to_string(),
        ),
        Arc::new(init_method),
    );

    let caller_frame = Frame::new(
        "Main".to_string(),
        "run".to_string(),
        "()V".to_string(),
        Arc::new(vec![
            Instruction::LdcString("DynamicTarget".to_string()),
            Instruction::InvokeStatic {
                class_name: "java/lang/Class".to_string(),
                method_name: "forName".to_string(),
                descriptor: "(Ljava/lang/String;)Ljava/lang/Class;".to_string(),
            },
            Instruction::InvokeVirtual {
                class_name: "java/lang/Class".to_string(),
                method_name: "newInstance".to_string(),
                descriptor: "()Ljava/lang/Object;".to_string(),
            },
            Instruction::AStore(0),
            Instruction::Return,
        ]),
        1,
        2,
        Vec::new(),
        Arc::new(HashMap::new()),
    );

    let _tid = emu.scheduler.spawn("MainThread", caller_frame, None);

    for _ in 0..20 {
        emu.tick(16.0);
    }

    println!("Total heap allocations: {}", emu.heap.total_allocations);
    println!("Recent calls: {:?}", emu.context.recent_native_calls);
    let mut found = false;
    for id_num in 1..=emu.heap.total_allocations {
        let oid = j2me_core::heap::object::ObjectId(id_num as u32);
        if let Some(ObjectData::Instance(inst)) = emu.heap.get(oid) {
            if inst.class_name == "DynamicTarget" {
                if emu.heap.get_field(oid, "initialized") == JavaValue::Int(999) {
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(
        found,
        "Class.newInstance() must invoke default constructor <init>()V and initialize fields"
    );
}

#[test]
fn test_input_stream_read_unsigned_byte_0_to_255() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // 1. ByteArrayInputStream with [0xFF (-1 signed), 0x80 (-128 signed), 0x01]
    let byte_arr = heap.alloc_array(ArrayData::Byte(vec![-1, -128, 1]));
    let bais_obj = heap.alloc_instance("java/io/ByteArrayInputStream");
    NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "<init>",
        "([B)V",
        Some(bais_obj),
        &[JavaValue::Object(Some(byte_arr))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let b1 = NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "read",
        "()I",
        Some(bais_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(b1, 255, "Byte 0xFF must be read as unsigned +255, not -1 (EOF)");

    let b2 = NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "read",
        "()I",
        Some(bais_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(b2, 128, "Byte 0x80 must be read as unsigned +128, not -128");

    let b3 = NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "read",
        "()I",
        Some(bais_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(b3, 1);

    let b4 = NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "read",
        "()I",
        Some(bais_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(b4, -1, "EOF must be returned when stream is exhausted");

    // 2. DataInputStream wrapping ByteArrayInputStream with 0xFF
    let byte_arr2 = heap.alloc_array(ArrayData::Byte(vec![-1]));
    let bais_obj2 = heap.alloc_instance("java/io/ByteArrayInputStream");
    NativeBridge::dispatch(
        "java/io/ByteArrayInputStream",
        "<init>",
        "([B)V",
        Some(bais_obj2),
        &[JavaValue::Object(Some(byte_arr2))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let dis_obj = heap.alloc_instance("java/io/DataInputStream");
    NativeBridge::dispatch(
        "java/io/DataInputStream",
        "<init>",
        "(Ljava/io/InputStream;)V",
        Some(dis_obj),
        &[JavaValue::Object(Some(bais_obj2))],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();

    let dis_b1 = NativeBridge::dispatch(
        "java/io/DataInputStream",
        "read",
        "()I",
        Some(dis_obj),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();
    assert_eq!(
        dis_b1, 255,
        "DataInputStream.read() for 0xFF must return unsigned +255"
    );
}

#[test]
fn test_offscreen_image_graphics_initial_clip_rect() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let img_ref = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "createImage",
        "(II)Ljavax/microedition/lcdui/Image;",
        None,
        &[JavaValue::Int(64), JavaValue::Int(48)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    let g_ref = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "getGraphics",
        "()Ljavax/microedition/lcdui/Graphics;",
        img_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_object()
    .unwrap();

    let cw = NativeBridge::dispatch(
        "javax/microedition/lcdui/Graphics",
        "getClipWidth",
        "()I",
        g_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    let ch = NativeBridge::dispatch(
        "javax/microedition/lcdui/Graphics",
        "getClipHeight",
        "()I",
        g_ref,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    assert_eq!(
        cw, 64,
        "Initial clip width on Image.getGraphics() must match image width"
    );
    assert_eq!(
        ch, 48,
        "Initial clip height on Image.getGraphics() must match image height"
    );
}

#[test]
fn test_canvas_dimensions_available_in_constructor_before_set_current() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(176, 208, 44100);

    assert!(context.current_displayable.is_none());

    let canvas_ref = heap.alloc_instance("javax/microedition/lcdui/Canvas");

    let w = NativeBridge::dispatch(
        "javax/microedition/lcdui/Canvas",
        "getWidth",
        "()I",
        Some(canvas_ref),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    let h = NativeBridge::dispatch(
        "javax/microedition/lcdui/Canvas",
        "getHeight",
        "()I",
        Some(canvas_ref),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap()
    .unwrap()
    .as_int()
    .unwrap();

    assert_eq!(
        w, 176,
        "Canvas.getWidth() before Display.setCurrent must return screen width"
    );
    assert!(
        h > 0,
        "Canvas.getHeight() before Display.setCurrent must be positive"
    );
    assert!(
        h == 208 || h == 184,
        "Canvas.getHeight() should reflect screen height (208 or 208-24=184)"
    );
}

#[test]
fn test_phase57_deadlock_breaker_spurious_wakeup() {
    let mut heap = Heap::new();
    let mut scheduler = Scheduler::new(1000);

    // Spawn 2 threads
    let dummy_frame1 = Frame::new(
        "Test1".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );
    let dummy_frame2 = Frame::new(
        "Test2".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );

    let tid1 = scheduler.spawn("Worker1", dummy_frame1, None);
    let _tid2 = scheduler.spawn("Worker2", dummy_frame2, None);

    let lock_obj = heap.alloc_instance("java/lang/Object");

    // Acquire lock and wait on monitor with no timeout
    assert!(scheduler.try_acquire_monitor(tid1, lock_obj));
    assert!(scheduler.wait_monitor(tid1, lock_obj, None).is_ok());

    // Worker2 also waits on another lock
    let lock2 = heap.alloc_instance("java/lang/Object");
    assert!(scheduler.try_acquire_monitor(_tid2, lock2));
    assert!(scheduler.wait_monitor(_tid2, lock2, None).is_ok());

    // In this state, neither thread is Ready and none are sleeping -> true deadlock condition
    assert_eq!(scheduler.next_ready_thread_index(), None);

    // Deadlock breaker should wake a waiting thread
    let broke_deadlock = scheduler.break_deadlocks();
    assert!(broke_deadlock, "break_deadlocks must break thread deadlock condition");
    assert!(scheduler.next_ready_thread_index().is_some(), "at least one thread must be ready after deadlock breaker");
}

#[test]
fn test_phase57_universal_stub_fallback_vendor_apis() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // 1. Siemens vibrator (void method)
    let res_void = NativeBridge::dispatch(
        "com/siemens/mp/game/Vibrator",
        "triggerVibrator",
        "(I)V",
        None,
        &[JavaValue::Int(100)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert_eq!(res_void, Ok(None));

    // 2. Motorola Lighting query (boolean query)
    let res_bool = NativeBridge::dispatch(
        "com/motorola/multimedia/Lighting",
        "isSupported",
        "()Z",
        None,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert_eq!(res_bool, Ok(Some(JavaValue::Int(1))));

    // 3. Bluetooth byte array retrieval (array return type)
    let res_arr = NativeBridge::dispatch(
        "javax/bluetooth/LocalDevice",
        "getBluetoothAddressBytes",
        "()[B",
        None,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(matches!(res_arr, Ok(Some(JavaValue::Object(Some(_))))), "Array return type must return non-null array instance");

    // 4. Vendor object factory (object return type)
    let res_obj = NativeBridge::dispatch(
        "com/siemens/mp/game/GraphicObject",
        "createGraphicObject",
        "()Lcom/siemens/mp/game/GraphicObject;",
        None,
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(matches!(res_obj, Ok(Some(JavaValue::Object(Some(_))))), "Object return type from factory must return non-null object instance");
}

#[test]
fn test_phase58_scheduler_cooperative_yield_and_starvation_prevention() {
    let mut scheduler = Scheduler::new(1000);

    let dummy_frame1 = Frame::new(
        "UIThread".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );
    let dummy_frame2 = Frame::new(
        "LoaderThread".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );

    let _tid1 = scheduler.spawn("UIThread", dummy_frame1, None);
    let _tid2 = scheduler.spawn("LoaderThread", dummy_frame2, None);

    // Both threads are ready. UIThread yields:
    scheduler.threads[0].on_yield();

    // Scheduler must prioritize LoaderThread (index 1) which has 0 yields, preventing starvation
    let next_idx = scheduler.next_ready_thread_index();
    assert_eq!(next_idx, Some(1), "LoaderThread must be prioritized over yielded UIThread");

    // UIThread yields again (yield_count_in_frame = 2), and LoaderThread finishes
    scheduler.threads[0].on_yield();
    scheduler.threads[1].state = j2me_core::threads::green_thread::ThreadState::Terminated;

    // UIThread has yielded >= 2 times in this frame and no other ready threads exist -> frame ends
    let next_idx2 = scheduler.next_ready_thread_index();
    assert_eq!(next_idx2, None, "When all ready threads have yielded >= 2 times, frame turn ends cleanly");
}

#[test]
fn test_phase58_aggressive_deadlock_breaker_200k_instructions() {
    let mut heap = Heap::new();
    let mut scheduler = Scheduler::new(1000);

    let dummy_frame1 = Frame::new(
        "Worker1".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );
    let dummy_frame2 = Frame::new(
        "Worker2".to_string(),
        "run".to_string(),
        "()V".to_string(),
        std::sync::Arc::new(Vec::new()),
        2,
        2,
        vec![],
        std::sync::Arc::new(std::collections::HashMap::new()),
    );

    let tid1 = scheduler.spawn("Worker1", dummy_frame1, None);
    let _tid2 = scheduler.spawn("Worker2", dummy_frame2, None);

    let lock_obj = heap.alloc_instance("java/lang/Object");

    // Acquire lock and enter wait_monitor with no notify
    assert!(scheduler.try_acquire_monitor(tid1, lock_obj));
    assert!(scheduler.wait_monitor(tid1, lock_obj, Some(10_000)).is_ok());

    // Worker1 is in Waiting state
    assert!(matches!(scheduler.threads[0].state, j2me_core::threads::green_thread::ThreadState::Waiting { .. }));

    // Worker2 runs 250,000 instructions in the background
    scheduler.threads[1].instructions_executed += 250_000;

    // Virtual time advances by only 10ms (deadline is 10,000ms), but instruction threshold > 200k has elapsed!
    scheduler.tick_time(10);

    // Worker1 should be automatically woken up by instruction threshold Deadlock Breaker!
    assert!(
        matches!(scheduler.threads[0].state, j2me_core::threads::green_thread::ThreadState::Ready),
        "Waiting thread must be woken up after exceeding 200k instructions limit"
    );
}

#[test]
fn test_phase59_round_rect_1px_clamp_resilience() {
    let mut pixels = vec![0u32; 100 * 100];
    let g = j2me_stdlib::midp::lcdui::graphics::Graphics::new(100, 100);

    // w = 1, h = 1, arc_w = 10, arc_h = 10 previously panicked at clamp(1, 0)
    g.fill_round_rect(&mut pixels, 10, 10, 1, 1, 10, 10);
    g.draw_round_rect(&mut pixels, 20, 20, 1, 1, 10, 10);
    g.fill_round_rect(&mut pixels, 30, 30, 0, 0, 5, 5);
    g.draw_round_rect(&mut pixels, 40, 40, -5, -5, 5, 5);
}

#[test]
fn test_phase59_connector_bluetooth_and_wma_stubs() {
    use j2me_stdlib::J2meEmulator;
    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;

    let dummy_thread = j2me_core::threads::green_thread::GreenThread::new(
        j2me_core::threads::green_thread::ThreadId(1),
        "TestThread".to_string(),
        j2me_core::vm::frame::Frame::new(
            "Test".to_string(),
            "test".to_string(),
            "()V".to_string(),
            std::sync::Arc::new(Vec::new()),
            1,
            2,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ),
        None,
    );
    emu.scheduler.threads.push(dummy_thread);

    // 1. Connector.open("sms://+123456789")
    let sms_url = emu.heap.alloc_string("sms://+123456789");
    let res = j2me_stdlib::bridge::native_bridge::NativeBridge::dispatch(
        "javax/microedition/io/Connector",
        "open",
        "(Ljava/lang/String;)Ljavax/microedition/io/Connection;",
        None,
        &[j2me_core::heap::value::JavaValue::Object(Some(sms_url))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    );
    assert!(res.is_ok());
    let conn_id = res.unwrap().unwrap().as_object().unwrap().unwrap();
    assert_eq!(
        emu.heap.get(conn_id).unwrap().as_instance().unwrap().class_name,
        "javax/wireless/messaging/MessageConnection"
    );

    // 2. LocalDevice.getLocalDevice()
    let ld_res = j2me_stdlib::bridge::native_bridge::NativeBridge::dispatch(
        "javax/bluetooth/LocalDevice",
        "getLocalDevice",
        "()Ljavax/bluetooth/LocalDevice;",
        None,
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    );
    assert!(ld_res.is_ok());
    let ld_id = ld_res.unwrap().unwrap().as_object().unwrap().unwrap();
    assert_eq!(
        emu.heap.get(ld_id).unwrap().as_instance().unwrap().class_name,
        "javax/bluetooth/LocalDevice"
    );

    // 3. DiscoveryAgent.startInquiry
    let da_res = j2me_stdlib::bridge::native_bridge::NativeBridge::dispatch(
        "javax/bluetooth/DiscoveryAgent",
        "startInquiry",
        "(ILjavax/bluetooth/DiscoveryListener;)Z",
        None,
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    );
    assert!(da_res.is_ok());
    assert_eq!(da_res.unwrap(), Some(j2me_core::heap::value::JavaValue::Int(0)));
}

#[test]
fn test_phase510_uncaught_athrow_auto_healing_and_region_clamping() {
    use j2me_core::vm::instruction::Instruction;
    use j2me_core::vm::interpreter::{Interpreter, StepResult};

    let mut heap = Heap::new();
    let cl = ClassLoader::new();

    // 1. Verify Uncaught Athrow Auto-Healing across nested frames
    // Parent frame: descriptor "()I", caller pc 0
    let parent_frame = Frame::new(
        "com/test/Caller".to_string(),
        "callerMethod".to_string(),
        "()I".to_string(),
        Arc::new(vec![Instruction::Nop, Instruction::IReturn]),
        10,
        10,
        vec![],
        Arc::new(HashMap::new()),
    );
    let mut thread = GreenThread::new(ThreadId(1), "test-thread".to_string(), parent_frame, None);

    // Child frame: descriptor "()I", throwing unhandled exception
    let child_frame = Frame::new(
        "com/test/Callee".to_string(),
        "failingMethod".to_string(),
        "()I".to_string(),
        Arc::new(vec![Instruction::AThrow]),
        10,
        10,
        vec![], // No catch block
        Arc::new(HashMap::new()),
    );
    thread.call_stack.push(child_frame);

    let exc_id = heap.alloc_instance("java/lang/RuntimeException");
    let res = Interpreter::handle_athrow(&mut thread, &mut heap, &cl, exc_id);

    // Verify it auto-heals: pops child frame, deposits default int (0) in parent frame, continues!
    assert!(matches!(res, StepResult::Continue));
    assert_eq!(thread.call_stack.len(), 1);
    let top = thread.call_stack.last_mut().unwrap();
    assert_eq!(top.pop().unwrap(), JavaValue::Int(0));

    // 2. Verify DirectGraphics safe bounds clamping without throwing
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Create Graphics object
    let g_inst = heap.alloc_instance("javax/microedition/lcdui/Graphics");
    let dg_inst = heap.alloc_instance("com/nokia/mid/ui/DirectGraphics");
    context.graphics_contexts.insert(
        g_inst,
        j2me_stdlib::bridge::context::GraphicsContext {
            graphics: j2me_stdlib::midp::lcdui::graphics::Graphics::new(240, 320),
            target: GraphicsTarget::Display,
        },
    );

    let pixels_arr = heap.alloc_array(ArrayData::Int(vec![0xFF0000; 10]));
    let dg_res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([Z[IIIIIIII)V",
        Some(dg_inst),
        &[
            JavaValue::Object(Some(pixels_arr)),
            JavaValue::Int(0),     // transparency
            JavaValue::Int(0),     // offset
            JavaValue::Int(100),   // scanlength > len (previously caused AIOOBE!)
            JavaValue::Int(0),     // x
            JavaValue::Int(0),     // y
            JavaValue::Int(100),   // width
            JavaValue::Int(100),   // height
            JavaValue::Int(0),     // manipulation
            JavaValue::Int(1),     // format
        ],
        &mut heap,
        &mut cl.clone(),
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(dg_res.is_ok(), "drawPixels must not throw exception on out-of-bound stride");

    let get_pixels_arr = heap.alloc_array(ArrayData::Int(vec![0; 5]));
    let get_res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "getPixels",
        "([IIIIIII)V",
        Some(dg_inst),
        &[
            JavaValue::Object(Some(get_pixels_arr)),
            JavaValue::Int(0),
            JavaValue::Int(50),
            JavaValue::Int(0),
            JavaValue::Int(0),
            JavaValue::Int(50),
            JavaValue::Int(50),
            JavaValue::Int(1),
        ],
        &mut heap,
        &mut cl.clone(),
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(get_res.is_ok(), "getPixels must not throw exception on out-of-bound dimensions");

    // 3. Verify Konami MIDlet stubs
    let req_res = NativeBridge::dispatch(
        "javax/microedition/midlet/MIDlet",
        "platformRequest",
        "(Ljava/lang/String;)Z",
        None,
        &[JavaValue::Object(None)],
        &mut heap,
        &mut cl.clone(),
        &mut scheduler,
        &mut context,
        0,
    );
    assert_eq!(req_res.unwrap(), Some(JavaValue::Int(1)));
}

#[test]
fn test_phase511_font_and_sprite_no_horizontal_scanline_gaps() {
    use j2me_stdlib::midp::lcdui::font::{FontSpec, FACE_SYSTEM, STYLE_PLAIN, SIZE_SMALL, SIZE_MEDIUM, SIZE_LARGE};

    // 1. Check all standard font sizes (SMALL=9, MEDIUM=13, LARGE=18)
    for &size in &[SIZE_SMALL, SIZE_MEDIUM, SIZE_LARGE] {
        let font = FontSpec::new(FACE_SYSTEM, STYLE_PLAIN, size);
        let font_h = font.get_height();

        // For glyphs with known solid vertical stems: "I", "L", "E", "B", "M"
        for &ch in &['I', 'L', 'E', 'B', 'M'] {
            let mut fb = vec![0u32; 100 * 100];
            font.render_string(
                &mut fb,
                100,
                100,
                (0, 0, 100, 100),
                &ch.to_string(),
                10,
                10,
                0xFFFFFFFF,
            );

            let char_w = font.char_width(' ');
            let glyph = j2me_stdlib::midp::lcdui::font::get_glyph_8x8(ch);

            // Verify that for every column and every scanline, pixels match source sampling perfectly
            // (i.e. every dy from 0 to font_h is rendered without skipped scanline gaps)
            for dx in 0..char_w {
                let src_col = ((dx * 8) / char_w).min(7) as usize;
                for dy in 0..font_h {
                    let src_row = ((dy * 8) / font_h).min(7) as usize;
                    let should_be_painted = (glyph[src_row] & (1 << (7 - src_col))) != 0;
                    let is_painted = fb[((10 + dy) * 100 + (10 + dx)) as usize] == 0xFFFFFFFF;
                    assert_eq!(
                        is_painted,
                        should_be_painted,
                        "Horizontal scanline gap at dy={}, dx={} for glyph '{}' size {}",
                        dy, dx, ch, size
                    );
                }
            }
        }
    }

    // 2. Check High-Level UI List rendering
    let mut emu = j2me_stdlib::emulator::J2meEmulator::new(240, 320, 44100);
    let list_id = emu.heap.alloc_instance("javax/microedition/lcdui/List");
    emu.context.lists.insert(
        list_id,
        j2me_stdlib::bridge::context::ListState {
            title: "Options".to_string(),
            list_type: 3, // IMPLICIT
            elements: vec![
                j2me_stdlib::bridge::context::ListElement {
                    string_part: "My List".to_string(),
                    image_part: None,
                    selected: false,
                },
                j2me_stdlib::bridge::context::ListElement {
                    string_part: "Opponent List".to_string(),
                    image_part: None,
                    selected: false,
                },
                j2me_stdlib::bridge::context::ListElement {
                    string_part: "Back".to_string(),
                    image_part: None,
                    selected: false,
                },
            ],
            selected_index: 0,
            fit_policy: 0,
            select_command: None,
        },
    );
    emu.render_list(list_id);

    // Verify non-empty pixels exist and letters do not have single-scanline gaps in vertical spans
    let text_pixels = emu
        .context
        .framebuffer
        .iter()
        .filter(|&&px| px == 0xFFFFFFFF || px == 0xFFE2E8F0)
        .count();
    assert!(text_pixels > 50, "List menu text must be rendered");
}

#[test]
fn test_phase511_frame_pacing_one_repaint_per_tick() {
    use j2me_core::classfile::class_file::ACC_PUBLIC;
    use j2me_core::vm::class_loader::PredecodedMethod;
    use j2me_core::vm::instruction::Instruction;

    let mut emu = j2me_stdlib::emulator::J2meEmulator::new(240, 320, 44100);
    emu.is_running = true;

    // Define Dummy Canvas with paint method
    let paint_method = PredecodedMethod {
        class_name: "PacedCanvas".to_string(),
        name: "paint".to_string(),
        descriptor: "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
        access_flags: ACC_PUBLIC,
        max_locals: 2,
        max_stack: 2,
        instructions: Arc::new(vec![Instruction::Return]),
        pc_map: Arc::new(HashMap::new()),
        exception_table: Vec::new(),
    };

    emu.class_loader.predecoded_methods.insert(
        (
            "PacedCanvas".to_string(),
            "paint".to_string(),
            "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
        ),
        Arc::new(paint_method),
    );

    let canvas_obj = emu.heap.alloc_instance("PacedCanvas");

    // Game loop bytecode:
    // 0: AConstNull
    // 1: InvokeStatic Display.getDisplay
    // 2: ALoad(0)
    // 3: InvokeVirtual Display.setCurrent
    // --- Loop ---
    // 4: ALoad(0)
    // 5: InvokeVirtual Canvas.repaint
    // 6: ALoad(0)
    // 7: InvokeVirtual Canvas.serviceRepaints
    // 8: InvokeStatic Thread.yield
    // 9: Goto(4)
    let loop_frame = Frame::new(
        "GameMain".to_string(),
        "run".to_string(),
        "()V".to_string(),
        Arc::new(vec![
            Instruction::AConstNull,
            Instruction::InvokeStatic {
                class_name: "javax/microedition/lcdui/Display".to_string(),
                method_name: "getDisplay".to_string(),
                descriptor: "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;".to_string(),
            },
            Instruction::ALoad(0),
            Instruction::InvokeVirtual {
                class_name: "javax/microedition/lcdui/Display".to_string(),
                method_name: "setCurrent".to_string(),
                descriptor: "(Ljavax/microedition/lcdui/Displayable;)V".to_string(),
            },
            // Loop start
            Instruction::ALoad(0),
            Instruction::InvokeVirtual {
                class_name: "javax/microedition/lcdui/Canvas".to_string(),
                method_name: "repaint".to_string(),
                descriptor: "()V".to_string(),
            },
            Instruction::ALoad(0),
            Instruction::InvokeVirtual {
                class_name: "PacedCanvas".to_string(),
                method_name: "serviceRepaints".to_string(),
                descriptor: "()V".to_string(),
            },
            Instruction::InvokeStatic {
                class_name: "java/lang/Thread".to_string(),
                method_name: "yield".to_string(),
                descriptor: "()V".to_string(),
            },
            Instruction::Goto(4),
        ]),
        2,
        4,
        Vec::new(),
        Arc::new(HashMap::new()),
    );

    let _tid = emu.scheduler.spawn("GameLoopThread", loop_frame, None);
    emu.scheduler.threads[0].call_stack[0].set_local(0, JavaValue::Object(Some(canvas_obj)));

    // Execute first tick: must execute exactly 1 paint, then yield until next tick
    assert_eq!(emu.context.paint_calls_count, 0);
    emu.tick(33.3);
    assert_eq!(
        emu.context.paint_calls_count, 1,
        "Exactly 1 paint must execute in first tick despite while(true) yield loop"
    );

    // Execute second tick: must execute exactly 1 more paint
    emu.tick(33.3);
    assert_eq!(
        emu.context.paint_calls_count, 2,
        "Exactly 1 paint must execute in second tick"
    );

    // Execute 5 more ticks
    for _ in 0..5 {
        emu.tick(33.3);
    }
    assert_eq!(
        emu.context.paint_calls_count, 7,
        "Exactly 7 paints must execute across 7 ticks (1 paint per tick)"
    );
}



