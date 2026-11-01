use j2me_core::heap::object::ArrayData;
use j2me_core::heap::value::JavaValue;
use j2me_core::vm::frame::Frame;
use j2me_core::vm::instruction::Instruction;
use j2me_stdlib::bridge::native_bridge::NativeBridge;
use j2me_stdlib::J2meEmulator;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_string_value_of_and_string_buffer_append_extracts_real_values() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // 1. StringBuffer containing "test_string"
    let str_val = emu.heap.alloc_string("test_string");
    let sb_id = emu.heap.alloc_instance("java/lang/StringBuffer");
    emu.heap
        .set_field(sb_id, "str", JavaValue::Object(Some(str_val)));

    let extracted = NativeBridge::object_to_java_string(&emu.heap, sb_id);
    assert_eq!(
        extracted, "test_string",
        "object_to_java_string on StringBuffer must return its text content, not 'Object'"
    );

    // 2. Integer containing 42
    let int_id = emu.heap.alloc_instance("java/lang/Integer");
    emu.heap.set_field(int_id, "value", JavaValue::Int(42));
    let int_str = NativeBridge::object_to_java_string(&emu.heap, int_id);
    assert_eq!(
        int_str, "42",
        "object_to_java_string on Integer must return '42'"
    );

    // 3. Boolean true and false
    let bool_id = emu.heap.alloc_instance("java/lang/Boolean");
    emu.heap.set_field(bool_id, "value", JavaValue::Int(1));
    let bool_str = NativeBridge::object_to_java_string(&emu.heap, bool_id);
    assert_eq!(
        bool_str, "true",
        "object_to_java_string on Boolean true must return 'true'"
    );

    let bool_false_id = emu.heap.alloc_instance("java/lang/Boolean");
    emu.heap
        .set_field(bool_false_id, "value", JavaValue::Int(0));
    let bool_false_str = NativeBridge::object_to_java_string(&emu.heap, bool_false_id);
    assert_eq!(
        bool_false_str, "false",
        "object_to_java_string on Boolean false must return 'false'"
    );
}

#[test]
fn test_string_value_of_all_boxed_and_array_types() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // null -> "null"
    let res_null = NativeBridge::dispatch(
        "java/lang/String",
        "valueOf",
        "(Ljava/lang/Object;)Ljava/lang/String;",
        None,
        &[JavaValue::Object(None)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten())
    .and_then(|id| emu.heap.get_string_utf8(id))
    .unwrap();
    assert_eq!(res_null, "null", "String.valueOf(null) must return 'null'");

    // Boxed Long
    let long_id = emu.heap.alloc_instance("java/lang/Long");
    emu.heap
        .set_field(long_id, "value", JavaValue::Long(9876543210123));
    let res_long = NativeBridge::object_to_java_string(&emu.heap, long_id);
    assert_eq!(res_long, "9876543210123");

    // Boxed Byte, Short, Character
    let byte_id = emu.heap.alloc_instance("java/lang/Byte");
    emu.heap.set_field(byte_id, "value", JavaValue::Int(-128));
    assert_eq!(
        NativeBridge::object_to_java_string(&emu.heap, byte_id),
        "-128"
    );

    let short_id = emu.heap.alloc_instance("java/lang/Short");
    emu.heap.set_field(short_id, "value", JavaValue::Int(32767));
    assert_eq!(
        NativeBridge::object_to_java_string(&emu.heap, short_id),
        "32767"
    );

    let char_id = emu.heap.alloc_instance("java/lang/Character");
    emu.heap
        .set_field(char_id, "value", JavaValue::Int('Q' as u32 as i32));
    assert_eq!(NativeBridge::object_to_java_string(&emu.heap, char_id), "Q");

    // Primitive array: int[]
    let int_arr = emu.heap.alloc_array(ArrayData::Int(vec![1, 2, 3]));
    let arr_str = NativeBridge::object_to_java_string(&emu.heap, int_arr);
    assert!(
        arr_str.starts_with("[I@"),
        "int[] must format as [I@hash, got {}",
        arr_str
    );

    // Object array: String[]
    let str_ref = emu.heap.alloc_string("elem");
    let obj_arr = emu
        .heap
        .alloc_array(ArrayData::Reference(vec![Some(str_ref)]));
    let obj_arr_str = NativeBridge::object_to_java_string(&emu.heap, obj_arr);
    assert!(
        obj_arr_str.starts_with("[Ljava.lang.String;@")
            || obj_arr_str.starts_with("[Ljava/lang/String;@"),
        "String[] must format as [Ljava.lang.String;@hash, got {}",
        obj_arr_str
    );

    // Object without toString override
    let plain_obj = emu.heap.alloc_instance("com/game/Player");
    let plain_str = NativeBridge::object_to_java_string(&emu.heap, plain_obj);
    assert!(
        plain_str.starts_with("com.game.Player@"),
        "Plain object must format as ClassName@hash, got {}",
        plain_str
    );
}

