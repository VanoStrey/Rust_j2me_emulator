use j2me_core::heap::object::{ArrayData, Heap, ObjectData};
use j2me_core::heap::value::JavaValue;
use j2me_core::threads::green_thread::{GreenThread, ThreadId};
use j2me_core::threads::scheduler::Scheduler;
use j2me_core::vm::class_loader::ClassLoader;
use j2me_stdlib::bridge::context::{GraphicsTarget, VmContext};
use j2me_stdlib::bridge::native_bridge::NativeBridge;
use j2me_stdlib::midp::lcdui::graphics::Graphics;
use j2me_stdlib::J2meEmulator;

#[test]
fn test_string_constructor_bounds() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let bytes_id = heap.alloc_array(ArrayData::Byte(vec![65, 66, 67]));
    let str_id = heap.alloc_instance("java/lang/String");

    // Negative offset
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "<init>",
        "([BII)V",
        Some(str_id),
        &[
            JavaValue::Object(Some(bytes_id)),
            JavaValue::Int(-1),
            JavaValue::Int(2),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("StringIndexOutOfBoundsException"));

    // Length exceeds array bounds
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "<init>",
        "([BII)V",
        Some(str_id),
        &[
            JavaValue::Object(Some(bytes_id)),
            JavaValue::Int(1),
            JavaValue::Int(5),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("StringIndexOutOfBoundsException"));

    // Char array out of bounds
    let chars_id = heap.alloc_array(ArrayData::Char(vec!['A' as u16, 'B' as u16]));
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "<init>",
        "([CII)V",
        Some(str_id),
        &[
            JavaValue::Object(Some(chars_id)),
            JavaValue::Int(0),
            JavaValue::Int(5),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("StringIndexOutOfBoundsException"));
}

#[test]
fn test_string_utf16_surrogates_and_methods() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Surrogate pair: U+1F600 (GRINNING FACE) -> 0xD83D, 0xDE00 (length in Java is 2!)
    let str_id = heap.alloc_string("😀 Hello");

    // length() must return UTF-16 code units count: 2 (emoji) + 1 (space) + 5 ("Hello") = 8
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "length",
        "()I",
        Some(str_id),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();
    assert_eq!(res, Some(JavaValue::Int(8)));

    // charAt(0) returns high surrogate 0xD83D
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "charAt",
        "(I)C",
        Some(str_id),
        &[JavaValue::Int(0)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();
    assert_eq!(res, Some(JavaValue::Int(0xD83D)));

    // charAt out of bounds throws StringIndexOutOfBoundsException
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "charAt",
        "(I)C",
        Some(str_id),
        &[JavaValue::Int(8)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("StringIndexOutOfBoundsException"));

    // substring(0, 2) extracts the surrogate pair
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "substring",
        "(II)Ljava/lang/String;",
        Some(str_id),
        &[JavaValue::Int(0), JavaValue::Int(2)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();
    if let Some(JavaValue::Object(Some(sub_id))) = res {
        assert_eq!(heap.get_string_utf8(sub_id), Some("😀".to_string()));
    } else {
        panic!("Expected string object");
    }

    // getChars into char array
    let dst_arr = heap.alloc_array(ArrayData::Char(vec![0; 4]));
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "getChars",
        "(II[CI)V",
        Some(str_id),
        &[
            JavaValue::Int(0),
            JavaValue::Int(2),
            JavaValue::Object(Some(dst_arr)),
            JavaValue::Int(1),
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_ok());
    if let Some(ObjectData::Array(ArrayData::Char(c))) = heap.get(dst_arr) {
        assert_eq!(c[1], 0xD83D);
        assert_eq!(c[2], 0xDE00);
    } else {
        panic!("Expected char array");
    }

    // Java trim on whitespace and control chars <= 0x20
    let ws_str = heap.alloc_string("\r\n\t  Test \u{001F} ");
    let res = NativeBridge::dispatch(
        "java/lang/String",
        "trim",
        "()Ljava/lang/String;",
        Some(ws_str),
        &[],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    )
    .unwrap();
    if let Some(JavaValue::Object(Some(t_id))) = res {
        assert_eq!(heap.get_string_utf8(t_id), Some("Test".to_string()));
    }
}

#[test]
fn test_image_create_and_graphics_clipping_bounds() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    // Negative width must throw IllegalArgumentException
    let res = NativeBridge::dispatch(
        "javax/microedition/lcdui/Image",
        "createImage",
        "(II)Ljavax/microedition/lcdui/Image;",
        None,
        &[JavaValue::Int(-20), JavaValue::Int(50)],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("IllegalArgumentException"));

    // Graphics with 0 or negative bounds clamps without panicking
    let mut g = Graphics::new(0, 0);
    g.set_clip(10, 10, -5, -5);
    assert_eq!(g.clip_w, 0);
    assert_eq!(g.clip_h, 0);

    let mut g2 = Graphics::new(-10, -20);
    g2.set_clip(0, 0, 10, 10);
    assert_eq!(g2.clip_w, 0);
    assert_eq!(g2.clip_h, 0);
}