#[test]
fn test_string_value_of_custom_guest_to_string_and_exception() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Define a class "CustomItem" with an overridden toString() method
    let custom_str = emu.heap.alloc_string("CustomItem[id=99]");
    let to_string_instructions = vec![
        Instruction::LdcString("CustomItem[id=99]".to_string()),
        Instruction::AReturn,
    ];
    let mut pc_map = HashMap::new();
    pc_map.insert(0, 0);
    pc_map.insert(1, 1);

    let method = j2me_core::vm::class_loader::PredecodedMethod {
        class_name: "CustomItem".to_string(),
        name: "toString".to_string(),
        descriptor: "()Ljava/lang/String;".to_string(),
        access_flags: 1, // public
        max_stack: 2,
        max_locals: 1,
        instructions: Arc::new(to_string_instructions),
        exception_table: Vec::new(),
        pc_map: Arc::new(pc_map),
    };
    emu.class_loader.predecoded_methods.insert(
        (
            "CustomItem".to_string(),
            "toString".to_string(),
            "()Ljava/lang/String;".to_string(),
        ),
        Arc::new(method),
    );

    let obj = emu.heap.alloc_instance("CustomItem");

    // Spawn a calling thread
    let caller_frame = Frame::new(
        "Main".to_string(),
        "run".to_string(),
        "()V".to_string(),
        Arc::new(vec![Instruction::Nop, Instruction::Return]),
        1,
        2,
        Vec::new(),
        Arc::new(HashMap::new()),
    );
    let tid = emu.scheduler.spawn("main", caller_frame, None);
    let thread_idx = tid.0 as usize - 1;

    // Call String.valueOf(obj)
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "valueOf",
        "(Ljava/lang/Object;)Ljava/lang/String;",
        None,
        &[JavaValue::Object(Some(obj))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        thread_idx,
    );
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        None,
        "Should push toString frame to execute guest method"
    );

    // Verify toString frame is pushed on call stack
    assert_eq!(emu.scheduler.threads[thread_idx].call_stack.len(), 2);
    assert_eq!(
        emu.scheduler.threads[thread_idx]
            .call_stack
            .last()
            .unwrap()
            .method_name,
        "toString"
    );

    // Step the emulator to execute toString and return value
    emu.is_running = true;
    emu.tick(50.0);

    let _ = custom_str;
}

#[test]
fn test_string_buffer_append_types_and_dynamic_path_construction() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    let sb_id = emu.heap.alloc_instance("java/lang/StringBuffer");
    let init_str = emu.heap.alloc_string("");
    emu.heap
        .set_field(sb_id, "str", JavaValue::Object(Some(init_str)));

    // 1. Append boolean true
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(Z)Ljava/lang/StringBuffer;",
        Some(sb_id),
        &[JavaValue::Int(1)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    // 2. Append char '/'
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(C)Ljava/lang/StringBuffer;",
        Some(sb_id),
        &[JavaValue::Int('/' as u32 as i32)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    // 3. Append int 42
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(I)Ljava/lang/StringBuffer;",
        Some(sb_id),
        &[JavaValue::Int(42)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    // 4. Append char[]
    let char_arr = emu.heap.alloc_array(ArrayData::Char(vec![
        '.' as u16, 'd' as u16, 'a' as u16, 't' as u16,
    ]));
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "([C)Ljava/lang/StringBuffer;",
        Some(sb_id),
        &[JavaValue::Object(Some(char_arr))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    let text = NativeBridge::object_to_java_string(&emu.heap, sb_id);
    assert_eq!(
        text, "true/42.dat",
        "StringBuffer append sequence must equal 'true/42.dat'"
    );

    // Test length and delete
    let len_res = NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "length",
        "()I",
        Some(sb_id),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();
    assert_eq!(len_res, Some(JavaValue::Int(11)));

    // Dynamic path construction for getResourceAsStream:
    let path_sb = emu.heap.alloc_instance("java/lang/StringBuffer");
    let p_empty = emu.heap.alloc_string("");
    emu.heap
        .set_field(path_sb, "str", JavaValue::Object(Some(p_empty)));

    let seg1 = emu.heap.alloc_string("/levels/level_");
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
        Some(path_sb),
        &[JavaValue::Object(Some(seg1))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(I)Ljava/lang/StringBuffer;",
        Some(path_sb),
        &[JavaValue::Int(1)],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    let seg2 = emu.heap.alloc_string(".bin");
    NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "append",
        "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
        Some(path_sb),
        &[JavaValue::Object(Some(seg2))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();

    // Call toString on path_sb
    let path_str_val = NativeBridge::dispatch(
        "java/lang/StringBuffer",
        "toString",
        "()Ljava/lang/String;",
        Some(path_sb),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .unwrap();

    // Populate mock JAR with "/levels/level_1.bin"
    let mut jar = j2me_core::jar::JarArchive {
        manifest: None,
        entries: HashMap::new(),
    };
    jar.entries
        .insert("levels/level_1.bin".to_string(), vec![0x11, 0x22, 0x33]);
    emu.context.jar = Some(jar);

    let class_obj = emu.heap.alloc_instance("java/lang/Class");
    let cname = emu.heap.alloc_string("com/game/Loader");
    emu.heap
        .set_field(class_obj, "name", JavaValue::Object(Some(cname)));

    let stream = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[path_str_val],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten());

    assert!(
        stream.is_some(),
        "Dynamic path constructed from StringBuffer must successfully resolve resource"
    );
}

#[test]
fn test_class_get_name_formats() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // Standard package class
    let class_obj = emu.heap.alloc_instance("java/lang/Class");
    let cname = emu.heap.alloc_string("ru/irnobi/mg/Check");
    emu.heap
        .set_field(class_obj, "name", JavaValue::Object(Some(cname)));

    let name_val = NativeBridge::dispatch(
        "java/lang/Class",
        "getName",
        "()Ljava/lang/String;",
        Some(class_obj),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten())
    .and_then(|id| emu.heap.get_string_utf8(id))
    .unwrap();

    assert_eq!(
        name_val, "ru.irnobi.mg.Check",
        "Class.getName() must return binary name with dots"
    );

    // Array type name
    let arr_class_obj = emu.heap.alloc_instance("java/lang/Class");
    let arr_cname = emu.heap.alloc_string("[Ljava/lang/String;");
    emu.heap
        .set_field(arr_class_obj, "name", JavaValue::Object(Some(arr_cname)));

    let arr_name_val = NativeBridge::dispatch(
        "java/lang/Class",
        "getName",
        "()Ljava/lang/String;",
        Some(arr_class_obj),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten())
    .and_then(|id| emu.heap.get_string_utf8(id))
    .unwrap();

    assert_eq!(
        arr_name_val, "[Ljava.lang.String;",
        "Array binary name must format with dots"
    );
}

#[test]
fn test_class_get_resource_as_stream_comprehensive() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    let mut jar = j2me_core::jar::JarArchive {
        manifest: None,
        entries: HashMap::new(),
    };
    // 1. Root resource
    jar.entries
        .insert("root_config.cfg".to_string(), vec![1, 2, 3]);
    // 2. Package resource
    jar.entries.insert(
        "com/game/pkg/data.bin".to_string(),
        vec![0x00, 0xFF, 0x00, 0x42],
    );
    // 3. Resource with dot segment
    jar.entries
        .insert("com/game/pkg/sub/asset.png".to_string(), vec![9, 8, 7]);
    emu.context.jar = Some(jar);

    let class_obj = emu.heap.alloc_instance("java/lang/Class");
    let cname = emu.heap.alloc_string("com/game/pkg/MyClass");
    emu.heap
        .set_field(class_obj, "name", JavaValue::Object(Some(cname)));

    // 1. Absolute path with '/'
    let abs_id = emu.heap.alloc_string("/root_config.cfg");
    let res1 = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(abs_id))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten());
    assert!(
        res1.is_some(),
        "Absolute path '/root_config.cfg' must resolve"
    );

    // 2. Package-relative path without '/'
    let rel_id = emu.heap.alloc_string("data.bin");
    let res2 = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(rel_id))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten());
    assert!(
        res2.is_some(),
        "Package-relative path 'data.bin' must resolve"
    );

    // Verify reading binary data with null bytes
    let stream_id = res2.unwrap();
    let b1 = NativeBridge::dispatch(
        "java/io/InputStream",
        "read",
        "()I",
        Some(stream_id),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();
    assert_eq!(b1, Some(JavaValue::Int(0x00)), "First byte must be 0x00");

    let b2 = NativeBridge::dispatch(
        "java/io/InputStream",
        "read",
        "()I",
        Some(stream_id),
        &[],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();
    assert_eq!(b2, Some(JavaValue::Int(0xFF)), "Second byte must be 0xFF");

    // 3. Path with '.' segments: "sub/./asset.png" from package "com/game/pkg"
    let dot_id = emu.heap.alloc_string("sub/./asset.png");
    let res3 = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(dot_id))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten());
    assert!(
        res3.is_some(),
        "Path with dot segment must normalize and resolve"
    );

    // 4. Missing resource must return null
    let missing_id = emu.heap.alloc_string("non_existent.dat");
    let res_miss = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(missing_id))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap();
    assert_eq!(
        res_miss,
        Some(JavaValue::NULL),
        "Missing resource must return null"
    );

    // 5. Case-insensitive fallback resolves DATA.BIN to data.bin
    let wrong_case_id = emu.heap.alloc_string("DATA.BIN");
    let res_case = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(wrong_case_id))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten());
    assert!(
        res_case.is_some(),
        "Case-insensitive fallback must resolve DATA.BIN to data.bin"
    );

    // 6. Repeated opening returns independent stream instance
    let rel_id_again = emu.heap.alloc_string("data.bin");
    let res_stream_2 = NativeBridge::dispatch(
        "java/lang/Class",
        "getResourceAsStream",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        Some(class_obj),
        &[JavaValue::Object(Some(rel_id_again))],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    )
    .unwrap()
    .and_then(|v| v.as_object().ok().flatten())
    .unwrap();

    assert_ne!(
        stream_id, res_stream_2,
        "Repeated getResourceAsStream must return fresh independent stream"
    );
    let pos2 = emu.heap.get_field(res_stream_2, "pos").as_int().unwrap();
    assert_eq!(pos2, 0, "Fresh stream must have position 0");
}