#[test]
fn test_nokia_direct_graphics_constants_and_aliases() {
    let mut heap = Heap::new();
    let mut cl = ClassLoader::new();
    let mut scheduler = Scheduler::new(1000);
    let mut context = VmContext::new(240, 320, 44100);

    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
    context.get_or_create_graphics(g_id, GraphicsTarget::Display);

    let dg_id = heap.alloc_instance("com/nokia/mid/ui/DirectGraphics");
    context.dg_to_graphics.insert(dg_id, g_id);

    // 1. Test INT format: 8888 (TYPE_INT_8888_ARGB) and 888 (TYPE_INT_888_RGB)
    let int_pixels = heap.alloc_array(ArrayData::Int(vec![0xFF00FF00u32 as i32; 100]));
    let res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([IZIIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(int_pixels)),
            JavaValue::Int(1),    // transparency
            JavaValue::Int(0),    // offset
            JavaValue::Int(10),   // scanlength
            JavaValue::Int(0),    // x
            JavaValue::Int(0),    // y
            JavaValue::Int(10),   // width
            JavaValue::Int(10),   // height
            JavaValue::Int(0),    // manipulation
            JavaValue::Int(8888), // official Nokia decimal format
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_ok(), "drawPixels with format 8888 must succeed");

    // 2. Test SHORT format: 4444 (TYPE_USHORT_4444_ARGB) and 565 (TYPE_USHORT_565_RGB)
    // with manipulation: FLIP_HORIZONTAL (8192) | ROTATE_90 (90) = 8282
    let short_pixels = heap.alloc_array(ArrayData::Short(vec![0x0F00; 100]));
    let res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([SZIIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(short_pixels)),
            JavaValue::Int(1),         // transparency
            JavaValue::Int(0),         // offset
            JavaValue::Int(10),        // scanlength
            JavaValue::Int(0),         // x
            JavaValue::Int(0),         // y
            JavaValue::Int(10),        // width
            JavaValue::Int(10),        // height
            JavaValue::Int(8192 | 90), // manipulation = FLIP_HORIZONTAL | ROTATE_90
            JavaValue::Int(4444),      // official Nokia decimal format
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(
        res.is_ok(),
        "drawPixels short with 4444 and 8192|90 must succeed"
    );

    // Check pixel write in framebuffer
    assert_ne!(
        context.framebuffer[0], 0,
        "Pixels should have been drawn to framebuffer"
    );

    // 3. Test getPixels with official format 8888
    let get_int_pixels = heap.alloc_array(ArrayData::Int(vec![0; 100]));
    let res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "getPixels",
        "([IIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(get_int_pixels)),
            JavaValue::Int(0),    // offset
            JavaValue::Int(10),   // scanlength
            JavaValue::Int(0),    // x
            JavaValue::Int(0),    // y
            JavaValue::Int(10),   // width
            JavaValue::Int(10),   // height
            JavaValue::Int(8888), // format
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_ok(), "getPixels int with format 8888 must succeed");

    // 4. Invalid format must throw IllegalArgumentException
    let res = NativeBridge::dispatch(
        "com/nokia/mid/ui/DirectGraphics",
        "drawPixels",
        "([IZIIIIIII)V",
        Some(dg_id),
        &[
            JavaValue::Object(Some(int_pixels)),
            JavaValue::Int(1),
            JavaValue::Int(0),
            JavaValue::Int(10),
            JavaValue::Int(0),
            JavaValue::Int(0),
            JavaValue::Int(10),
            JavaValue::Int(10),
            JavaValue::Int(0),
            JavaValue::Int(9999), // invalid format
        ],
        &mut heap,
        &mut cl,
        &mut scheduler,
        &mut context,
        0,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("IllegalArgumentException"));
}

#[test]
fn test_gamecanvas_suppress_key_events_contract() {
    assert_ne!(J2meEmulator::key_to_game_action(50), 0, "Key 2 maps to UP");
    assert_ne!(
        J2meEmulator::key_to_game_action(53),
        0,
        "Key 5 maps to FIRE"
    );
    assert_ne!(J2meEmulator::key_to_game_action(-1), 0, "Key UP maps to UP");
    assert_eq!(
        J2meEmulator::key_to_game_action(48),
        0,
        "Key 0 does not map to game action"
    );
    assert_eq!(
        J2meEmulator::key_to_game_action(42),
        0,
        "Key * does not map to game action"
    );
    assert_eq!(
        J2meEmulator::key_to_game_action(35),
        0,
        "Key # does not map to game action"
    );
}

#[test]
fn test_gc_graphics_retaining_target_contract() {
    let mut emu = J2meEmulator::new(240, 320, 44100);

    // 1. Allocate mutable Image and Graphics
    let img_id = emu.heap.alloc_instance("javax/microedition/lcdui/Image");
    let img = j2me_stdlib::midp::lcdui::image::Image::new_mutable(50, 50);
    emu.context.images.insert(img_id, img);

    let g_id = emu.heap.alloc_instance("javax/microedition/lcdui/Graphics");
    emu.context
        .get_or_create_graphics(g_id, GraphicsTarget::Image(img_id));

    // Put Graphics in a thread frame local so Graphics is reachable by GC
    let mut thread = GreenThread::new(
        ThreadId(1),
        "test-worker".to_string(),
        j2me_core::vm::frame::Frame::new(
            "TestClass".to_string(),
            "run".to_string(),
            "()V".to_string(),
            std::sync::Arc::new(vec![]),
            2,
            2,
            vec![],
            std::sync::Arc::new(std::collections::HashMap::new()),
        ),
        None,
    );
    thread.call_stack[0].set_local(0, JavaValue::Object(Some(g_id)));
    emu.scheduler.threads.push(thread);

    // Notice: img_id is NOT in locals or operand stack! Only g_id is reachable.
    // 2. Run GC
    let collected = emu.force_gc();
    assert_eq!(
        collected, 0,
        "img_id must be rooted by g_id target and NOT collected"
    );
    assert!(
        emu.heap.get(img_id).is_some(),
        "Image must still exist on heap"
    );
    assert!(
        emu.context.images.get(&img_id).is_some(),
        "Image pixel buffer must still exist in context"
    );

    // 3. Allocate new objects on heap
    let new_obj1 = emu.heap.alloc_instance("java/lang/Object");
    let new_obj2 = emu.heap.alloc_instance("java/lang/Object");
    assert_ne!(new_obj1, img_id);
    assert_ne!(new_obj2, img_id);

    // 4. Graphics can still draw to its target image without panic or corrupting new_obj
    let draw_res = NativeBridge::dispatch(
        "javax/microedition/lcdui/Graphics",
        "fillRect",
        "(IIII)V",
        Some(g_id),
        &[
            JavaValue::Int(0),
            JavaValue::Int(0),
            JavaValue::Int(10),
            JavaValue::Int(10),
        ],
        &mut emu.heap,
        &mut emu.class_loader,
        &mut emu.scheduler,
        &mut emu.context,
        0,
    );
    assert!(
        draw_res.is_ok(),
        "Drawing through Graphics after GC must succeed"
    );

    // 5. Verify the backing buffer actually received pixels and was not corrupted
    let img_ref = emu.context.images.get(&img_id).expect("Image must exist");
    assert_eq!(img_ref.width, 50);
    assert_eq!(img_ref.height, 50);
    assert_ne!(
        img_ref.pixels[0], 0,
        "fillRect must modify target image pixels"
    );
}
