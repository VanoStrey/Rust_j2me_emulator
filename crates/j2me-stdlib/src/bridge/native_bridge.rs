use crate::bridge::context::{
    FormState, GraphicsTarget, ListElement, ListState, StringItemState, TextBoxState,
    TiledLayerState, VmContext,
};
use crate::midp::lcdui::font::FontSpec;
use crate::midp::lcdui::image::{make_rgba, unpack_rgba, Image};
use crate::nokia::direct_graphics::DirectGraphics;
use j2me_core::heap::object::{ArrayData, Heap, ObjectData, ObjectId};
use j2me_core::heap::value::JavaValue;
use j2me_core::threads::green_thread::ThreadState;
use j2me_core::threads::scheduler::Scheduler;
use j2me_core::vm::class_loader::ClassLoader;
use j2me_core::vm::frame::Frame;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub enum ArrayCopyError {
    NullPointer,
    IndexOutOfBounds,
    ArrayStore,
}

#[inline]
pub(crate) fn with_graphics_target_mut<F>(context: &mut VmContext, target: GraphicsTarget, f: F)
where
    F: FnOnce(&mut [u32]),
{
    match target {
        GraphicsTarget::Display => {
            f(&mut context.framebuffer);
            context.screen_dirty = true;
        }
        GraphicsTarget::Image(t_id) => {
            if let Some(img) = context.images.get_mut(&t_id) {
                f(&mut img.pixels);
            }
        }
        GraphicsTarget::GameCanvas(c_id) => {
            if let Some(buf) = context.game_canvas_buffers.get_mut(&c_id) {
                f(buf);
            }
        }
    }
}

#[inline]
pub(crate) fn with_graphics_target_ref<F, R>(
    context: &VmContext,
    target: GraphicsTarget,
    f: F,
) -> Option<R>
where
    F: FnOnce(&[u32]) -> R,
{
    match target {
        GraphicsTarget::Display => Some(f(&context.framebuffer)),
        GraphicsTarget::Image(t_id) => context.images.get(&t_id).map(|img| f(&img.pixels)),
        GraphicsTarget::GameCanvas(c_id) => {
            context.game_canvas_buffers.get(&c_id).map(|buf| f(buf))
        }
    }
}

pub struct NativeBridge;

impl NativeBridge {
    pub fn object_to_java_string(heap: &Heap, oid: ObjectId) -> String {
        if let Some(s) = heap.get_string_utf8(oid) {
            s
        } else if let Some(ObjectData::Instance(inst)) = heap.get(oid) {
            if inst.class_name == "java/lang/StringBuffer"
                || inst.class_name == "java/lang/StringBuilder"
            {
                let sid = heap.get_field(oid, "str").as_object().ok().flatten();
                sid.and_then(|id| heap.get_string_utf8(id))
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Integer" {
                heap.get_field(oid, "value")
                    .as_int()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Long" {
                heap.get_field(oid, "value")
                    .as_long()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Boolean" {
                heap.get_field(oid, "value")
                    .as_int()
                    .map(|v| if v != 0 { "true" } else { "false" })
                    .unwrap_or("false")
                    .to_string()
            } else if inst.class_name == "java/lang/Byte" || inst.class_name == "java/lang/Short" {
                heap.get_field(oid, "value")
                    .as_int()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Character" {
                heap.get_field(oid, "value")
                    .as_int()
                    .ok()
                    .and_then(|v| char::from_u32(v as u32))
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Float" {
                heap.get_field(oid, "value")
                    .as_float()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Double" {
                heap.get_field(oid, "value")
                    .as_double()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            } else if inst.class_name == "java/lang/Class" {
                let name = heap
                    .get_field(oid, "name")
                    .as_object()
                    .ok()
                    .flatten()
                    .and_then(|id| heap.get_string_utf8(id))
                    .unwrap_or_else(|| "java.lang.Class".to_string());
                format!("class {}", name.replace('/', "."))
            } else {
                format!("{}@{:x}", inst.class_name.replace('/', "."), oid.0)
            }
        } else if let Some(ObjectData::Array(arr)) = heap.get(oid) {
            match arr {
                ArrayData::Boolean(_) => format!("[Z@{:x}", oid.0),
                ArrayData::Byte(_) => format!("[B@{:x}", oid.0),
                ArrayData::Char(_) => format!("[C@{:x}", oid.0),
                ArrayData::Short(_) => format!("[S@{:x}", oid.0),
                ArrayData::Int(_) => format!("[I@{:x}", oid.0),
                ArrayData::Long(_) => format!("[J@{:x}", oid.0),
                ArrayData::Float(_) => format!("[F@{:x}", oid.0),
                ArrayData::Double(_) => format!("[D@{:x}", oid.0),
                ArrayData::Reference(elements) => {
                    let elem_cls = elements
                        .iter()
                        .filter_map(|opt| *opt)
                        .find_map(|id| match heap.get(id) {
                            Some(ObjectData::Instance(i)) => Some(i.class_name.clone()),
                            Some(ObjectData::String(_)) => Some("java/lang/String".to_string()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "java/lang/Object".to_string());
                    format!("[L{};@{:x}", elem_cls.replace('/', "."), oid.0)
                }
            }
        } else {
            format!("Object@{:x}", oid.0)
        }
    }

    pub fn normalize_jar_resource_path(path: &str) -> String {
        let mut segments = Vec::new();
        for seg in path.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            } else if seg == ".." {
                segments.pop();
            } else {
                segments.push(seg);
            }
        }
        segments.join("/")
    }

    pub fn validate_direct_graphics_bounds(
        offset: i32,
        scanlength: i32,
        width: i32,
        height: i32,
        arr_len: usize,
    ) -> bool {
        if offset < 0 {
            return false;
        }
        if width > 0 && height > 0 {
            let h_sub_1 = (height as i64) - 1;
            let w_sub_1 = (width as i64) - 1;
            let scan64 = scanlength as i64;
            let off64 = offset as i64;
            let stride_prod = match h_sub_1.checked_mul(scan64) {
                Some(p) => p,
                None => return false,
            };
            let last_start = match off64.checked_add(stride_prod) {
                Some(s) => s,
                None => return false,
            };
            let min_idx = off64.min(last_start);
            let max_start = off64.max(last_start);
            let max_idx = match max_start.checked_add(w_sub_1) {
                Some(m) => m,
                None => return false,
            };
            if min_idx < 0 || max_idx >= (arr_len as i64) {
                return false;
            }
        }
        true
    }

    pub fn validate_direct_graphics_byte_bounds(
        offset: i32,
        scanlength: i32,
        width: i32,
        height: i32,
        arr_len: usize,
        format: i32,
    ) -> bool {
        if offset < 0 {
            return false;
        }
        if width <= 0 || height <= 0 {
            return true;
        }
        if Self::validate_direct_graphics_bounds(offset, scanlength, width, height, arr_len) {
            return true;
        }
        let off = offset as usize;
        let w = width as usize;
        let h = height as usize;
        let scan = scanlength.max(0) as usize;
        match format {
            DirectGraphics::TYPE_BYTE_1_GRAY => {
                let bytes_needed_pixels = off + ((h - 1) * scan + w + 7) / 8;
                let bytes_needed_bytes = off + (h - 1) * scan + (w + 7) / 8;
                bytes_needed_pixels <= arr_len || bytes_needed_bytes <= arr_len
            }
            DirectGraphics::TYPE_BYTE_1_GRAY_VERTICAL => {
                let bytes_needed = off + ((h + 7) / 8 - 1) * scan + w;
                bytes_needed <= arr_len
            }
            DirectGraphics::TYPE_BYTE_2_GRAY => {
                let bytes_needed = off + ((h - 1) * scan + w + 3) / 4;
                bytes_needed <= arr_len
            }
            DirectGraphics::TYPE_BYTE_4_GRAY => {
                let bytes_needed = off + ((h - 1) * scan + w + 1) / 2;
                bytes_needed <= arr_len
            }
            _ => false,
        }
    }

    pub fn dispatch(
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        receiver: Option<ObjectId>,
        args: &[JavaValue],
        heap: &mut Heap,
        class_loader: &mut ClassLoader,
        scheduler: &mut Scheduler,
        context: &mut VmContext,
        thread_idx: usize,
    ) -> Result<Option<JavaValue>, String> {
        let cls = class_name.replace('.', "/");

        // 1. java/lang/System
        if cls == "java/lang/System" {
            match method_name {
                "currentTimeMillis" => {
                    return Ok(Some(JavaValue::Long(scheduler.current_time_ms as i64)));
                }
                "identityHashCode" => {
                    let id = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .map(|o| o.0 as i32)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(id)));
                }
                "gc" => {
                    return Ok(None);
                }
                "arraycopy" => match Self::handle_arraycopy(args, heap) {
                    Ok(()) => return Ok(None),
                    Err(ArrayCopyError::NullPointer) => {
                        let exc_id = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc_id.0));
                    }
                    Err(ArrayCopyError::IndexOutOfBounds) => {
                        let exc_id = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/IndexOutOfBoundsException:{}",
                            exc_id.0
                        ));
                    }
                    Err(ArrayCopyError::ArrayStore) => {
                        let exc_id = heap.alloc_instance("java/lang/ArrayStoreException");
                        return Err(format!("EXC:java/lang/ArrayStoreException:{}", exc_id.0));
                    }
                },
                "getProperty" => {
                    let key_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let key = key_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let val: Option<&str> = match key.as_str() {
                        "microedition.platform" => Some("Nokia6230i/3.80"),
                        "microedition.profiles" => Some("MIDP-2.0"),
                        "microedition.configuration" => Some("CLDC-1.1"),
                        "microedition.locale" => Some("en-US"),
                        "microedition.encoding" => Some("ISO-8859-1"),
                        "file.separator" => Some("/"),
                        "path.separator" => Some(":"),
                        "line.separator" => Some("\n"),
                        "microedition.commports" => Some(""),
                        "microedition.hostname" => Some("localhost"),
                        "microedition.jtwi.version" => Some("1.0"),
                        "microedition.m3g.version" => Some("1.1"),
                        "microedition.media.version" => Some("1.1"),
                        "com.nokia.mid.dateformat" => Some("dd.MM.yyyy"),
                        "com.nokia.mid.timeformat" => Some("HH:mm:ss"),
                        "com.nokia.network.access" => Some("true"),
                        "com.nokia.mid.ui.softkey.height" => Some("24"),
                        _ => None,
                    };
                    if let Some(v) = val {
                        let str_id = heap.alloc_string(v);
                        return Ok(Some(JavaValue::Object(Some(str_id))));
                    }
                    if let Some(prop) = context.get_app_property(&key) {
                        let str_id = heap.alloc_string(prop);
                        return Ok(Some(JavaValue::Object(Some(str_id))));
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "exit" => return Ok(None),
                _ => {}
            }
        }

        // java/io/PrintStream
        if cls == "java/io/PrintStream" {
            match method_name {
                "println" | "print" => {
                    if let Some(arg) = args.first() {
                        let text = match arg {
                            JavaValue::Object(Some(id)) => heap
                                .get_string_utf8(*id)
                                .unwrap_or_else(|| format!("Object({:?})", id)),
                            JavaValue::Object(None) => "null".to_string(),
                            JavaValue::Int(v) => v.to_string(),
                            JavaValue::Long(v) => v.to_string(),
                            JavaValue::Float(v) => v.to_string(),
                            JavaValue::Double(v) => v.to_string(),
                        };
                        println!("[STDOUT] {}", text);
                    } else {
                        println!("[STDOUT]");
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        // 2. java/lang/Runtime
        if cls == "java/lang/Runtime" {
            match method_name {
                "getRuntime" => {
                    let id = heap.alloc_instance("java/lang/Runtime");
                    return Ok(Some(JavaValue::Object(Some(id))));
                }
                "totalMemory" => return Ok(Some(JavaValue::Long(32 * 1024 * 1024))), // 32MB
                "freeMemory" => return Ok(Some(JavaValue::Long(16 * 1024 * 1024))),  // 16MB
                "gc" => return Ok(None),
                _ => {}
            }
        }

        // 3. java/lang/Object methods & framework constructors (inherited/called by all classes)
        match method_name {
            "<init>" if cls == "javax/microedition/lcdui/game/GameCanvas" => {
                let suppress = if let Some(arg) = args.first() {
                    arg.as_int().unwrap_or(0) != 0
                } else {
                    false
                };
                if let Some(c_id) = receiver {
                    context.game_canvas_suppress_keys.insert(c_id, suppress);
                    context.get_or_create_game_canvas_buffer(c_id);
                }
                context.suppress_key_events = suppress;
                return Ok(None);
            }
            "<init>"
                if cls == "java/lang/Object"
                    || cls == "javax/microedition/midlet/MIDlet"
                    || cls == "javax/microedition/lcdui/Displayable"
                    || cls == "javax/microedition/lcdui/Canvas"
                    || cls == "com/nokia/mid/ui/FullCanvas"
                    || cls == "java/util/TimerTask"
                    || cls.ends_with("Exception")
                    || cls.ends_with("Error")
                    || cls.ends_with("Throwable") =>
            {
                return Ok(None);
            }
            "getClass" if args.is_empty() => {
                let rec_cls = if let Some(id) = receiver {
                    if let Some(obj) = heap.get(id) {
                        match obj {
                            ObjectData::Instance(inst) => inst.class_name.clone(),
                            ObjectData::Array(arr) => match arr {
                                ArrayData::Boolean(_) => "[Z".to_string(),
                                ArrayData::Byte(_) => "[B".to_string(),
                                ArrayData::Char(_) => "[C".to_string(),
                                ArrayData::Short(_) => "[S".to_string(),
                                ArrayData::Int(_) => "[I".to_string(),
                                ArrayData::Long(_) => "[J".to_string(),
                                ArrayData::Float(_) => "[F".to_string(),
                                ArrayData::Double(_) => "[D".to_string(),
                                ArrayData::Reference(elements) => {
                                    let elem_cls = elements
                                        .iter()
                                        .filter_map(|opt| *opt)
                                        .find_map(|id| {
                                            if let Some(ObjectData::Instance(i)) = heap.get(id) {
                                                Some(i.class_name.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| "java/lang/Object".to_string());
                                    format!("[L{};", elem_cls.replace('/', "."))
                                }
                            },
                            ObjectData::String(_) => "java/lang/String".to_string(),
                        }
                    } else {
                        cls.to_string()
                    }
                } else {
                    cls.to_string()
                };
                let cls_obj = class_loader.get_or_create_class_object(heap, &rec_cls);
                return Ok(Some(JavaValue::Object(Some(cls_obj))));
            }
            "hashCode" if args.is_empty() => {
                let code = receiver.map(|id| id.0 as i32).unwrap_or(0);
                return Ok(Some(JavaValue::Int(code)));
            }
            "equals" if args.len() == 1 => {
                let other = args[0].as_object().ok().flatten();
                let eq = Self::java_objects_equal(heap, receiver, other);
                return Ok(Some(JavaValue::Int(if eq { 1 } else { 0 })));
            }
            "toString" if args.is_empty() => {
                if cls == "java/lang/String" {
                    return Ok(Some(JavaValue::Object(receiver)));
                }
                if cls == "java/lang/StringBuffer" || cls == "java/lang/StringBuilder" {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let sid = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .unwrap_or(r_id);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                let s = if let Some(r_id) = receiver {
                    Self::object_to_java_string(heap, r_id)
                } else {
                    "null".to_string()
                };
                let sid = heap.alloc_string(&s);
                return Ok(Some(JavaValue::Object(Some(sid))));
            }
            "wait" => {
                let ms = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                if ms < 0 {
                    let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                    return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                }
                let m = match receiver {
                    Some(m) => m,
                    None => {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                };
                let tid = scheduler.threads[thread_idx].id;
                let wake_ms = if ms > 0 {
                    Some(scheduler.current_time_ms + ms as u64)
                } else {
                    // Safe spurious wakeup timeout for wait(0) to prevent eternal deadlock (JLS §17.8.1)
                    Some(scheduler.current_time_ms + 1000)
                };
                if let Err(_) = scheduler.wait_monitor(tid, m, wake_ms) {
                    let exc = heap.alloc_instance("java/lang/IllegalMonitorStateException");
                    return Err(format!(
                        "EXC:java/lang/IllegalMonitorStateException:{}",
                        exc.0
                    ));
                }
                return Ok(None);
            }
            "notify" => {
                let m = match receiver {
                    Some(m) => m,
                    None => {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                };
                let tid = scheduler.threads[thread_idx].id;
                if let Err(_) = scheduler.notify_monitor(tid, m) {
                    let exc = heap.alloc_instance("java/lang/IllegalMonitorStateException");
                    return Err(format!(
                        "EXC:java/lang/IllegalMonitorStateException:{}",
                        exc.0
                    ));
                }
                return Ok(None);
            }
            "notifyAll" => {
                let m = match receiver {
                    Some(m) => m,
                    None => {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                };
                let tid = scheduler.threads[thread_idx].id;
                if let Err(_) = scheduler.notify_all_monitor(tid, m) {
                    let exc = heap.alloc_instance("java/lang/IllegalMonitorStateException");
                    return Err(format!(
                        "EXC:java/lang/IllegalMonitorStateException:{}",
                        exc.0
                    ));
                }
                return Ok(None);
            }
            _ => {}
        }

        if method_name == "getResourceAsStream" {
            let path_id = args.first().and_then(|v| v.as_object().ok()).flatten();
            let raw_path = path_id
                .and_then(|id| heap.get_string_utf8(id))
                .unwrap_or_default();
            if raw_path.is_empty() {
                return Ok(Some(JavaValue::NULL));
            }

            if let Some(ref jar) = context.jar {
                let canonical_path = if raw_path.starts_with('/') {
                    Self::normalize_jar_resource_path(&raw_path[1..])
                } else {
                    let pkg_prefix = receiver
                        .and_then(|id| heap.get_field(id, "name").as_object().ok().flatten())
                        .and_then(|id| heap.get_string_utf8(id))
                        .map(|name| {
                            let norm = name.replace('.', "/");
                            if let Some(idx) = norm.rfind('/') {
                                norm[..idx].to_string()
                            } else {
                                String::new()
                            }
                        })
                        .unwrap_or_default();

                    let combined = if pkg_prefix.is_empty() {
                        raw_path.clone()
                    } else {
                        format!("{}/{}", pkg_prefix, raw_path)
                    };
                    Self::normalize_jar_resource_path(&combined)
                };

                let entry_data = jar
                    .get_entry(&canonical_path)
                    .or_else(|| jar.get_entry(&raw_path));

                if let Some(entry_data) = entry_data {
                    let byte_arr = heap.alloc_array(ArrayData::Byte(
                        entry_data.iter().map(|&b| b as i8).collect(),
                    ));
                    let stream_id = heap.alloc_instance("java/io/ByteArrayInputStream");
                    heap.set_field(stream_id, "buf", JavaValue::Object(Some(byte_arr)));
                    heap.set_field(stream_id, "count", JavaValue::Int(entry_data.len() as i32));
                    heap.set_field(stream_id, "pos", JavaValue::Int(0));
                    heap.set_field(stream_id, "mark", JavaValue::Int(0));
                    return Ok(Some(JavaValue::Object(Some(stream_id))));
                }
            }
            return Ok(Some(JavaValue::NULL));
        }

        if method_name == "getAppProperty" {
            let key_id = args.first().and_then(|v| v.as_object().ok()).flatten();
            let key = key_id
                .and_then(|id| heap.get_string_utf8(id))
                .unwrap_or_default();
            if let Some(val) = context.get_app_property(&key) {
                let val_id = heap.alloc_string(val);
                return Ok(Some(JavaValue::Object(Some(val_id))));
            }
            if std::env::var_os("J2ME_DEBUG_PROPS").is_some() {
                eprintln!("[PROP] MISSING key='{}'", key);
            }
            return Ok(Some(JavaValue::NULL));
        }

        // 4. java/lang/Class
        if cls == "java/lang/Class" {
            match method_name {
                "getName" => {
                    let name = receiver
                        .and_then(|id| heap.get_field(id, "name").as_object().ok().flatten())
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_else(|| cls.clone());
                    let binary_name = if name.starts_with('[') {
                        // Array descriptor: replace internal slashes with dots in class reference,
                        // e.g. "[Ljava/lang/String;" -> "[Ljava.lang.String;", "[B" -> "[B"
                        name.replace('/', ".")
                    } else {
                        name.replace('/', ".")
                    };
                    let name_id = heap.alloc_string(&binary_name);
                    return Ok(Some(JavaValue::Object(Some(name_id))));
                }
                "forName" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let cls_name = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let cls_obj = class_loader.get_or_create_class_object(heap, &cls_name);
                    return Ok(Some(JavaValue::Object(Some(cls_obj))));
                }
                "newInstance" => {
                    let target_class = receiver
                        .and_then(|id| {
                            heap.get_field(id, "target_class")
                                .as_object()
                                .ok()
                                .flatten()
                        })
                        .and_then(|id| heap.get_string_utf8(id))
                        .or_else(|| {
                            receiver
                                .and_then(|id| {
                                    heap.get_field(id, "name").as_object().ok().flatten()
                                })
                                .and_then(|id| heap.get_string_utf8(id))
                                .map(|s| s.replace('.', "/"))
                        })
                        .unwrap_or_else(|| "java/lang/Object".to_string());
                    let new_id = heap.alloc_instance(&target_class);
                    let has_init = if let Some(init_method) =
                        class_loader.find_method(&target_class, "<init>", "()V")
                    {
                        if !init_method.is_native() {
                            if let Some(caller_frame) =
                                scheduler.threads[thread_idx].current_frame_mut()
                            {
                                caller_frame.push(JavaValue::Object(Some(new_id)));
                            }
                            let mut new_frame = Frame::new(
                                target_class.clone(),
                                "<init>".to_string(),
                                "()V".to_string(),
                                Arc::clone(&init_method.instructions),
                                init_method.max_locals,
                                init_method.max_stack,
                                init_method.exception_table.clone(),
                                Arc::clone(&init_method.pc_map),
                            );
                            new_frame.set_local(0, JavaValue::Object(Some(new_id)));
                            if init_method.is_synchronized() {
                                new_frame.sync_monitor = Some(new_id);
                            }
                            scheduler.threads[thread_idx].call_stack.push(new_frame);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if has_init {
                        return Ok(None);
                    }
                    return Ok(Some(JavaValue::Object(Some(new_id))));
                }
                "isInstance" => {
                    return Ok(Some(JavaValue::Int(1)));
                }
                _ => {}
            }
        }

        // 5. java/lang/Throwable
        if cls == "java/lang/Throwable" || class_loader.is_instance_of(&cls, "java/lang/Throwable")
        {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if let Some(msg) = args.first() {
                            heap.set_field(r_id, "detailMessage", *msg);
                        }
                    }
                    return Ok(None);
                }
                "getMessage" => {
                    let msg = receiver.and_then(|r| {
                        heap.get_field(r, "detailMessage")
                            .as_object()
                            .ok()
                            .flatten()
                    });
                    return Ok(Some(JavaValue::Object(msg)));
                }
                "toString" => {
                    let cls_opt = receiver.and_then(|r| {
                        if let Some(ObjectData::Instance(inst)) = heap.get(r) {
                            Some(inst.class_name.clone())
                        } else {
                            None
                        }
                    });
                    let msg = cls_opt.map(|c| heap.alloc_string(&c));
                    return Ok(Some(JavaValue::Object(msg)));
                }
                "printStackTrace" => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 6. java/lang/Thread
        if cls == "java/lang/Thread" || class_loader.is_instance_of(&cls, "java/lang/Thread") {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if let Some(target) = args.first() {
                            if let Ok(Some(obj_id)) = target.as_object() {
                                let is_string = heap
                                    .get(obj_id)
                                    .map(|o| match o {
                                        ObjectData::Instance(i) => {
                                            i.class_name == "java/lang/String"
                                        }
                                        _ => false,
                                    })
                                    .unwrap_or(false);
                                if !is_string {
                                    heap.set_field(r_id, "target", *target);
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "start" => {
                    if let Some(r_id) = receiver {
                        let target_obj = heap
                            .get_field(r_id, "target")
                            .as_object()
                            .ok()
                            .flatten()
                            .unwrap_or(r_id);
                        let target_cls =
                            if let Some(ObjectData::Instance(inst)) = heap.get(target_obj) {
                                inst.class_name.clone()
                            } else {
                                "java/lang/Runnable".to_string()
                            };

                        if let Some(run_m) = class_loader.find_method(&target_cls, "run", "()V") {
                            let mut run_frame = Frame::new(
                                target_cls.clone(),
                                "run".to_string(),
                                "()V".to_string(),
                                Arc::clone(&run_m.instructions),
                                run_m.max_locals,
                                run_m.max_stack,
                                run_m.exception_table.clone(),
                                Arc::clone(&run_m.pc_map),
                            );
                            run_frame.set_local(0, JavaValue::Object(Some(target_obj)));
                            if run_m.is_synchronized() {
                                run_frame.sync_monitor = Some(target_obj);
                            }
                            let _new_tid = scheduler.spawn(
                                &format!("{}-run", target_cls),
                                run_frame,
                                Some(r_id),
                            );
                        }
                    }
                    return Ok(None);
                }
                "sleep" => {
                    let ms = args
                        .first()
                        .and_then(|v| match v {
                            JavaValue::Long(l) => Some(*l),
                            JavaValue::Int(i) => Some(*i as i64),
                            _ => v.as_long().ok(),
                        })
                        .unwrap_or(0);
                    if ms < 0 {
                        let exc_id = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!(
                            "EXC:java/lang/IllegalArgumentException:{}",
                            exc_id.0
                        ));
                    }
                    if std::env::var_os("J2ME_DEBUG_THREADS").is_some() {
                        eprintln!(
                            "[THREAD SLEEP] thread={} ms={}",
                            scheduler
                                .threads
                                .get(thread_idx)
                                .map(|t| t.name.as_str())
                                .unwrap_or("?"),
                            ms
                        );
                    }
                    if ms <= 15 && context.repaints_this_tick >= context.max_repaints_per_tick {
                        context.tick_budget_exhausted = true;
                    }
                    if ms > 0 && thread_idx < scheduler.threads.len() {
                        scheduler.threads[thread_idx].consecutive_yields = 0;
                        scheduler.threads[thread_idx].state = ThreadState::Sleeping {
                            wake_time_ms: scheduler.current_time_ms + ms as u64,
                        };
                    } else if thread_idx < scheduler.threads.len() {
                        scheduler.threads[thread_idx].on_yield();
                        scheduler.threads[thread_idx].state = ThreadState::Ready;
                    }
                    return Ok(None);
                }
                "yield" => {
                    if thread_idx < scheduler.threads.len() {
                        scheduler.threads[thread_idx].on_yield();
                        scheduler.threads[thread_idx].state = ThreadState::Ready;
                    }
                    if context.repaints_this_tick >= context.max_repaints_per_tick {
                        context.tick_budget_exhausted = true;
                    }
                    return Ok(None);
                }
                "isAlive" => {
                    let alive = receiver
                        .map(|rid| {
                            scheduler.threads.iter().any(|t| {
                                (t.java_thread_obj == Some(rid)
                                    || heap.get_field(rid, "target").as_object().ok().flatten()
                                        == t.java_thread_obj)
                                    && t.state != ThreadState::Terminated
                            })
                        })
                        .unwrap_or(false);
                    return Ok(Some(JavaValue::Int(if alive { 1 } else { 0 })));
                }
                "join" => {
                    if let Some(target_rid) = receiver {
                        let is_target_alive = scheduler.threads.iter().any(|t| {
                            (t.java_thread_obj == Some(target_rid)
                                || heap.get_field(target_rid, "target").as_object().ok().flatten()
                                    == t.java_thread_obj)
                                && t.state != ThreadState::Terminated
                                && t.id != scheduler.threads[thread_idx].id
                        });
                        if is_target_alive {
                            let timeout_ms = args
                                .first()
                                .and_then(|v| match v {
                                    JavaValue::Long(l) => Some(*l as u64),
                                    JavaValue::Int(i) => Some(*i as u64),
                                    _ => None,
                                })
                                .filter(|&t| t > 0)
                                .unwrap_or(5000);
                            scheduler.threads[thread_idx].state = ThreadState::Waiting {
                                monitor: Some(target_rid),
                                timeout_wake_ms: Some(scheduler.current_time_ms + timeout_ms),
                                saved_recursion: 0,
                            };
                        }
                    }
                    return Ok(None);
                }
                "setPriority" | "getPriority" => return Ok(Some(JavaValue::Int(5))),
                "currentThread" => {
                    let t_id = receiver.unwrap_or(ObjectId(1));
                    return Ok(Some(JavaValue::Object(Some(t_id))));
                }
                _ => {}
            }
        }

        // 6. java/lang/Math
        if cls == "java/lang/Math" {
            match method_name {
                "sin" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.sin())));
                }
                "cos" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.cos())));
                }
                "tan" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.tan())));
                }
                "sqrt" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.sqrt())));
                }
                "toRadians" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.to_radians())));
                }
                "toDegrees" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.to_degrees())));
                }
                "ceil" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.ceil())));
                }
                "floor" => {
                    let a = args[0].as_double().unwrap_or(0.0);
                    return Ok(Some(JavaValue::Double(a.floor())));
                }
                "abs" => {
                    if let Ok(i) = args[0].as_int() {
                        return Ok(Some(JavaValue::Int(i.abs())));
                    }
                    if let Ok(l) = args[0].as_long() {
                        return Ok(Some(JavaValue::Long(l.abs())));
                    }
                    if let Ok(f) = args[0].as_float() {
                        return Ok(Some(JavaValue::Float(f.abs())));
                    }
                    if let Ok(d) = args[0].as_double() {
                        return Ok(Some(JavaValue::Double(d.abs())));
                    }
                }
                "min" => {
                    if let (Ok(a), Ok(b)) = (args[0].as_int(), args[1].as_int()) {
                        return Ok(Some(JavaValue::Int(a.min(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_long(), args[1].as_long()) {
                        return Ok(Some(JavaValue::Long(a.min(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_float(), args[1].as_float()) {
                        return Ok(Some(JavaValue::Float(a.min(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_double(), args[1].as_double()) {
                        return Ok(Some(JavaValue::Double(a.min(b))));
                    }
                }
                "max" => {
                    if let (Ok(a), Ok(b)) = (args[0].as_int(), args[1].as_int()) {
                        return Ok(Some(JavaValue::Int(a.max(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_long(), args[1].as_long()) {
                        return Ok(Some(JavaValue::Long(a.max(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_float(), args[1].as_float()) {
                        return Ok(Some(JavaValue::Float(a.max(b))));
                    }
                    if let (Ok(a), Ok(b)) = (args[0].as_double(), args[1].as_double()) {
                        return Ok(Some(JavaValue::Double(a.max(b))));
                    }
                }
                _ => {}
            }
        }

        let is_data_input_stream = cls == "java/io/DataInputStream"
            || class_loader.is_instance_of(&cls, "java/io/DataInputStream");
        let is_other_input_stream = !is_data_input_stream
            && (cls.ends_with("InputStream")
                || cls == "java/io/InputStream"
                || cls == "java/io/ByteArrayInputStream"
                || class_loader.is_instance_of(&cls, "java/io/InputStream"));

        // 7. java/io/InputStream & java/io/ByteArrayInputStream
        if is_other_input_stream {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if !args.is_empty() {
                            let buf_val = match args[0].as_object().ok().flatten() {
                                Some(id) => JavaValue::Object(Some(id)),
                                None => {
                                    let empty_arr = heap.alloc_array(ArrayData::Byte(Vec::new()));
                                    JavaValue::Object(Some(empty_arr))
                                }
                            };
                            heap.set_field(r_id, "buf", buf_val);
                            let offset = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                            heap.set_field(r_id, "pos", JavaValue::Int(offset));
                            heap.set_field(r_id, "mark", JavaValue::Int(offset));
                            let count = if args.len() >= 3 {
                                let len = args[2].as_int().unwrap_or(0);
                                offset + len
                            } else {
                                if let Ok(Some(arr_id)) = buf_val.as_object() {
                                    heap.get(arr_id)
                                        .and_then(|o| o.as_array())
                                        .map(|a| a.len() as i32)
                                        .unwrap_or(0)
                                } else {
                                    0
                                }
                            };
                            heap.set_field(r_id, "count", JavaValue::Int(count));
                        }
                    }
                    return Ok(None);
                }
                "read" => {
                    let r_id = receiver
                        .map(|id| Self::resolve_stream_source(heap, id))
                        .unwrap_or(ObjectId(0));
                    let pos = heap.get_field(r_id, "pos").as_int().unwrap_or(0);
                    let count = heap.get_field(r_id, "count").as_int().unwrap_or(0);
                    let buf_id = heap.get_field(r_id, "buf").as_object().ok().flatten();

                    if args.is_empty() {
                        // read() -> int
                        if pos >= count {
                            return Ok(Some(JavaValue::Int(-1)));
                        }
                        if let Some(bid) = buf_id {
                            if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(bid) {
                                if (pos as usize) < b.len() {
                                    let val = b[pos as usize] as u8 as i32;
                                    heap.set_field(r_id, "pos", JavaValue::Int(pos + 1));
                                    return Ok(Some(JavaValue::Int(val)));
                                }
                            }
                        }
                        return Ok(Some(JavaValue::Int(-1)));
                    } else {
                        // read(byte[] b) or read(byte[] b, int off, int len)
                        let dst_arr_id = args[0].as_object().ok().flatten();
                        let dst_off =
                            args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                        let dst_len = if args.len() >= 3 {
                            args[2].as_int().unwrap_or(0) as usize
                        } else {
                            dst_arr_id
                                .and_then(|id| heap.get(id))
                                .and_then(|o| o.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0)
                        };

                        if pos >= count {
                            return Ok(Some(JavaValue::Int(-1)));
                        }
                        let avail = (count - pos) as usize;
                        let to_read = dst_len.min(avail);
                        if to_read == 0 {
                            return Ok(Some(JavaValue::Int(0)));
                        }

                        let src_bytes: Option<Vec<i8>> =
                            buf_id.and_then(|bid| match heap.get(bid) {
                                Some(ObjectData::Array(ArrayData::Byte(b))) => {
                                    let p = pos as usize;
                                    Some(b[p..p + to_read].to_vec())
                                }
                                _ => None,
                            });

                        if let (Some(bytes), Some(dst_id)) = (src_bytes, dst_arr_id) {
                            if let Some(ObjectData::Array(ArrayData::Byte(dst))) =
                                heap.get_mut(dst_id)
                            {
                                if dst_off + to_read <= dst.len() {
                                    dst[dst_off..dst_off + to_read].copy_from_slice(&bytes[..]);
                                    heap.set_field(
                                        r_id,
                                        "pos",
                                        JavaValue::Int(pos + to_read as i32),
                                    );
                                    return Ok(Some(JavaValue::Int(to_read as i32)));
                                }
                            }
                        }
                        return Ok(Some(JavaValue::Int(-1)));
                    }
                }
                "skip" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let n = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0) as i32;
                    let pos = heap.get_field(r_id, "pos").as_int().unwrap_or(0);
                    let count = heap.get_field(r_id, "count").as_int().unwrap_or(0);
                    let actual = n.min(count - pos).max(0);
                    heap.set_field(r_id, "pos", JavaValue::Int(pos + actual));
                    return Ok(Some(JavaValue::Long(actual as i64)));
                }
                "available" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let pos = heap.get_field(r_id, "pos").as_int().unwrap_or(0);
                    let count = heap.get_field(r_id, "count").as_int().unwrap_or(0);
                    return Ok(Some(JavaValue::Int((count - pos).max(0))));
                }
                "close" => return Ok(None),
                "reset" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let mark = heap.get_field(r_id, "mark").as_int().unwrap_or(0);
                    heap.set_field(r_id, "pos", JavaValue::Int(mark));
                    return Ok(None);
                }
                "mark" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let pos = heap.get_field(r_id, "pos").as_int().unwrap_or(0);
                    heap.set_field(r_id, "mark", JavaValue::Int(pos));
                    return Ok(None);
                }
                "markSupported" => return Ok(Some(JavaValue::Int(1))),
                _ => {}
            }
        }

        // 8. java/io/DataInputStream
        if is_data_input_stream {
            match method_name {
                "<init>" => {
                    println!(
                        "[DataInputStream.<init>] receiver: {:?}, in: {:?}",
                        receiver,
                        args.first()
                    );
                    if let Some(r_id) = receiver {
                        if let Some(stream_arg) = args.first() {
                            heap.set_field(r_id, "in", *stream_arg);
                        }
                    }
                    return Ok(None);
                }
                "readFully" => {
                    let in_stream = match receiver
                        .and_then(|r| heap.get_field(r, "in").as_object().ok().flatten())
                    {
                        Some(id) => id,
                        None => {
                            let exc_id = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc_id.0));
                        }
                    };
                    let dst_id = match args.first().and_then(|v| v.as_object().ok()).flatten() {
                        Some(id) => id,
                        None => {
                            let exc_id = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc_id.0));
                        }
                    };
                    let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = if args.len() >= 3 {
                        args[2].as_int().unwrap_or(0) as usize
                    } else {
                        heap.get(dst_id)
                            .and_then(|o| o.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0)
                    };

                    Self::stream_read_bytes(heap, in_stream, dst_id, off, len)?;
                    return Ok(None);
                }
                "readByte" => {
                    let b = Self::data_stream_read_u8(heap, receiver)?;
                    return Ok(Some(JavaValue::Int(b as i8 as i32)));
                }
                "readUnsignedByte" => {
                    let b = Self::data_stream_read_u8(heap, receiver)?;
                    return Ok(Some(JavaValue::Int(b as i32)));
                }
                "readShort" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as u16;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as u16;
                    let val = ((b1 << 8) | b2) as i16 as i32;
                    return Ok(Some(JavaValue::Int(val)));
                }
                "readUnsignedShort" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let val = (b1 << 8) | b2;
                    return Ok(Some(JavaValue::Int(val as i32)));
                }
                "readInt" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b3 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b4 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let val = ((b1 << 24) | (b2 << 16) | (b3 << 8) | b4) as i32;
                    return Ok(Some(JavaValue::Int(val)));
                }
                "readLong" => {
                    let mut bytes = [0u8; 8];
                    for i in 0..8 {
                        bytes[i] = Self::data_stream_read_u8(heap, receiver)?;
                    }
                    let val = i64::from_be_bytes(bytes);
                    return Ok(Some(JavaValue::Long(val)));
                }
                "readFloat" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b3 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b4 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let bits = (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;
                    return Ok(Some(JavaValue::Float(f32::from_bits(bits))));
                }
                "readDouble" => {
                    let mut bytes = [0u8; 8];
                    for i in 0..8 {
                        bytes[i] = Self::data_stream_read_u8(heap, receiver)?;
                    }
                    let bits = u64::from_be_bytes(bytes);
                    return Ok(Some(JavaValue::Double(f64::from_bits(bits))));
                }
                "readBoolean" => {
                    let b = Self::data_stream_read_u8(heap, receiver)?;
                    return Ok(Some(JavaValue::Int(if b != 0 { 1 } else { 0 })));
                }
                "readChar" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as u32;
                    return Ok(Some(JavaValue::Int(((b1 << 8) | b2) as i32)));
                }
                "readUTF" => {
                    let b1 = Self::data_stream_read_u8(heap, receiver)? as usize;
                    let b2 = Self::data_stream_read_u8(heap, receiver)? as usize;
                    let utflen = (b1 << 8) | b2;
                    let mut buf = Vec::with_capacity(utflen);
                    for _ in 0..utflen {
                        buf.push(Self::data_stream_read_u8(heap, receiver)?);
                    }
                    match Self::decode_modified_utf8(&buf) {
                        Ok(s) => {
                            let sid = heap.alloc_string(&s);
                            return Ok(Some(JavaValue::Object(Some(sid))));
                        }
                        Err(_) => {
                            let exc = heap.alloc_instance("java/io/UTFDataFormatException");
                            return Err(format!("EXC:java/io/UTFDataFormatException:{}", exc.0));
                        }
                    }
                }
                "read" => {
                    if args.is_empty() {
                        let b = Self::data_stream_read_u8(heap, receiver);
                        match b {
                            Ok(val) => return Ok(Some(JavaValue::Int(val as i32))),
                            Err(_) => return Ok(Some(JavaValue::Int(-1))),
                        }
                    } else {
                        let in_stream = receiver.map(|r| Self::resolve_stream_source(heap, r));
                        let dst_id = args[0].as_object().ok().flatten();
                        let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                        let len = if args.len() >= 3 {
                            args[2].as_int().unwrap_or(0) as usize
                        } else {
                            dst_id
                                .and_then(|id| heap.get(id))
                                .and_then(|o| o.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0)
                        };
                        if let (Some(s_id), Some(d_id)) = (in_stream, dst_id) {
                            let read_bytes = Self::stream_read_bytes(heap, s_id, d_id, off, len);
                            match read_bytes {
                                Ok(n) => return Ok(Some(JavaValue::Int(n as i32))),
                                Err(_) => return Ok(Some(JavaValue::Int(-1))),
                            }
                        }
                        return Ok(Some(JavaValue::Int(-1)));
                    }
                }
                "skipBytes" => {
                    let n = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let in_stream = receiver.map(|r| Self::resolve_stream_source(heap, r));
                    if let Some(s_id) = in_stream {
                        let pos = heap.get_field(s_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(s_id, "count").as_int().unwrap_or(0);
                        let actual = n.min(count - pos).max(0);
                        heap.set_field(s_id, "pos", JavaValue::Int(pos + actual));
                        return Ok(Some(JavaValue::Int(actual)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "skip" => {
                    let n = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0) as i32;
                    let in_stream = receiver.map(|r| Self::resolve_stream_source(heap, r));
                    if let Some(s_id) = in_stream {
                        let pos = heap.get_field(s_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(s_id, "count").as_int().unwrap_or(0);
                        let actual = n.min(count - pos).max(0);
                        heap.set_field(s_id, "pos", JavaValue::Int(pos + actual));
                        return Ok(Some(JavaValue::Long(actual as i64)));
                    }
                    return Ok(Some(JavaValue::Long(0)));
                }
                "available" => {
                    let in_stream = receiver.map(|r| Self::resolve_stream_source(heap, r));
                    if let Some(s_id) = in_stream {
                        let pos = heap.get_field(s_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(s_id, "count").as_int().unwrap_or(0);
                        return Ok(Some(JavaValue::Int((count - pos).max(0))));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "mark" => {
                    let in_stream = receiver.map(|r| Self::resolve_stream_source(heap, r));
                    if let Some(s_id) = in_stream {
                        let pos = heap.get_field(s_id, "pos").as_int().unwrap_or(0);
                        heap.set_field(s_id, "mark", JavaValue::Int(pos));
                    }
                    return Ok(None);
                }
                "reset" => {
                    let in_stream =
                        receiver.and_then(|r| heap.get_field(r, "in").as_object().ok().flatten());
                    if let Some(s_id) = in_stream {
                        let mark = heap.get_field(s_id, "mark").as_int().unwrap_or(0);
                        heap.set_field(s_id, "pos", JavaValue::Int(mark));
                    }
                    return Ok(None);
                }
                "markSupported" => return Ok(Some(JavaValue::Int(1))),
                "close" => return Ok(None),
                _ => {}
            }
        }

        // 9. java/io/ByteArrayOutputStream & DataOutputStream
        if cls == "java/io/ByteArrayOutputStream"
            || cls == "java/io/DataOutputStream"
            || cls.ends_with("OutputStream")
            || class_loader.is_instance_of(&cls, "java/io/OutputStream")
        {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if cls == "java/io/ByteArrayOutputStream" {
                            let cap = args
                                .first()
                                .and_then(|v| v.as_int().ok())
                                .unwrap_or(32)
                                .max(0) as usize;
                            let buf_id = heap.alloc_array(ArrayData::Byte(Vec::with_capacity(cap)));
                            heap.set_field(r_id, "buf", JavaValue::Object(Some(buf_id)));
                        } else {
                            // FilterOutputStream / DataOutputStream
                            if let Some(out_arg) = args.first() {
                                heap.set_field(r_id, "out", *out_arg);
                            }
                        }
                    }
                    return Ok(None);
                }
                "write" => {
                    if args.len() == 1 {
                        if let Ok(Some(arr_id)) = args[0].as_object() {
                            if let Some(ObjectData::Array(ArrayData::Byte(src))) = heap.get(arr_id)
                            {
                                let slice = src.clone();
                                Self::output_stream_write_bytes(heap, receiver, &slice);
                            }
                        } else {
                            let b = args[0].as_int().unwrap_or(0) as i8;
                            Self::output_stream_write_byte(heap, receiver, b);
                        }
                    } else if args.len() >= 3 {
                        let arr_id = args[0].as_object().ok().flatten();
                        let off = args[1].as_int().unwrap_or(0) as usize;
                        let len = args[2].as_int().unwrap_or(0) as usize;
                        if let Some(aid) = arr_id {
                            if let Some(ObjectData::Array(ArrayData::Byte(src))) = heap.get(aid) {
                                let end = (off + len).min(src.len());
                                if off <= end {
                                    let slice = src[off..end].to_vec();
                                    Self::output_stream_write_bytes(heap, receiver, &slice);
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "writeByte" => {
                    let b = args[0].as_int().unwrap_or(0) as i8;
                    Self::output_stream_write_byte(heap, receiver, b);
                    return Ok(None);
                }
                "writeShort" => {
                    let s = args[0].as_int().unwrap_or(0) as u16;
                    Self::output_stream_write_bytes(heap, receiver, &[(s >> 8) as i8, s as i8]);
                    return Ok(None);
                }
                "writeChar" => {
                    let c = args[0].as_int().unwrap_or(0) as u16;
                    Self::output_stream_write_bytes(heap, receiver, &[(c >> 8) as i8, c as i8]);
                    return Ok(None);
                }
                "writeInt" => {
                    let i = args[0].as_int().unwrap_or(0) as u32;
                    Self::output_stream_write_bytes(
                        heap,
                        receiver,
                        &[(i >> 24) as i8, (i >> 16) as i8, (i >> 8) as i8, i as i8],
                    );
                    return Ok(None);
                }
                "writeLong" => {
                    let l = args[0].as_long().unwrap_or(0) as u64;
                    let mut b = [0i8; 8];
                    for shift in 0..8 {
                        b[7 - shift] = (l >> (shift * 8)) as i8;
                    }
                    Self::output_stream_write_bytes(heap, receiver, &b);
                    return Ok(None);
                }
                "writeBoolean" => {
                    let b = args[0].as_int().unwrap_or(0) != 0;
                    Self::output_stream_write_byte(heap, receiver, if b { 1 } else { 0 });
                    return Ok(None);
                }
                "writeUTF" => {
                    let str_id = args[0].as_object().ok().flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let bytes = s.as_bytes();
                    let len = bytes.len() as u16;
                    let mut payload = Vec::with_capacity(bytes.len() + 2);
                    payload.push((len >> 8) as i8);
                    payload.push(len as i8);
                    for &b in bytes {
                        payload.push(b as i8);
                    }
                    Self::output_stream_write_bytes(heap, receiver, &payload);
                    return Ok(None);
                }
                "flush" | "close" => return Ok(None),
                "toByteArray" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let buf_id = heap.get_field(r_id, "buf").as_object().ok().flatten();
                    if let Some(bid) = buf_id {
                        let cloned_b =
                            if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(bid) {
                                Some(b.clone())
                            } else {
                                None
                            };
                        if let Some(b) = cloned_b {
                            let len = b.len();
                            let copy_id = heap.alloc_array(ArrayData::Byte(b));
                            eprintln!(
                                "[TOBYTEARRAY] buf_id={:?} copy_id={:?} len={}",
                                bid, copy_id, len
                            );
                            return Ok(Some(JavaValue::Object(Some(copy_id))));
                        }
                    }
                    let empty_id = heap.alloc_array(ArrayData::Byte(Vec::new()));
                    eprintln!("[TOBYTEARRAY EMPTY] id={:?}", empty_id);
                    return Ok(Some(JavaValue::Object(Some(empty_id))));
                }
                "size" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let buf_id = heap.get_field(r_id, "buf").as_object().ok().flatten();
                    let len = buf_id
                        .and_then(|id| heap.get(id))
                        .and_then(|o| o.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(len as i32)));
                }
                "reset" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let buf_id = heap.get_field(r_id, "buf").as_object().ok().flatten();
                    if let Some(bid) = buf_id {
                        if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get_mut(bid) {
                            b.clear();
                        }
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 9b. java/io/InputStreamReader & java/io/Reader
        if cls == "java/io/InputStreamReader"
            || cls == "java/io/Reader"
            || class_loader.is_instance_of(&cls, "java/io/InputStreamReader")
            || class_loader.is_instance_of(&cls, "java/io/Reader")
        {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if let Some(in_stream) = args.first() {
                            heap.set_field(r_id, "in", *in_stream);
                        }
                    }
                    return Ok(None);
                }
                "read" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let in_val = heap.get_field(r_id, "in").as_object().ok().flatten();
                    let stream_id = match in_val {
                        Some(sid) => sid,
                        None => return Ok(Some(JavaValue::Int(-1))),
                    };

                    if args.is_empty() {
                        let pos = heap.get_field(stream_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(stream_id, "count").as_int().unwrap_or(0);
                        let buf_id = heap.get_field(stream_id, "buf").as_object().ok().flatten();
                        if pos < count {
                            if let Some(bid) = buf_id {
                                if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(bid) {
                                    if (pos as usize) < b.len() {
                                        let ch = (b[pos as usize] as u8) as i32;
                                        heap.set_field(stream_id, "pos", JavaValue::Int(pos + 1));
                                        return Ok(Some(JavaValue::Int(ch)));
                                    }
                                }
                            }
                        }
                        return Ok(Some(JavaValue::Int(-1)));
                    } else {
                        let cbuf_id = match args[0].as_object().ok().flatten() {
                            Some(cid) => cid,
                            None => {
                                let exc = heap.alloc_instance("java/lang/NullPointerException");
                                return Err(format!(
                                    "EXC:java/lang/NullPointerException:{}",
                                    exc.0
                                ));
                            }
                        };
                        let off = args
                            .get(1)
                            .and_then(|v| v.as_int().ok())
                            .unwrap_or(0)
                            .max(0) as usize;
                        let max_len = if args.len() >= 3 {
                            args[2].as_int().unwrap_or(0).max(0) as usize
                        } else {
                            heap.get(cbuf_id)
                                .and_then(|o| o.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0)
                        };

                        if max_len == 0 {
                            return Ok(Some(JavaValue::Int(0)));
                        }

                        let pos = heap.get_field(stream_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(stream_id, "count").as_int().unwrap_or(0);
                        let buf_id = heap.get_field(stream_id, "buf").as_object().ok().flatten();

                        if pos >= count {
                            return Ok(Some(JavaValue::Int(-1)));
                        }

                        let available = (count - pos).max(0) as usize;
                        let to_read = available.min(max_len);

                        if let Some(bid) = buf_id {
                            let bytes_to_copy = if let Some(ObjectData::Array(ArrayData::Byte(b))) =
                                heap.get(bid)
                            {
                                let start = pos as usize;
                                let end = (start + to_read).min(b.len());
                                b[start..end]
                                    .iter()
                                    .map(|&x| x as u8 as u16)
                                    .collect::<Vec<_>>()
                            } else {
                                Vec::new()
                            };

                            let read_len = bytes_to_copy.len();
                            if let Some(ObjectData::Array(ArrayData::Char(chars))) =
                                heap.get_mut(cbuf_id)
                            {
                                for (i, &ch) in bytes_to_copy.iter().enumerate() {
                                    if off + i < chars.len() {
                                        chars[off + i] = ch;
                                    }
                                }
                            }
                            heap.set_field(stream_id, "pos", JavaValue::Int(pos + read_len as i32));
                            return Ok(Some(JavaValue::Int(if read_len == 0 {
                                -1
                            } else {
                                read_len as i32
                            })));
                        }
                        return Ok(Some(JavaValue::Int(-1)));
                    }
                }
                "ready" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let in_val = heap.get_field(r_id, "in").as_object().ok().flatten();
                    if let Some(stream_id) = in_val {
                        let pos = heap.get_field(stream_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(stream_id, "count").as_int().unwrap_or(0);
                        return Ok(Some(JavaValue::Int(if pos < count { 1 } else { 0 })));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "close" => {
                    return Ok(None);
                }
                "skip" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let n = args
                        .first()
                        .and_then(|v| v.as_long().ok())
                        .unwrap_or(0)
                        .max(0) as i32;
                    let in_val = heap.get_field(r_id, "in").as_object().ok().flatten();
                    if let Some(stream_id) = in_val {
                        let pos = heap.get_field(stream_id, "pos").as_int().unwrap_or(0);
                        let count = heap.get_field(stream_id, "count").as_int().unwrap_or(0);
                        let actual = n.min(count - pos).max(0);
                        heap.set_field(stream_id, "pos", JavaValue::Int(pos + actual));
                        return Ok(Some(JavaValue::Long(actual as i64)));
                    }
                    return Ok(Some(JavaValue::Long(0)));
                }
                _ => {}
            }
        }

        // 10. java/lang/String
        if cls == "java/lang/String" {
            match method_name {
                "<init>" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    if args.is_empty() {
                        heap.set_string(r_id, "");
                    } else if let Ok(Some(obj_id)) = args[0].as_object() {
                        match heap.get(obj_id) {
                            Some(ObjectData::Array(ArrayData::Byte(b))) => {
                                let offset_i =
                                    args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                                let len_i = args.get(2).and_then(|v| v.as_int().ok());
                                let enc = args
                                    .get(3)
                                    .and_then(|v| v.as_object().ok())
                                    .flatten()
                                    .and_then(|id| heap.get_string_utf8(id));
                                if offset_i < 0 {
                                    return Err("EXC:java/lang/StringIndexOutOfBoundsException: offset is negative".to_string());
                                }
                                let offset = offset_i as usize;
                                let actual_len = match len_i {
                                    Some(l) => {
                                        if l < 0 {
                                            return Err("EXC:java/lang/StringIndexOutOfBoundsException: length is negative".to_string());
                                        }
                                        l as usize
                                    }
                                    None => b.len().saturating_sub(offset),
                                };
                                if offset + actual_len > b.len() {
                                    return Err("EXC:java/lang/StringIndexOutOfBoundsException: offset + length exceeds byte array length".to_string());
                                }
                                let slice: Vec<u8> = b[offset..offset + actual_len]
                                    .iter()
                                    .map(|&x| x as u8)
                                    .collect();
                                let decoded = Self::decode_string_bytes(&slice, enc.as_deref());
                                heap.set_string(r_id, &decoded);
                            }
                            Some(ObjectData::Array(ArrayData::Char(c))) => {
                                let offset_i =
                                    args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                                let len_i = args.get(2).and_then(|v| v.as_int().ok());
                                if offset_i < 0 {
                                    return Err("EXC:java/lang/StringIndexOutOfBoundsException: offset is negative".to_string());
                                }
                                let offset = offset_i as usize;
                                let actual_len = match len_i {
                                    Some(l) => {
                                        if l < 0 {
                                            return Err("EXC:java/lang/StringIndexOutOfBoundsException: length is negative".to_string());
                                        }
                                        l as usize
                                    }
                                    None => c.len().saturating_sub(offset),
                                };
                                if offset + actual_len > c.len() {
                                    return Err("EXC:java/lang/StringIndexOutOfBoundsException: offset + length exceeds char array length".to_string());
                                }
                                let chars: String =
                                    String::from_utf16_lossy(&c[offset..offset + actual_len]);
                                heap.set_string(r_id, &chars);
                            }
                            _ => {
                                let s = heap.get_string_utf8(obj_id).unwrap_or_default();
                                heap.set_string(r_id, &s);
                            }
                        }
                    }
                    return Ok(None);
                }
                "length" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let u16_len = s.encode_utf16().count();
                    return Ok(Some(JavaValue::Int(u16_len as i32)));
                }
                "charAt" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let idx_i = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let utf16: Vec<u16> = s.encode_utf16().collect();
                    if idx_i < 0 || (idx_i as usize) >= utf16.len() {
                        return Err(format!(
                            "EXC:java/lang/StringIndexOutOfBoundsException: index {} out of bounds for length {}",
                            idx_i,
                            utf16.len()
                        ));
                    }
                    let c = utf16[idx_i as usize] as i32;
                    return Ok(Some(JavaValue::Int(c)));
                }
                "substring" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let utf16: Vec<u16> = s.encode_utf16().collect();
                    let start_i = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let end_i = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(utf16.len() as i32);
                    if start_i < 0
                        || end_i < 0
                        || (start_i as usize) > (end_i as usize)
                        || (end_i as usize) > utf16.len()
                    {
                        return Err(format!(
                            "EXC:java/lang/StringIndexOutOfBoundsException: start {}, end {}, length {}",
                            start_i,
                            end_i,
                            utf16.len()
                        ));
                    }
                    let sub = String::from_utf16_lossy(&utf16[start_i as usize..end_i as usize]);
                    let sid = heap.alloc_string(&sub);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "getChars" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let utf16: Vec<u16> = s.encode_utf16().collect();
                    let src_begin = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let src_end = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let dst_obj = args.get(2).and_then(|v| v.as_object().ok()).flatten();
                    let dst_begin = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    let dst_id = match dst_obj {
                        Some(id) => id,
                        None => {
                            return Err(
                                "EXC:java/lang/NullPointerException: dst is null".to_string()
                            );
                        }
                    };

                    if src_begin < 0
                        || src_begin > src_end
                        || (src_end as usize) > utf16.len()
                        || dst_begin < 0
                    {
                        return Err(
                            "EXC:java/lang/StringIndexOutOfBoundsException: index out of bounds"
                                .to_string(),
                        );
                    }
                    let count = (src_end - src_begin) as usize;
                    match heap.get_mut(dst_id) {
                        Some(ObjectData::Array(ArrayData::Char(dst_chars))) => {
                            let dst_idx = dst_begin as usize;
                            if dst_idx + count > dst_chars.len() {
                                return Err(
                                    "EXC:java/lang/StringIndexOutOfBoundsException: dst array overflow"
                                        .to_string(),
                                );
                            }
                            dst_chars[dst_idx..dst_idx + count]
                                .copy_from_slice(&utf16[src_begin as usize..src_end as usize]);
                        }
                        _ => {
                            return Err(
                                "EXC:java/lang/ArrayStoreException: dst is not char[]".to_string()
                            );
                        }
                    }
                    return Ok(None);
                }
                "indexOf" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    if let Ok(ch) = args[0].as_int() {
                        let from = args
                            .get(1)
                            .and_then(|v| v.as_int().ok())
                            .unwrap_or(0)
                            .max(0) as usize;
                        let target = char::from_u32(ch as u32).unwrap_or('\0');
                        let pos = s
                            .chars()
                            .enumerate()
                            .skip(from)
                            .find(|&(_, c)| c == target)
                            .map(|(i, _)| i as i32)
                            .unwrap_or(-1);
                        return Ok(Some(JavaValue::Int(pos)));
                    } else if let Ok(Some(str_id)) = args[0].as_object() {
                        let target = heap.get_string_utf8(str_id).unwrap_or_default();
                        let from = args
                            .get(1)
                            .and_then(|v| v.as_int().ok())
                            .unwrap_or(0)
                            .max(0) as usize;
                        // Java semantics: search in CHAR units from `from`,
                        // returning a char index (never a byte offset).
                        let chars: Vec<char> = s.chars().collect();
                        let needle: Vec<char> = target.chars().collect();
                        let mut found = -1i32;
                        if !needle.is_empty() && needle.len() <= chars.len() {
                            let start = from.min(chars.len());
                            let last = chars.len() - needle.len();
                            let mut i = start;
                            while i <= last {
                                if chars[i..i + needle.len()] == needle[..] {
                                    found = i as i32;
                                    break;
                                }
                                i += 1;
                            }
                        }
                        return Ok(Some(JavaValue::Int(found)));
                    }
                    return Ok(Some(JavaValue::Int(-1)));
                }
                "lastIndexOf" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let ch = args[0].as_int().unwrap_or(0) as u32;
                    let target = char::from_u32(ch).unwrap_or('\0');
                    let pos = s
                        .chars()
                        .enumerate()
                        .filter(|&(_, c)| c == target)
                        .last()
                        .map(|(i, _)| i as i32)
                        .unwrap_or(-1);
                    return Ok(Some(JavaValue::Int(pos)));
                }
                "startsWith" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let prefix_id = args[0].as_object().ok().flatten();
                    let prefix = prefix_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    return Ok(Some(JavaValue::Int(if s.starts_with(&prefix) {
                        1
                    } else {
                        0
                    })));
                }
                "endsWith" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let suffix_id = args[0].as_object().ok().flatten();
                    let suffix = suffix_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    return Ok(Some(JavaValue::Int(if s.ends_with(&suffix) {
                        1
                    } else {
                        0
                    })));
                }
                "equals" => {
                    let s1 = receiver.and_then(|r| heap.get_string_utf8(r));
                    let s2 = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id));
                    let eq = s1.is_some() && s1 == s2;
                    return Ok(Some(JavaValue::Int(if eq { 1 } else { 0 })));
                }
                "equalsIgnoreCase" => {
                    let s1 = receiver.and_then(|r| heap.get_string_utf8(r));
                    let s2 = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id));
                    let eq = match (s1, s2) {
                        (Some(a), Some(b)) => a.eq_ignore_ascii_case(&b),
                        _ => false,
                    };
                    return Ok(Some(JavaValue::Int(if eq { 1 } else { 0 })));
                }
                "compareTo" => {
                    let s1 = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let s2 = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    return Ok(Some(JavaValue::Int(s1.cmp(&s2) as i32)));
                }
                "toCharArray" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let chars: Vec<u16> = s.encode_utf16().collect();
                    let aid = heap.alloc_array(ArrayData::Char(chars));
                    return Ok(Some(JavaValue::Object(Some(aid))));
                }
                "getBytes" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let enc = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id));
                    let bytes = Self::encode_string_bytes(&s, enc.as_deref());
                    let aid = heap.alloc_array(ArrayData::Byte(
                        bytes.into_iter().map(|b| b as i8).collect(),
                    ));
                    return Ok(Some(JavaValue::Object(Some(aid))));
                }
                "valueOf" => {
                    let s = if let Some(first) = args.first() {
                        match first {
                            JavaValue::Int(i) => {
                                if descriptor.contains('C') {
                                    char::from_u32(*i as u32)
                                        .map(|c| c.to_string())
                                        .unwrap_or_default()
                                } else if descriptor.contains('Z') {
                                    if *i != 0 {
                                        "true".to_string()
                                    } else {
                                        "false".to_string()
                                    }
                                } else {
                                    i.to_string()
                                }
                            }
                            JavaValue::Long(l) => l.to_string(),
                            JavaValue::Float(f) => f.to_string(),
                            JavaValue::Double(d) => d.to_string(),
                            JavaValue::Object(Some(oid)) => {
                                if let Some(ObjectData::Array(ArrayData::Char(chars))) =
                                    heap.get(*oid)
                                {
                                    if args.len() >= 3 && descriptor.contains("[CII") {
                                        let offset = args[1].as_int().unwrap_or(0).max(0) as usize;
                                        let len = args[2].as_int().unwrap_or(0).max(0) as usize;
                                        let slice = if offset < chars.len() {
                                            let end = (offset + len).min(chars.len());
                                            &chars[offset..end]
                                        } else {
                                            &[]
                                        };
                                        slice
                                            .iter()
                                            .filter_map(|&c| char::from_u32(c as u32))
                                            .collect()
                                    } else {
                                        chars
                                            .iter()
                                            .filter_map(|&c| char::from_u32(c as u32))
                                            .collect()
                                    }
                                } else {
                                    if let Some(ObjectData::Instance(inst)) = heap.get(*oid) {
                                        if inst.class_name != "java/lang/String"
                                            && inst.class_name != "java/lang/StringBuffer"
                                            && inst.class_name != "java/lang/StringBuilder"
                                            && inst.class_name != "java/lang/Integer"
                                            && inst.class_name != "java/lang/Long"
                                            && inst.class_name != "java/lang/Boolean"
                                            && inst.class_name != "java/lang/Byte"
                                            && inst.class_name != "java/lang/Short"
                                            && inst.class_name != "java/lang/Character"
                                            && inst.class_name != "java/lang/Float"
                                            && inst.class_name != "java/lang/Double"
                                            && inst.class_name != "java/lang/Class"
                                        {
                                            if let Some(m) = class_loader.find_method(
                                                &inst.class_name,
                                                "toString",
                                                "()Ljava/lang/String;",
                                            ) {
                                                if thread_idx < scheduler.threads.len()
                                                    && !m.instructions.is_empty()
                                                {
                                                    let mut to_string_frame = Frame::new(
                                                        inst.class_name.clone(),
                                                        "toString".to_string(),
                                                        "()Ljava/lang/String;".to_string(),
                                                        Arc::clone(&m.instructions),
                                                        m.max_locals,
                                                        m.max_stack,
                                                        m.exception_table.clone(),
                                                        Arc::clone(&m.pc_map),
                                                    );
                                                    to_string_frame.set_local(
                                                        0,
                                                        JavaValue::Object(Some(*oid)),
                                                    );
                                                    scheduler.threads[thread_idx]
                                                        .call_stack
                                                        .push(to_string_frame);
                                                    return Ok(None);
                                                }
                                            }
                                        }
                                    }
                                    Self::object_to_java_string(heap, *oid)
                                }
                            }
                            JavaValue::Object(None) => "null".to_string(),
                        }
                    } else {
                        String::new()
                    };
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "trim" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let utf16: Vec<u16> = s.encode_utf16().collect();
                    let mut start = 0;
                    let mut end = utf16.len();
                    while start < end && utf16[start] <= 0x20 {
                        start += 1;
                    }
                    while end > start && utf16[end - 1] <= 0x20 {
                        end -= 1;
                    }
                    let trimmed = String::from_utf16_lossy(&utf16[start..end]);
                    let sid = heap.alloc_string(&trimmed);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toLowerCase" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let sid = heap.alloc_string(&s.to_lowercase());
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toUpperCase" => {
                    let s = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let sid = heap.alloc_string(&s.to_uppercase());
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "concat" => {
                    let s1 = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let s2 = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let sid = heap.alloc_string(&format!("{}{}", s1, s2));
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "intern" => {
                    // JLS: String.intern() returns the string instance itself
                    if let Some(r_id) = receiver {
                        return Ok(Some(JavaValue::Object(Some(r_id))));
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "regionMatches" => {
                    let (ignore_case, toffset_raw, other_id, ooffset_raw, len_raw) =
                        if args.len() == 5 {
                            (
                                args[0].as_int().unwrap_or(0) != 0,
                                args[1].as_int().unwrap_or(0),
                                args[2].as_object().ok().flatten(),
                                args[3].as_int().unwrap_or(0),
                                args[4].as_int().unwrap_or(0),
                            )
                        } else if args.len() == 4 {
                            (
                                false,
                                args[0].as_int().unwrap_or(0),
                                args[1].as_object().ok().flatten(),
                                args[2].as_int().unwrap_or(0),
                                args[3].as_int().unwrap_or(0),
                            )
                        } else {
                            return Ok(Some(JavaValue::Int(0)));
                        };

                    if toffset_raw < 0 || ooffset_raw < 0 || len_raw < 0 {
                        return Ok(Some(JavaValue::Int(0)));
                    }

                    let this_str = receiver
                        .and_then(|r| heap.get_string_utf8(r))
                        .unwrap_or_default();
                    let other_str = other_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();

                    let this_chars: Vec<char> = this_str.chars().collect();
                    let other_chars: Vec<char> = other_str.chars().collect();

                    let toffset = toffset_raw as usize;
                    let ooffset = ooffset_raw as usize;
                    let len = len_raw as usize;

                    if toffset + len > this_chars.len() || ooffset + len > other_chars.len() {
                        return Ok(Some(JavaValue::Int(0)));
                    }

                    for i in 0..len {
                        let c1 = this_chars[toffset + i];
                        let c2 = other_chars[ooffset + i];
                        if c1 == c2 {
                            continue;
                        }
                        if ignore_case {
                            if c1.to_ascii_uppercase() == c2.to_ascii_uppercase()
                                || c1.to_ascii_lowercase() == c2.to_ascii_lowercase()
                            {
                                continue;
                            }
                        }
                        return Ok(Some(JavaValue::Int(0)));
                    }
                    return Ok(Some(JavaValue::Int(1)));
                }
                "replace" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let old_c = args.get(0).and_then(|a| a.as_int().ok()).unwrap_or(0) as u16;
                    let new_c = args.get(1).and_then(|a| a.as_int().ok()).unwrap_or(0) as u16;
                    let s = heap.get_string_utf8(r_id).unwrap_or_default();
                    if let (Some(old_char), Some(new_char)) =
                        (char::from_u32(old_c as u32), char::from_u32(new_c as u32))
                    {
                        let replaced: String = s
                            .chars()
                            .map(|c| if c == old_char { new_char } else { c })
                            .collect();
                        let sid = heap.alloc_string(&replaced);
                        return Ok(Some(JavaValue::Object(Some(sid))));
                    } else {
                        return Ok(Some(JavaValue::Object(Some(r_id))));
                    }
                }
                _ => {}
            }
        }

        // 11. java/lang/StringBuffer
        if cls == "java/lang/StringBuffer" {
            match method_name {
                "<init>" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let init_s = if let Some(arg) = args.first() {
                        if let Ok(Some(sid)) = arg.as_object() {
                            heap.get_string_utf8(sid).unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    let sid = heap.alloc_string(&init_s);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(sid)));
                    return Ok(None);
                }
                "append" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let app = if let Some(first) = args.first() {
                        match first {
                            JavaValue::Int(i) => {
                                if descriptor.contains('C') {
                                    char::from_u32(*i as u32)
                                        .map(|c| c.to_string())
                                        .unwrap_or_default()
                                } else if descriptor.contains('Z') {
                                    if *i != 0 {
                                        "true".to_string()
                                    } else {
                                        "false".to_string()
                                    }
                                } else {
                                    i.to_string()
                                }
                            }
                            JavaValue::Long(l) => l.to_string(),
                            JavaValue::Float(f) => f.to_string(),
                            JavaValue::Double(d) => d.to_string(),
                            JavaValue::Object(Some(oid)) => {
                                if let Some(ObjectData::Array(ArrayData::Char(chars))) =
                                    heap.get(*oid)
                                {
                                    if args.len() >= 3 && descriptor.contains("[CII") {
                                        let offset = args[1].as_int().unwrap_or(0).max(0) as usize;
                                        let len = args[2].as_int().unwrap_or(0).max(0) as usize;
                                        let slice = if offset < chars.len() {
                                            let end = (offset + len).min(chars.len());
                                            &chars[offset..end]
                                        } else {
                                            &[]
                                        };
                                        slice
                                            .iter()
                                            .filter_map(|&c| char::from_u32(c as u32))
                                            .collect()
                                    } else {
                                        chars
                                            .iter()
                                            .filter_map(|&c| char::from_u32(c as u32))
                                            .collect()
                                    }
                                } else {
                                    Self::object_to_java_string(heap, *oid)
                                }
                            }
                            JavaValue::Object(None) => "null".to_string(),
                        }
                    } else {
                        String::new()
                    };
                    let new_sid = heap.alloc_string(&format!("{}{}", cur_s, app));
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(Some(JavaValue::Object(Some(r_id))));
                }
                "toString" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let sid = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| heap.alloc_string(""));
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "length" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    return Ok(Some(JavaValue::Int(cur_s.chars().count() as i32)));
                }
                "setLength" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let new_len = args[0].as_int().unwrap_or(0).max(0) as usize;
                    let mut chars: Vec<char> = cur_s.chars().collect();
                    if new_len > chars.len() {
                        chars.resize(new_len, '\0');
                    } else {
                        chars.truncate(new_len);
                    }
                    let updated: String = chars.into_iter().collect();
                    let new_sid = heap.alloc_string(&updated);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(None);
                }
                "charAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let idx = args[0].as_int().unwrap_or(0) as usize;
                    let c = cur_s.chars().nth(idx).unwrap_or('\0') as u32 as i32;
                    return Ok(Some(JavaValue::Int(c)));
                }
                "getChars" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let src_begin = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let src_end = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let dst_arr_id = args.get(2).and_then(|v| v.as_object().ok()).flatten();
                    let dst_begin = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if dst_arr_id.is_none() {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                    let aid = dst_arr_id.unwrap();
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let utf16_chars: Vec<u16> = cur_s.encode_utf16().collect();
                    let src_len = utf16_chars.len() as i32;

                    let dst_len = match heap.get(aid) {
                        Some(ObjectData::Array(ArrayData::Char(chars))) => chars.len() as i32,
                        _ => 0,
                    };

                    if src_begin < 0
                        || dst_begin < 0
                        || src_begin > src_end
                        || src_end > src_len
                        || (dst_begin + (src_end - src_begin)) > dst_len
                    {
                        let exc = heap.alloc_instance("java/lang/StringIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/StringIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }

                    if let Some(ObjectData::Array(ArrayData::Char(ref mut chars))) =
                        heap.get_mut(aid)
                    {
                        let count = (src_end - src_begin) as usize;
                        let src_slice = &utf16_chars[src_begin as usize..src_end as usize];
                        chars[dst_begin as usize..dst_begin as usize + count]
                            .copy_from_slice(src_slice);
                    }
                    return Ok(None);
                }
                "delete" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let start = args
                        .get(0)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let end = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let chars: Vec<char> = cur_s.chars().collect();
                    let total = chars.len();
                    let real_start = start.min(total);
                    let real_end = end.min(total);
                    let mut new_chars = Vec::new();
                    if real_start < total {
                        new_chars.extend_from_slice(&chars[..real_start]);
                    }
                    if real_end < total {
                        new_chars.extend_from_slice(&chars[real_end..]);
                    }
                    let new_s: String = new_chars.into_iter().collect();
                    let new_sid = heap.alloc_string(&new_s);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(Some(JavaValue::Object(Some(r_id))));
                }
                "deleteCharAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let idx = args
                        .get(0)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let mut chars: Vec<char> = cur_s.chars().collect();
                    if idx < chars.len() {
                        chars.remove(idx);
                    }
                    let new_s: String = chars.into_iter().collect();
                    let new_sid = heap.alloc_string(&new_s);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(Some(JavaValue::Object(Some(r_id))));
                }
                "insert" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let offset = args
                        .get(0)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let insert_s = if let Some(val) = args.get(1) {
                        match val {
                            JavaValue::Int(i) => {
                                if descriptor.contains("C)") {
                                    char::from_u32(*i as u32)
                                        .map(|c| c.to_string())
                                        .unwrap_or_default()
                                } else {
                                    i.to_string()
                                }
                            }
                            JavaValue::Long(l) => l.to_string(),
                            JavaValue::Float(f) => f.to_string(),
                            JavaValue::Double(d) => d.to_string(),
                            JavaValue::Object(Some(oid)) => heap
                                .get_string_utf8(*oid)
                                .unwrap_or_else(|| "Object".to_string()),
                            JavaValue::Object(None) => "null".to_string(),
                        }
                    } else {
                        String::new()
                    };
                    let chars: Vec<char> = cur_s.chars().collect();
                    let real_offset = offset.min(chars.len());
                    let mut new_chars = Vec::new();
                    new_chars.extend_from_slice(&chars[..real_offset]);
                    new_chars.extend(insert_s.chars());
                    new_chars.extend_from_slice(&chars[real_offset..]);
                    let new_s: String = new_chars.into_iter().collect();
                    let new_sid = heap.alloc_string(&new_s);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(Some(JavaValue::Object(Some(r_id))));
                }
                "reverse" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let rev: String = cur_s.chars().rev().collect();
                    let new_sid = heap.alloc_string(&rev);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(Some(JavaValue::Object(Some(r_id))));
                }
                "setCharAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cur_s = heap
                        .get_field(r_id, "str")
                        .as_object()
                        .ok()
                        .flatten()
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let idx_raw = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let ch_code = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as u32;
                    let mut chars: Vec<char> = cur_s.chars().collect();
                    if idx_raw < 0 || (idx_raw as usize) >= chars.len() {
                        let exc = heap.alloc_instance("java/lang/StringIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/StringIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }
                    if let Some(ch) = char::from_u32(ch_code) {
                        chars[idx_raw as usize] = ch;
                    }
                    let updated: String = chars.into_iter().collect();
                    let new_sid = heap.alloc_string(&updated);
                    heap.set_field(r_id, "str", JavaValue::Object(Some(new_sid)));
                    return Ok(None);
                }
                "ensureCapacity" => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 12. java/lang/Integer, Long, Boolean, etc.
        if cls == "java/lang/Integer" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                        heap.set_field(r_id, "value", v);
                    }
                    return Ok(None);
                }
                "intValue" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "valueOf" => {
                    let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                    let obj = heap.alloc_instance("java/lang/Integer");
                    heap.set_field(obj, "value", v);
                    return Ok(Some(JavaValue::Object(Some(obj))));
                }
                "parseInt" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let radix = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(10) as u32;
                    let val = i32::from_str_radix(s.trim(), radix).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(val)));
                }
                "toString" => {
                    let i = if !args.is_empty() {
                        args[0].as_int().unwrap_or(0)
                    } else {
                        receiver
                            .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                            .unwrap_or(0)
                    };
                    let sid = heap.alloc_string(&i.to_string());
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toHexString" => {
                    let i = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let s = format!("{:x}", i as u32);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toOctalString" => {
                    let i = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let s = format!("{:o}", i as u32);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toBinaryString" => {
                    let i = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let s = format!("{:b}", i as u32);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "equals" => {
                    let other = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let eq = if let (Some(r), Some(o)) = (receiver, other) {
                        heap.get_field(r, "value").as_int() == heap.get_field(o, "value").as_int()
                    } else {
                        false
                    };
                    return Ok(Some(JavaValue::Int(if eq { 1 } else { 0 })));
                }
                "hashCode" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                _ => {}
            }
        }

        if cls == "java/lang/Long" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Long(0));
                        heap.set_field(r_id, "value", v);
                    }
                    return Ok(None);
                }
                "longValue" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_long().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Long(v)));
                }
                "valueOf" => {
                    let v = args.first().copied().unwrap_or(JavaValue::Long(0));
                    let obj = heap.alloc_instance("java/lang/Long");
                    heap.set_field(obj, "value", v);
                    return Ok(Some(JavaValue::Object(Some(obj))));
                }
                "parseLong" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let val = s.trim().parse::<i64>().unwrap_or(0);
                    return Ok(Some(JavaValue::Long(val)));
                }
                "toString" => {
                    let l = if !args.is_empty() {
                        args[0].as_long().unwrap_or(0)
                    } else {
                        receiver
                            .map(|id| heap.get_field(id, "value").as_long().unwrap_or(0))
                            .unwrap_or(0)
                    };
                    let sid = heap.alloc_string(&l.to_string());
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toHexString" => {
                    let l = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                    let s = format!("{:x}", l as u64);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toOctalString" => {
                    let l = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                    let s = format!("{:o}", l as u64);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                "toBinaryString" => {
                    let l = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                    let s = format!("{:b}", l as u64);
                    let sid = heap.alloc_string(&s);
                    return Ok(Some(JavaValue::Object(Some(sid))));
                }
                _ => {}
            }
        }

        if cls == "java/lang/Boolean" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                        heap.set_field(r_id, "value", v);
                    }
                    return Ok(None);
                }
                "booleanValue" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "valueOf" => {
                    let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                    let obj = heap.alloc_instance("java/lang/Boolean");
                    heap.set_field(obj, "value", v);
                    return Ok(Some(JavaValue::Object(Some(obj))));
                }
                _ => {}
            }
        }

        if cls == "java/lang/Byte" || cls == "java/lang/Short" || cls == "java/lang/Character" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                        heap.set_field(r_id, "value", v);
                    }
                    return Ok(None);
                }
                "byteValue" | "shortValue" | "charValue" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "toLowerCase" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as u32;
                    let low = std::char::from_u32(c)
                        .map(|ch| ch.to_ascii_lowercase() as i32)
                        .unwrap_or(c as i32);
                    return Ok(Some(JavaValue::Int(low)));
                }
                "toUpperCase" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as u32;
                    let up = std::char::from_u32(c)
                        .map(|ch| ch.to_ascii_uppercase() as i32)
                        .unwrap_or(c as i32);
                    return Ok(Some(JavaValue::Int(up)));
                }
                "isDigit" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as u32;
                    let d = std::char::from_u32(c)
                        .map(|ch| ch.is_ascii_digit())
                        .unwrap_or(false);
                    return Ok(Some(JavaValue::Int(if d { 1 } else { 0 })));
                }
                "parseShort" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let radix = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(10) as u32;
                    let val = i16::from_str_radix(s.trim(), radix).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(val as i32)));
                }
                "parseByte" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let radix = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(10) as u32;
                    let val = i8::from_str_radix(s.trim(), radix).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(val as i32)));
                }
                _ => {}
            }
        }

        // 12b. java/util/Calendar
        if cls == "java/util/Calendar" || class_loader.is_instance_of(&cls, "java/util/Calendar") {
            match method_name {
                "getInstance" => {
                    let cal = heap.alloc_instance("java/util/Calendar");
                    return Ok(Some(JavaValue::Object(Some(cal))));
                }
                "get" => {
                    let field = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    // Standard Calendar fields in CLDC:
                    // YEAR = 1, MONTH = 2 (0-based), DATE = 5, HOUR = 10, HOUR_OF_DAY = 11,
                    // MINUTE = 12, SECOND = 13, MILLISECOND = 14, DAY_OF_WEEK = 7
                    let val = match field {
                        1 => 2026,
                        2 => 9,   // October (0-indexed)
                        5 => 25,  // 25th
                        7 => 1,   // Sunday
                        11 => 12, // 12h
                        12 => 0,
                        13 => 0,
                        14 => 0,
                        _ => 0,
                    };
                    return Ok(Some(JavaValue::Int(val)));
                }
                "setTime" | "setTimeZone" => {
                    return Ok(None);
                }
                "getTime" => {
                    let date_id = heap.alloc_instance("java/util/Date");
                    let now = if scheduler.current_time_ms > 0 {
                        scheduler.current_time_ms as i64
                    } else {
                        1792940000000i64
                    };
                    heap.set_field(date_id, "fastTime", JavaValue::Long(now));
                    context.dates.insert(date_id, now);
                    return Ok(Some(JavaValue::Object(Some(date_id))));
                }
                "getTimeInMillis" => {
                    return Ok(Some(JavaValue::Long(1792940000000i64)));
                }
                _ => {}
            }
        }

        // 12c. java/util/Date
        if cls == "java/util/Date" || class_loader.is_instance_of(&cls, "java/util/Date") {
            let r_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let time = if let Some(first) = args.first() {
                        first.as_long().unwrap_or_else(|_| {
                            if scheduler.current_time_ms > 0 {
                                scheduler.current_time_ms as i64
                            } else {
                                1792940000000i64
                            }
                        })
                    } else {
                        if scheduler.current_time_ms > 0 {
                            scheduler.current_time_ms as i64
                        } else {
                            1792940000000i64
                        }
                    };
                    heap.set_field(r_id, "fastTime", JavaValue::Long(time));
                    context.dates.insert(r_id, time);
                    return Ok(None);
                }
                "getTime" => {
                    let time = context.dates.get(&r_id).copied().unwrap_or_else(|| {
                        heap.get_field(r_id, "fastTime")
                            .as_long()
                            .unwrap_or_else(|_| {
                                if scheduler.current_time_ms > 0 {
                                    scheduler.current_time_ms as i64
                                } else {
                                    1792940000000i64
                                }
                            })
                    });
                    return Ok(Some(JavaValue::Long(time)));
                }
                "setTime" => {
                    let time = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                    heap.set_field(r_id, "fastTime", JavaValue::Long(time));
                    context.dates.insert(r_id, time);
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        // 12d. java/security/MessageDigest
        if cls == "java/security/MessageDigest"
            || class_loader.is_instance_of(&cls, "java/security/MessageDigest")
        {
            let r_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "getInstance" => {
                    let algo_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let algo = algo_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_else(|| "MD5".to_string())
                        .to_uppercase();
                    let md_id = heap.alloc_instance("java/security/MessageDigest");
                    context.message_digests.insert(
                        md_id,
                        crate::bridge::context::MessageDigestState {
                            algorithm: algo,
                            buffer: Vec::new(),
                        },
                    );
                    return Ok(Some(JavaValue::Object(Some(md_id))));
                }
                "<init>" => return Ok(None),
                "update" => {
                    let state = context.message_digests.entry(r_id).or_default();
                    if args.len() == 1 {
                        if let Ok(v) = args[0].as_int() {
                            state.buffer.push(v as u8);
                            return Ok(None);
                        } else if let Ok(arr_opt) = args[0].as_object() {
                            let arr_id = match arr_opt {
                                Some(id) => id,
                                None => {
                                    let exc = heap.alloc_instance("java/lang/NullPointerException");
                                    return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                                }
                            };
                            if let Some(ObjectData::Array(ArrayData::Byte(bytes))) = heap.get(arr_id) {
                                state.buffer.extend(bytes.iter().map(|&b| b as u8));
                            }
                            return Ok(None);
                        }
                    } else if args.len() >= 3 {
                        let arr_id = match args[0].as_object().ok().flatten() {
                            Some(id) => id,
                            None => {
                                let exc = heap.alloc_instance("java/lang/NullPointerException");
                                return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                            }
                        };
                        let offset = args[1].as_int().unwrap_or(0);
                        let len = args[2].as_int().unwrap_or(0);
                        if offset < 0 || len < 0 {
                            let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                        }
                        if let Some(ObjectData::Array(ArrayData::Byte(bytes))) = heap.get(arr_id) {
                            let u_offset = offset as usize;
                            let u_len = len as usize;
                            if u_offset + u_len > bytes.len() {
                                let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                            }
                            state
                                .buffer
                                .extend(bytes[u_offset..u_offset + u_len].iter().map(|&b| b as u8));
                        }
                        return Ok(None);
                    }
                    return Ok(None);
                }
                "digest" => {
                    let state = context.message_digests.entry(r_id).or_default();
                    if let Some(first) = args.first() {
                        if args.len() == 1 {
                            // digest(byte[] input)
                            let arr_id = match first.as_object().ok().flatten() {
                                Some(id) => id,
                                None => {
                                    let exc = heap.alloc_instance("java/lang/NullPointerException");
                                    return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                                }
                            };
                            if let Some(ObjectData::Array(ArrayData::Byte(bytes))) = heap.get(arr_id) {
                                state.buffer.extend(bytes.iter().map(|&b| b as u8));
                            }
                        } else if args.len() >= 3 {
                            // digest(byte[] buf, int offset, int len) -> int
                            let arr_id = match first.as_object().ok().flatten() {
                                Some(id) => id,
                                None => {
                                    let exc = heap.alloc_instance("java/lang/NullPointerException");
                                    return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                                }
                            };
                            let offset = args[1].as_int().unwrap_or(0);
                            let len = args[2].as_int().unwrap_or(0);
                            let digest_bytes = if state.algorithm.contains("SHA") {
                                crate::cldc::sha1_digest(&state.buffer).to_vec()
                            } else {
                                crate::cldc::md5_digest(&state.buffer).to_vec()
                            };
                            state.buffer.clear();

                            if offset < 0
                                || len < 0
                                || (offset as usize + digest_bytes.len())
                                    > (offset as usize + len as usize)
                            {
                                let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                            }
                            if let Some(dst_obj) = heap.get_mut(arr_id) {
                                if let Some(ArrayData::Byte(target_bytes)) = dst_obj.as_array_mut() {
                                    let u_offset = offset as usize;
                                    if u_offset + digest_bytes.len() > target_bytes.len() {
                                        let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                        return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                                    }
                                    for (i, &b) in digest_bytes.iter().enumerate() {
                                        target_bytes[u_offset + i] = b as i8;
                                    }
                                    return Ok(Some(JavaValue::Int(digest_bytes.len() as i32)));
                                }
                            }
                            return Ok(Some(JavaValue::Int(0)));
                        }
                    }
                    // digest() -> byte[]
                    let digest_bytes = if state.algorithm.contains("SHA") {
                        crate::cldc::sha1_digest(&state.buffer).to_vec()
                    } else {
                        crate::cldc::md5_digest(&state.buffer).to_vec()
                    };
                    state.buffer.clear();
                    let out_arr = heap.alloc_array(ArrayData::Byte(
                        digest_bytes.into_iter().map(|b| b as i8).collect(),
                    ));
                    return Ok(Some(JavaValue::Object(Some(out_arr))));
                }
                "reset" => {
                    if let Some(state) = context.message_digests.get_mut(&r_id) {
                        state.buffer.clear();
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        // 13. java/util/Vector & java/util/Stack
        if cls == "java/util/Vector"
            || cls == "java/util/Stack"
            || class_loader.is_instance_of(&cls, "java/util/Vector")
            || class_loader.is_instance_of(&cls, "java/util/Stack")
        {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let elements_id =
                            heap.alloc_array(ArrayData::Reference(Vec::with_capacity(16)));
                        heap.set_field(r_id, "elementData", JavaValue::Object(Some(elements_id)));
                        heap.set_field(r_id, "elementCount", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "addElement" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            v.push(elem);
                            let count = v.len() as i32;
                            heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                        }
                    }
                    return Ok(None);
                }
                "elementAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let idx = args[0].as_int().unwrap_or(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            let elem = v.get(idx).copied().flatten();
                            return Ok(Some(JavaValue::Object(elem)));
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "setElementAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args[0].as_object().ok().flatten();
                    let idx = args[1].as_int().unwrap_or(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            if idx < v.len() {
                                v[idx] = elem;
                            }
                        }
                    }
                    return Ok(None);
                }
                "size" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let count = heap.get_field(r_id, "elementCount").as_int().unwrap_or(0);
                    return Ok(Some(JavaValue::Int(count)));
                }
                "isEmpty" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let count = heap.get_field(r_id, "elementCount").as_int().unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if count == 0 { 1 } else { 0 })));
                }
                "removeAllElements" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            v.clear();
                            heap.set_field(r_id, "elementCount", JavaValue::Int(0));
                        }
                    }
                    return Ok(None);
                }
                "removeElementAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let idx = args[0].as_int().unwrap_or(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            if idx < v.len() {
                                v.remove(idx);
                                let count = v.len() as i32;
                                heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                            }
                        }
                    }
                    return Ok(None);
                }
                "insertElementAt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let idx = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            let insert_pos = idx.min(v.len());
                            v.insert(insert_pos, elem);
                            let count = v.len() as i32;
                            heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                        }
                    }
                    return Ok(None);
                }
                "removeElement" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        let pos = if let Some(ObjectData::Array(ArrayData::Reference(v))) =
                            heap.get(eid)
                        {
                            v.iter()
                                .position(|&x| Self::java_objects_equal(heap, x, elem))
                        } else {
                            None
                        };
                        if let Some(p) = pos {
                            if let Some(ObjectData::Array(ArrayData::Reference(v))) =
                                heap.get_mut(eid)
                            {
                                v.remove(p);
                                let count = v.len() as i32;
                                heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                                return Ok(Some(JavaValue::Int(1)));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "contains" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            let found = v.iter().any(|&x| Self::java_objects_equal(heap, x, elem));
                            return Ok(Some(JavaValue::Int(if found { 1 } else { 0 })));
                        }
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "indexOf" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let start_from = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            for i in start_from..v.len() {
                                if Self::java_objects_equal(heap, v[i], elem) {
                                    return Ok(Some(JavaValue::Int(i as i32)));
                                }
                            }
                        }
                    }
                    return Ok(Some(JavaValue::Int(-1)));
                }
                "lastIndexOf" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            if let Some(pos) = v
                                .iter()
                                .rposition(|&x| Self::java_objects_equal(heap, x, elem))
                            {
                                return Ok(Some(JavaValue::Int(pos as i32)));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::Int(-1)));
                }
                "firstElement" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            if let Some(&first) = v.first() {
                                return Ok(Some(JavaValue::Object(first)));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "lastElement" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            if let Some(&last) = v.last() {
                                return Ok(Some(JavaValue::Object(last)));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "copyInto" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let dst_arr_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let (Some(eid), Some(did)) = (ed_id, dst_arr_id) {
                        let src_vec = if let Some(ObjectData::Array(ArrayData::Reference(v))) =
                            heap.get(eid)
                        {
                            v.clone()
                        } else {
                            Vec::new()
                        };
                        if let Some(ObjectData::Array(ArrayData::Reference(dst))) =
                            heap.get_mut(did)
                        {
                            for (i, elem) in src_vec.iter().enumerate() {
                                if i < dst.len() {
                                    dst[i] = *elem;
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "elements" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let enum_obj = heap.alloc_instance("java/util/VectorEnumerator");
                    heap.set_field(enum_obj, "target", JavaValue::Object(Some(r_id)));
                    heap.set_field(enum_obj, "cursor", JavaValue::Int(0));
                    return Ok(Some(JavaValue::Object(Some(enum_obj))));
                }
                "capacity" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    let cap = ed_id
                        .and_then(|id| heap.get(id))
                        .and_then(|o| o.as_array())
                        .map(|a| a.len())
                        .unwrap_or(16);
                    return Ok(Some(JavaValue::Int(cap as i32)));
                }
                "ensureCapacity" | "trimToSize" => {
                    return Ok(None);
                }
                "setSize" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let new_size = args
                        .first()
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            v.resize(new_size, None);
                            heap.set_field(r_id, "elementCount", JavaValue::Int(new_size as i32));
                        }
                    }
                    return Ok(None);
                }
                "push" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let elem = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            v.push(elem);
                            let count = v.len() as i32;
                            heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                        }
                    }
                    return Ok(Some(JavaValue::Object(elem)));
                }
                "pop" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(eid)
                        {
                            if let Some(item) = v.pop() {
                                let count = v.len() as i32;
                                heap.set_field(r_id, "elementCount", JavaValue::Int(count));
                                return Ok(Some(JavaValue::Object(item)));
                            }
                        }
                    }
                    let exc = heap.alloc_instance("java/util/EmptyStackException");
                    return Err(format!("EXC:java/util/EmptyStackException:{}", exc.0));
                }
                "peek" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            if let Some(&item) = v.last() {
                                return Ok(Some(JavaValue::Object(item)));
                            }
                        }
                    }
                    let exc = heap.alloc_instance("java/util/EmptyStackException");
                    return Err(format!("EXC:java/util/EmptyStackException:{}", exc.0));
                }
                "empty" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            return Ok(Some(JavaValue::Int(if v.is_empty() { 1 } else { 0 })));
                        }
                    }
                    return Ok(Some(JavaValue::Int(1)));
                }
                "search" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let target = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let ed_id = heap
                        .get_field(r_id, "elementData")
                        .as_object()
                        .ok()
                        .flatten();
                    if let Some(eid) = ed_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(eid) {
                            for (idx, &item) in v.iter().rev().enumerate() {
                                if item == target {
                                    return Ok(Some(JavaValue::Int((idx + 1) as i32)));
                                }
                            }
                        }
                    }
                    return Ok(Some(JavaValue::Int(-1)));
                }
                _ => {}
            }
        }

        // 14. java/util/Hashtable
        if cls == "java/util/Hashtable" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let keys_id = heap.alloc_array(ArrayData::Reference(Vec::new()));
                        let vals_id = heap.alloc_array(ArrayData::Reference(Vec::new()));
                        heap.set_field(r_id, "keys", JavaValue::Object(Some(keys_id)));
                        heap.set_field(r_id, "vals", JavaValue::Object(Some(vals_id)));
                    }
                    return Ok(None);
                }
                "put" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let key = args[0].as_object().ok().flatten();
                    let val = args[1].as_object().ok().flatten();
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();

                    if let (Some(kid), Some(vid)) = (keys_id, vals_id) {
                        let found_idx =
                            if let Some(ObjectData::Array(ArrayData::Reference(k_vec))) =
                                heap.get(kid)
                            {
                                k_vec
                                    .iter()
                                    .position(|&k| Self::java_objects_equal(heap, k, key))
                            } else {
                                None
                            };

                        let mut replaced = None;
                        if let Some(idx) = found_idx {
                            if let Some(ObjectData::Array(ArrayData::Reference(v_vec))) =
                                heap.get_mut(vid)
                            {
                                replaced = v_vec.get(idx).copied().flatten();
                                v_vec[idx] = val;
                            }
                        } else {
                            if let Some(ObjectData::Array(ArrayData::Reference(k_vec))) =
                                heap.get_mut(kid)
                            {
                                k_vec.push(key);
                            }
                            if let Some(ObjectData::Array(ArrayData::Reference(v_vec))) =
                                heap.get_mut(vid)
                            {
                                v_vec.push(val);
                            }
                        }
                        return Ok(Some(JavaValue::Object(replaced)));
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "get" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let key = args[0].as_object().ok().flatten();
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();

                    if let (Some(kid), Some(vid)) = (keys_id, vals_id) {
                        if let (
                            Some(ObjectData::Array(ArrayData::Reference(k_vec))),
                            Some(ObjectData::Array(ArrayData::Reference(v_vec))),
                        ) = (heap.get(kid), heap.get(vid))
                        {
                            for (i, &k) in k_vec.iter().enumerate() {
                                if Self::java_objects_equal(heap, k, key) {
                                    return Ok(Some(JavaValue::Object(
                                        v_vec.get(i).copied().flatten(),
                                    )));
                                }
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "containsKey" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let key = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    if let Some(kid) = keys_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(k_vec))) = heap.get(kid)
                        {
                            let found = k_vec
                                .iter()
                                .any(|&k| Self::java_objects_equal(heap, k, key));
                            return Ok(Some(JavaValue::Int(if found { 1 } else { 0 })));
                        }
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "contains" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let val = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();
                    if let Some(vid) = vals_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v_vec))) = heap.get(vid)
                        {
                            let found = v_vec
                                .iter()
                                .any(|&v| Self::java_objects_equal(heap, v, val));
                            return Ok(Some(JavaValue::Int(if found { 1 } else { 0 })));
                        }
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "remove" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let key = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();

                    if let (Some(kid), Some(vid)) = (keys_id, vals_id) {
                        let found_idx =
                            if let Some(ObjectData::Array(ArrayData::Reference(k_vec))) =
                                heap.get(kid)
                            {
                                k_vec
                                    .iter()
                                    .position(|&k| Self::java_objects_equal(heap, k, key))
                            } else {
                                None
                            };

                        if let Some(idx) = found_idx {
                            if let Some(ObjectData::Array(ArrayData::Reference(k_vec))) =
                                heap.get_mut(kid)
                            {
                                k_vec.remove(idx);
                            }
                            let removed_val =
                                if let Some(ObjectData::Array(ArrayData::Reference(v_vec))) =
                                    heap.get_mut(vid)
                                {
                                    if idx < v_vec.len() {
                                        v_vec.remove(idx)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                            return Ok(Some(JavaValue::Object(removed_val)));
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "size" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let len = keys_id
                        .and_then(|id| heap.get(id))
                        .and_then(|o| o.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(len as i32)));
                }
                "isEmpty" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let len = keys_id
                        .and_then(|id| heap.get(id))
                        .and_then(|o| o.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if len == 0 { 1 } else { 0 })));
                }
                "clear" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();
                    if let Some(kid) = keys_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(k))) = heap.get_mut(kid)
                        {
                            k.clear();
                        }
                    }
                    if let Some(vid) = vals_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get_mut(vid)
                        {
                            v.clear();
                        }
                    }
                    return Ok(None);
                }
                "keys" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let keys_id = heap.get_field(r_id, "keys").as_object().ok().flatten();
                    let enum_obj = heap.alloc_instance("java/util/HashtableEnumerator");
                    heap.set_field(enum_obj, "array", JavaValue::Object(keys_id));
                    heap.set_field(enum_obj, "cursor", JavaValue::Int(0));
                    return Ok(Some(JavaValue::Object(Some(enum_obj))));
                }
                "elements" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let vals_id = heap.get_field(r_id, "vals").as_object().ok().flatten();
                    let enum_obj = heap.alloc_instance("java/util/HashtableEnumerator");
                    heap.set_field(enum_obj, "array", JavaValue::Object(vals_id));
                    heap.set_field(enum_obj, "cursor", JavaValue::Int(0));
                    return Ok(Some(JavaValue::Object(Some(enum_obj))));
                }
                _ => {}
            }
        }

        // 14.1 java/util/Enumeration
        if cls.ends_with("Enumerator") || cls == "java/util/Enumeration" {
            match method_name {
                "hasMoreElements" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cursor = heap.get_field(r_id, "cursor").as_int().unwrap_or(0);
                    let arr_id = heap
                        .get_field(r_id, "array")
                        .as_object()
                        .ok()
                        .flatten()
                        .or_else(|| {
                            let target =
                                heap.get_field(r_id, "target").as_object().ok().flatten()?;
                            heap.get_field(target, "elementData")
                                .as_object()
                                .ok()
                                .flatten()
                        });
                    let len = arr_id
                        .and_then(|id| heap.get(id))
                        .and_then(|o| o.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if (cursor as usize) < len {
                        1
                    } else {
                        0
                    })));
                }
                "nextElement" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let cursor = heap.get_field(r_id, "cursor").as_int().unwrap_or(0);
                    heap.set_field(r_id, "cursor", JavaValue::Int(cursor + 1));
                    let arr_id = heap
                        .get_field(r_id, "array")
                        .as_object()
                        .ok()
                        .flatten()
                        .or_else(|| {
                            let target =
                                heap.get_field(r_id, "target").as_object().ok().flatten()?;
                            heap.get_field(target, "elementData")
                                .as_object()
                                .ok()
                                .flatten()
                        });
                    if let Some(aid) = arr_id {
                        if let Some(ObjectData::Array(ArrayData::Reference(v))) = heap.get(aid) {
                            if let Some(elem) = v.get(cursor as usize).copied().flatten() {
                                return Ok(Some(JavaValue::Object(Some(elem))));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                _ => {}
            }
        }

        // 15. java/util/Random
        if cls == "java/util/Random" {
            match method_name {
                "<init>" => {
                    let seed = args
                        .first()
                        .and_then(|v| v.as_long().ok())
                        .map(|l| l as u64)
                        .unwrap_or(context.rng_seed);
                    if let Some(r_id) = receiver {
                        heap.set_field(r_id, "seed", JavaValue::Long(seed as i64));
                    }
                    return Ok(None);
                }
                "setSeed" => {
                    let seed = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0) as u64;
                    if let Some(r_id) = receiver {
                        heap.set_field(r_id, "seed", JavaValue::Long(seed as i64));
                    }
                    return Ok(None);
                }
                "nextInt" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    let mut seed =
                        heap.get_field(r_id, "seed")
                            .as_long()
                            .unwrap_or(context.rng_seed as i64) as u64;
                    // Standard Java LCG
                    seed = (seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1u64 << 48) - 1);
                    heap.set_field(r_id, "seed", JavaValue::Long(seed as i64));
                    context.rng_seed = seed;

                    let raw_val = (seed >> 17) as i32;
                    if let Some(first) = args.first() {
                        let n = first.as_int().unwrap_or(1);
                        if n <= 0 {
                            return Ok(Some(JavaValue::Int(0)));
                        }
                        return Ok(Some(JavaValue::Int((raw_val % n).abs())));
                    }
                    return Ok(Some(JavaValue::Int(raw_val)));
                }
                _ => {}
            }
        }

        // 16. java/util/Timer & TimerTask
        if cls == "java/util/Timer" || cls.ends_with("/Timer") {
            match method_name {
                "<init>" => return Ok(None),
                "schedule" | "scheduleAtFixedRate" => {
                    if let Some(task_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        let delay_ms = if args.len() >= 2 {
                            if let Ok(l) = args[1].as_long() {
                                l.max(0) as u64
                            } else if let Ok(i) = args[1].as_int() {
                                i.max(0) as u64
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let period_ms = if args.len() >= 3 {
                            if let Ok(l) = args[2].as_long() {
                                if l > 0 {
                                    Some(l as u64)
                                } else {
                                    None
                                }
                            } else if let Ok(i) = args[2].as_int() {
                                if i > 0 {
                                    Some(i as u64)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let is_fixed_rate = method_name == "scheduleAtFixedRate";
                        let timer_id = receiver.unwrap_or(ObjectId(1));
                        if let Err(_e) = context.timers.schedule(
                            timer_id,
                            task_id,
                            delay_ms,
                            period_ms,
                            is_fixed_rate,
                            scheduler.current_time_ms,
                        ) {
                            let exc_id = heap.alloc_instance("java/lang/IllegalStateException");
                            return Err(format!(
                                "EXC:java/lang/IllegalStateException:{}",
                                exc_id.0
                            ));
                        }
                    }
                    return Ok(None);
                }
                "cancel" => {
                    if let Some(timer_id) = receiver {
                        context.timers.cancel_timer(timer_id);
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        if cls == "java/util/TimerTask"
            || cls.ends_with("/TimerTask")
            || class_loader.is_instance_of(&cls, "java/util/TimerTask")
        {
            match method_name {
                "<init>" => return Ok(None),
                "cancel" => {
                    let res = receiver
                        .map(|id| context.timers.cancel_task(id))
                        .unwrap_or(false);
                    return Ok(Some(JavaValue::Int(if res { 1 } else { 0 })));
                }
                "scheduledExecutionTime" => {
                    let t = receiver
                        .and_then(|id| context.timers.scheduled_execution_time(id))
                        .unwrap_or(scheduler.current_time_ms);
                    return Ok(Some(JavaValue::Long(t as i64)));
                }
                "run" => return Ok(None),
                _ => return Ok(None),
            }
        }

        // 17. MIDlet
        if cls.ends_with("MIDlet")
            || cls == "javax/microedition/midlet/MIDlet"
            || class_loader.is_instance_of(&cls, "javax/microedition/midlet/MIDlet")
        {
            match method_name {
                "getAppProperty" => {
                    let key_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let key = key_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    if let Some(ref jar) = context.jar {
                        if let Some(ref manifest) = jar.manifest {
                            if let Some(val) = manifest.get(&key) {
                                let val_id = heap.alloc_string(val);
                                return Ok(Some(JavaValue::Object(Some(val_id))));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "platformRequest" => {
                    return Ok(Some(JavaValue::Int(1)));
                }
                "checkPermission" => {
                    return Ok(Some(JavaValue::Int(1)));
                }
                "notifyDestroyed" | "notifyPaused" => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 18. Display & Displayable
        if cls == "javax/microedition/lcdui/Display" {
            match method_name {
                "getDisplay" => {
                    let disp_id = heap.alloc_instance("javax/microedition/lcdui/Display");
                    return Ok(Some(JavaValue::Object(Some(disp_id))));
                }
                "setCurrent" => {
                    let dispable = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let prev_dispable = context.current_displayable;
                    context.current_displayable = dispable;
                    context.screen_dirty = true;
                    context.repaint_requested = true;

                    if prev_dispable != dispable {
                        context.ui_queue.canvas_changed(prev_dispable, dispable);
                        context.key_states = 0;
                        context.latched_key_states = 0;

                        if thread_idx < scheduler.threads.len() {
                            // 1. Prepare showNotify on new displayable (pushed first, executes last)
                            if let Some(new_id) = dispable {
                                context.ui_queue.cancel_show_notify_for(new_id);
                                if let Some(ObjectData::Instance(inst)) = heap.get(new_id) {
                                    let cls = inst.class_name.clone();
                                    if let Some(show_m) = class_loader.find_method(&cls, "showNotify", "()V") {
                                        let mut show_frame = Frame::new(
                                            cls.clone(),
                                            "showNotify".to_string(),
                                            "()V".to_string(),
                                            Arc::clone(&show_m.instructions),
                                            show_m.max_locals,
                                            show_m.max_stack,
                                            show_m.exception_table.clone(),
                                            Arc::clone(&show_m.pc_map),
                                        );
                                        show_frame.set_local(0, JavaValue::Object(Some(new_id)));
                                        if show_m.is_synchronized() {
                                            show_frame.sync_monitor = Some(new_id);
                                            let tid = scheduler.threads[thread_idx].id;
                                            if !scheduler.try_acquire_monitor(tid, new_id) {
                                                scheduler.block_on_monitor(tid, new_id, 1);
                                            }
                                        }
                                        scheduler.threads[thread_idx].call_stack.push(show_frame);
                                    }
                                }
                            }

                            // 3. Prepare hideNotify on old displayable (pushed third, executes first)
                            if let Some(old_id) = prev_dispable {
                                context.ui_queue.cancel_hide_notify_for(old_id);
                                if let Some(ObjectData::Instance(inst)) = heap.get(old_id) {
                                    let cls = inst.class_name.clone();
                                    if let Some(hide_m) = class_loader.find_method(&cls, "hideNotify", "()V") {
                                        let mut hide_frame = Frame::new(
                                            cls,
                                            "hideNotify".to_string(),
                                            "()V".to_string(),
                                            Arc::clone(&hide_m.instructions),
                                            hide_m.max_locals,
                                            hide_m.max_stack,
                                            hide_m.exception_table.clone(),
                                            Arc::clone(&hide_m.pc_map),
                                        );
                                        hide_frame.set_local(0, JavaValue::Object(Some(old_id)));
                                        if hide_m.is_synchronized() {
                                            hide_frame.sync_monitor = Some(old_id);
                                            let tid = scheduler.threads[thread_idx].id;
                                            if !scheduler.try_acquire_monitor(tid, old_id) {
                                                scheduler.block_on_monitor(tid, old_id, 1);
                                            }
                                        }
                                        scheduler.threads[thread_idx].call_stack.push(hide_frame);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(disp_id) = dispable {
                        context.request_repaint(disp_id);
                    }
                    return Ok(None);
                }
                "getCurrent" => {
                    return Ok(Some(JavaValue::Object(context.current_displayable)));
                }
                "isColor" => return Ok(Some(JavaValue::Int(1))),
                "numColors" => return Ok(Some(JavaValue::Int(65536))),
                "numAlphaLevels" => return Ok(Some(JavaValue::Int(256))),
                "flashBacklight" => return Ok(Some(JavaValue::Int(1))),
                "getColor" => return Ok(Some(JavaValue::Int(0x00FFFFFF))),
                "getBorderStyle" => return Ok(Some(JavaValue::Int(0))),
                "getBestImageWidth" | "getBestImageHeight" => return Ok(Some(JavaValue::Int(16))),
                "vibrate" => {
                    let dur = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    context.vibration_active = dur > 0;
                    return Ok(Some(JavaValue::Int(1)));
                }
                "callSerially" => {
                    if let Some(r_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        context.post_call_serially(r_id);
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        let is_displayable = cls.ends_with("Canvas")
            || cls.ends_with("FullCanvas")
            || cls == "javax/microedition/lcdui/Displayable"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/Displayable")
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/Canvas")
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/game/GameCanvas");

        if is_displayable {
            match method_name {
                "getWidth" => return Ok(Some(JavaValue::Int(context.screen_width as i32))),
                "getHeight" => {
                    let disp_id = receiver.or(context.current_displayable);
                    let is_full = if let Some(id) = disp_id {
                        let is_full_canvas = if let Some(ObjectData::Instance(inst)) = heap.get(id)
                        {
                            class_loader
                                .is_instance_of(&inst.class_name, "com/nokia/mid/ui/FullCanvas")
                        } else {
                            false
                        };
                        context
                            .displayable_commands
                            .get(&id)
                            .map(|e| e.is_full_screen)
                            .unwrap_or(is_full_canvas)
                    } else {
                        false
                    };
                    let h = if is_full {
                        context.screen_height as i32
                    } else {
                        (context.screen_height as i32) - 24
                    };
                    return Ok(Some(JavaValue::Int(h)));
                }
                "isShown" => {
                    let is_current = receiver.is_none() || receiver == context.current_displayable;
                    return Ok(Some(JavaValue::Int(if is_current { 1 } else { 0 })));
                }
                "repaint" => {
                    context.repaint_requested = true;
                    context.screen_dirty = true;
                    if let Some(disp_id) = receiver.or(context.current_displayable) {
                        context.request_repaint(disp_id);
                    }
                    return Ok(None);
                }
                "serviceRepaints" => {
                    context.repaint_requested = false;
                    if context.repaints_this_tick >= context.max_repaints_per_tick {
                        context.tick_budget_exhausted = true;
                    }
                    context.repaints_this_tick += 1;
                    context.paint_calls_count += 1;
                    let target_disp = receiver.or(context.current_displayable);
                    if let Some(disp_id) = target_disp {
                        context.ui_queue.cancel_repaints_for(disp_id);
                        if let Some(ObjectData::Instance(inst)) = heap.get(disp_id) {
                            let cls = inst.class_name.clone();
                            if let Some(paint_m) = class_loader.find_method(
                                &cls,
                                "paint",
                                "(Ljavax/microedition/lcdui/Graphics;)V",
                            ) {
                                let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
                                context.get_or_create_graphics(g_id, GraphicsTarget::Display);
                                let mut paint_frame = Frame::new(
                                    cls.clone(),
                                    "paint".to_string(),
                                    "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
                                    Arc::clone(&paint_m.instructions),
                                    paint_m.max_locals,
                                    paint_m.max_stack,
                                    paint_m.exception_table.clone(),
                                    Arc::clone(&paint_m.pc_map),
                                );
                                paint_frame.set_local(0, JavaValue::Object(Some(disp_id)));
                                paint_frame.set_local(1, JavaValue::Object(Some(g_id)));
                                if paint_m.is_synchronized() {
                                    paint_frame.sync_monitor = Some(disp_id);
                                    let tid = scheduler.threads[thread_idx].id;
                                    if !scheduler.try_acquire_monitor(tid, disp_id) {
                                        scheduler.block_on_monitor(tid, disp_id, 1);
                                    }
                                }
                                if thread_idx < scheduler.threads.len() {
                                    scheduler.threads[thread_idx].call_stack.push(paint_frame);

                                    // If showNotify is still pending in ui_queue, push it ON TOP so it runs BEFORE paint
                                    let has_pending_show = context.ui_queue.has_pending_show_notify(disp_id);
                                    if has_pending_show {
                                        context.ui_queue.cancel_show_notify_for(disp_id);
                                        if let Some(show_m) = class_loader.find_method(&cls, "showNotify", "()V") {
                                            let mut show_frame = Frame::new(
                                                cls,
                                                "showNotify".to_string(),
                                                "()V".to_string(),
                                                Arc::clone(&show_m.instructions),
                                                show_m.max_locals,
                                                show_m.max_stack,
                                                show_m.exception_table.clone(),
                                                Arc::clone(&show_m.pc_map),
                                            );
                                            show_frame.set_local(0, JavaValue::Object(Some(disp_id)));
                                            if show_m.is_synchronized() {
                                                show_frame.sync_monitor = Some(disp_id);
                                                let tid = scheduler.threads[thread_idx].id;
                                                if !scheduler.try_acquire_monitor(tid, disp_id) {
                                                    scheduler.block_on_monitor(tid, disp_id, 1);
                                                }
                                            }
                                            scheduler.threads[thread_idx].call_stack.push(show_frame);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    context.screen_dirty = true;
                    return Ok(None);
                }
                "isDoubleBuffered" => return Ok(Some(JavaValue::Int(1))),
                "setFullScreenMode" => {
                    let mode = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;
                    if let Some(disp_id) = receiver.or(context.current_displayable) {
                        let prev_mode = context.displayable_commands.get(&disp_id).map(|e| e.is_full_screen).unwrap_or(false);
                        let entry = context.displayable_commands.entry(disp_id).or_default();
                        entry.is_full_screen = mode;

                        let w = context.screen_width as i32;
                        let h = if mode { context.screen_height as i32 } else { (context.screen_height as i32) - 24 };

                        if prev_mode != mode {
                            if let Some(ObjectData::Instance(inst)) = heap.get(disp_id) {
                                let cls = inst.class_name.clone();
                                if let Some(m) = class_loader.find_method(&cls, "sizeChanged", "(II)V") {
                                    let mut frame = Frame::new(
                                        cls,
                                        "sizeChanged".to_string(),
                                        "(II)V".to_string(),
                                        Arc::clone(&m.instructions),
                                        m.max_locals,
                                        m.max_stack,
                                        m.exception_table.clone(),
                                        Arc::clone(&m.pc_map),
                                    );
                                    frame.set_local(0, JavaValue::Object(Some(disp_id)));
                                    frame.set_local(1, JavaValue::Int(w));
                                    frame.set_local(2, JavaValue::Int(h));
                                    if m.is_synchronized() {
                                        frame.sync_monitor = Some(disp_id);
                                        if thread_idx < scheduler.threads.len() {
                                            let tid = scheduler.threads[thread_idx].id;
                                            if !scheduler.try_acquire_monitor(tid, disp_id) {
                                                scheduler.block_on_monitor(tid, disp_id, 1);
                                            }
                                        }
                                    }
                                    if thread_idx < scheduler.threads.len() {
                                        scheduler.threads[thread_idx].call_stack.push(frame);
                                    }
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "flushGraphics" => {
                    let canvas_id = receiver.unwrap_or(ObjectId(0));
                    if context.current_displayable == Some(canvas_id) {
                        if let Some(src_buf) = context.game_canvas_buffers.get(&canvas_id) {
                            if src_buf.len() == context.framebuffer.len() {
                                if context.repaints_this_tick >= context.max_repaints_per_tick {
                                    context.tick_budget_exhausted = true;
                                }
                                context.repaints_this_tick += 1;
                                if args.len() >= 4 {
                                    let x = args[0].as_int().unwrap_or(0);
                                    let y = args[1].as_int().unwrap_or(0);
                                    let w = args[2].as_int().unwrap_or(0);
                                    let h = args[3].as_int().unwrap_or(0);
                                    if w > 0 && h > 0 {
                                        let sw = context.screen_width as i32;
                                        let sh = context.screen_height as i32;
                                        let x1 = x.max(0).min(sw);
                                        let y1 = y.max(0).min(sh);
                                        let x2 = (x + w).max(0).min(sw);
                                        let y2 = (y + h).max(0).min(sh);
                                        if x2 > x1 && y2 > y1 {
                                            let copy_w = (x2 - x1) as usize;
                                            for row in y1..y2 {
                                                let row_src = (row * sw + x1) as usize;
                                                let row_dst = (row * sw + x1) as usize;
                                                context.framebuffer[row_dst..row_dst + copy_w]
                                                    .copy_from_slice(
                                                        &src_buf[row_src..row_src + copy_w],
                                                    );
                                            }
                                            context.screen_dirty = true;
                                            context.fb_publishes_count += 1;
                                            let new_hash = crate::bridge::context::hash_framebuffer(&context.framebuffer);
                                            if new_hash != context.last_framebuffer_hash {
                                                context.last_framebuffer_hash = new_hash;
                                                context.framebuffer_changes_count += 1;
                                            }
                                        }
                                    }
                                } else {
                                    context.framebuffer.copy_from_slice(src_buf);
                                    context.screen_dirty = true;
                                    context.fb_publishes_count += 1;
                                    let new_hash = crate::bridge::context::hash_framebuffer(&context.framebuffer);
                                    if new_hash != context.last_framebuffer_hash {
                                        context.last_framebuffer_hash = new_hash;
                                        context.framebuffer_changes_count += 1;
                                    }
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "getGraphics" => {
                    let canvas_id = receiver.unwrap_or(ObjectId(0));
                    context.get_or_create_game_canvas_buffer(canvas_id);
                    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
                    let sw = context.screen_width as i32;
                    let sh = context.screen_height as i32;
                    let gc =
                        context.get_or_create_graphics(g_id, GraphicsTarget::GameCanvas(canvas_id));
                    gc.target = GraphicsTarget::GameCanvas(canvas_id);
                    gc.graphics.reset_state(sw, sh);
                    return Ok(Some(JavaValue::Object(Some(g_id))));
                }
                "getKeyStates" => {
                    let states = context.get_and_clear_key_states();
                    if states != 0 {
                        context.input_affected_game_state = true;
                    }
                    return Ok(Some(JavaValue::Int(states)));
                }
                "getGameAction" => {
                    let key = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let action = match key {
                        -1 | 50 => 1,           // UP (or KEY_NUM2)
                        -3 | 52 => 2,           // LEFT (or KEY_NUM4)
                        -4 | 54 => 5,           // RIGHT (or KEY_NUM6)
                        -2 | 56 => 6,           // DOWN (or KEY_NUM8)
                        -5 | 53 | 10 | 32 => 8, // FIRE (or KEY_NUM5, Enter, Space)
                        49 | -6 => 9,           // GAME_A / SOFT1
                        51 | -7 => 10,          // GAME_B / SOFT2
                        55 => 11,               // GAME_C (KEY_NUM7)
                        57 => 12,               // GAME_D (KEY_NUM9)
                        _ => 0,
                    };
                    return Ok(Some(JavaValue::Int(action)));
                }
                "getKeyCode" => {
                    let action = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let key = match action {
                        1 => -1,  // UP
                        2 => -3,  // LEFT
                        5 => -4,  // RIGHT
                        6 => -2,  // DOWN
                        8 => -5,  // FIRE
                        9 => 49,  // GAME_A
                        10 => 51, // GAME_B
                        11 => 55, // GAME_C
                        12 => 57, // GAME_D
                        _ => 0,
                    };
                    return Ok(Some(JavaValue::Int(key)));
                }
                "getKeyName" => {
                    let key = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let name = match key {
                        -1 => "UP",
                        -2 => "DOWN",
                        -3 => "LEFT",
                        -4 => "RIGHT",
                        -5 => "FIRE",
                        -6 => "SOFT1",
                        -7 => "SOFT2",
                        -8 => "CLEAR",
                        48..=57 => {
                            let ch = (b'0' + (key - 48) as u8) as char;
                            let s_id = heap.alloc_string(&ch.to_string());
                            return Ok(Some(JavaValue::Object(Some(s_id))));
                        }
                        42 => "*",
                        35 => "#",
                        _ => "KEY",
                    };
                    let s_id = heap.alloc_string(name);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "hasRepeatEvents" => return Ok(Some(JavaValue::Int(1))),
                "hasPointerEvents" => return Ok(Some(JavaValue::Int(0))),
                "hasPointerMotionEvents" => return Ok(Some(JavaValue::Int(0))),
                "keyPressed" | "keyReleased" | "keyRepeated" | "pointerPressed"
                | "pointerReleased" | "pointerDragged" | "showNotify" | "hideNotify"
                | "sizeChanged" | "addCommand" | "removeCommand" | "setCommandListener"
                | "setTitle" => return Ok(None),
                _ => {}
            }
        }

        if cls == "javax/microedition/lcdui/Alert" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if let Some(arg) = args.first() {
                            heap.set_field(r_id, "title", *arg);
                        }
                    }
                    return Ok(None);
                }
                "setType" | "setTimeout" | "setString" | "setImage" | "setIndicator"
                | "addCommand" | "setCommandListener" => return Ok(None),
                "getString" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    return Ok(Some(heap.get_field(r_id, "title")));
                }
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/AlertType" {
            return Ok(Some(JavaValue::Object(receiver)));
        }

        if cls == "javax/microedition/lcdui/Command" {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        if let Some(label_arg) = args.first() {
                            heap.set_field(r_id, "label", *label_arg);
                        }
                        if let Some(type_arg) = args.get(1) {
                            heap.set_field(r_id, "commandType", *type_arg);
                        }
                        if let Some(prio_arg) = args.get(2) {
                            heap.set_field(r_id, "priority", *prio_arg);
                        }
                    }
                    return Ok(None);
                }
                "getLabel" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    return Ok(Some(heap.get_field(r_id, "label")));
                }
                "getCommandType" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    return Ok(Some(heap.get_field(r_id, "commandType")));
                }
                "getPriority" => {
                    let r_id = receiver.unwrap_or(ObjectId(0));
                    return Ok(Some(heap.get_field(r_id, "priority")));
                }
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/Gauge"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/Gauge")
        {
            match method_name {
                "<init>" => {
                    if let Some(r_id) = receiver {
                        let label = args.get(0).copied().unwrap_or(JavaValue::Object(None));
                        let interactive = args.get(1).copied().unwrap_or(JavaValue::Int(0));
                        let max_val = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(100);
                        let init_val = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                        heap.set_field(r_id, "label", label);
                        heap.set_field(r_id, "interactive", interactive);
                        heap.set_field(r_id, "maxValue", JavaValue::Int(max_val));
                        heap.set_field(r_id, "value", JavaValue::Int(init_val));
                    }
                    return Ok(None);
                }
                "getValue" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "value").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "setValue" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Int(0));
                        heap.set_field(r_id, "value", v);
                    }
                    return Ok(None);
                }
                "getMaxValue" => {
                    let v = receiver
                        .map(|id| {
                            let mv = heap.get_field(id, "maxValue").as_int().unwrap_or(100);
                            if mv <= 0 {
                                100
                            } else {
                                mv
                            }
                        })
                        .unwrap_or(100);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "setMaxValue" => {
                    if let Some(r_id) = receiver {
                        let v = args.first().copied().unwrap_or(JavaValue::Int(100));
                        heap.set_field(r_id, "maxValue", v);
                    }
                    return Ok(None);
                }
                "isInteractive" => {
                    let v = receiver
                        .map(|id| heap.get_field(id, "interactive").as_int().unwrap_or(0))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(v)));
                }
                _ => return Ok(None),
            }
        }

        // javax/microedition/lcdui/StringItem & Item
        if cls == "javax/microedition/lcdui/StringItem"
            || cls == "javax/microedition/lcdui/Item"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/StringItem")
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/Item")
        {
            let item_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let label_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let text_opt = args.get(1).and_then(|v| v.as_object().ok()).flatten();
                    let mode = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0); // 0 = PLAIN

                    let label_str = label_opt.and_then(|id| heap.get_string_utf8(id));
                    let text_str = text_opt.and_then(|id| heap.get_string_utf8(id));

                    heap.set_field(item_id, "label", JavaValue::Object(label_opt));
                    heap.set_field(item_id, "text", JavaValue::Object(text_opt));
                    heap.set_field(item_id, "appearanceMode", JavaValue::Int(mode));

                    context.string_items.insert(
                        item_id,
                        StringItemState {
                            label: label_str,
                            text: text_str,
                            appearance_mode: mode,
                            font: None,
                        },
                    );
                    return Ok(None);
                }
                "getText" => {
                    let text_val = heap.get_field(item_id, "text");
                    return Ok(Some(text_val));
                }
                "setText" => {
                    let text_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let text_str = text_opt.and_then(|id| heap.get_string_utf8(id));
                    heap.set_field(item_id, "text", JavaValue::Object(text_opt));
                    if let Some(state) = context.string_items.get_mut(&item_id) {
                        state.text = text_str;
                    }
                    context.screen_dirty = true;
                    return Ok(None);
                }
                "getLabel" => {
                    let label_val = heap.get_field(item_id, "label");
                    return Ok(Some(label_val));
                }
                "setLabel" => {
                    let label_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let label_str = label_opt.and_then(|id| heap.get_string_utf8(id));
                    heap.set_field(item_id, "label", JavaValue::Object(label_opt));
                    if let Some(state) = context.string_items.get_mut(&item_id) {
                        state.label = label_str;
                    }
                    context.screen_dirty = true;
                    return Ok(None);
                }
                "getAppearanceMode" => {
                    let mode = context
                        .string_items
                        .get(&item_id)
                        .map(|s| s.appearance_mode)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(mode)));
                }
                "setFont" => {
                    let font_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    if let Some(state) = context.string_items.get_mut(&item_id) {
                        state.font = font_opt;
                    }
                    return Ok(None);
                }
                "getFont" => {
                    let font_opt = context.string_items.get(&item_id).and_then(|s| s.font);
                    return Ok(Some(JavaValue::Object(font_opt)));
                }
                "setPreferredSize" | "setLayout" | "addCommand" | "setItemCommandListener"
                | "setDefaultCommand" => return Ok(None),
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/Form"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/Form")
        {
            let form_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let mut form_state = FormState::default();
                    if let Some(t_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(t_id) {
                            form_state.title = s;
                        }
                        heap.set_field(form_id, "title", JavaValue::Object(Some(t_id)));
                    }
                    context.forms.insert(form_id, form_state);
                    return Ok(None);
                }
                "append" => {
                    let arg_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let form_state = context.forms.entry(form_id).or_default();
                    if let Some(aid) = arg_id {
                        // Check if it's already an item or a plain string
                        let is_item = context.string_items.contains_key(&aid)
                            || heap
                                .get(aid)
                                .map(|o| {
                                    if let ObjectData::Instance(ref inst) = o {
                                        inst.class_name.contains("Item")
                                    } else {
                                        false
                                    }
                                })
                                .unwrap_or(false);

                        let final_item_id = if !is_item {
                            // If passed a String, auto-create a StringItem
                            if let Some(s) = heap.get_string_utf8(aid) {
                                let si_id =
                                    heap.alloc_instance("javax/microedition/lcdui/StringItem");
                                heap.set_field(si_id, "text", JavaValue::Object(Some(aid)));
                                context.string_items.insert(
                                    si_id,
                                    StringItemState {
                                        label: None,
                                        text: Some(s),
                                        appearance_mode: 0,
                                        font: None,
                                    },
                                );
                                si_id
                            } else {
                                aid
                            }
                        } else {
                            aid
                        };

                        form_state.items.push(final_item_id);
                        let idx = (form_state.items.len() - 1) as i32;
                        context.screen_dirty = true;
                        return Ok(Some(JavaValue::Int(idx)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "delete" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(form_state) = context.forms.get_mut(&form_id) {
                        if idx >= 0 && (idx as usize) < form_state.items.len() {
                            form_state.items.remove(idx as usize);
                            context.screen_dirty = true;
                        }
                    }
                    return Ok(None);
                }
                "deleteAll" => {
                    if let Some(form_state) = context.forms.get_mut(&form_id) {
                        form_state.items.clear();
                        context.screen_dirty = true;
                    }
                    return Ok(None);
                }
                "insert" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let item_id = args.get(1).and_then(|v| v.as_object().ok()).flatten();
                    if let (Some(form_state), Some(iid)) =
                        (context.forms.get_mut(&form_id), item_id)
                    {
                        let i = (idx.max(0) as usize).min(form_state.items.len());
                        form_state.items.insert(i, iid);
                        context.screen_dirty = true;
                    }
                    return Ok(None);
                }
                "set" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let item_id = args.get(1).and_then(|v| v.as_object().ok()).flatten();
                    if let (Some(form_state), Some(iid)) =
                        (context.forms.get_mut(&form_id), item_id)
                    {
                        if idx >= 0 && (idx as usize) < form_state.items.len() {
                            form_state.items[idx as usize] = iid;
                            context.screen_dirty = true;
                        }
                    }
                    return Ok(None);
                }
                "get" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(form_state) = context.forms.get(&form_id) {
                        if idx >= 0 && (idx as usize) < form_state.items.len() {
                            return Ok(Some(JavaValue::Object(Some(
                                form_state.items[idx as usize],
                            ))));
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                "size" => {
                    let s = context
                        .forms
                        .get(&form_id)
                        .map(|f| f.items.len() as i32)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(s)));
                }
                "getTitle" => {
                    let t_val = heap.get_field(form_id, "title");
                    return Ok(Some(t_val));
                }
                "setTitle" => {
                    let t_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    heap.set_field(form_id, "title", JavaValue::Object(t_id));
                    if let (Some(form_state), Some(tid)) = (context.forms.get_mut(&form_id), t_id) {
                        if let Some(s) = heap.get_string_utf8(tid) {
                            form_state.title = s;
                        }
                    }
                    context.screen_dirty = true;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/List"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/List")
        {
            let list_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let mut list_state = ListState::default();
                    if let Some(t_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(t_id) {
                            list_state.title = s;
                        }
                    }
                    if let Some(lt) = args.get(1).and_then(|v| v.as_int().ok()) {
                        list_state.list_type = lt;
                    }
                    if args.len() >= 4 {
                        if let Some(str_arr_id) =
                            args.get(2).and_then(|v| v.as_object().ok()).flatten()
                        {
                            if let Some(ObjectData::Array(ArrayData::Reference(ref str_ids))) =
                                heap.get(str_arr_id).cloned()
                            {
                                let img_ids = args
                                    .get(3)
                                    .and_then(|v| v.as_object().ok())
                                    .flatten()
                                    .and_then(|aid| {
                                        if let Some(ObjectData::Array(ArrayData::Reference(
                                            ref imgs,
                                        ))) = heap.get(aid)
                                        {
                                            Some(imgs.clone())
                                        } else {
                                            None
                                        }
                                    });
                                for (i, &s_opt) in str_ids.iter().enumerate() {
                                    let s_text = s_opt
                                        .and_then(|sid| heap.get_string_utf8(sid))
                                        .unwrap_or_default();
                                    let img_opt = img_ids
                                        .as_ref()
                                        .and_then(|imgs| imgs.get(i).copied().flatten());
                                    list_state.elements.push(ListElement {
                                        string_part: s_text,
                                        image_part: img_opt,
                                        selected: i == 0,
                                    });
                                }
                            }
                        }
                    }
                    context.lists.insert(list_id, list_state);
                    return Ok(None);
                }
                "append" => {
                    let s_text = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let img_part = args.get(1).and_then(|v| v.as_object().ok()).flatten();
                    let list = context.lists.entry(list_id).or_default();
                    let idx = list.elements.len() as i32;
                    let is_first = list.elements.is_empty();
                    list.elements.push(ListElement {
                        string_part: s_text,
                        image_part: img_part,
                        selected: is_first,
                    });
                    return Ok(Some(JavaValue::Int(idx)));
                }
                "insert" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let s_text = args
                        .get(1)
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let img_part = args.get(2).and_then(|v| v.as_object().ok()).flatten();
                    let list = context.lists.entry(list_id).or_default();
                    let pos = (idx as usize).min(list.elements.len());
                    list.elements.insert(
                        pos,
                        ListElement {
                            string_part: s_text,
                            image_part: img_part,
                            selected: false,
                        },
                    );
                    return Ok(None);
                }
                "delete" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        list.elements.remove(idx as usize);
                        if list.selected_index >= list.elements.len() as i32 {
                            list.selected_index = (list.elements.len() as i32 - 1).max(0);
                        }
                    }
                    return Ok(None);
                }
                "deleteAll" => {
                    let list = context.lists.entry(list_id).or_default();
                    list.elements.clear();
                    list.selected_index = 0;
                    return Ok(None);
                }
                "set" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let s_text = args
                        .get(1)
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let img_part = args.get(2).and_then(|v| v.as_object().ok()).flatten();
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        list.elements[idx as usize].string_part = s_text;
                        list.elements[idx as usize].image_part = img_part;
                    }
                    return Ok(None);
                }
                "getString" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        let text = list.elements[idx as usize].string_part.clone();
                        let s_id = heap.alloc_string(&text);
                        return Ok(Some(JavaValue::Object(Some(s_id))));
                    }
                    let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                    return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                }
                "getImage" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        return Ok(Some(JavaValue::Object(
                            list.elements[idx as usize].image_part,
                        )));
                    }
                    return Ok(Some(JavaValue::Object(None)));
                }
                "size" => {
                    let list = context.lists.entry(list_id).or_default();
                    return Ok(Some(JavaValue::Int(list.elements.len() as i32)));
                }
                "getSelectedIndex" => {
                    let list = context.lists.entry(list_id).or_default();
                    if list.elements.is_empty() {
                        return Ok(Some(JavaValue::Int(-1)));
                    }
                    return Ok(Some(JavaValue::Int(list.selected_index)));
                }
                "setSelectedIndex" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let sel = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(1) != 0;
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        list.selected_index = idx;
                        if list.list_type == 1 {
                            for (i, elem) in list.elements.iter_mut().enumerate() {
                                elem.selected = i == idx as usize && sel;
                            }
                        } else {
                            list.elements[idx as usize].selected = sel;
                        }
                    }
                    return Ok(None);
                }
                "isSelected" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.lists.entry(list_id).or_default();
                    if idx >= 0 && (idx as usize) < list.elements.len() {
                        let is_sel = if list.list_type == 3 {
                            list.selected_index == idx
                        } else {
                            list.elements[idx as usize].selected
                        };
                        return Ok(Some(JavaValue::Int(if is_sel { 1 } else { 0 })));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "setSelectCommand" => {
                    let cmd_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let list = context.lists.entry(list_id).or_default();
                    list.select_command = cmd_id;
                    return Ok(None);
                }
                "setFitPolicy" => {
                    let fp = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.lists.entry(list_id).or_default();
                    list.fit_policy = fp;
                    return Ok(None);
                }
                "getFitPolicy" => {
                    let list = context.lists.entry(list_id).or_default();
                    return Ok(Some(JavaValue::Int(list.fit_policy)));
                }
                "getTitle" => {
                    let list = context.lists.entry(list_id).or_default();
                    let s_id = heap.alloc_string(&list.title);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "setTitle" => {
                    let s_opt = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid));
                    let list = context.lists.entry(list_id).or_default();
                    list.title = s_opt.unwrap_or_default();
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/TextBox"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/TextBox")
        {
            let tb_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let mut tb_state = TextBoxState::default();
                    if let Some(t_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(t_id) {
                            tb_state.title = s;
                        }
                    }
                    if let Some(txt_id) = args.get(1).and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(txt_id) {
                            tb_state.text = s;
                        }
                    }
                    if let Some(ms) = args.get(2).and_then(|v| v.as_int().ok()) {
                        tb_state.max_size = ms;
                    }
                    if let Some(c) = args.get(3).and_then(|v| v.as_int().ok()) {
                        tb_state.constraints = c;
                    }
                    tb_state.caret_position = tb_state.text.len() as i32;
                    context.text_boxes.insert(tb_id, tb_state);
                    return Ok(None);
                }
                "getString" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    let s_id = heap.alloc_string(&tb.text);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "setString" => {
                    let s = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    tb.text = s;
                    tb.caret_position = tb.text.len() as i32;
                    return Ok(None);
                }
                "size" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.text.len() as i32)));
                }
                "getMaxSize" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.max_size.max(256))));
                }
                "setMaxSize" => {
                    let ms = args.first().and_then(|v| v.as_int().ok()).unwrap_or(256);
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    tb.max_size = ms;
                    return Ok(Some(JavaValue::Int(ms)));
                }
                "getCaretPosition" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.caret_position)));
                }
                "getConstraints" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.constraints)));
                }
                "setConstraints" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    tb.constraints = c;
                    return Ok(None);
                }
                "getTitle" => {
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    let s_id = heap.alloc_string(&tb.title);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "setTitle" => {
                    let s = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let tb = context.text_boxes.entry(tb_id).or_default();
                    tb.title = s;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        if cls == "javax/microedition/lcdui/TextField"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/TextField")
        {
            let tf_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let mut tb_state = TextBoxState::default();
                    if let Some(t_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(t_id) {
                            tb_state.title = s;
                        }
                    }
                    if let Some(txt_id) = args.get(1).and_then(|v| v.as_object().ok()).flatten() {
                        if let Some(s) = heap.get_string_utf8(txt_id) {
                            tb_state.text = s;
                        }
                    }
                    if let Some(ms) = args.get(2).and_then(|v| v.as_int().ok()) {
                        tb_state.max_size = ms;
                    }
                    if let Some(c) = args.get(3).and_then(|v| v.as_int().ok()) {
                        tb_state.constraints = c;
                    }
                    tb_state.caret_position = tb_state.text.len() as i32;
                    context.text_boxes.insert(tf_id, tb_state);
                    return Ok(None);
                }
                "getString" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    let s_id = heap.alloc_string(&tb.text);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "setString" => {
                    let s = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    tb.text = s;
                    tb.caret_position = tb.text.len() as i32;
                    return Ok(None);
                }
                "size" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.text.len() as i32)));
                }
                "getMaxSize" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.max_size.max(256))));
                }
                "setMaxSize" => {
                    let ms = args.first().and_then(|v| v.as_int().ok()).unwrap_or(256);
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    tb.max_size = ms;
                    return Ok(Some(JavaValue::Int(ms)));
                }
                "getCaretPosition" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.caret_position)));
                }
                "getConstraints" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    return Ok(Some(JavaValue::Int(tb.constraints)));
                }
                "setConstraints" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    tb.constraints = c;
                    return Ok(None);
                }
                "getLabel" => {
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    let s_id = heap.alloc_string(&tb.title);
                    return Ok(Some(JavaValue::Object(Some(s_id))));
                }
                "setLabel" => {
                    let s = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .and_then(|sid| heap.get_string_utf8(sid))
                        .unwrap_or_default();
                    let tb = context.text_boxes.entry(tf_id).or_default();
                    tb.title = s;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        if method_name == "addCommand" {
            if let (Some(disp_id), Some(cmd_id)) = (
                receiver,
                args.first().and_then(|v| v.as_object().ok().flatten()),
            ) {
                context.add_command(disp_id, cmd_id, heap);
            }
            return Ok(None);
        }

        if method_name == "removeCommand" {
            if let (Some(disp_id), Some(cmd_id)) = (
                receiver,
                args.first().and_then(|v| v.as_object().ok().flatten()),
            ) {
                context.remove_command(disp_id, cmd_id);
            }
            return Ok(None);
        }

        if method_name == "setCommandListener" {
            if let Some(disp_id) = receiver {
                let listener = args.first().and_then(|v| v.as_object().ok().flatten());
                context.set_command_listener(disp_id, listener);
            }
            return Ok(None);
        }

        // 20. Image
        if cls == "javax/microedition/lcdui/Image" {
            match method_name {
                "createImage" => {
                    if args.len() == 2 {
                        // createImage(width, height) -> mutable off-screen (clamp 0 dimensions to >= 1, reject negative)
                        let raw_w = args[0].as_int().unwrap_or(1);
                        let raw_h = args[1].as_int().unwrap_or(1);
                        if raw_w < 0 || raw_h < 0 {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                        }
                        let w = raw_w.max(1);
                        let h = raw_h.max(1);
                        let img = Image::new_mutable(w, h);
                        let id = heap.alloc_instance("javax/microedition/lcdui/Image");
                        context.images.insert(id, img);
                        return Ok(Some(JavaValue::Object(Some(id))));
                    } else if args.len() == 3 {
                        // createImage(byte[] data, int offset, int len) -> Non-strict PNG
                        let byte_arr_id = args[0].as_object().ok().flatten();
                        let offset_i = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                        let len_i = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);

                        if let Some(id) = byte_arr_id {
                            if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(id) {
                                if offset_i < 0
                                    || len_i < 0
                                    || (offset_i as usize) + (len_i as usize) > b.len()
                                {
                                    return Err(
                                        "EXC:java/lang/ArrayIndexOutOfBoundsException: array index out of bounds"
                                            .to_string(),
                                    );
                                }
                                let offset = offset_i as usize;
                                let len = len_i as usize;
                                let raw_slice: Vec<u8> =
                                    b[offset..offset + len].iter().map(|&x| x as u8).collect();
                                if let Ok(img) = Image::from_bytes(&raw_slice) {
                                    let img_id =
                                        heap.alloc_instance("javax/microedition/lcdui/Image");
                                    context.images.insert(img_id, img);
                                    return Ok(Some(JavaValue::Object(Some(img_id))));
                                }
                            }
                        }
                    } else if args.len() == 6 {
                        // createImage(Image image, int x, int y, int width, int height, int transform)
                        let src_id = args[0].as_object().ok().flatten();
                        let sx = args[1].as_int().unwrap_or(0);
                        let sy = args[2].as_int().unwrap_or(0);
                        let sw = args[3].as_int().unwrap_or(1).max(1);
                        let sh = args[4].as_int().unwrap_or(1).max(1);
                        let _transform = args[5].as_int().unwrap_or(0);
                        if let Some(sid) = src_id {
                            if let Some(src) = context.images.get(&sid) {
                                let mut sub_pixels = vec![0u32; (sw * sh) as usize];
                                for row in 0..sh {
                                    for col in 0..sw {
                                        let px = sx + col;
                                        let py = sy + row;
                                        if px >= 0 && px < src.width && py >= 0 && py < src.height {
                                            sub_pixels[(row * sw + col) as usize] =
                                                src.pixels[(py * src.width + px) as usize];
                                        }
                                    }
                                }
                                let img = Image::new_immutable(sw, sh, sub_pixels);
                                let img_id = heap.alloc_instance("javax/microedition/lcdui/Image");
                                context.images.insert(img_id, img);
                                return Ok(Some(JavaValue::Object(Some(img_id))));
                            }
                        }
                    } else if args.len() == 1 {
                        // createImage(String name) or createImage(InputStream stream)
                        if let Ok(Some(arg_id)) = args[0].as_object() {
                            // Check if String
                            if let Some(path) = heap.get_string_utf8(arg_id) {
                                if let Some(ref jar) = context.jar {
                                    if let Some(bytes) = jar.get_entry(&path) {
                                        if let Ok(img) = Image::from_bytes(bytes) {
                                            let img_id = heap
                                                .alloc_instance("javax/microedition/lcdui/Image");
                                            context.images.insert(img_id, img);
                                            return Ok(Some(JavaValue::Object(Some(img_id))));
                                        }
                                    }
                                }
                                let exc = heap.alloc_instance("java/io/IOException");
                                return Err(format!("EXC:java/io/IOException:{}", exc.0));
                            } else {
                                // InputStream: read all bytes
                                let buf_id =
                                    heap.get_field(arg_id, "buf").as_object().ok().flatten();
                                if let Some(bid) = buf_id {
                                    if let Some(ObjectData::Array(ArrayData::Byte(b))) =
                                        heap.get(bid)
                                    {
                                        let raw: Vec<u8> = b.iter().map(|&x| x as u8).collect();
                                        if let Ok(img) = Image::from_bytes(&raw) {
                                            let img_id = heap
                                                .alloc_instance("javax/microedition/lcdui/Image");
                                            context.images.insert(img_id, img);
                                            return Ok(Some(JavaValue::Object(Some(img_id))));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Fallback empty image
                    let img = Image::new_mutable(1, 1);
                    let id = heap.alloc_instance("javax/microedition/lcdui/Image");
                    context.images.insert(id, img);
                    return Ok(Some(JavaValue::Object(Some(id))));
                }
                "createRGBImage" => {
                    let rgb_arr_id = args[0].as_object().ok().flatten();
                    let w = args[1].as_int().unwrap_or(1);
                    let h = args[2].as_int().unwrap_or(1);
                    let process_alpha = args[3].as_int().unwrap_or(0) != 0;

                    let mut pixels = vec![0u32; (w * h) as usize];
                    if let Some(aid) = rgb_arr_id {
                        if let Some(ObjectData::Array(ArrayData::Int(data))) = heap.get(aid) {
                            for (i, &argb) in data.iter().take(pixels.len()).enumerate() {
                                let a = if process_alpha {
                                    ((argb >> 24) & 0xFF) as u8
                                } else {
                                    255
                                };
                                let r = ((argb >> 16) & 0xFF) as u8;
                                let g = ((argb >> 8) & 0xFF) as u8;
                                let b = (argb & 0xFF) as u8;
                                pixels[i] = make_rgba(r, g, b, a);
                            }
                        }
                    }
                    let img = Image::new_immutable(w, h, pixels);
                    let id = heap.alloc_instance("javax/microedition/lcdui/Image");
                    context.images.insert(id, img);
                    return Ok(Some(JavaValue::Object(Some(id))));
                }
                "getGraphics" => {
                    let g_id = heap.alloc_instance("javax/microedition/lcdui/Graphics");
                    let target = receiver
                        .map(GraphicsTarget::Image)
                        .unwrap_or(GraphicsTarget::Display);
                    context.get_or_create_graphics(g_id, target);
                    return Ok(Some(JavaValue::Object(Some(g_id))));
                }
                "getWidth" => {
                    let w = receiver
                        .and_then(|id| context.images.get(&id))
                        .map(|i| i.width)
                        .unwrap_or(1);
                    return Ok(Some(JavaValue::Int(w)));
                }
                "getHeight" => {
                    let h = receiver
                        .and_then(|id| context.images.get(&id))
                        .map(|i| i.height)
                        .unwrap_or(1);
                    return Ok(Some(JavaValue::Int(h)));
                }
                "isMutable" => {
                    let m = receiver
                        .and_then(|id| context.images.get(&id))
                        .map(|i| i.is_mutable)
                        .unwrap_or(false);
                    return Ok(Some(JavaValue::Int(if m { 1 } else { 0 })));
                }
                "getRGB" => {
                    let rgb_arr_id = args[0].as_object().ok().flatten();
                    let offset = args[1].as_int().unwrap_or(0) as usize;
                    let scan = args[2].as_int().unwrap_or(0) as usize;
                    let x = args[3].as_int().unwrap_or(0);
                    let y = args[4].as_int().unwrap_or(0);
                    let w = args[5].as_int().unwrap_or(0);
                    let h = args[6].as_int().unwrap_or(0);

                    if let (Some(r_id), Some(aid)) = (receiver, rgb_arr_id) {
                        if let Some(img) = context.images.get(&r_id) {
                            if let Some(ObjectData::Array(ArrayData::Int(dst))) = heap.get_mut(aid)
                            {
                                img.get_rgb(dst, offset, scan, x, y, w, h);
                            }
                        }
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 21. Font
        if cls == "javax/microedition/lcdui/Font" {
            let get_font_spec = |r: Option<ObjectId>, h: &Heap| -> FontSpec {
                if let Some(fid) = r {
                    let face = h.get_field(fid, "face").as_int().unwrap_or(0);
                    let style = h.get_field(fid, "style").as_int().unwrap_or(0);
                    let size = h.get_field(fid, "size").as_int().unwrap_or(0);
                    FontSpec::new(face, style, size)
                } else {
                    FontSpec::default()
                }
            };

            match method_name {
                "getFont" => {
                    let face = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let style = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let size = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let font_id = heap.alloc_instance("javax/microedition/lcdui/Font");
                    heap.set_field(font_id, "face", JavaValue::Int(face));
                    heap.set_field(font_id, "style", JavaValue::Int(style));
                    heap.set_field(font_id, "size", JavaValue::Int(size));
                    return Ok(Some(JavaValue::Object(Some(font_id))));
                }
                "getDefaultFont" => {
                    let font_id = heap.alloc_instance("javax/microedition/lcdui/Font");
                    heap.set_field(font_id, "face", JavaValue::Int(0));
                    heap.set_field(font_id, "style", JavaValue::Int(0));
                    heap.set_field(font_id, "size", JavaValue::Int(0));
                    return Ok(Some(JavaValue::Object(Some(font_id))));
                }
                "stringWidth" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let font = get_font_spec(receiver, heap);
                    return Ok(Some(JavaValue::Int(font.string_width(&s))));
                }
                "substringWidth" => {
                    let str_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let s = str_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let offset = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let len = args
                        .get(2)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let font = get_font_spec(receiver, heap);
                    let sub: String = s.chars().skip(offset).take(len).collect();
                    return Ok(Some(JavaValue::Int(font.string_width(&sub))));
                }
                "charsWidth" => {
                    let arr_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let offset = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let len = args
                        .get(2)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(0)
                        .max(0) as usize;
                    let font = get_font_spec(receiver, heap);
                    if let Some(aid) = arr_id {
                        if let Some(ObjectData::Array(ArrayData::Char(c))) = heap.get(aid) {
                            let s: String = c
                                .iter()
                                .skip(offset)
                                .take(len)
                                .filter_map(|&ch| char::from_u32(ch as u32))
                                .collect();
                            return Ok(Some(JavaValue::Int(font.string_width(&s))));
                        }
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "charWidth" => {
                    let c = args.first().and_then(|v| v.as_int().ok()).unwrap_or(32) as u8 as char;
                    let font = get_font_spec(receiver, heap);
                    return Ok(Some(JavaValue::Int(font.char_width(c))));
                }
                "getHeight" => {
                    let font = get_font_spec(receiver, heap);
                    return Ok(Some(JavaValue::Int(font.get_height())));
                }
                "getBaselinePosition" => {
                    let font = get_font_spec(receiver, heap);
                    return Ok(Some(JavaValue::Int(font.get_baseline_position())));
                }
                "getStyle" => {
                    let style = receiver.map(|fid| heap.get_field(fid, "style").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(style)));
                }
                "getSize" => {
                    let size = receiver.map(|fid| heap.get_field(fid, "size").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(size)));
                }
                "getFace" => {
                    let face = receiver.map(|fid| heap.get_field(fid, "face").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(face)));
                }
                "isPlain" => {
                    let style = receiver.map(|fid| heap.get_field(fid, "style").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if style == 0 { 1 } else { 0 })));
                }
                "isBold" => {
                    let style = receiver.map(|fid| heap.get_field(fid, "style").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if style & 1 != 0 { 1 } else { 0 })));
                }
                "isItalic" => {
                    let style = receiver.map(|fid| heap.get_field(fid, "style").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if style & 2 != 0 { 1 } else { 0 })));
                }
                "isUnderlined" => {
                    let style = receiver.map(|fid| heap.get_field(fid, "style").as_int().unwrap_or(0)).unwrap_or(0);
                    return Ok(Some(JavaValue::Int(if style & 4 != 0 { 1 } else { 0 })));
                }
                _ => {}
            }
        }

        // 21b. javax/microedition/lcdui/game/Sprite & Layer
        let is_sprite_or_layer = cls == "javax/microedition/lcdui/game/Sprite"
            || cls == "javax/microedition/lcdui/game/Layer"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/game/Sprite")
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/game/Layer");
        if is_sprite_or_layer {
            match method_name {
                "<init>" => {
                    if let Some(r) = receiver {
                        let img_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                        let (w, h) = if let Some(id) = img_id {
                            context
                                .images
                                .get(&id)
                                .map(|img| (img.width, img.height))
                                .unwrap_or((16, 16))
                        } else {
                            (16, 16)
                        };
                        let fw = args
                            .get(1)
                            .and_then(|v| v.as_int().ok())
                            .unwrap_or(w)
                            .max(1);
                        let fh = args
                            .get(2)
                            .and_then(|v| v.as_int().ok())
                            .unwrap_or(h)
                            .max(1);
                        heap.set_field(r, "image", JavaValue::Object(img_id));
                        heap.set_field(r, "frameWidth", JavaValue::Int(fw));
                        heap.set_field(r, "frameHeight", JavaValue::Int(fh));
                        heap.set_field(r, "x", JavaValue::Int(0));
                        heap.set_field(r, "y", JavaValue::Int(0));
                        heap.set_field(r, "visible", JavaValue::Int(1));
                        heap.set_field(r, "frame", JavaValue::Int(0));
                        heap.set_field(r, "transform", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "getWidth" => {
                    let w = receiver
                        .and_then(|r| heap.get_field(r, "frameWidth").as_int().ok())
                        .unwrap_or(16)
                        .max(1);
                    return Ok(Some(JavaValue::Int(w)));
                }
                "getHeight" => {
                    let h = receiver
                        .and_then(|r| heap.get_field(r, "frameHeight").as_int().ok())
                        .unwrap_or(16)
                        .max(1);
                    return Ok(Some(JavaValue::Int(h)));
                }
                "getX" => {
                    let x = receiver
                        .and_then(|r| heap.get_field(r, "x").as_int().ok())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(x)));
                }
                "getY" => {
                    let y = receiver
                        .and_then(|r| heap.get_field(r, "y").as_int().ok())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(y)));
                }
                "setPosition" => {
                    let x = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        heap.set_field(r, "x", JavaValue::Int(x));
                        heap.set_field(r, "y", JavaValue::Int(y));
                    }
                    return Ok(None);
                }
                "move" => {
                    let dx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let dy = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        let x = heap.get_field(r, "x").as_int().unwrap_or(0);
                        let y = heap.get_field(r, "y").as_int().unwrap_or(0);
                        heap.set_field(r, "x", JavaValue::Int(x + dx));
                        heap.set_field(r, "y", JavaValue::Int(y + dy));
                    }
                    return Ok(None);
                }
                "setVisible" => {
                    let v = args.first().and_then(|v| v.as_int().ok()).unwrap_or(1);
                    if let Some(r) = receiver {
                        heap.set_field(r, "visible", JavaValue::Int(v));
                    }
                    return Ok(None);
                }
                "isVisible" => {
                    let v = receiver
                        .and_then(|r| heap.get_field(r, "visible").as_int().ok())
                        .unwrap_or(1);
                    return Ok(Some(JavaValue::Int(v)));
                }
                "defineReferencePixel" => {
                    let rx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let ry = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        heap.set_field(r, "refX", JavaValue::Int(rx));
                        heap.set_field(r, "refY", JavaValue::Int(ry));
                        let spr = context.sprites.entry(r).or_default();
                        spr.ref_x = rx;
                        spr.ref_y = ry;
                    }
                    return Ok(None);
                }
                "setRefPixelPosition" => {
                    let target_x = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let target_y = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        let spr = context.sprites.entry(r).or_default();
                        let ref_x = spr.ref_x;
                        let ref_y = spr.ref_y;
                        let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                        let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                        let transform = heap.get_field(r, "transform").as_int().unwrap_or(0);

                        let (dx, dy) = match transform {
                            0 /* TRANS_NONE */ => (ref_x, ref_y),
                            1 /* TRANS_MIRROR_ROT180 */ => (ref_x, fh - 1 - ref_y),
                            2 /* TRANS_MIRROR */ => (fw - 1 - ref_x, ref_y),
                            3 /* TRANS_ROT180 */ => (fw - 1 - ref_x, fh - 1 - ref_y),
                            4 /* TRANS_MIRROR_ROT270 */ => (ref_y, ref_x),
                            5 /* TRANS_ROT90 */ => (fh - 1 - ref_y, ref_x),
                            6 /* TRANS_ROT270 */ => (ref_y, fw - 1 - ref_x),
                            7 /* TRANS_MIRROR_ROT90 */ => (fh - 1 - ref_y, fw - 1 - ref_x),
                            _ => (ref_x, ref_y),
                        };

                        let new_x = target_x - dx;
                        let new_y = target_y - dy;
                        heap.set_field(r, "x", JavaValue::Int(new_x));
                        heap.set_field(r, "y", JavaValue::Int(new_y));
                    }
                    return Ok(None);
                }
                "getRefPixelX" => {
                    if let Some(r) = receiver {
                        let sx = heap.get_field(r, "x").as_int().unwrap_or(0);
                        let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                        let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                        let transform = heap.get_field(r, "transform").as_int().unwrap_or(0);
                        let spr = context.sprites.entry(r).or_default();
                        let ref_x = spr.ref_x;
                        let ref_y = spr.ref_y;

                        let dx = match transform {
                            0 => ref_x,
                            1 => ref_x,
                            2 => fw - 1 - ref_x,
                            3 => fw - 1 - ref_x,
                            4 => ref_y,
                            5 => fh - 1 - ref_y,
                            6 => ref_y,
                            7 => fh - 1 - ref_y,
                            _ => ref_x,
                        };
                        return Ok(Some(JavaValue::Int(sx + dx)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "getRefPixelY" => {
                    if let Some(r) = receiver {
                        let sy = heap.get_field(r, "y").as_int().unwrap_or(0);
                        let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                        let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                        let transform = heap.get_field(r, "transform").as_int().unwrap_or(0);
                        let spr = context.sprites.entry(r).or_default();
                        let ref_x = spr.ref_x;
                        let ref_y = spr.ref_y;

                        let dy = match transform {
                            0 => ref_y,
                            1 => fh - 1 - ref_y,
                            2 => ref_y,
                            3 => fh - 1 - ref_y,
                            4 => ref_x,
                            5 => ref_x,
                            6 => fw - 1 - ref_x,
                            7 => fw - 1 - ref_x,
                            _ => ref_y,
                        };
                        return Ok(Some(JavaValue::Int(sy + dy)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "setFrameSequence" => {
                    let r = match receiver {
                        Some(r) => r,
                        None => return Ok(None),
                    };
                    let raw_frames = {
                        let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                        let img = img_id.and_then(|id| context.images.get(&id));
                        let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                        let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                        img.map(|im| ((im.width / fw) * (im.height / fh)) as i32)
                            .unwrap_or(1)
                            .max(1)
                    };

                    let seq_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let spr = context.sprites.entry(r).or_default();
                    if let Some(arr_id) = seq_opt {
                        let ints: Vec<i32> = match heap.get(arr_id) {
                            Some(ObjectData::Array(ArrayData::Int(arr))) => arr.clone(),
                            _ => Vec::new(),
                        };
                        if ints.is_empty() {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                        }
                        for &frame_idx in &ints {
                            if frame_idx < 0 || frame_idx >= raw_frames {
                                let exc =
                                    heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!(
                                    "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                    exc.0
                                ));
                            }
                        }
                        spr.frame_sequence = Some(ints.clone());
                        spr.sequence_index = 0;
                        heap.set_field(r, "frame", JavaValue::Int(ints[0]));
                    } else {
                        spr.frame_sequence = None;
                        spr.sequence_index = 0;
                        heap.set_field(r, "frame", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "getFrameSequenceLength" => {
                    let r = receiver.unwrap_or(ObjectId(0));
                    let spr = context.sprites.entry(r).or_default();
                    if let Some(ref seq) = spr.frame_sequence {
                        return Ok(Some(JavaValue::Int(seq.len() as i32)));
                    } else {
                        let raw_frames = {
                            let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                            let img = img_id.and_then(|id| context.images.get(&id));
                            let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                            let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                            img.map(|im| ((im.width / fw) * (im.height / fh)) as i32)
                                .unwrap_or(1)
                                .max(1)
                        };
                        return Ok(Some(JavaValue::Int(raw_frames)));
                    }
                }
                "setFrame" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        let spr = context.sprites.entry(r).or_default();
                        let seq_len = if let Some(ref seq) = spr.frame_sequence {
                            seq.len() as i32
                        } else {
                            let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                            let img = img_id.and_then(|id| context.images.get(&id));
                            let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                            let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                            img.map(|im| ((im.width / fw) * (im.height / fh)) as i32)
                                .unwrap_or(1)
                                .max(1)
                        };
                        if idx < 0 || idx >= seq_len {
                            let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                        }
                        spr.sequence_index = idx as usize;
                        let real_frame = if let Some(ref seq) = spr.frame_sequence {
                            seq[spr.sequence_index]
                        } else {
                            idx
                        };
                        heap.set_field(r, "frame", JavaValue::Int(real_frame));
                    }
                    return Ok(None);
                }
                "getFrame" => {
                    let r = receiver.unwrap_or(ObjectId(0));
                    let spr = context.sprites.entry(r).or_default();
                    return Ok(Some(JavaValue::Int(spr.sequence_index as i32)));
                }
                "nextFrame" => {
                    if let Some(r) = receiver {
                        let spr = context.sprites.entry(r).or_default();
                        let seq_len = if let Some(ref seq) = spr.frame_sequence {
                            seq.len()
                        } else {
                            let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                            let img = img_id.and_then(|id| context.images.get(&id));
                            let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                            let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                            img.map(|im| ((im.width / fw) * (im.height / fh)) as usize)
                                .unwrap_or(1)
                                .max(1)
                        };
                        spr.sequence_index = (spr.sequence_index + 1) % seq_len;
                        let real_frame = if let Some(ref seq) = spr.frame_sequence {
                            seq[spr.sequence_index]
                        } else {
                            spr.sequence_index as i32
                        };
                        heap.set_field(r, "frame", JavaValue::Int(real_frame));
                    }
                    return Ok(None);
                }
                "prevFrame" => {
                    if let Some(r) = receiver {
                        let spr = context.sprites.entry(r).or_default();
                        let seq_len = if let Some(ref seq) = spr.frame_sequence {
                            seq.len()
                        } else {
                            let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                            let img = img_id.and_then(|id| context.images.get(&id));
                            let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                            let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                            img.map(|im| ((im.width / fw) * (im.height / fh)) as usize)
                                .unwrap_or(1)
                                .max(1)
                        };
                        spr.sequence_index = if spr.sequence_index == 0 {
                            seq_len - 1
                        } else {
                            spr.sequence_index - 1
                        };
                        let real_frame = if let Some(ref seq) = spr.frame_sequence {
                            seq[spr.sequence_index]
                        } else {
                            spr.sequence_index as i32
                        };
                        heap.set_field(r, "frame", JavaValue::Int(real_frame));
                    }
                    return Ok(None);
                }
                "getRawFrameCount" => {
                    let total = receiver
                        .and_then(|r| {
                            let img_id = heap.get_field(r, "image").as_object().ok().flatten()?;
                            let img = context.images.get(&img_id)?;
                            let fw = heap
                                .get_field(r, "frameWidth")
                                .as_int()
                                .ok()
                                .unwrap_or(16)
                                .max(1);
                            let fh = heap
                                .get_field(r, "frameHeight")
                                .as_int()
                                .ok()
                                .unwrap_or(16)
                                .max(1);
                            Some(((img.width / fw) * (img.height / fh)) as i32)
                        })
                        .unwrap_or(1);
                    return Ok(Some(JavaValue::Int(total.max(1))));
                }
                "setImage" => {
                    let img_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let fw = args
                        .get(1)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(16)
                        .max(1);
                    let fh = args
                        .get(2)
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(16)
                        .max(1);
                    if let Some(r) = receiver {
                        heap.set_field(r, "image", JavaValue::Object(img_id));
                        heap.set_field(r, "frameWidth", JavaValue::Int(fw));
                        heap.set_field(r, "frameHeight", JavaValue::Int(fh));
                    }
                    return Ok(None);
                }
                "setTransform" => {
                    let new_t = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        let old_t = heap.get_field(r, "transform").as_int().unwrap_or(0);
                        if old_t != new_t {
                            let fw = heap.get_field(r, "frameWidth").as_int().unwrap_or(16).max(1);
                            let fh = heap.get_field(r, "frameHeight").as_int().unwrap_or(16).max(1);
                            let spr = context.sprites.entry(r).or_default();
                            let ref_x = spr.ref_x;
                            let ref_y = spr.ref_y;

                            let calc_d = |t: i32| -> (i32, i32) {
                                match t {
                                    0 /* TRANS_NONE */ => (ref_x, ref_y),
                                    1 /* TRANS_MIRROR_ROT180 */ => (ref_x, fh - 1 - ref_y),
                                    2 /* TRANS_MIRROR */ => (fw - 1 - ref_x, ref_y),
                                    3 /* TRANS_ROT180 */ => (fw - 1 - ref_x, fh - 1 - ref_y),
                                    4 /* TRANS_MIRROR_ROT270 */ => (ref_y, ref_x),
                                    5 /* TRANS_ROT90 */ => (fh - 1 - ref_y, ref_x),
                                    6 /* TRANS_ROT270 */ => (ref_y, fw - 1 - ref_x),
                                    7 /* TRANS_MIRROR_ROT90 */ => (fh - 1 - ref_y, fw - 1 - ref_x),
                                    _ => (ref_x, ref_y),
                                }
                            };

                            let (dx_old, dy_old) = calc_d(old_t);
                            let (dx_new, dy_new) = calc_d(new_t);

                            let cur_x = heap.get_field(r, "x").as_int().unwrap_or(0);
                            let cur_y = heap.get_field(r, "y").as_int().unwrap_or(0);

                            let new_x = cur_x + dx_old - dx_new;
                            let new_y = cur_y + dy_old - dy_new;

                            heap.set_field(r, "x", JavaValue::Int(new_x));
                            heap.set_field(r, "y", JavaValue::Int(new_y));
                            heap.set_field(r, "transform", JavaValue::Int(new_t));
                        }
                    }
                    return Ok(None);
                }
                "paint" => {
                    let g_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    if let (Some(r), Some(gid)) = (receiver, g_id) {
                        let visible = heap.get_field(r, "visible").as_int().unwrap_or(1) != 0;
                        let img_id = heap.get_field(r, "image").as_object().ok().flatten();
                        if visible && img_id.is_some() {
                            let sx = heap.get_field(r, "x").as_int().unwrap_or(0);
                            let sy = heap.get_field(r, "y").as_int().unwrap_or(0);
                            let fw = heap
                                .get_field(r, "frameWidth")
                                .as_int()
                                .ok()
                                .unwrap_or(16)
                                .max(1);
                            let fh = heap
                                .get_field(r, "frameHeight")
                                .as_int()
                                .ok()
                                .unwrap_or(16)
                                .max(1);
                            let frame = heap.get_field(r, "frame").as_int().unwrap_or(0);
                            let transform = heap.get_field(r, "transform").as_int().unwrap_or(0);
                            if let Some(img) = context.images.get(&img_id.unwrap()).cloned() {
                                let cols = (img.width / fw).max(1);
                                let col = frame % cols;
                                let row = frame / cols;
                                let src_x = col * fw;
                                let src_y = row * fh;
                                if let Some(gc) = context.graphics_contexts.get(&gid) {
                                    let target = gc.target;
                                    let g_clone = gc.graphics.clone();
                                    with_graphics_target_mut(context, target, |target_pixels| {
                                        g_clone.draw_region(
                                            target_pixels,
                                            &img,
                                            src_x,
                                            src_y,
                                            fw,
                                            fh,
                                            transform,
                                            sx,
                                            sy,
                                            0,
                                        );
                                    });
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                "collidesWith" => {
                    return Ok(Some(JavaValue::Int(0)));
                }
                _ => {}
            }
        }

        // 21c. javax/microedition/lcdui/game/LayerManager
        if cls == "javax/microedition/lcdui/game/LayerManager"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/game/LayerManager")
        {
            let lm_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    context.layer_managers.entry(lm_id).or_default();
                    return Ok(None);
                }
                "append" => {
                    if let Some(layer_id) =
                        args.first().and_then(|v| v.as_object().ok()).flatten()
                    {
                        let lm = context.layer_managers.entry(lm_id).or_default();
                        lm.layers.retain(|&id| id != layer_id);
                        lm.layers.push(layer_id);
                    }
                    return Ok(None);
                }
                "insert" => {
                    let layer_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let idx = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(lid) = layer_id {
                        let lm = context.layer_managers.entry(lm_id).or_default();
                        lm.layers.retain(|&id| id != lid);
                        let pos = (idx as usize).min(lm.layers.len());
                        lm.layers.insert(pos, lid);
                    }
                    return Ok(None);
                }
                "getLayerAt" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let lm = context.layer_managers.entry(lm_id).or_default();
                    if idx >= 0 && (idx as usize) < lm.layers.len() {
                        return Ok(Some(JavaValue::Object(Some(lm.layers[idx as usize]))));
                    }
                    let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                    return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                }
                "getSize" => {
                    let lm = context.layer_managers.entry(lm_id).or_default();
                    return Ok(Some(JavaValue::Int(lm.layers.len() as i32)));
                }
                "remove" => {
                    if let Some(layer_id) =
                        args.first().and_then(|v| v.as_object().ok()).flatten()
                    {
                        let lm = context.layer_managers.entry(lm_id).or_default();
                        lm.layers.retain(|&id| id != layer_id);
                    }
                    return Ok(None);
                }
                "setViewWindow" => {
                    let vx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let vy = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let vw = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let vh = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let lm = context.layer_managers.entry(lm_id).or_default();
                    lm.view_x = vx;
                    lm.view_y = vy;
                    lm.view_w = vw;
                    lm.view_h = vh;
                    return Ok(None);
                }
                "paint" => {
                    let g_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let px = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let py = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(gid) = g_id {
                        let lm_state = context.layer_managers.entry(lm_id).or_default().clone();
                        let layers_to_paint: Vec<ObjectId> =
                            lm_state.layers.into_iter().rev().collect();

                        for layer_id in layers_to_paint {
                            let visible =
                                heap.get_field(layer_id, "visible").as_int().unwrap_or(1) != 0;
                            if !visible {
                                continue;
                            }
                            let lx = heap.get_field(layer_id, "x").as_int().unwrap_or(0);
                            let ly = heap.get_field(layer_id, "y").as_int().unwrap_or(0);
                            let draw_x = px + (lx - lm_state.view_x);
                            let draw_y = py + (ly - lm_state.view_y);

                            // Check if this layer is a TiledLayer
                            if let Some(tl) = context.tiled_layers.get(&layer_id).cloned() {
                                if let Some(iid) = tl.image_id {
                                    if let Some(img) = context.images.get(&iid).cloned() {
                                        let tiles_per_row = (img.width / tl.tile_width).max(1);
                                        if let Some(gc) = context.graphics_contexts.get_mut(&gid) {
                                            let target = gc.target;
                                            let orig_clip_x = gc.graphics.clip_x;
                                            let orig_clip_y = gc.graphics.clip_y;
                                            let orig_clip_w = gc.graphics.clip_w;
                                            let orig_clip_h = gc.graphics.clip_h;

                                            if lm_state.view_w > 0 && lm_state.view_h > 0 {
                                                gc.graphics.clip_rect(
                                                    px,
                                                    py,
                                                    lm_state.view_w,
                                                    lm_state.view_h,
                                                );
                                            }

                                            let g_clone = gc.graphics.clone();
                                            with_graphics_target_mut(context, target, |buf| {
                                                for r in 0..tl.rows {
                                                    for c in 0..tl.columns {
                                                        let cell_val =
                                                            tl.cells[(r * tl.columns + c) as usize];
                                                        let static_idx = if cell_val < 0 {
                                                            let anim_slot =
                                                                (-cell_val - 1) as usize;
                                                            tl.animated_tiles
                                                                .get(anim_slot)
                                                                .copied()
                                                                .unwrap_or(0)
                                                        } else {
                                                            cell_val
                                                        };
                                                        if static_idx <= 0 {
                                                            continue;
                                                        }
                                                        let src_col =
                                                            (static_idx - 1) % tiles_per_row;
                                                        let src_row =
                                                            (static_idx - 1) / tiles_per_row;
                                                        let src_x = src_col * tl.tile_width;
                                                        let src_y = src_row * tl.tile_height;
                                                        let dest_x = draw_x + c * tl.tile_width;
                                                        let dest_y = draw_y + r * tl.tile_height;

                                                        g_clone.draw_region(
                                                            buf,
                                                            &img,
                                                            src_x,
                                                            src_y,
                                                            tl.tile_width,
                                                            tl.tile_height,
                                                            0,
                                                            dest_x,
                                                            dest_y,
                                                            0,
                                                        );
                                                    }
                                                }
                                            });

                                            if let Some(gc_rest) =
                                                context.graphics_contexts.get_mut(&gid)
                                            {
                                                gc_rest.graphics.clip_x = orig_clip_x;
                                                gc_rest.graphics.clip_y = orig_clip_y;
                                                gc_rest.graphics.clip_w = orig_clip_w;
                                                gc_rest.graphics.clip_h = orig_clip_h;
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            let img_id =
                                heap.get_field(layer_id, "image").as_object().ok().flatten();
                            if let Some(iid) = img_id {
                                let fw = heap
                                    .get_field(layer_id, "frameWidth")
                                    .as_int()
                                    .ok()
                                    .unwrap_or(16)
                                    .max(1);
                                let fh = heap
                                    .get_field(layer_id, "frameHeight")
                                    .as_int()
                                    .ok()
                                    .unwrap_or(16)
                                    .max(1);
                                let frame =
                                    heap.get_field(layer_id, "frame").as_int().unwrap_or(0);
                                let transform = heap
                                    .get_field(layer_id, "transform")
                                    .as_int()
                                    .unwrap_or(0);

                                if let Some(img) = context.images.get(&iid).cloned() {
                                    let cols = (img.width / fw).max(1);
                                    let col = frame % cols;
                                    let row = frame / cols;
                                    let src_x = col * fw;
                                    let src_y = row * fh;

                                    if let Some(gc) = context.graphics_contexts.get_mut(&gid) {
                                        let target = gc.target;
                                        let orig_clip_x = gc.graphics.clip_x;
                                        let orig_clip_y = gc.graphics.clip_y;
                                        let orig_clip_w = gc.graphics.clip_w;
                                        let orig_clip_h = gc.graphics.clip_h;

                                        if lm_state.view_w > 0 && lm_state.view_h > 0 {
                                            gc.graphics.clip_rect(
                                                px,
                                                py,
                                                lm_state.view_w,
                                                lm_state.view_h,
                                            );
                                        }

                                        let g_clone = gc.graphics.clone();
                                        with_graphics_target_mut(context, target, |buf| {
                                            g_clone.draw_region(
                                                buf, &img, src_x, src_y, fw, fh, transform,
                                                draw_x, draw_y, 0,
                                            );
                                        });

                                        if let Some(gc_rest) =
                                            context.graphics_contexts.get_mut(&gid)
                                        {
                                            gc_rest.graphics.clip_x = orig_clip_x;
                                            gc_rest.graphics.clip_y = orig_clip_y;
                                            gc_rest.graphics.clip_w = orig_clip_w;
                                            gc_rest.graphics.clip_h = orig_clip_h;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 21d. javax/microedition/lcdui/game/TiledLayer
        if cls == "javax/microedition/lcdui/game/TiledLayer"
            || class_loader.is_instance_of(&cls, "javax/microedition/lcdui/game/TiledLayer")
        {
            let tl_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => {
                    let cols = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let rows = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let img_id = args.get(2).and_then(|v| v.as_object().ok()).flatten();
                    let tw = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let th = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if img_id.is_none() {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                    if cols <= 0 || rows <= 0 || tw <= 0 || th <= 0 {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                    }

                    if let Some(img) = context.images.get(&img_id.unwrap()) {
                        if img.width % tw != 0 || img.height % th != 0 {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                        }
                    }

                    heap.set_field(tl_id, "columns", JavaValue::Int(cols));
                    heap.set_field(tl_id, "rows", JavaValue::Int(rows));
                    heap.set_field(tl_id, "tileWidth", JavaValue::Int(tw));
                    heap.set_field(tl_id, "tileHeight", JavaValue::Int(th));
                    heap.set_field(tl_id, "width", JavaValue::Int(cols * tw));
                    heap.set_field(tl_id, "height", JavaValue::Int(rows * th));
                    heap.set_field(tl_id, "x", JavaValue::Int(0));
                    heap.set_field(tl_id, "y", JavaValue::Int(0));
                    heap.set_field(tl_id, "visible", JavaValue::Int(1));
                    heap.set_field(tl_id, "image", JavaValue::Object(img_id));

                    context.tiled_layers.insert(
                        tl_id,
                        TiledLayerState {
                            columns: cols,
                            rows,
                            tile_width: tw,
                            tile_height: th,
                            image_id: img_id,
                            cells: vec![0; (cols * rows) as usize],
                            animated_tiles: Vec::new(),
                        },
                    );
                    return Ok(None);
                }
                "setCell" => {
                    let col = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let row = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let tile_idx = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    let (cols, rows, num_static, anim_count) = {
                        let tl = match context.tiled_layers.get(&tl_id) {
                            Some(t) => t,
                            None => return Ok(None),
                        };
                        let img_w = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.width)
                            .unwrap_or(16);
                        let img_h = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.height)
                            .unwrap_or(16);
                        let n_stat = (img_w / tl.tile_width) * (img_h / tl.tile_height);
                        (tl.columns, tl.rows, n_stat, tl.animated_tiles.len())
                    };

                    if col < 0 || col >= cols || row < 0 || row >= rows {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if tile_idx < 0 {
                        let anim_idx = (-tile_idx - 1) as usize;
                        if anim_idx >= anim_count {
                            let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                        }
                    } else if tile_idx > num_static {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if let Some(tl) = context.tiled_layers.get_mut(&tl_id) {
                        tl.cells[(row * cols + col) as usize] = tile_idx;
                    }
                    return Ok(None);
                }
                "getCell" => {
                    let col = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let row = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(tl) = context.tiled_layers.get(&tl_id) {
                        if col < 0 || col >= tl.columns || row < 0 || row >= tl.rows {
                            let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                        }
                        let val = tl.cells[(row * tl.columns + col) as usize];
                        return Ok(Some(JavaValue::Int(val)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "fillCells" => {
                    let col = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let row = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let num_cols = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let num_rows = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let tile_idx = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    let (cols, rows, num_static, anim_count) = {
                        let tl = match context.tiled_layers.get(&tl_id) {
                            Some(t) => t,
                            None => return Ok(None),
                        };
                        let img_w = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.width)
                            .unwrap_or(16);
                        let img_h = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.height)
                            .unwrap_or(16);
                        let n_stat = (img_w / tl.tile_width) * (img_h / tl.tile_height);
                        (tl.columns, tl.rows, n_stat, tl.animated_tiles.len())
                    };

                    if num_cols < 0
                        || num_rows < 0
                        || col < 0
                        || col + num_cols > cols
                        || row < 0
                        || row + num_rows > rows
                    {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if tile_idx < 0 {
                        let anim_idx = (-tile_idx - 1) as usize;
                        if anim_idx >= anim_count {
                            let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                        }
                    } else if tile_idx > num_static {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if let Some(tl) = context.tiled_layers.get_mut(&tl_id) {
                        for r in row..row + num_rows {
                            for c in col..col + num_cols {
                                tl.cells[(r * cols + c) as usize] = tile_idx;
                            }
                        }
                    }
                    return Ok(None);
                }
                "createAnimatedTile" => {
                    let static_idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let num_static = {
                        let tl = match context.tiled_layers.get(&tl_id) {
                            Some(t) => t,
                            None => return Ok(Some(JavaValue::Int(-1))),
                        };
                        let img_w = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.width)
                            .unwrap_or(16);
                        let img_h = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.height)
                            .unwrap_or(16);
                        (img_w / tl.tile_width) * (img_h / tl.tile_height)
                    };

                    if static_idx < 0 || static_idx > num_static {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if let Some(tl) = context.tiled_layers.get_mut(&tl_id) {
                        tl.animated_tiles.push(static_idx);
                        let anim_id = -(tl.animated_tiles.len() as i32);
                        return Ok(Some(JavaValue::Int(anim_id)));
                    }
                    return Ok(Some(JavaValue::Int(-1)));
                }
                "setAnimatedTile" => {
                    let anim_idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let static_idx = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    let (num_static, anim_count) = {
                        let tl = match context.tiled_layers.get(&tl_id) {
                            Some(t) => t,
                            None => return Ok(None),
                        };
                        let img_w = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.width)
                            .unwrap_or(16);
                        let img_h = tl
                            .image_id
                            .and_then(|id| context.images.get(&id))
                            .map(|i| i.height)
                            .unwrap_or(16);
                        (
                            (img_w / tl.tile_width) * (img_h / tl.tile_height),
                            tl.animated_tiles.len(),
                        )
                    };

                    if anim_idx >= 0 {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }
                    let slot = (-anim_idx - 1) as usize;
                    if slot >= anim_count || static_idx < 0 || static_idx > num_static {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }

                    if let Some(tl) = context.tiled_layers.get_mut(&tl_id) {
                        tl.animated_tiles[slot] = static_idx;
                    }
                    return Ok(None);
                }
                "getAnimatedTile" => {
                    let anim_idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if anim_idx >= 0 {
                        let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                        return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                    }
                    let slot = (-anim_idx - 1) as usize;
                    if let Some(tl) = context.tiled_layers.get(&tl_id) {
                        if slot >= tl.animated_tiles.len() {
                            let exc = heap.alloc_instance("java/lang/IndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/IndexOutOfBoundsException:{}", exc.0));
                        }
                        return Ok(Some(JavaValue::Int(tl.animated_tiles[slot])));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "setStaticTileSet" => {
                    let img_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let tw = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let th = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if img_id.is_none() {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                    }
                    if tw <= 0 || th <= 0 {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                    }
                    if let Some(img) = context.images.get(&img_id.unwrap()) {
                        if img.width % tw != 0 || img.height % th != 0 {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                        }
                    }

                    if let Some(tl) = context.tiled_layers.get_mut(&tl_id) {
                        tl.image_id = img_id;
                        tl.tile_width = tw;
                        tl.tile_height = th;
                        heap.set_field(tl_id, "tileWidth", JavaValue::Int(tw));
                        heap.set_field(tl_id, "tileHeight", JavaValue::Int(th));
                        heap.set_field(tl_id, "width", JavaValue::Int(tl.columns * tw));
                        heap.set_field(tl_id, "height", JavaValue::Int(tl.rows * th));
                        heap.set_field(tl_id, "image", JavaValue::Object(img_id));
                    }
                    return Ok(None);
                }
                "paint" => {
                    let g_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    if let (Some(gid), Some(tl)) =
                        (g_id, context.tiled_layers.get(&tl_id).cloned())
                    {
                        let visible = heap.get_field(tl_id, "visible").as_int().unwrap_or(1) != 0;
                        if !visible {
                            return Ok(None);
                        }
                        let lx = heap.get_field(tl_id, "x").as_int().unwrap_or(0);
                        let ly = heap.get_field(tl_id, "y").as_int().unwrap_or(0);

                        if let Some(iid) = tl.image_id {
                            if let Some(img) = context.images.get(&iid).cloned() {
                                let tiles_per_row = (img.width / tl.tile_width).max(1);
                                if let Some(gc) = context.graphics_contexts.get(&gid) {
                                    let target = gc.target;
                                    let g_clone = gc.graphics.clone();

                                    with_graphics_target_mut(context, target, |target_pixels| {
                                        for r in 0..tl.rows {
                                            for c in 0..tl.columns {
                                                let cell_val =
                                                    tl.cells[(r * tl.columns + c) as usize];
                                                let static_idx = if cell_val < 0 {
                                                    let anim_slot =
                                                        (-cell_val - 1) as usize;
                                                    tl.animated_tiles
                                                        .get(anim_slot)
                                                        .copied()
                                                        .unwrap_or(0)
                                                } else {
                                                    cell_val
                                                };
                                                if static_idx <= 0 {
                                                    continue;
                                                }
                                                let src_col =
                                                    (static_idx - 1) % tiles_per_row;
                                                let src_row =
                                                    (static_idx - 1) / tiles_per_row;
                                                let src_x = src_col * tl.tile_width;
                                                let src_y = src_row * tl.tile_height;
                                                let dest_x = lx + c * tl.tile_width;
                                                let dest_y = ly + r * tl.tile_height;

                                                g_clone.draw_region(
                                                    target_pixels,
                                                    &img,
                                                    src_x,
                                                    src_y,
                                                    tl.tile_width,
                                                    tl.tile_height,
                                                    0,
                                                    dest_x,
                                                    dest_y,
                                                    0,
                                                );
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        // 22. Graphics
        if cls == "javax/microedition/lcdui/Graphics" {
            let g_id = receiver.unwrap_or(ObjectId(0));
            context.get_or_create_graphics(g_id, GraphicsTarget::Display);
            let target = context
                .graphics_contexts
                .get(&g_id)
                .map(|gc| gc.target)
                .unwrap_or(GraphicsTarget::Display);

            match method_name {
                "setColor" => {
                    if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                        if args.len() == 1 {
                            let rgb = args[0].as_int().unwrap_or(0);
                            gc.graphics.set_color_rgb(rgb);
                        } else if args.len() >= 3 {
                            let r = args[0].as_int().unwrap_or(0) as u8;
                            let g = args[1].as_int().unwrap_or(0) as u8;
                            let b = args[2].as_int().unwrap_or(0) as u8;
                            gc.graphics.current_color = make_rgba(r, g, b, 255);
                        }
                    }
                    return Ok(None);
                }
                "getColor" => {
                    if let Some(gc) = context.graphics_contexts.get(&g_id) {
                        let (r, g, b, _) = unpack_rgba(gc.graphics.current_color);
                        let rgb = ((r as i32) << 16) | ((g as i32) << 8) | (b as i32);
                        return Ok(Some(JavaValue::Int(rgb)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "translate" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                        gc.graphics.translate(x, y);
                    }
                    return Ok(None);
                }
                "getTranslateX" => {
                    let tx = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.trans_x)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(tx)));
                }
                "getTranslateY" => {
                    let ty = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.trans_y)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(ty)));
                }
                "setClip" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                        gc.graphics.set_clip(x, y, w, h);
                    }
                    return Ok(None);
                }
                "clipRect" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                        gc.graphics.clip_rect(x, y, w, h);
                    }
                    return Ok(None);
                }
                "getClipX" => {
                    let cx = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clip_x - gc.graphics.trans_x)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(cx)));
                }
                "getClipY" => {
                    let cy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clip_y - gc.graphics.trans_y)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(cy)));
                }
                "getClipWidth" => {
                    let cw = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clip_w)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(cw)));
                }
                "getClipHeight" => {
                    let ch = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clip_h)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(ch)));
                }
                "fillRect" => {
                    context.fill_rect_count = context.fill_rect_count.wrapping_add(1);
                    let x = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let w = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let h = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(gc) = context.graphics_contexts.get(&g_id) {
                        let g_copy = gc.graphics.clone();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.fill_rect(buf, x, y, w, h);
                        });
                    }
                    return Ok(None);
                }
                "drawRect" => {
                    let x = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let w = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let h = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(gc) = context.graphics_contexts.get(&g_id) {
                        let g_copy = gc.graphics.clone();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.draw_rect(buf, x, y, w, h);
                        });
                    }
                    return Ok(None);
                }
                "drawLine" => {
                    let x1 = args.get(0).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y1 = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let x2 = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y2 = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(gc) = context.graphics_contexts.get(&g_id) {
                        let g_copy = gc.graphics.clone();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.draw_line(buf, x1, y1, x2, y2);
                        });
                    }
                    return Ok(None);
                }
                "drawImage" => {
                    context.draw_image_count = context.draw_image_count.wrapping_add(1);
                    let img_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let x = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let anchor = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(src_id) = img_id {
                        let src_img = if let Some(img) = context.images.get(&src_id) {
                            img.clone()
                        } else {
                            Image::new_immutable(1, 1, vec![0])
                        };
                        if let Some(gc) = context.graphics_contexts.get(&g_id) {
                            let g_copy = gc.graphics.clone();
                            with_graphics_target_mut(context, target, |buf| {
                                g_copy.draw_image(buf, &src_img, x, y, anchor);
                            });
                        }
                    }
                    return Ok(None);
                }
                "drawRegion" => {
                    context.draw_region_count = context.draw_region_count.wrapping_add(1);
                    let img_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let src_x = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let src_y = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let w = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let h = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let transform = args.get(5).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let dst_x = args.get(6).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let dst_y = args.get(7).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let anchor = args.get(8).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(src_id) = img_id {
                        let src_img = if let Some(img) = context.images.get(&src_id) {
                            img.clone()
                        } else {
                            Image::new_immutable(1, 1, vec![0])
                        };
                        if let Some(gc) = context.graphics_contexts.get(&g_id) {
                            let g_copy = gc.graphics.clone();
                            with_graphics_target_mut(context, target, |buf| {
                                g_copy.draw_region(
                                    buf, &src_img, src_x, src_y, w, h, transform, dst_x, dst_y,
                                    anchor,
                                );
                            });
                        }
                    }
                    return Ok(None);
                }
                "drawRGB" => {
                    let rgb_arr_id = args[0].as_object().ok().flatten();
                    let offset = args[1].as_int().unwrap_or(0) as usize;
                    let scanlength = args[2].as_int().unwrap_or(0) as usize;
                    let x = args[3].as_int().unwrap_or(0);
                    let y = args[4].as_int().unwrap_or(0);
                    let width = args[5].as_int().unwrap_or(0);
                    let height = args[6].as_int().unwrap_or(0);
                    let process_alpha = args[7].as_int().unwrap_or(0) != 0;

                    if let Some(aid) = rgb_arr_id {
                        if let Some(ObjectData::Array(ArrayData::Int(data))) =
                            heap.get(aid).cloned()
                        {
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                g_copy.draw_rgb(
                                    buf,
                                    &data,
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    process_alpha,
                                );
                            });
                        }
                    }
                    return Ok(None);
                }
                "copyArea" => {
                    let sx = args[0].as_int().unwrap_or(0);
                    let sy = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    let dx = args[4].as_int().unwrap_or(0);
                    let dy = args[5].as_int().unwrap_or(0);
                    let anchor = args[6].as_int().unwrap_or(0);

                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        g_copy.copy_area(buf, sx, sy, w, h, dx, dy, anchor);
                    });
                    return Ok(None);
                }
                "drawString" => {
                    context.draw_string_count = context.draw_string_count.wrapping_add(1);
                    let str_id = args[0].as_object().ok().flatten();
                    let x = args[1].as_int().unwrap_or(0);
                    let y = args[2].as_int().unwrap_or(0);
                    let anchor = args[3].as_int().unwrap_or(0);

                    if let Some(sid) = str_id {
                        if let Some(s) = heap.get_string_utf8(sid) {
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                g_copy.draw_string(buf, &s, x, y, anchor);
                            });
                        }
                    }
                    return Ok(None);
                }
                "drawSubstring" => {
                    context.draw_string_count = context.draw_string_count.wrapping_add(1);
                    let str_id = args[0].as_object().ok().flatten();
                    let offset = args[1].as_int().unwrap_or(0) as usize;
                    let len = args[2].as_int().unwrap_or(0) as usize;
                    let x = args[3].as_int().unwrap_or(0);
                    let y = args[4].as_int().unwrap_or(0);
                    let anchor = args[5].as_int().unwrap_or(0);

                    if let Some(sid) = str_id {
                        if let Some(s) = heap.get_string_utf8(sid) {
                            let sub: String = s.chars().skip(offset).take(len).collect();
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                g_copy.draw_string(buf, &sub, x, y, anchor);
                            });
                        }
                    }
                    return Ok(None);
                }
                "drawChar" => {
                    context.draw_string_count = context.draw_string_count.wrapping_add(1);
                    let ch = args[0].as_int().unwrap_or(0) as u32;
                    let x = args[1].as_int().unwrap_or(0);
                    let y = args[2].as_int().unwrap_or(0);
                    let anchor = args[3].as_int().unwrap_or(0);

                    if let Some(c) = char::from_u32(ch) {
                        let g_copy = context
                            .graphics_contexts
                            .get(&g_id)
                            .map(|gc| gc.graphics.clone())
                            .unwrap();
                        let s = c.to_string();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.draw_string(buf, &s, x, y, anchor);
                        });
                    }
                    return Ok(None);
                }
                "drawChars" => {
                    context.draw_string_count = context.draw_string_count.wrapping_add(1);
                    let data_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let offset = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let x = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let anchor = args.get(5).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    if let Some(did) = data_id {
                        if let Some(ObjectData::Array(ArrayData::Char(chars))) = heap.get(did) {
                            if offset < chars.len() {
                                let slice_len = len.min(chars.len() - offset);
                                let s = String::from_utf16_lossy(&chars[offset..offset + slice_len]);
                                let g_copy = context
                                    .graphics_contexts
                                    .get(&g_id)
                                    .map(|gc| gc.graphics.clone())
                                    .unwrap();
                                with_graphics_target_mut(context, target, |buf| {
                                    g_copy.draw_string(buf, &s, x, y, anchor);
                                });
                            }
                        }
                    }
                    return Ok(None);
                }
                "getFont" => {
                    let font_spec = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.font)
                        .unwrap_or_default();
                    let font_id = heap.alloc_instance("javax/microedition/lcdui/Font");
                    heap.set_field(font_id, "face", JavaValue::Int(font_spec.face));
                    heap.set_field(font_id, "style", JavaValue::Int(font_spec.style));
                    heap.set_field(font_id, "size", JavaValue::Int(font_spec.size));
                    return Ok(Some(JavaValue::Object(Some(font_id))));
                }
                "setFont" => {
                    if let Some(font_id) = args.first().and_then(|v| v.as_object().ok()).flatten() {
                        let face = heap.get_field(font_id, "face").as_int().unwrap_or(0);
                        let style = heap.get_field(font_id, "style").as_int().unwrap_or(0);
                        let size = heap.get_field(font_id, "size").as_int().unwrap_or(0);
                        if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                            gc.graphics.font = FontSpec::new(face, style, size);
                        }
                    }
                    return Ok(None);
                }
                "fillRoundRect" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    let arc_w = args[4].as_int().unwrap_or(0);
                    let arc_h = args[5].as_int().unwrap_or(0);
                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        g_copy.fill_round_rect(buf, x, y, w, h, arc_w, arc_h);
                    });
                    return Ok(None);
                }
                "drawRoundRect" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    let arc_w = args[4].as_int().unwrap_or(0);
                    let arc_h = args[5].as_int().unwrap_or(0);
                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        g_copy.draw_round_rect(buf, x, y, w, h, arc_w, arc_h);
                    });
                    return Ok(None);
                }
                "fillTriangle" => {
                    let x1 = args[0].as_int().unwrap_or(0);
                    let y1 = args[1].as_int().unwrap_or(0);
                    let x2 = args[2].as_int().unwrap_or(0);
                    let y2 = args[3].as_int().unwrap_or(0);
                    let x3 = args[4].as_int().unwrap_or(0);
                    let y3 = args[5].as_int().unwrap_or(0);

                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        g_copy.fill_triangle(buf, x1, y1, x2, y2, x3, y3);
                    });
                    return Ok(None);
                }
                "drawTriangle" => {
                    let x1 = args[0].as_int().unwrap_or(0);
                    let y1 = args[1].as_int().unwrap_or(0);
                    let x2 = args[2].as_int().unwrap_or(0);
                    let y2 = args[3].as_int().unwrap_or(0);
                    let x3 = args[4].as_int().unwrap_or(0);
                    let y3 = args[5].as_int().unwrap_or(0);

                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        g_copy.draw_triangle(buf, x1, y1, x2, y2, x3, y3);
                    });
                    return Ok(None);
                }
                "fillArc" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    let start_angle = args[4].as_int().unwrap_or(0);
                    let arc_angle = args[5].as_int().unwrap_or(0);
                    if w > 0 && h > 0 {
                        let g_copy = context
                            .graphics_contexts
                            .get(&g_id)
                            .map(|gc| gc.graphics.clone())
                            .unwrap();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.fill_arc(buf, x, y, w, h, start_angle, arc_angle);
                        });
                    }
                    return Ok(None);
                }
                "drawArc" => {
                    let x = args[0].as_int().unwrap_or(0);
                    let y = args[1].as_int().unwrap_or(0);
                    let w = args[2].as_int().unwrap_or(0);
                    let h = args[3].as_int().unwrap_or(0);
                    let start_angle = args[4].as_int().unwrap_or(0);
                    let arc_angle = args[5].as_int().unwrap_or(0);
                    if w > 0 && h > 0 {
                        let g_copy = context
                            .graphics_contexts
                            .get(&g_id)
                            .map(|gc| gc.graphics.clone())
                            .unwrap();
                        with_graphics_target_mut(context, target, |buf| {
                            g_copy.draw_arc(buf, x, y, w, h, start_angle, arc_angle);
                        });
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 23. Nokia DirectUtils & DirectGraphics
        if cls == "com/nokia/mid/ui/DirectUtils" {
            match method_name {
                "getDirectGraphics" => {
                    let g_id = args
                        .first()
                        .and_then(|v| v.as_object().ok())
                        .flatten()
                        .unwrap_or(ObjectId(0));
                    let dg_id = heap.alloc_instance("com/nokia/mid/ui/DirectGraphics");
                    context.dg_to_graphics.insert(dg_id, g_id);
                    return Ok(Some(JavaValue::Object(Some(dg_id))));
                }
                "createImage" => {
                    if descriptor.starts_with("([BII)")
                        || (args.len() == 3
                            && args[0].as_object().is_ok()
                            && args[0].as_object().unwrap().is_some())
                    {
                        let byte_arr_id = args[0].as_object().ok().flatten();
                        let offset = args[1].as_int().unwrap_or(0);
                        let len = args[2].as_int().unwrap_or(0);

                        if offset < 0 || len < 0 {
                            let exc =
                                heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                            return Err(format!(
                                "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                exc.0
                            ));
                        }
                        let (off_u, len_u) = (offset as usize, len as usize);

                        if let Some(id) = byte_arr_id {
                            if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(id) {
                                if off_u + len_u > b.len() {
                                    let exc = heap
                                        .alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                    return Err(format!(
                                        "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                        exc.0
                                    ));
                                }
                                let raw_slice: Vec<u8> =
                                    b[off_u..off_u + len_u].iter().map(|&x| x as u8).collect();
                                if let Ok(img) = Image::from_bytes(&raw_slice) {
                                    let img_id =
                                        heap.alloc_instance("javax/microedition/lcdui/Image");
                                    context.images.insert(img_id, img);
                                    return Ok(Some(JavaValue::Object(Some(img_id))));
                                }
                            }
                        }
                        return Ok(Some(JavaValue::NULL));
                    } else if descriptor.starts_with("(III)")
                        || (args.len() == 3 && args[0].as_int().is_ok())
                    {
                        let w = args[0].as_int().unwrap_or(0);
                        let h = args[1].as_int().unwrap_or(0);
                        let argb = args[2].as_int().unwrap_or(0) as u32;

                        if w <= 0 || h <= 0 {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }

                        let img = Image::new_mutable_with_color(w, h, argb);
                        let id = heap.alloc_instance("javax/microedition/lcdui/Image");
                        context.images.insert(id, img);
                        return Ok(Some(JavaValue::Object(Some(id))));
                    } else if args.len() == 2 {
                        let w = args[0].as_int().unwrap_or(0);
                        let h = args[1].as_int().unwrap_or(0);

                        if w <= 0 || h <= 0 {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }

                        let img = Image::new_mutable(w, h);
                        let id = heap.alloc_instance("javax/microedition/lcdui/Image");
                        context.images.insert(id, img);
                        return Ok(Some(JavaValue::Object(Some(id))));
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                _ => {}
            }
        }

        if cls == "com/nokia/mid/ui/DirectGraphics"
            || cls.ends_with("/DirectGraphics")
            || cls.ends_with("/DirectGraphicsImpl")
            || class_loader.is_instance_of(&cls, "com/nokia/mid/ui/DirectGraphics")
        {
            if method_name == "getNativePixelFormat" {
                return Ok(Some(JavaValue::Int(4444))); // DirectGraphics::TYPE_USHORT_4444_ARGB
            }

            let dg_id = receiver.unwrap_or(ObjectId(0));
            let g_id = context
                .dg_to_graphics
                .get(&dg_id)
                .copied()
                .unwrap_or(ObjectId(0));
            context.get_or_create_graphics(g_id, GraphicsTarget::Display);
            let target = context
                .graphics_contexts
                .get(&g_id)
                .map(|gc| gc.target)
                .unwrap_or(GraphicsTarget::Display);

            match method_name {
                "setARGBColor" => {
                    let argb = args[0].as_int().unwrap_or(0);
                    if let Some(gc) = context.graphics_contexts.get_mut(&g_id) {
                        gc.graphics.set_color_argb(argb);
                    }
                    return Ok(None);
                }
                "drawPixels" => {
                    context.draw_pixels_count = context.draw_pixels_count.wrapping_add(1);
                    // drawPixels(int[] or short[], boolean transparency, int offset, int scanlength, int x, int y, int width, int height, int manipulation, int format)
                    let arr_id = match args[0].as_object().ok().flatten() {
                        Some(id) => id,
                        None => {
                            let exc = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                        }
                    };
                    let transparency = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;
                    let offset = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let scanlength = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let x = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(5).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let width = args.get(6).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let height = args.get(7).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let manipulation = args.get(8).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let format = args.get(9).and_then(|v| v.as_int().ok()).unwrap_or(1);

                    if offset < 0 {
                        let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }
                    if width <= 0 || height <= 0 {
                        return Ok(None);
                    }
                    let rot = manipulation & 0x1FFF;
                    let allowed_rot = rot == 0 || rot == 90 || rot == 180 || rot == 270;
                    let flags = manipulation & !0x1FFF;
                    let allowed_flags = (flags & !(0x2000 | 0x4000)) == 0;
                    if !allowed_rot || !allowed_flags {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!(
                            "EXC:java/lang/IllegalArgumentException:{}",
                            exc.0
                        ));
                    }

                    let arr_obj = heap.get(arr_id).cloned();
                    match arr_obj {
                        Some(ObjectData::Array(ArrayData::Int(ref i_vec))) => {
                            let norm_format = match format {
                                DirectGraphics::TYPE_INT_8888_ARGB | 1 => {
                                    DirectGraphics::TYPE_INT_8888_ARGB
                                }
                                DirectGraphics::TYPE_INT_888_RGB | 6 => {
                                    DirectGraphics::TYPE_INT_888_RGB
                                }
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            };
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                DirectGraphics::draw_pixels_int(
                                    &g_copy,
                                    buf,
                                    i_vec,
                                    transparency,
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    manipulation,
                                    norm_format,
                                );
                            });
                        }
                        Some(ObjectData::Array(ArrayData::Short(ref s_vec))) => {
                            let norm_format = match format {
                                DirectGraphics::TYPE_USHORT_4444_ARGB | 2 => {
                                    DirectGraphics::TYPE_USHORT_4444_ARGB
                                }
                                DirectGraphics::TYPE_USHORT_565_RGB | 3 => {
                                    DirectGraphics::TYPE_USHORT_565_RGB
                                }
                                DirectGraphics::TYPE_USHORT_444_RGB | 4 => {
                                    DirectGraphics::TYPE_USHORT_444_RGB
                                }
                                DirectGraphics::TYPE_USHORT_1555_ARGB | 5 => {
                                    DirectGraphics::TYPE_USHORT_1555_ARGB
                                }
                                DirectGraphics::TYPE_USHORT_555_RGB => {
                                    DirectGraphics::TYPE_USHORT_555_RGB
                                }
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            };
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                DirectGraphics::draw_pixels_short(
                                    &g_copy,
                                    buf,
                                    s_vec,
                                    transparency,
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    manipulation,
                                    norm_format,
                                );
                            });
                        }
                        Some(ObjectData::Array(ArrayData::Byte(ref b_vec))) => {
                            match format {
                                DirectGraphics::TYPE_BYTE_1_GRAY
                                | DirectGraphics::TYPE_BYTE_1_GRAY_VERTICAL
                                | DirectGraphics::TYPE_BYTE_2_GRAY
                                | DirectGraphics::TYPE_BYTE_4_GRAY
                                | DirectGraphics::TYPE_BYTE_8_GRAY
                                | DirectGraphics::TYPE_BYTE_332_RGB => {}
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            }
                            let norm_format = format;
                            let mask_vec = match args[1].as_object().ok().flatten() {
                                Some(mask_id) => {
                                    if let Some(ObjectData::Array(ArrayData::Byte(ref m))) =
                                        heap.get(mask_id)
                                    {
                                        Some(m.clone())
                                    } else {
                                        None
                                    }
                                }
                                None => None,
                            };
                            let g_copy = context
                                .graphics_contexts
                                .get(&g_id)
                                .map(|gc| gc.graphics.clone())
                                .unwrap();
                            with_graphics_target_mut(context, target, |buf| {
                                DirectGraphics::draw_pixels_byte(
                                    &g_copy,
                                    buf,
                                    b_vec,
                                    mask_vec.as_deref(),
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    manipulation,
                                    norm_format,
                                );
                            });
                        }
                        _ => {}
                    }
                    return Ok(None);
                }
                "getPixels" => {
                    let arr_id = match args[0].as_object().ok().flatten() {
                        Some(id) => id,
                        None => return Ok(None),
                    };
                    let offset = args[1].as_int().unwrap_or(0);
                    let scanlength = args[2].as_int().unwrap_or(0);
                    let x = args[3].as_int().unwrap_or(0);
                    let y = args[4].as_int().unwrap_or(0);
                    let width = args[5].as_int().unwrap_or(0);
                    let height = args[6].as_int().unwrap_or(0);
                    let format = args[7].as_int().unwrap_or(1);

                    if offset < 0 {
                        let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }
                    if width <= 0 || height <= 0 {
                        return Ok(None);
                    }

                    let g_copy = match context.graphics_contexts.get(&g_id) {
                        Some(gc) => gc.graphics.clone(),
                        None => return Ok(None),
                    };

                    match heap.get_mut(arr_id) {
                        Some(ObjectData::Array(ArrayData::Int(ref mut i_vec))) => {
                            let norm_format = match format {
                                DirectGraphics::TYPE_INT_8888_ARGB | 1 => {
                                    DirectGraphics::TYPE_INT_8888_ARGB
                                }
                                DirectGraphics::TYPE_INT_888_RGB | 6 => {
                                    DirectGraphics::TYPE_INT_888_RGB
                                }
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            };
                            let req_len = ((offset + (height - 1) * scanlength + width).max(0)) as usize;
                            if i_vec.len() < req_len {
                                i_vec.resize(req_len, 0);
                            }
                            with_graphics_target_ref(context, target, |buf| {
                                DirectGraphics::get_pixels_int(
                                    &g_copy,
                                    buf,
                                    i_vec,
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    norm_format,
                                );
                            });
                        }
                        Some(ObjectData::Array(ArrayData::Short(ref mut s_vec))) => {
                            let norm_format = match format {
                                DirectGraphics::TYPE_USHORT_4444_ARGB | 2 => {
                                    DirectGraphics::TYPE_USHORT_4444_ARGB
                                }
                                DirectGraphics::TYPE_USHORT_565_RGB | 3 => {
                                    DirectGraphics::TYPE_USHORT_565_RGB
                                }
                                DirectGraphics::TYPE_USHORT_444_RGB | 4 => {
                                    DirectGraphics::TYPE_USHORT_444_RGB
                                }
                                DirectGraphics::TYPE_USHORT_1555_ARGB | 5 => {
                                    DirectGraphics::TYPE_USHORT_1555_ARGB
                                }
                                DirectGraphics::TYPE_USHORT_555_RGB => {
                                    DirectGraphics::TYPE_USHORT_555_RGB
                                }
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            };
                            let req_len = ((offset + (height - 1) * scanlength + width).max(0)) as usize;
                            if s_vec.len() < req_len {
                                s_vec.resize(req_len, 0);
                            }
                            with_graphics_target_ref(context, target, |buf| {
                                DirectGraphics::get_pixels_short(
                                    &g_copy,
                                    buf,
                                    s_vec,
                                    offset,
                                    scanlength,
                                    x,
                                    y,
                                    width,
                                    height,
                                    norm_format,
                                );
                            });
                        }
                        Some(ObjectData::Array(ArrayData::Byte(ref mut b_vec))) => {
                            match format {
                                DirectGraphics::TYPE_BYTE_1_GRAY
                                | DirectGraphics::TYPE_BYTE_1_GRAY_VERTICAL
                                | DirectGraphics::TYPE_BYTE_2_GRAY
                                | DirectGraphics::TYPE_BYTE_4_GRAY
                                | DirectGraphics::TYPE_BYTE_8_GRAY
                                | DirectGraphics::TYPE_BYTE_332_RGB => {}
                                _ => {
                                    let exc =
                                        heap.alloc_instance("java/lang/IllegalArgumentException");
                                    return Err(format!(
                                        "EXC:java/lang/IllegalArgumentException:{}",
                                        exc.0
                                    ));
                                }
                            }
                            let norm_format = format;
                            with_graphics_target_ref(context, target, |buf| {
                                DirectGraphics::get_pixels_byte(
                                    &g_copy, buf, b_vec, offset, scanlength, x, y, width,
                                    height, norm_format,
                                );
                            });
                        }
                        _ => {}
                    }
                    return Ok(None);
                }
                "drawImage" => {
                    let img_id = match args[0].as_object().ok().flatten() {
                        Some(id) => id,
                        None => return Ok(None),
                    };
                    let x = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let y = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let anchor = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let manipulation = args.get(4).and_then(|v| v.as_int().ok()).unwrap_or(0);

                    let rot = manipulation & 0x1FFF;
                    let allowed_rot = rot == 0 || rot == 90 || rot == 180 || rot == 270;
                    let flags = manipulation & !0x1FFF;
                    let allowed_flags = (flags & !(0x2000 | 0x4000)) == 0;
                    if !allowed_rot || !allowed_flags {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                    }

                    let src_img = if let Some(img) = context.images.get(&img_id) {
                        img.clone()
                    } else {
                        Image::new_immutable(1, 1, vec![0])
                    };
                    if let Some(gc) = context.graphics_contexts.get(&g_id) {
                        let g_copy = gc.graphics.clone();
                        with_graphics_target_mut(context, target, |buf| {
                            DirectGraphics::draw_image(
                                &g_copy,
                                buf,
                                &src_img,
                                x,
                                y,
                                anchor,
                                manipulation,
                            );
                        });
                    }
                    return Ok(None);
                }
                "fillPolygon" => {
                    let x_arr_id = match args[0].as_object().ok().flatten() {
                        Some(id) => id,
                        None => {
                            let exc = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                        }
                    };
                    let x_off = args[1].as_int().unwrap_or(0);
                    let y_arr_id = match args[2].as_object().ok().flatten() {
                        Some(id) => id,
                        None => {
                            let exc = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                        }
                    };
                    let y_off = args[3].as_int().unwrap_or(0);
                    let n_pts = args[4].as_int().unwrap_or(0);
                    let argb = args[5].as_int().unwrap_or(0);

                    if n_pts < 0 {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                    }
                    if x_off < 0 || y_off < 0 {
                        let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }
                    let x_pts: Vec<i32> = match heap.get(x_arr_id).and_then(|o| o.as_array()) {
                        Some(ArrayData::Int(v)) => {
                            let end = x_off as usize + n_pts as usize;
                            if end > v.len() {
                                let exc =
                                    heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!(
                                    "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                    exc.0
                                ));
                            }
                            v[x_off as usize..end].to_vec()
                        }
                        _ => {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }
                    };
                    let y_pts: Vec<i32> = match heap.get(y_arr_id).and_then(|o| o.as_array()) {
                        Some(ArrayData::Int(v)) => {
                            let end = y_off as usize + n_pts as usize;
                            if end > v.len() {
                                let exc =
                                    heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!(
                                    "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                    exc.0
                                ));
                            }
                            v[y_off as usize..end].to_vec()
                        }
                        _ => {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }
                    };

                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        DirectGraphics::fill_polygon(
                            &g_copy,
                            buf,
                            &x_pts,
                            &y_pts,
                            n_pts as usize,
                            argb,
                        );
                    });
                    return Ok(None);
                }
                "drawPolygon" => {
                    let x_arr_id = match args[0].as_object().ok().flatten() {
                        Some(id) => id,
                        None => {
                            let exc = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                        }
                    };
                    let x_off = args[1].as_int().unwrap_or(0);
                    let y_arr_id = match args[2].as_object().ok().flatten() {
                        Some(id) => id,
                        None => {
                            let exc = heap.alloc_instance("java/lang/NullPointerException");
                            return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                        }
                    };
                    let y_off = args[3].as_int().unwrap_or(0);
                    let n_pts = args[4].as_int().unwrap_or(0);
                    let argb = args[5].as_int().unwrap_or(0);

                    if n_pts < 0 {
                        let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                        return Err(format!("EXC:java/lang/IllegalArgumentException:{}", exc.0));
                    }
                    if x_off < 0 || y_off < 0 {
                        let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                        return Err(format!(
                            "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                            exc.0
                        ));
                    }
                    let x_pts: Vec<i32> = match heap.get(x_arr_id).and_then(|o| o.as_array()) {
                        Some(ArrayData::Int(v)) => {
                            let end = x_off as usize + n_pts as usize;
                            if end > v.len() {
                                let exc =
                                    heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!(
                                    "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                    exc.0
                                ));
                            }
                            v[x_off as usize..end].to_vec()
                        }
                        _ => {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }
                    };
                    let y_pts: Vec<i32> = match heap.get(y_arr_id).and_then(|o| o.as_array()) {
                        Some(ArrayData::Int(v)) => {
                            let end = y_off as usize + n_pts as usize;
                            if end > v.len() {
                                let exc =
                                    heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                return Err(format!(
                                    "EXC:java/lang/ArrayIndexOutOfBoundsException:{}",
                                    exc.0
                                ));
                            }
                            v[y_off as usize..end].to_vec()
                        }
                        _ => {
                            let exc = heap.alloc_instance("java/lang/IllegalArgumentException");
                            return Err(format!(
                                "EXC:java/lang/IllegalArgumentException:{}",
                                exc.0
                            ));
                        }
                    };

                    let g_copy = context
                        .graphics_contexts
                        .get(&g_id)
                        .map(|gc| gc.graphics.clone())
                        .unwrap();
                    with_graphics_target_mut(context, target, |buf| {
                        DirectGraphics::draw_polygon(
                            &g_copy,
                            buf,
                            &x_pts,
                            &y_pts,
                            n_pts as usize,
                            argb,
                        );
                    });
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 24. Nokia DeviceControl
        if cls == "com/nokia/mid/ui/DeviceControl" {
            match method_name {
                "startVibra" => {
                    context.vibration_active = true;
                    return Ok(None);
                }
                "stopVibra" => {
                    context.vibration_active = false;
                    return Ok(None);
                }
                "setLights" => {
                    let lvl = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(100);
                    context.lights_level = lvl;
                    return Ok(None);
                }
                _ => {}
            }
        }

        // Nokia Sound
        if cls == "com/nokia/mid/sound/Sound" {
            match method_name {
                "<init>" => {
                    if let Some(r) = receiver {
                        if !args.is_empty() {
                            if let Some(arr_id) = args[0].as_object().ok().flatten() {
                                if let Some(ObjectData::Array(ArrayData::Byte(ref b))) =
                                    heap.get(arr_id)
                                {
                                    let bytes: Vec<u8> = b.iter().map(|&x| x as u8).collect();
                                    let content_type = if bytes.starts_with(b"MThd") {
                                        "audio/midi"
                                    } else if bytes.starts_with(b"RIFF") {
                                        "audio/wav"
                                    } else {
                                        "audio/midi"
                                    };
                                    let (_, player) = context.alloc_player(content_type, bytes);
                                    context.players.insert(r, player);
                                }
                            } else if let Ok(freq) = args[0].as_int() {
                                let dur = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(200);
                                heap.set_field(r, "freq", JavaValue::Int(freq));
                                heap.set_field(r, "duration", JavaValue::Int(dur));
                            }
                        }
                    }
                    return Ok(None);
                }
                "play" => {
                    let loop_count = args.first().and_then(|v| v.as_int().ok()).unwrap_or(1);
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            let loops = if loop_count <= 0 { -1 } else { loop_count };
                            player.set_loop_count(loops);
                            player.start(&mut context.audio);
                        } else {
                            let freq = heap.get_field(r, "freq").as_int().unwrap_or(0);
                            let dur = heap.get_field(r, "duration").as_int().unwrap_or(0);
                            if freq > 0 && dur > 0 {
                                let note =
                                    (69.0 + 12.0 * ((freq as f32) / 440.0).log2()).round() as i32;
                                context.audio.synthesize_tone(note.clamp(0, 127), dur, 80);
                            }
                        }
                    }
                    return Ok(None);
                }
                "stop" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.stop();
                        }
                    }
                    return Ok(None);
                }
                "getState" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get(&r) {
                            if player.state == crate::media::manager::STATE_STARTED {
                                return Ok(Some(JavaValue::Int(0))); // SOUND_PLAYING
                            }
                        }
                    }
                    return Ok(Some(JavaValue::Int(1))); // SOUND_STOPPED
                }
                "setGain" => {
                    let gain = args.first().and_then(|v| v.as_int().ok()).unwrap_or(100);
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.set_volume(gain);
                        }
                    }
                    return Ok(None);
                }
                "release" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.close();
                        }
                    }
                    return Ok(None);
                }
                "init" => {
                    if let Some(r) = receiver {
                        if !args.is_empty() {
                            if let Some(arr_id) = args[0].as_object().ok().flatten() {
                                if let Some(ObjectData::Array(ArrayData::Byte(ref b))) =
                                    heap.get(arr_id)
                                {
                                    let bytes: Vec<u8> = b.iter().map(|&x| x as u8).collect();
                                    let content_type = if bytes.starts_with(b"MThd") {
                                        "audio/midi"
                                    } else if bytes.starts_with(b"RIFF") {
                                        "audio/wav"
                                    } else {
                                        "audio/midi"
                                    };
                                    let (_, player) = context.alloc_player(content_type, bytes);
                                    context.players.insert(r, player);
                                }
                            }
                        }
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // 25. MMAPI (Media)
        if cls == "javax/microedition/media/Manager" {
            match method_name {
                "createPlayer" => {
                    let mut media_bytes = Vec::new();
                    let mut content_type = "audio/midi".to_string();

                    if !args.is_empty() {
                        if let Some(stream_id) = args[0].as_object().ok().flatten() {
                            let mut cur_stream = stream_id;
                            for _ in 0..10 {
                                if let Some(buf_id) =
                                    heap.get_field(cur_stream, "buf").as_object().ok().flatten()
                                {
                                    if let Some(ObjectData::Array(ArrayData::Byte(ref b))) =
                                        heap.get(buf_id)
                                    {
                                        let pos = heap
                                            .get_field(cur_stream, "pos")
                                            .as_int()
                                            .unwrap_or(0)
                                            .max(0) as usize;
                                        let count = heap
                                            .get_field(cur_stream, "count")
                                            .as_int()
                                            .unwrap_or(b.len() as i32)
                                            .max(0) as usize;
                                        let end = count.min(b.len());
                                        if pos < end {
                                            media_bytes =
                                                b[pos..end].iter().map(|&x| x as u8).collect();
                                        } else if !b.is_empty() {
                                            media_bytes = b.iter().map(|&x| x as u8).collect();
                                        }
                                        break;
                                    }
                                }
                                if let Some(inner) =
                                    heap.get_field(cur_stream, "in").as_object().ok().flatten()
                                {
                                    cur_stream = inner;
                                } else {
                                    break;
                                }
                            }
                        } else if let Some(locator_id) = args[0].as_object().ok().flatten() {
                            if let Some(s) = heap.get_string_utf8(locator_id) {
                                content_type = s.clone();
                                // If locator points to a resource in JAR, e.g. "/menu.mid"
                                if let Some(ref jar) = context.jar {
                                    let norm = Self::normalize_jar_resource_path(s.trim_start_matches('/'));
                                    if let Some(entry_data) = jar.get_entry(&norm) {
                                        media_bytes = entry_data.to_vec();
                                    }
                                }
                            }
                        }
                    }

                    if args.len() >= 2 {
                        if let Some(type_id) = args[1].as_object().ok().flatten() {
                            if let Some(t) = heap.get_string_utf8(type_id) {
                                content_type = t;
                            }
                        }
                    }

                    // Auto-detect format from magic bytes if available
                    if media_bytes.len() >= 4 {
                        if &media_bytes[0..4] == b"MThd" {
                            content_type = "audio/midi".to_string();
                        } else if &media_bytes[0..4] == b"RIFF" {
                            content_type = "audio/x-wav".to_string();
                        }
                    }

                    let (pid, player) = context.alloc_player(&content_type, media_bytes);
                    let inst = heap.alloc_instance("javax/microedition/media/Player");
                    heap.set_field(inst, "playerId", JavaValue::Int(pid));
                    context.players.insert(inst, player);
                    return Ok(Some(JavaValue::Object(Some(inst))));
                }
                "playTone" => {
                    let note = args.first().and_then(|v| v.as_int().ok()).unwrap_or(69);
                    let dur = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(100);
                    let vol = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(80);
                    context.audio.synthesize_tone(note, dur, vol);
                    return Ok(None);
                }
                "getSupportedContentTypes" => {
                    let types = vec![
                        Some(heap.alloc_string("audio/x-wav")),
                        Some(heap.alloc_string("audio/wav")),
                        Some(heap.alloc_string("audio/midi")),
                        Some(heap.alloc_string("audio/mid")),
                        Some(heap.alloc_string("audio/x-mid")),
                    ];
                    let arr_id = heap.alloc_array(ArrayData::Reference(types));
                    return Ok(Some(JavaValue::Object(Some(arr_id))));
                }
                "getSupportedProtocols" => {
                    let protos = vec![
                        Some(heap.alloc_string("device")),
                        Some(heap.alloc_string("capture")),
                    ];
                    let arr_id = heap.alloc_array(ArrayData::Reference(protos));
                    return Ok(Some(JavaValue::Object(Some(arr_id))));
                }
                _ => {}
            }
        }

        if cls == "javax/microedition/media/Player" {
            match method_name {
                "start" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.start(&mut context.audio);
                        }
                    }
                    return Ok(None);
                }
                "stop" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.stop();
                        }
                    }
                    return Ok(None);
                }
                "close" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.close();
                        }
                    }
                    return Ok(None);
                }
                "prefetch" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.prefetch();
                        }
                    }
                    return Ok(None);
                }
                "realize" => {
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.realize();
                        }
                    }
                    return Ok(None);
                }
                "deallocate" => {
                    return Ok(None);
                }
                "setLoopCount" => {
                    let count = args.first().and_then(|v| v.as_int().ok()).unwrap_or(1);
                    if let Some(r) = receiver {
                        if let Some(player) = context.players.get_mut(&r) {
                            player.set_loop_count(count);
                        }
                    }
                    return Ok(None);
                }
                "getState" => {
                    let state = receiver
                        .and_then(|r| context.players.get(&r))
                        .map(|p| p.state)
                        .unwrap_or(crate::media::manager::STATE_UNREALIZED);
                    return Ok(Some(JavaValue::Int(state)));
                }
                "setMediaTime" => {
                    let ms = args.first().and_then(|v| v.as_long().ok()).unwrap_or(0);
                    let landed = receiver
                        .and_then(|r| context.players.get_mut(&r))
                        .map(|p| p.set_media_time(ms))
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Long(landed)));
                }
                "getMediaTime" => {
                    let ms = receiver
                        .and_then(|r| context.players.get(&r))
                        .map(|p| p.media_time())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Long(ms)));
                }
                "getControl" => {
                    let vc_id = if let Some(r) = receiver {
                        if let Some(&id) = context.player_volume_controls.get(&r) {
                            id
                        } else {
                            let id =
                                heap.alloc_instance("javax/microedition/media/control/VolumeControl");
                            heap.set_field(id, "playerObj", JavaValue::Object(Some(r)));
                            context.player_volume_controls.insert(r, id);
                            context.volume_controls.insert(
                                id,
                                crate::bridge::context::VolumeControlState {
                                    level: 100,
                                    muted: false,
                                    player_obj: Some(r),
                                },
                            );
                            id
                        }
                    } else {
                        let id =
                            heap.alloc_instance("javax/microedition/media/control/VolumeControl");
                        context
                            .volume_controls
                            .insert(id, crate::bridge::context::VolumeControlState::default());
                        id
                    };
                    return Ok(Some(JavaValue::Object(Some(vc_id))));
                }
                "getControls" => {
                    let vc_id = if let Some(r) = receiver {
                        if let Some(&id) = context.player_volume_controls.get(&r) {
                            id
                        } else {
                            let id =
                                heap.alloc_instance("javax/microedition/media/control/VolumeControl");
                            heap.set_field(id, "playerObj", JavaValue::Object(Some(r)));
                            context.player_volume_controls.insert(r, id);
                            context.volume_controls.insert(
                                id,
                                crate::bridge::context::VolumeControlState {
                                    level: 100,
                                    muted: false,
                                    player_obj: Some(r),
                                },
                            );
                            id
                        }
                    } else {
                        let id =
                            heap.alloc_instance("javax/microedition/media/control/VolumeControl");
                        context
                            .volume_controls
                            .insert(id, crate::bridge::context::VolumeControlState::default());
                        id
                    };
                    let arr_id = heap.alloc_array(ArrayData::Reference(vec![Some(vc_id)]));
                    return Ok(Some(JavaValue::Object(Some(arr_id))));
                }
                "addPlayerListener" | "removePlayerListener" => return Ok(None),
                _ => {}
            }
        }

        if cls == "javax/microedition/media/control/VolumeControl"
            || class_loader.is_instance_of(&cls, "javax/microedition/media/control/VolumeControl")
        {
            let rec = receiver.unwrap_or(ObjectId(0));
            let state = context.volume_controls.entry(rec).or_default();
            match method_name {
                "<init>" => return Ok(None),
                "setLevel" => {
                    let level = args
                        .first()
                        .and_then(|v| v.as_int().ok())
                        .unwrap_or(100)
                        .clamp(0, 100);
                    state.level = level;
                    let p_obj = state.player_obj.or_else(|| {
                        heap.get_field(rec, "playerObj").as_object().ok().flatten()
                    });
                    if let Some(player_obj) = p_obj {
                        if let Some(player) = context.players.get_mut(&player_obj) {
                            player.set_volume(if state.muted { 0 } else { level });
                        }
                    }
                    return Ok(Some(JavaValue::Int(level)));
                }
                "getLevel" => {
                    return Ok(Some(JavaValue::Int(state.level)));
                }
                "setMute" => {
                    let mute = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;
                    state.muted = mute;
                    let p_obj = state.player_obj.or_else(|| {
                        heap.get_field(rec, "playerObj").as_object().ok().flatten()
                    });
                    if let Some(player_obj) = p_obj {
                        if let Some(player) = context.players.get_mut(&player_obj) {
                            player.set_volume(if mute { 0 } else { state.level });
                        }
                    }
                    return Ok(None);
                }
                "isMuted" => {
                    return Ok(Some(JavaValue::Int(if state.muted { 1 } else { 0 })));
                }
                _ => {}
            }
        }

        // 26. RMS (Record Management System)
        if cls == "javax/microedition/rms/RecordStore" {
            match method_name {
                "openRecordStore" => {
                    let name_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let name = name_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_else(|| "default".to_string());
                    let create = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(1) != 0;
                    if std::env::var_os("J2ME_DEBUG_RMS").is_some() {
                        eprintln!(
                            "[RMS] openRecordStore name='{}' create={} known={:?}",
                            name,
                            create,
                            context.rms.names()
                        );
                    }

                    match context.rms.open(&name, create) {
                        Ok(_) => {
                            let rs_id = heap.alloc_instance("javax/microedition/rms/RecordStore");
                            let str_id = heap.alloc_string(&name);
                            heap.set_field(rs_id, "name", JavaValue::Object(Some(str_id)));
                            return Ok(Some(JavaValue::Object(Some(rs_id))));
                        }
                        Err(e) => {
                            let exc_id = heap.alloc_instance(
                                "javax/microedition/rms/RecordStoreNotFoundException",
                            );
                            let msg_id = heap.alloc_string(&e);
                            heap.set_field(
                                exc_id,
                                "detailMessage",
                                JavaValue::Object(Some(msg_id)),
                            );
                            return Err(format!(
                                "EXC:javax/microedition/rms/RecordStoreNotFoundException:{}",
                                exc_id.0
                            ));
                        }
                    }
                }
                "closeRecordStore" => return Ok(None),
                "deleteRecordStore" => {
                    let name_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let name = name_id
                        .and_then(|id| heap.get_string_utf8(id))
                        .unwrap_or_default();
                    let _ = context.rms.delete(&name);
                    return Ok(None);
                }
                "enumerateRecords" => {
                    // JSR-75: returns a live enumeration over the store's ids.
                    // The filter/comparator arguments are ignored (most MIDlets
                    // pass null); keepUpdated is stored for API compatibility.
                    let name = Self::get_rms_name(receiver, heap);
                    let keep_updated = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;

                    let ids: Vec<i32> = match context.rms.open(&name, false) {
                        Ok(store) => store.ids(),
                        Err(_) => Vec::new(),
                    };

                    let enum_id = heap.alloc_instance("javax/microedition/rms/RecordEnumeration");
                    let str_id = heap.alloc_string(&name);
                    let ids_arr = heap.alloc_array(ArrayData::Int(ids));
                    heap.set_field(enum_id, "store", JavaValue::Object(Some(str_id)));
                    heap.set_field(enum_id, "ids", JavaValue::Object(Some(ids_arr)));
                    heap.set_field(enum_id, "index", JavaValue::Int(0));
                    heap.set_field(
                        enum_id,
                        "keepUpdated",
                        JavaValue::Int(if keep_updated { 1 } else { 0 }),
                    );
                    return Ok(Some(JavaValue::Object(Some(enum_id))));
                }
                "getNumRecords" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let count = context
                        .rms
                        .open(&name, false)
                        .map(|s| s.get_num_records() as i32)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(count)));
                }
                "getNextRecordID" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let next_id = context
                        .rms
                        .open(&name, false)
                        .map(|s| s.next_id)
                        .unwrap_or(1);
                    return Ok(Some(JavaValue::Int(next_id)));
                }
                "getRecord" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let record_id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if record_id < 1 {
                        let exc = heap
                            .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                        return Err(format!(
                            "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                            exc.0
                        ));
                    }

                    if args.len() >= 3 {
                        let byte_arr_id = match args.get(1).and_then(|v| v.as_object().ok()).flatten() {
                            Some(id) => id,
                            None => {
                                let exc = heap.alloc_instance("java/lang/NullPointerException");
                                return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                            }
                        };
                        let offset = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0);
                        if offset < 0 {
                            let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                            return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                        }

                        if let Ok(store) = context.rms.open(&name, false) {
                            if let Some(data) = store.get_record(record_id) {
                                if let Some(dst_obj) = heap.get_mut(byte_arr_id) {
                                    if let Some(ArrayData::Byte(b)) = dst_obj.as_array_mut() {
                                        let u_offset = offset as usize;
                                        if u_offset + data.len() > b.len() {
                                            let exc = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
                                            return Err(format!("EXC:java/lang/ArrayIndexOutOfBoundsException:{}", exc.0));
                                        }
                                        for (i, &byte) in data.iter().enumerate() {
                                            b[u_offset + i] = byte as i8;
                                        }
                                        return Ok(Some(JavaValue::Int(data.len() as i32)));
                                    }
                                }
                            }
                        }
                        let exc = heap
                            .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                        return Err(format!(
                            "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                            exc.0
                        ));
                    }

                    if let Ok(store) = context.rms.open(&name, false) {
                        if let Some(data) = store.get_record(record_id) {
                            let byte_arr = heap.alloc_array(ArrayData::Byte(
                                data.iter().map(|&b| b as i8).collect(),
                            ));
                            return Ok(Some(JavaValue::Object(Some(byte_arr))));
                        }
                    }
                    let exc = heap
                        .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc.0
                    ));
                }
                "addRecord" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let byte_arr_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let offset = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;

                    let slice: Vec<u8> = if let Some(id) = byte_arr_id {
                        if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(id) {
                            let end = (offset + len).min(b.len());
                            if offset <= end {
                                b[offset..end].iter().map(|&x| x as u8).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    if let Ok(store) = context.rms.open(&name, true) {
                        let rid = store.add_record(&slice);
                        return Ok(Some(JavaValue::Int(rid as i32)));
                    }
                    return Ok(Some(JavaValue::Int(0)));
                }
                "setRecord" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let record_id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if record_id < 1 {
                        let exc = heap
                            .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                        return Err(format!(
                            "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                            exc.0
                        ));
                    }
                    let byte_arr_id = args.get(1).and_then(|v| v.as_object().ok()).flatten();
                    let offset = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;

                    let slice: Vec<u8> = if let Some(id) = byte_arr_id {
                        if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(id) {
                            let end = (offset + len).min(b.len());
                            if offset <= end {
                                b[offset..end].iter().map(|&x| x as u8).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    if let Ok(store) = context.rms.open(&name, true) {
                        if store.records.contains_key(&record_id) {
                            store.set_record(record_id, &slice);
                        } else if record_id == store.next_id || store.records.is_empty() || record_id == 1 {
                            store.records.insert(record_id, slice);
                            store.next_id = store.next_id.max(record_id + 1);
                            store.version += 1;
                        } else {
                            let exc = heap
                                .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                            return Err(format!(
                                "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                                exc.0
                            ));
                        }
                    }
                    return Ok(None);
                }
                "getRecordSize" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let record_id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if record_id < 1 {
                        let exc = heap
                            .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                        return Err(format!(
                            "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                            exc.0
                        ));
                    }
                    if let Ok(store) = context.rms.open(&name, false) {
                        if let Some(data) = store.get_record(record_id) {
                            return Ok(Some(JavaValue::Int(data.len() as i32)));
                        }
                    }
                    let exc = heap
                        .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc.0
                    ));
                }
                "deleteRecord" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let record_id = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Ok(store) = context.rms.open(&name, false) {
                        if !store.delete_record(record_id) {
                            let exc = heap
                                .alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                            return Err(format!(
                                "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                                exc.0
                            ));
                        }
                    }
                    return Ok(None);
                }
                "getSizeAvailable" => {
                    // MIDP 2.0: available storage size in bytes
                    return Ok(Some(JavaValue::Int(1_048_576)));
                }
                "getSize" => {
                    let name = Self::get_rms_name(receiver, heap);
                    let size = if let Ok(store) = context.rms.open(&name, false) {
                        store.records.values().map(|v| v.len()).sum::<usize>() as i32
                    } else {
                        0
                    };
                    return Ok(Some(JavaValue::Int(size)));
                }
                "listRecordStores" => {
                    let names: Vec<String> = context.rms.stores.keys().cloned().collect();
                    if names.is_empty() {
                        return Ok(Some(JavaValue::NULL));
                    }
                    let mut str_objs = Vec::new();
                    for n in names {
                        let sid = heap.alloc_string(&n);
                        str_objs.push(Some(sid));
                    }
                    let arr_id = heap.alloc_array(ArrayData::Reference(str_objs));
                    return Ok(Some(JavaValue::Object(Some(arr_id))));
                }
                _ => {}
            }
        }

        // 26b. RMS RecordEnumeration (JSR-75 enumeration over a RecordStore)
        if cls == "javax/microedition/rms/RecordEnumeration" {
            let ids: Vec<i32> = receiver
                .and_then(|r| heap.get_field(r, "ids").as_object().ok().flatten())
                .and_then(|id| match heap.get(id) {
                    Some(ObjectData::Array(ArrayData::Int(v))) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let index = receiver
                .and_then(|r| heap.get_field(r, "index").as_int().ok())
                .unwrap_or(0) as usize;
            let store_name = receiver
                .and_then(|r| heap.get_field(r, "store").as_object().ok().flatten())
                .and_then(|id| heap.get_string_utf8(id))
                .unwrap_or_default();

            match method_name {
                "numRecords" => return Ok(Some(JavaValue::Int(ids.len() as i32))),
                "hasNextElement" => {
                    return Ok(Some(JavaValue::Int(if index < ids.len() { 1 } else { 0 })))
                }
                "hasPreviousElement" => {
                    return Ok(Some(JavaValue::Int(if index > 0 && !ids.is_empty() {
                        1
                    } else {
                        0
                    })))
                }
                "nextRecordId" => {
                    if index < ids.len() {
                        let rid = ids[index];
                        if let Some(r) = receiver {
                            heap.set_field(r, "index", JavaValue::Int((index + 1) as i32));
                        }
                        return Ok(Some(JavaValue::Int(rid)));
                    }
                    let exc_id =
                        heap.alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc_id.0
                    ));
                }
                "previousRecordId" => {
                    if index > 0 && !ids.is_empty() {
                        let prev_idx = index - 1;
                        let rid = ids[prev_idx];
                        if let Some(r) = receiver {
                            heap.set_field(r, "index", JavaValue::Int(prev_idx as i32));
                        }
                        return Ok(Some(JavaValue::Int(rid)));
                    }
                    let exc_id =
                        heap.alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc_id.0
                    ));
                }
                "nextRecord" => {
                    if index < ids.len() {
                        let rid = ids[index];
                        let dest = args.first().and_then(|v| v.as_object().ok()).flatten();
                        let data = context
                            .rms
                            .open(&store_name, false)
                            .ok()
                            .and_then(|st| st.get_record(rid).map(|d| d.to_vec()))
                            .unwrap_or_default();
                        if let Some(r) = receiver {
                            heap.set_field(r, "index", JavaValue::Int((index + 1) as i32));
                        }
                        if let Some(dst_id) = dest {
                            let off =
                                args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                            if let Some(ObjectData::Array(ArrayData::Byte(buf))) =
                                heap.get_mut(dst_id)
                            {
                                for (i, &b) in data.iter().enumerate() {
                                    if off + i < buf.len() {
                                        buf[off + i] = b as i8;
                                    }
                                }
                            }
                            return Ok(Some(JavaValue::Int(data.len() as i32)));
                        } else {
                            let byte_arr = heap.alloc_array(ArrayData::Byte(
                                data.iter().map(|&b| b as i8).collect(),
                            ));
                            return Ok(Some(JavaValue::Object(Some(byte_arr))));
                        }
                    }
                    let exc_id =
                        heap.alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc_id.0
                    ));
                }
                "previousRecord" => {
                    if index > 0 && !ids.is_empty() {
                        let prev_idx = index - 1;
                        let rid = ids[prev_idx];
                        let dest = args.first().and_then(|v| v.as_object().ok()).flatten();
                        let data = context
                            .rms
                            .open(&store_name, false)
                            .ok()
                            .and_then(|st| st.get_record(rid).map(|d| d.to_vec()))
                            .unwrap_or_default();
                        if let Some(r) = receiver {
                            heap.set_field(r, "index", JavaValue::Int(prev_idx as i32));
                        }
                        if let Some(dst_id) = dest {
                            let off =
                                args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                            if let Some(ObjectData::Array(ArrayData::Byte(buf))) =
                                heap.get_mut(dst_id)
                            {
                                for (i, &b) in data.iter().enumerate() {
                                    if off + i < buf.len() {
                                        buf[off + i] = b as i8;
                                    }
                                }
                            }
                            return Ok(Some(JavaValue::Int(data.len() as i32)));
                        } else {
                            let byte_arr = heap.alloc_array(ArrayData::Byte(
                                data.iter().map(|&b| b as i8).collect(),
                            ));
                            return Ok(Some(JavaValue::Object(Some(byte_arr))));
                        }
                    }
                    let exc_id =
                        heap.alloc_instance("javax/microedition/rms/InvalidRecordIDException");
                    return Err(format!(
                        "EXC:javax/microedition/rms/InvalidRecordIDException:{}",
                        exc_id.0
                    ));
                }
                "reset" => {
                    if let Some(r) = receiver {
                        heap.set_field(r, "index", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "rebuild" => {
                    let ids: Vec<i32> = match context.rms.open(&store_name, false) {
                        Ok(store) => store.ids(),
                        Err(_) => Vec::new(),
                    };
                    if let Some(r) = receiver {
                        let ids_arr = heap.alloc_array(ArrayData::Int(ids));
                        heap.set_field(r, "ids", JavaValue::Object(Some(ids_arr)));
                        heap.set_field(r, "index", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "destroy" => {
                    if let Some(r) = receiver {
                        heap.set_field(r, "ids", JavaValue::Object(None));
                        heap.set_field(r, "index", JavaValue::Int(0));
                    }
                    return Ok(None);
                }
                "keepUpdated" => {
                    let flag = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    if let Some(r) = receiver {
                        heap.set_field(r, "keepUpdated", JavaValue::Int(flag));
                    }
                    return Ok(None);
                }
                "isKeptUpdated" => {
                    let flag = receiver
                        .and_then(|r| heap.get_field(r, "keepUpdated").as_int().ok())
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(flag)));
                }
                _ => {}
            }
        }

        // 26b. javax/microedition/io/Connector & Connection Stubs
        if cls == "javax/microedition/io/Connector" {
            match method_name {
                "open" | "openDataInputStream" | "openInputStream" | "openDataOutputStream" | "openOutputStream" => {
                    let url_id = args.first().and_then(|v| v.as_object().ok()).flatten();
                    let url = url_id.and_then(|id| heap.get_string_utf8(id)).unwrap_or_default();
                    if method_name == "openInputStream" || method_name == "openDataInputStream" {
                        let stream_id = heap.alloc_instance("java/io/ByteArrayInputStream");
                        let empty_buf = heap.alloc_array(ArrayData::Byte(Vec::new()));
                        heap.set_field(stream_id, "buf", JavaValue::Object(Some(empty_buf)));
                        heap.set_field(stream_id, "pos", JavaValue::Int(0));
                        heap.set_field(stream_id, "count", JavaValue::Int(0));
                        if method_name == "openDataInputStream" {
                            let dis_id = heap.alloc_instance("java/io/DataInputStream");
                            heap.set_field(dis_id, "in", JavaValue::Object(Some(stream_id)));
                            return Ok(Some(JavaValue::Object(Some(dis_id))));
                        }
                        return Ok(Some(JavaValue::Object(Some(stream_id))));
                    }
                    if method_name == "openOutputStream" || method_name == "openDataOutputStream" {
                        let stream_id = heap.alloc_instance("java/io/ByteArrayOutputStream");
                        let empty_buf = heap.alloc_array(ArrayData::Byte(Vec::new()));
                        heap.set_field(stream_id, "buf", JavaValue::Object(Some(empty_buf)));
                        heap.set_field(stream_id, "count", JavaValue::Int(0));
                        if method_name == "openDataOutputStream" {
                            let dos_id = heap.alloc_instance("java/io/DataOutputStream");
                            heap.set_field(dos_id, "out", JavaValue::Object(Some(stream_id)));
                            return Ok(Some(JavaValue::Object(Some(dos_id))));
                        }
                        return Ok(Some(JavaValue::Object(Some(stream_id))));
                    }

                    let conn_cls = if url.starts_with("sms://") {
                        "javax/wireless/messaging/MessageConnection"
                    } else if url.starts_with("btspp://") {
                        "javax/bluetooth/StreamConnection"
                    } else if url.starts_with("socket://") {
                        "javax/microedition/io/SocketConnection"
                    } else {
                        "javax/microedition/io/HttpConnection"
                    };
                    let conn_id = heap.alloc_instance(conn_cls);
                    return Ok(Some(JavaValue::Object(Some(conn_id))));
                }
                _ => {}
            }
        }

        // 26c. javax/bluetooth/* (JSR-82) Stubs
        if cls.starts_with("javax/bluetooth/") {
            match cls.as_str() {
                "javax/bluetooth/LocalDevice" => match method_name {
                    "getLocalDevice" => {
                        let ld_id = heap.alloc_instance("javax/bluetooth/LocalDevice");
                        return Ok(Some(JavaValue::Object(Some(ld_id))));
                    }
                    "getDiscoveryAgent" => {
                        let da_id = heap.alloc_instance("javax/bluetooth/DiscoveryAgent");
                        return Ok(Some(JavaValue::Object(Some(da_id))));
                    }
                    "getBluetoothAddress" => {
                        let addr = heap.alloc_string("001122334455");
                        return Ok(Some(JavaValue::Object(Some(addr))));
                    }
                    "getFriendlyName" => {
                        let name = heap.alloc_string("RustJ2ME");
                        return Ok(Some(JavaValue::Object(Some(name))));
                    }
                    "getProperty" => {
                        let val = heap.alloc_string("1.1");
                        return Ok(Some(JavaValue::Object(Some(val))));
                    }
                    _ => {}
                },
                "javax/bluetooth/DiscoveryAgent" => match method_name {
                    "startInquiry" => return Ok(Some(JavaValue::Int(0))),
                    "cancelInquiry" => return Ok(Some(JavaValue::Int(1))),
                    "searchServices" => return Ok(Some(JavaValue::Int(1))),
                    "cancelServiceSearch" => return Ok(Some(JavaValue::Int(1))),
                    "selectService" => {
                        let s = heap.alloc_string("btspp://localhost:1");
                        return Ok(Some(JavaValue::Object(Some(s))));
                    }
                    _ => {}
                },
                "javax/bluetooth/RemoteDevice" => match method_name {
                    "getRemoteDevice" => {
                        let rd_id = heap.alloc_instance("javax/bluetooth/RemoteDevice");
                        return Ok(Some(JavaValue::Object(Some(rd_id))));
                    }
                    "getBluetoothAddress" => {
                        let addr = heap.alloc_string("001122334466");
                        return Ok(Some(JavaValue::Object(Some(addr))));
                    }
                    "getFriendlyName" => {
                        let name = heap.alloc_string("RemotePeer");
                        return Ok(Some(JavaValue::Object(Some(name))));
                    }
                    "isAuthenticated" | "isAuthorized" | "isEncrypted" => {
                        return Ok(Some(JavaValue::Int(1)));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // 26d. javax/wireless/messaging/* (WMA) Stubs
        if cls.starts_with("javax/wireless/messaging/") {
            match cls.as_str() {
                "javax/wireless/messaging/MessageConnection" => match method_name {
                    "newMessage" => {
                        let msg_id = heap.alloc_instance("javax/wireless/messaging/TextMessage");
                        return Ok(Some(JavaValue::Object(Some(msg_id))));
                    }
                    "send" | "close" => return Ok(None),
                    "receive" => {
                        let msg_id = heap.alloc_instance("javax/wireless/messaging/TextMessage");
                        return Ok(Some(JavaValue::Object(Some(msg_id))));
                    }
                    _ => {}
                },
                "javax/wireless/messaging/TextMessage" => match method_name {
                    "setPayloadText" | "setAddress" => return Ok(None),
                    "getPayloadText" => {
                        let s = heap.alloc_string("");
                        return Ok(Some(JavaValue::Object(Some(s))));
                    }
                    "getAddress" => {
                        let s = heap.alloc_string("sms://+10000000000");
                        return Ok(Some(JavaValue::Object(Some(s))));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // 26e. Vendor Extensions: Siemens & Motorola Multimedia Stubs
        if cls.starts_with("com/siemens/mp/") || cls.starts_with("com/motorola/multimedia/") {
            match method_name {
                "play" | "vibrate" | "setLightOn" | "setColor" | "startVibrate" | "stopVibrate" => {
                    return Ok(None);
                }
                "getMelody" => {
                    let m_id = heap.alloc_instance("com/siemens/mp/game/Melody");
                    return Ok(Some(JavaValue::Object(Some(m_id))));
                }
                _ => {}
            }
        }

        // 27. M3G (JSR 184) & Mascot Capsule Micro3D Engine Stubs
        if cls == "javax/microedition/m3g/Background" {
            let r = match receiver {
                Some(r) => r,
                None => {
                    let exc = heap.alloc_instance("java/lang/NullPointerException");
                    return Err(format!("EXC:java/lang/NullPointerException:{}", exc.0));
                }
            };
            match method_name {
                "<init>" => {
                    heap.set_field(r, "colorClearEnable", JavaValue::Int(1));
                    heap.set_field(r, "depthClearEnable", JavaValue::Int(1));
                    heap.set_field(r, "color", JavaValue::Int(0));
                    return Ok(None);
                }
                "setColorClearEnable" => {
                    let enable = args.first().and_then(|v| v.as_int().ok()).unwrap_or(1);
                    heap.set_field(
                        r,
                        "colorClearEnable",
                        JavaValue::Int(if enable != 0 { 1 } else { 0 }),
                    );
                    return Ok(None);
                }
                "isColorClearEnable" => {
                    let val = heap.get_field(r, "colorClearEnable").as_int().unwrap_or(1);
                    return Ok(Some(JavaValue::Int(val)));
                }
                "setDepthClearEnable" => {
                    let enable = args.first().and_then(|v| v.as_int().ok()).unwrap_or(1);
                    heap.set_field(
                        r,
                        "depthClearEnable",
                        JavaValue::Int(if enable != 0 { 1 } else { 0 }),
                    );
                    return Ok(None);
                }
                "isDepthClearEnable" => {
                    let val = heap.get_field(r, "depthClearEnable").as_int().unwrap_or(1);
                    return Ok(Some(JavaValue::Int(val)));
                }
                "setColor" => {
                    let col = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    heap.set_field(r, "color", JavaValue::Int(col));
                    return Ok(None);
                }
                "getColor" => {
                    let col = heap.get_field(r, "color").as_int().unwrap_or(0);
                    return Ok(Some(JavaValue::Int(col)));
                }
                _ => {}
            }
        }

        if cls == "javax/microedition/m3g/Graphics3D" {
            match method_name {
                "getInstance" => {
                    let inst = heap.alloc_instance("javax/microedition/m3g/Graphics3D");
                    return Ok(Some(JavaValue::Object(Some(inst))));
                }
                "bindTarget" | "releaseTarget" | "render" | "setViewport" | "setCamera" => {
                    return Ok(None);
                }
                "clear" => {
                    if let Some(bg_obj) = args.first().and_then(|v| v.as_object().ok().flatten()) {
                        let color_enabled = heap
                            .get_field(bg_obj, "colorClearEnable")
                            .as_int()
                            .unwrap_or(1)
                            != 0;
                        if color_enabled {
                            let argb = heap.get_field(bg_obj, "color").as_int().unwrap_or(0) as u32;
                            if argb != 0 {
                                context.framebuffer.fill(argb);
                                context.screen_dirty = true;
                            }
                        }
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        if cls == "javax/microedition/m3g/Loader" {
            match method_name {
                "load" => {
                    // JSR-184: returns Object3D[] where the root is a World instance
                    let world_id = heap.alloc_instance("javax/microedition/m3g/World");
                    let arr_id = heap.alloc_array(ArrayData::Reference(vec![Some(world_id)]));
                    return Ok(Some(JavaValue::Object(Some(arr_id))));
                }
                _ => {}
            }
        }

        if cls == "javax/microedition/m3g/Group"
            || cls == "javax/microedition/m3g/World"
            || class_loader.is_instance_of(&cls, "javax/microedition/m3g/Group")
            || class_loader.is_instance_of(&cls, "javax/microedition/m3g/World")
        {
            let parent_id = receiver.unwrap_or(ObjectId(0));
            match method_name {
                "<init>" => return Ok(None),
                "addChild" => {
                    let child_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    if let Some(child_id) = child_opt {
                        let list = context.m3g_children.entry(parent_id).or_default();
                        if !list.contains(&child_id) {
                            list.push(child_id);
                        }
                    }
                    return Ok(None);
                }
                "removeChild" => {
                    let child_opt = args.first().and_then(|v| v.as_object().ok()).flatten();
                    if let Some(child_id) = child_opt {
                        if let Some(list) = context.m3g_children.get_mut(&parent_id) {
                            list.retain(|&c| c != child_id);
                        }
                    }
                    return Ok(None);
                }
                "getChildCount" => {
                    let count = context
                        .m3g_children
                        .get(&parent_id)
                        .map(|v| v.len() as i32)
                        .unwrap_or(0);
                    return Ok(Some(JavaValue::Int(count)));
                }
                "getChild" => {
                    let idx = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let list = context.m3g_children.entry(parent_id).or_default();
                    if idx >= 0 && (idx as usize) < list.len() {
                        return Ok(Some(JavaValue::Object(Some(list[idx as usize]))));
                    }
                    // Fallback for scenes loaded from Loader.load or empty World/Group:
                    // Allocate a valid stub Node, register it in m3g_children up to idx so subsequent
                    // calls return the same object.
                    let target_idx = idx.max(0) as usize;
                    while list.len() <= target_idx {
                        let stub_node = heap.alloc_instance("javax/microedition/m3g/Node");
                        list.push(stub_node);
                    }
                    return Ok(Some(JavaValue::Object(Some(list[target_idx]))));
                }
                "getActiveCamera" => {
                    let cam_id = heap.alloc_instance("javax/microedition/m3g/Camera");
                    return Ok(Some(JavaValue::Object(Some(cam_id))));
                }
                "setActiveCamera" => return Ok(None),
                "getBackground" => {
                    let bg_id = heap.alloc_instance("javax/microedition/m3g/Background");
                    return Ok(Some(JavaValue::Object(Some(bg_id))));
                }
                "setBackground" => return Ok(None),
                _ => {}
            }
        }

        if cls == "javax/microedition/m3g/Node"
            || class_loader.is_instance_of(&cls, "javax/microedition/m3g/Node")
        {
            match method_name {
                "<init>" => return Ok(None),
                "getParent" => {
                    if let Some(rec) = receiver {
                        for (&parent, children) in &context.m3g_children {
                            if children.contains(&rec) {
                                return Ok(Some(JavaValue::Object(Some(parent))));
                            }
                        }
                    }
                    return Ok(Some(JavaValue::NULL));
                }
                _ => {}
            }
        }

        // Generic M3G & MascotCapsule fallback: strictly respect return_desc for primitives and objects
        if cls.contains("micro3d") || cls.contains("m3g") {
            let return_desc = if let Some(idx) = descriptor.rfind(')') {
                &descriptor[idx + 1..]
            } else {
                descriptor
            };
            if return_desc == "V" {
                return Ok(None);
            }
            if return_desc == "Z" {
                return Ok(Some(JavaValue::Int(1)));
            }
            if return_desc == "F" {
                return Ok(Some(JavaValue::Float(0.0)));
            }
            if return_desc == "D" {
                return Ok(Some(JavaValue::Double(0.0)));
            }
            if return_desc == "J" {
                return Ok(Some(JavaValue::Long(0)));
            }
            if return_desc == "I" || return_desc == "S" || return_desc == "B" || return_desc == "C"
            {
                return Ok(Some(JavaValue::Int(0)));
            }
            if return_desc.starts_with('L') && return_desc.ends_with(';') {
                let target_cls = &return_desc[1..return_desc.len() - 1];
                let stub_id = heap.alloc_instance(target_cls);
                return Ok(Some(JavaValue::Object(Some(stub_id))));
            }
        }

        // Universal Stub Fallback (Siemens / Motorola / Nokia / MIDP 2.1 / Vendor extensions)
        context
            .unhandled_natives
            .insert(format!("{}.{}{}", cls, method_name, descriptor));
        context.unhandled_native_calls += 1;
        if std::env::var_os("J2ME_DEBUG_NATIVE").is_some() {
            println!(
                "[UNHANDLED NATIVE] {}.{}{} -> universal stub fallback",
                cls, method_name, descriptor
            );
        }
        let return_desc = if let Some(idx) = descriptor.rfind(')') {
            &descriptor[idx + 1..]
        } else {
            descriptor
        };

        if return_desc == "V" || method_name == "<init>" {
            return Ok(None);
        }
        if return_desc == "Z" {
            let val = if method_name.starts_with("isSupported")
                || method_name.starts_with("isAvailable")
                || method_name.starts_with("can")
                || method_name.starts_with("has")
            {
                1
            } else {
                0
            };
            return Ok(Some(JavaValue::Int(val)));
        }
        if return_desc == "I" || return_desc == "S" || return_desc == "B" || return_desc == "C" {
            return Ok(Some(JavaValue::Int(0)));
        }
        if return_desc == "J" {
            return Ok(Some(JavaValue::Long(0)));
        }
        if return_desc == "F" {
            return Ok(Some(JavaValue::Float(0.0)));
        }
        if return_desc == "D" {
            return Ok(Some(JavaValue::Double(0.0)));
        }
        if return_desc.starts_with('[') {
            let arr_id = match return_desc {
                "[Z" => heap.alloc_array(ArrayData::Boolean(Vec::new())),
                "[B" => heap.alloc_array(ArrayData::Byte(Vec::new())),
                "[C" => heap.alloc_array(ArrayData::Char(Vec::new())),
                "[S" => heap.alloc_array(ArrayData::Short(Vec::new())),
                "[I" => heap.alloc_array(ArrayData::Int(Vec::new())),
                "[J" => heap.alloc_array(ArrayData::Long(Vec::new())),
                "[F" => heap.alloc_array(ArrayData::Float(Vec::new())),
                "[D" => heap.alloc_array(ArrayData::Double(Vec::new())),
                _ => heap.alloc_array(ArrayData::Reference(Vec::new())),
            };
            return Ok(Some(JavaValue::Object(Some(arr_id))));
        }
        if return_desc.starts_with('L') && return_desc.ends_with(';') {
            let target_cls = &return_desc[1..return_desc.len() - 1];
            if method_name.starts_with("get") || method_name.starts_with("create") || method_name == "instance" {
                let stub_id = heap.alloc_instance(target_cls);
                return Ok(Some(JavaValue::Object(Some(stub_id))));
            }
            return Ok(Some(JavaValue::Object(None)));
        }

        Ok(Some(JavaValue::default_for_descriptor(return_desc)))
    }

    fn handle_arraycopy(args: &[JavaValue], heap: &mut Heap) -> Result<(), ArrayCopyError> {
        if args.len() < 5 {
            return Err(ArrayCopyError::ArrayStore);
        }
        let src_id = match args[0]
            .as_object()
            .map_err(|_| ArrayCopyError::ArrayStore)?
        {
            Some(id) => id,
            None => return Err(ArrayCopyError::NullPointer),
        };
        let src_pos_raw = args[1].as_int().map_err(|_| ArrayCopyError::ArrayStore)?;
        let dst_id = match args[2]
            .as_object()
            .map_err(|_| ArrayCopyError::ArrayStore)?
        {
            Some(id) => id,
            None => return Err(ArrayCopyError::NullPointer),
        };
        let dst_pos_raw = args[3].as_int().map_err(|_| ArrayCopyError::ArrayStore)?;
        let len_raw = args[4].as_int().map_err(|_| ArrayCopyError::ArrayStore)?;

        if src_pos_raw < 0 || dst_pos_raw < 0 || len_raw < 0 {
            return Err(ArrayCopyError::IndexOutOfBounds);
        }

        let src_pos = src_pos_raw as usize;
        let dst_pos = dst_pos_raw as usize;
        let len = len_raw as usize;

        let src_data = match heap.get(src_id) {
            Some(ObjectData::Array(arr)) => arr.clone(),
            _ => return Err(ArrayCopyError::ArrayStore),
        };

        let dst_obj = heap.get_mut(dst_id).ok_or(ArrayCopyError::NullPointer)?;
        let dst_arr = dst_obj.as_array_mut().ok_or(ArrayCopyError::ArrayStore)?;

        let src_len = match &src_data {
            ArrayData::Byte(s) => s.len(),
            ArrayData::Int(s) => s.len(),
            ArrayData::Short(s) => s.len(),
            ArrayData::Char(s) => s.len(),
            ArrayData::Boolean(s) => s.len(),
            ArrayData::Long(s) => s.len(),
            ArrayData::Float(s) => s.len(),
            ArrayData::Double(s) => s.len(),
            ArrayData::Reference(s) => s.len(),
        };

        let dst_len = match &dst_arr {
            ArrayData::Byte(d) => d.len(),
            ArrayData::Int(d) => d.len(),
            ArrayData::Short(d) => d.len(),
            ArrayData::Char(d) => d.len(),
            ArrayData::Boolean(d) => d.len(),
            ArrayData::Long(d) => d.len(),
            ArrayData::Float(d) => d.len(),
            ArrayData::Double(d) => d.len(),
            ArrayData::Reference(d) => d.len(),
        };

        if src_pos.saturating_add(len) > src_len || dst_pos.saturating_add(len) > dst_len {
            return Err(ArrayCopyError::IndexOutOfBounds);
        }

        if len == 0 {
            let types_match = matches!(
                (&src_data, &dst_arr),
                (ArrayData::Byte(_), ArrayData::Byte(_))
                    | (ArrayData::Int(_), ArrayData::Int(_))
                    | (ArrayData::Short(_), ArrayData::Short(_))
                    | (ArrayData::Char(_), ArrayData::Char(_))
                    | (ArrayData::Boolean(_), ArrayData::Boolean(_))
                    | (ArrayData::Long(_), ArrayData::Long(_))
                    | (ArrayData::Float(_), ArrayData::Float(_))
                    | (ArrayData::Double(_), ArrayData::Double(_))
                    | (ArrayData::Reference(_), ArrayData::Reference(_))
            );
            if !types_match {
                return Err(ArrayCopyError::ArrayStore);
            }
            return Ok(());
        }

        match (src_data, dst_arr) {
            (ArrayData::Byte(s), ArrayData::Byte(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Int(s), ArrayData::Int(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Short(s), ArrayData::Short(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Char(s), ArrayData::Char(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Boolean(s), ArrayData::Boolean(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Long(s), ArrayData::Long(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Float(s), ArrayData::Float(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Double(s), ArrayData::Double(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            (ArrayData::Reference(s), ArrayData::Reference(d)) => {
                d[dst_pos..dst_pos + len].copy_from_slice(&s[src_pos..src_pos + len]);
            }
            _ => return Err(ArrayCopyError::ArrayStore),
        }

        Ok(())
    }

    fn get_rms_name(receiver: Option<ObjectId>, heap: &Heap) -> String {
        receiver
            .and_then(|r| heap.get_field(r, "name").as_object().ok().flatten())
            .and_then(|id| heap.get_string_utf8(id))
            .unwrap_or_else(|| "default".to_string())
    }

    fn decode_modified_utf8(bytes: &[u8]) -> Result<String, ()> {
        let mut chars = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            let b1 = bytes[i];
            if b1 & 0x80 == 0 {
                // 1-byte format (0..127): standard ASCII
                chars.push(b1 as u16);
                i += 1;
            } else if (b1 & 0xE0) == 0xC0 {
                // 2-byte format: 110xxxxx 10xxxxxx (including null character 0xC0 0x80)
                if i + 1 >= bytes.len() {
                    return Err(());
                }
                let b2 = bytes[i + 1];
                if (b2 & 0xC0) != 0x80 {
                    return Err(());
                }
                let ch = (((b1 & 0x1F) as u16) << 6) | ((b2 & 0x3F) as u16);
                chars.push(ch);
                i += 2;
            } else if (b1 & 0xF0) == 0xE0 {
                // 3-byte format: 1110xxxx 10xxxxxx 10xxxxxx
                if i + 2 >= bytes.len() {
                    return Err(());
                }
                let b2 = bytes[i + 1];
                let b3 = bytes[i + 2];
                if (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80 {
                    return Err(());
                }
                let ch = (((b1 & 0x0F) as u16) << 12)
                    | (((b2 & 0x3F) as u16) << 6)
                    | ((b3 & 0x3F) as u16);
                chars.push(ch);
                i += 3;
            } else {
                return Err(());
            }
        }
        String::from_utf16(&chars).map_err(|_| ())
    }

    pub fn resolve_stream_source(heap: &Heap, mut stream_id: ObjectId) -> ObjectId {
        let mut depth = 0;
        while depth < 16 {
            if let Some(buf_id) = heap.get_field(stream_id, "buf").as_object().ok().flatten() {
                if heap.get(buf_id).map(|o| o.as_array().is_some()).unwrap_or(false) {
                    return stream_id;
                }
            }
            if let Some(inner) = heap.get_field(stream_id, "in").as_object().ok().flatten() {
                stream_id = inner;
                depth += 1;
            } else {
                break;
            }
        }
        stream_id
    }

    fn data_stream_read_u8(heap: &mut Heap, dis_id: Option<ObjectId>) -> Result<u8, String> {
        let r = match dis_id {
            Some(id) => id,
            None => {
                let exc_id = heap.alloc_instance("java/lang/NullPointerException");
                return Err(format!("EXC:java/lang/NullPointerException:{}", exc_id.0));
            }
        };
        let inner_id = match heap.get_field(r, "in").as_object().ok().flatten() {
            Some(id) => id,
            None => {
                let exc_id = heap.alloc_instance("java/lang/NullPointerException");
                return Err(format!("EXC:java/lang/NullPointerException:{}", exc_id.0));
            }
        };
        let in_stream = Self::resolve_stream_source(heap, inner_id);
        let buf_id = match heap.get_field(in_stream, "buf").as_object().ok().flatten() {
            Some(id) => id,
            None => {
                let exc_id = heap.alloc_instance("java/io/IOException");
                return Err(format!("EXC:java/io/IOException:{}", exc_id.0));
            }
        };
        let pos = heap.get_field(in_stream, "pos").as_int().unwrap_or(0);
        let count = heap.get_field(in_stream, "count").as_int().unwrap_or(0);

        if pos >= count {
            let exc_id = heap.alloc_instance("java/io/EOFException");
            return Err(format!("EXC:java/io/EOFException:{}", exc_id.0));
        }

        if let Some(ObjectData::Array(ArrayData::Byte(b))) = heap.get(buf_id) {
            let val = b[pos as usize] as u8;
            heap.set_field(in_stream, "pos", JavaValue::Int(pos + 1));
            return Ok(val);
        }

        let exc_id = heap.alloc_instance("java/io/IOException");
        Err(format!("EXC:java/io/IOException:{}", exc_id.0))
    }

    fn stream_read_bytes(
        heap: &mut Heap,
        stream_id: ObjectId,
        dst_arr_id: ObjectId,
        off: usize,
        len: usize,
    ) -> Result<usize, String> {
        let stream_id = Self::resolve_stream_source(heap, stream_id);
        let buf_id = match heap.get_field(stream_id, "buf").as_object().ok().flatten() {
            Some(id) => id,
            None => {
                let exc_id = heap.alloc_instance("java/io/IOException");
                return Err(format!("EXC:java/io/IOException:{}", exc_id.0));
            }
        };
        let pos = heap.get_field(stream_id, "pos").as_int().unwrap_or(0) as usize;
        let count = heap.get_field(stream_id, "count").as_int().unwrap_or(0) as usize;

        if pos >= count && len > 0 {
            let exc_id = heap.alloc_instance("java/io/EOFException");
            return Err(format!("EXC:java/io/EOFException:{}", exc_id.0));
        }

        let avail = count.saturating_sub(pos);
        let to_copy = len.min(avail);
        if to_copy == 0 {
            return Ok(0);
        }

        let slice = match heap.get(buf_id) {
            Some(ObjectData::Array(ArrayData::Byte(b))) => {
                if pos + to_copy <= b.len() {
                    b[pos..pos + to_copy].to_vec()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        if let Some(ObjectData::Array(ArrayData::Byte(dst))) = heap.get_mut(dst_arr_id) {
            if off + to_copy <= dst.len() {
                dst[off..off + to_copy].copy_from_slice(&slice);
            }
        }
        heap.set_field(stream_id, "pos", JavaValue::Int((pos + to_copy) as i32));
        Ok(to_copy)
    }

    fn output_stream_write_bytes(heap: &mut Heap, receiver: Option<ObjectId>, bytes: &[i8]) {
        if let Some(r_id) = receiver {
            let out_id = heap.get_field(r_id, "out").as_object().ok().flatten();
            if let Some(oid) = out_id {
                Self::output_stream_write_bytes(heap, Some(oid), bytes);
            } else {
                let buf_id = heap.get_field(r_id, "buf").as_object().ok().flatten();
                if let Some(bid) = buf_id {
                    if let Some(ObjectData::Array(ArrayData::Byte(buf))) = heap.get_mut(bid) {
                        buf.extend_from_slice(bytes);
                    }
                }
            }
        }
    }

    fn output_stream_write_byte(heap: &mut Heap, receiver: Option<ObjectId>, b: i8) {
        Self::output_stream_write_bytes(heap, receiver, &[b]);
    }

    fn decode_string_bytes(bytes: &[u8], enc: Option<&str>) -> String {
        let is_cp1251 = enc
            .map(|e| e.eq_ignore_ascii_case("windows-1251") || e.eq_ignore_ascii_case("cp1251"))
            .unwrap_or(false);
        if is_cp1251 {
            let mut s = String::with_capacity(bytes.len());
            for &b in bytes {
                if b < 0x80 {
                    s.push(b as char);
                } else if b == 0xA8 {
                    s.push('Ё');
                } else if b == 0xB8 {
                    s.push('ё');
                } else if b >= 0xC0 {
                    // 0xC0..=0xFF map to 0x0410..=0x044F
                    if let Some(c) = char::from_u32(0x0410 + (b - 0xC0) as u32) {
                        s.push(c);
                    } else {
                        s.push('?');
                    }
                } else {
                    s.push('?');
                }
            }
            s
        } else {
            String::from_utf8_lossy(bytes).to_string()
        }
    }

    fn encode_string_bytes(s: &str, enc: Option<&str>) -> Vec<u8> {
        let is_cp1251 = enc
            .map(|e| e.eq_ignore_ascii_case("windows-1251") || e.eq_ignore_ascii_case("cp1251"))
            .unwrap_or(false);
        if is_cp1251 {
            let mut bytes = Vec::with_capacity(s.len());
            for c in s.chars() {
                let u = c as u32;
                if u < 0x80 {
                    bytes.push(u as u8);
                } else if c == 'Ё' {
                    bytes.push(0xA8);
                } else if c == 'ё' {
                    bytes.push(0xB8);
                } else if (0x0410..=0x044F).contains(&u) {
                    bytes.push((0xC0 + (u - 0x0410)) as u8);
                } else {
                    bytes.push(b'?');
                }
            }
            bytes
        } else {
            s.as_bytes().to_vec()
        }
    }

    pub fn java_objects_equal(heap: &Heap, a: Option<ObjectId>, b: Option<ObjectId>) -> bool {
        if a == b {
            return true;
        }
        match (a, b) {
            (Some(id_a), Some(id_b)) => {
                let str_a = heap.get_string_utf8(id_a);
                let str_b = heap.get_string_utf8(id_b);
                if str_a.is_some() || str_b.is_some() {
                    return str_a.is_some() && str_a == str_b;
                }

                let cls_a = heap
                    .get(id_a)
                    .and_then(|o| o.as_instance())
                    .map(|i| i.class_name.clone());
                let cls_b = heap
                    .get(id_b)
                    .and_then(|o| o.as_instance())
                    .map(|i| i.class_name.clone());
                match (cls_a, cls_b) {
                    (Some(ca), Some(cb)) if ca == cb => {
                        if matches!(
                            ca.as_str(),
                            "java/lang/Integer"
                                | "java/lang/Byte"
                                | "java/lang/Short"
                                | "java/lang/Character"
                                | "java/lang/Boolean"
                        ) {
                            let va = heap.get_field(id_a, "value").as_int().ok();
                            let vb = heap.get_field(id_b, "value").as_int().ok();
                            va.is_some() && va == vb
                        } else if ca == "java/lang/Long" {
                            let va = heap.get_field(id_a, "value").as_long().ok();
                            let vb = heap.get_field(id_b, "value").as_long().ok();
                            va.is_some() && va == vb
                        } else if ca == "java/lang/Double" {
                            let va = heap.get_field(id_a, "value").as_double().ok();
                            let vb = heap.get_field(id_b, "value").as_double().ok();
                            va.is_some() && va.map(|v| v.to_bits()) == vb.map(|v| v.to_bits())
                        } else if ca == "java/lang/Float" {
                            let va = heap.get_field(id_a, "value").as_float().ok();
                            let vb = heap.get_field(id_b, "value").as_float().ok();
                            va.is_some() && va.map(|v| v.to_bits()) == vb.map(|v| v.to_bits())
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod vm_tests {
    use super::*;

    #[test]
    fn arraycopy_null_src_or_dst() {
        let mut heap = Heap::new();
        let arr = heap.alloc(ObjectData::Array(ArrayData::Int(vec![0; 10])));
        // null src
        let err1 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(None),
                JavaValue::Int(0),
                JavaValue::Object(Some(arr)),
                JavaValue::Int(0),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(err1, Err(ArrayCopyError::NullPointer));

        // null dst
        let err2 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(arr)),
                JavaValue::Int(0),
                JavaValue::Object(None),
                JavaValue::Int(0),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(err2, Err(ArrayCopyError::NullPointer));
    }

    #[test]
    fn arraycopy_bounds_and_negative_indices() {
        let mut heap = Heap::new();
        let src = heap.alloc(ObjectData::Array(ArrayData::Int(vec![0; 10])));
        let dst = heap.alloc(ObjectData::Array(ArrayData::Int(vec![0; 10])));

        // Negative src_pos
        let e1 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(src)),
                JavaValue::Int(-1),
                JavaValue::Object(Some(dst)),
                JavaValue::Int(0),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(e1, Err(ArrayCopyError::IndexOutOfBounds));

        // Negative dst_pos
        let e2 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(src)),
                JavaValue::Int(0),
                JavaValue::Object(Some(dst)),
                JavaValue::Int(-1),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(e2, Err(ArrayCopyError::IndexOutOfBounds));

        // Negative length
        let e3 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(src)),
                JavaValue::Int(0),
                JavaValue::Object(Some(dst)),
                JavaValue::Int(0),
                JavaValue::Int(-5),
            ],
            &mut heap,
        );
        assert_eq!(e3, Err(ArrayCopyError::IndexOutOfBounds));

        // Out of bounds (srcPos + len > src.length)
        let e4 = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(src)),
                JavaValue::Int(6),
                JavaValue::Object(Some(dst)),
                JavaValue::Int(0),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(e4, Err(ArrayCopyError::IndexOutOfBounds));
    }

    #[test]
    fn arraycopy_type_mismatch() {
        let mut heap = Heap::new();
        let src_int = heap.alloc(ObjectData::Array(ArrayData::Int(vec![0; 10])));
        let dst_byte = heap.alloc(ObjectData::Array(ArrayData::Byte(vec![0; 10])));

        let err = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(src_int)),
                JavaValue::Int(0),
                JavaValue::Object(Some(dst_byte)),
                JavaValue::Int(0),
                JavaValue::Int(5),
            ],
            &mut heap,
        );
        assert_eq!(err, Err(ArrayCopyError::ArrayStore));
    }

    #[test]
    fn arraycopy_overlapping_ranges() {
        let mut heap = Heap::new();
        let arr = heap.alloc(ObjectData::Array(ArrayData::Int(vec![10, 20, 30, 40, 50])));

        // Copy arr[0..3] to arr[1..4] -> should become [10, 10, 20, 30, 50]
        let res = NativeBridge::handle_arraycopy(
            &[
                JavaValue::Object(Some(arr)),
                JavaValue::Int(0),
                JavaValue::Object(Some(arr)),
                JavaValue::Int(1),
                JavaValue::Int(3),
            ],
            &mut heap,
        );
        assert_eq!(res, Ok(()));

        if let Some(ObjectData::Array(ArrayData::Int(ref s))) = heap.get(arr) {
            assert_eq!(s, &[10, 10, 20, 30, 50]);
        }
    }

    #[test]
    fn thread_sleep_semantics() {
        let mut heap = Heap::new();
        let mut class_loader = ClassLoader::new();
        let mut scheduler = Scheduler::new(5000);
        let mut context = VmContext::new(240, 320, 44100);

        // Dummy thread
        let frame = Frame::new(
            "Test".to_string(),
            "main".to_string(),
            "()V".to_string(),
            Arc::new(vec![]),
            2,
            2,
            vec![],
            Arc::new(std::collections::HashMap::new()),
        );
        let th_id = scheduler.spawn("test_th", frame, None);
        let th_idx = scheduler
            .threads
            .iter()
            .position(|t| t.id == th_id)
            .unwrap();

        // 1. Negative ms throws IllegalArgumentException
        let res_neg = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(-50)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            th_idx,
        );
        assert!(res_neg.is_err());
        assert!(res_neg.unwrap_err().contains("IllegalArgumentException"));

        // 2. sleep(0) yields quantum without blocking
        let res_zero = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(0)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            th_idx,
        );
        assert_eq!(res_zero, Ok(None));
        assert_eq!(scheduler.threads[th_idx].state, ThreadState::Ready);

        // 3. Positive sleep enters Sleeping state with wake_time
        scheduler.current_time_ms = 1000;
        let res_pos = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(250)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            th_idx,
        );
        assert_eq!(res_pos, Ok(None));
        assert_eq!(
            scheduler.threads[th_idx].state,
            ThreadState::Sleeping { wake_time_ms: 1250 }
        );
    }

    #[test]
    fn scheduler_sleep_quantum_and_multiple_threads_same_wake_time() {
        let mut scheduler = Scheduler::new(1000);

        let dummy_frame = || {
            Frame::new(
                "Test".to_string(),
                "main".to_string(),
                "()V".to_string(),
                Arc::new(vec![]),
                1,
                1,
                vec![],
                Arc::new(std::collections::HashMap::new()),
            )
        };

        let t1 = scheduler.spawn("th1", dummy_frame(), None);
        let t2 = scheduler.spawn("th2", dummy_frame(), None);

        // Put both to sleep with wake_time 100
        scheduler.current_time_ms = 80;
        scheduler.threads[0].state = ThreadState::Sleeping { wake_time_ms: 100 };
        scheduler.threads[1].state = ThreadState::Sleeping { wake_time_ms: 100 };

        // At t=96 (16ms tick), neither wakes up
        scheduler.tick_time(16);
        assert_eq!(scheduler.current_time_ms, 96);
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { .. }
        ));
        assert!(matches!(
            scheduler.threads[1].state,
            ThreadState::Sleeping { .. }
        ));
        assert_eq!(scheduler.next_ready_thread_index(), None);

        // At t=112 (another 16ms tick), both wake up and become Ready
        scheduler.tick_time(16);
        assert_eq!(scheduler.current_time_ms, 112);
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
        assert_eq!(scheduler.threads[1].state, ThreadState::Ready);

        // Round-robin picks t1 then t2 without starvation
        let idx1 = scheduler.next_ready_thread_index().unwrap();
        assert_eq!(scheduler.threads[idx1].id, t1);
        scheduler.threads[idx1].state = ThreadState::Ready;
        let idx2 = scheduler.next_ready_thread_index().unwrap();
        assert_eq!(scheduler.threads[idx2].id, t2);
    }

    #[test]
    fn m3g_background_set_color_clear_enable_and_color_state() {
        let mut heap = Heap::new();
        let mut class_loader = ClassLoader::new();
        let mut scheduler = Scheduler::new(1000);
        let mut context = crate::bridge::context::VmContext::new(240, 320, 44100);

        let bg_id = heap.alloc_instance("javax/microedition/m3g/Background");

        // <init>()V -> void (returns Ok(None))
        let res_init = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "<init>",
            "()V",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_init.unwrap(), None);

        // Initial state is true
        let res_is_clear = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "isColorClearEnable",
            "()Z",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_is_clear.unwrap(), Some(JavaValue::Int(1)));

        // setColorClearEnable(false) -> void (returns Ok(None))
        let res_set_false = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "setColorClearEnable",
            "(Z)V",
            Some(bg_id),
            &[JavaValue::Int(0)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_set_false.unwrap(), None);

        // Verified changed state to 0
        let res_is_clear2 = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "isColorClearEnable",
            "()Z",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_is_clear2.unwrap(), Some(JavaValue::Int(0)));

        // setColor(0x00FF00AA) -> void
        let res_set_col = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "setColor",
            "(I)V",
            Some(bg_id),
            &[JavaValue::Int(0x00FF00AA)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_set_col.unwrap(), None);

        let res_get_col = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "getColor",
            "()I",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_get_col.unwrap(), Some(JavaValue::Int(0x00FF00AA)));

        // setColorClearEnable(true)
        let res_set_true = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "setColorClearEnable",
            "(Z)V",
            Some(bg_id),
            &[JavaValue::Int(1)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_set_true.unwrap(), None);

        let res_is_clear3 = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "isColorClearEnable",
            "()Z",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_is_clear3.unwrap(), Some(JavaValue::Int(1)));

        // Depth clear enable testing
        let res_set_depth_f = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "setDepthClearEnable",
            "(Z)V",
            Some(bg_id),
            &[JavaValue::Int(0)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_set_depth_f.unwrap(), None);

        let res_is_depth = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "isDepthClearEnable",
            "()Z",
            Some(bg_id),
            &[],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res_is_depth.unwrap(), Some(JavaValue::Int(0)));

        // Call without receiver must fail gracefully (NPE)
        let res_null_recv = NativeBridge::dispatch(
            "javax/microedition/m3g/Background",
            "setColorClearEnable",
            "(Z)V",
            None,
            &[JavaValue::Int(1)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(res_null_recv.is_err());
    }

    #[test]
    fn scheduler_sleep_boundary_0_1_16_17() {
        let mut heap = Heap::new();
        let mut class_loader = ClassLoader::new();
        let mut scheduler = Scheduler::new(1000);
        let mut context = crate::bridge::context::VmContext::new(240, 320, 44100);

        let dummy_frame = || {
            Frame::new(
                "Test".to_string(),
                "main".to_string(),
                "()V".to_string(),
                Arc::new(vec![]),
                1,
                1,
                vec![],
                Arc::new(std::collections::HashMap::new()),
            )
        };
        let _t1 = scheduler.spawn("th1", dummy_frame(), None);

        // 1. sleep(0): acts as yield, thread remains Ready immediately
        let res0 = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(0)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res0, Ok(None));
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);

        // 2. sleep(1): wake_time_ms is 1001
        scheduler.current_time_ms = 1000;
        let res1 = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(1)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res1, Ok(None));
        assert_eq!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { wake_time_ms: 1001 }
        );
        scheduler.tick_time(16); // t = 1016 >= 1001 -> wakes up on 1st tick
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);

        // 3. sleep(16): wake_time_ms is 1016 + 16 = 1032
        let res16 = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(16)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res16, Ok(None));
        assert_eq!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { wake_time_ms: 1032 }
        );
        scheduler.tick_time(16); // t = 1032 >= 1032 -> wakes up exactly at quantum boundary
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);

        // 4. sleep(17): wake_time_ms is 1032 + 17 = 1049
        let res17 = NativeBridge::dispatch(
            "java/lang/Thread",
            "sleep",
            "(J)V",
            None,
            &[JavaValue::Long(17)],
            &mut heap,
            &mut class_loader,
            &mut scheduler,
            &mut context,
            0,
        );
        assert_eq!(res17, Ok(None));
        assert_eq!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { wake_time_ms: 1049 }
        );
        scheduler.tick_time(16); // t = 1048 < 1049 -> stays Sleeping!
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Sleeping { wake_time_ms: 1049 }
        ));
        scheduler.tick_time(16); // t = 1064 >= 1049 -> wakes up on 2nd tick!
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
    }

    #[test]
    fn midp_command_and_softkey_contract() {
        let mut heap = Heap::new();
        let mut context = crate::bridge::context::VmContext::new(240, 320, 44100);

        let disp_id = heap.alloc_instance("javax/microedition/lcdui/Canvas");

        // Command 1: OK (type 4, priority 1) -> SOFT1
        let cmd1 = heap.alloc_instance("javax/microedition/lcdui/Command");
        let ok_str = heap.alloc_string("OK");
        heap.set_field(cmd1, "label", JavaValue::Object(Some(ok_str)));
        heap.set_field(cmd1, "commandType", JavaValue::Int(4));
        heap.set_field(cmd1, "priority", JavaValue::Int(1));

        // Command 2: Exit (type 7, priority 1) -> SOFT2
        let cmd2 = heap.alloc_instance("javax/microedition/lcdui/Command");
        let exit_str = heap.alloc_string("Exit");
        heap.set_field(cmd2, "label", JavaValue::Object(Some(exit_str)));
        heap.set_field(cmd2, "commandType", JavaValue::Int(7));
        heap.set_field(cmd2, "priority", JavaValue::Int(1));

        // Listener
        let listener_id = heap.alloc_instance("TestListener");

        // 1. Add single command
        context.add_command(disp_id, cmd1, &heap);
        let (s1, s2) = context.get_soft_commands(disp_id, &heap);
        assert_eq!(s1, Some(cmd1));
        assert_eq!(s2, None);

        // 2. Add second command
        context.add_command(disp_id, cmd2, &heap);
        let (s1_both, s2_both) = context.get_soft_commands(disp_id, &heap);
        assert_eq!(s1_both, Some(cmd1)); // OK -> SOFT1
        assert_eq!(s2_both, Some(cmd2)); // Exit -> SOFT2

        // 3. Set listener
        context.set_command_listener(disp_id, Some(listener_id));
        assert_eq!(
            context
                .displayable_commands
                .get(&disp_id)
                .and_then(|e| e.listener),
            Some(listener_id)
        );

        // 4. Replace listener
        let listener2_id = heap.alloc_instance("TestListener2");
        context.set_command_listener(disp_id, Some(listener2_id));
        assert_eq!(
            context
                .displayable_commands
                .get(&disp_id)
                .and_then(|e| e.listener),
            Some(listener2_id)
        );

        // 5. Remove Command 1
        context.remove_command(disp_id, cmd1);
        let (s1_after_rem, s2_after_rem) = context.get_soft_commands(disp_id, &heap);
        assert_eq!(s1_after_rem, None);
        assert_eq!(s2_after_rem, Some(cmd2)); // Exit remains on SOFT2

        // 6. Switch Displayable cleans up or has isolated commands
        let disp2_id = heap.alloc_instance("javax/microedition/lcdui/Canvas");
        let (d2_s1, d2_s2) = context.get_soft_commands(disp2_id, &heap);
        assert_eq!(d2_s1, None);
        assert_eq!(d2_s2, None);
    }

    #[test]
    fn test_data_input_stream_read_utf_modified_utf8_and_exceptions() {
        let mut heap = Heap::new();
        let mut cl = ClassLoader::new();
        let mut scheduler = Scheduler::new(5000);
        let mut context = VmContext::new(240, 320, 44100);

        // Helper to construct a DataInputStream wrapping a ByteArrayInputStream with given bytes
        let make_dis = |bytes: &[u8], heap: &mut Heap| -> ObjectId {
            let b_arr = heap.alloc_array(ArrayData::Byte(bytes.iter().map(|&b| b as i8).collect()));
            let bais = heap.alloc_instance("java/io/ByteArrayInputStream");
            heap.set_field(bais, "buf", JavaValue::Object(Some(b_arr)));
            heap.set_field(bais, "pos", JavaValue::Int(0));
            heap.set_field(bais, "count", JavaValue::Int(bytes.len() as i32));

            let dis = heap.alloc_instance("java/io/DataInputStream");
            heap.set_field(dis, "in", JavaValue::Object(Some(bais)));
            dis
        };

        // 1. Valid Modified UTF-8 containing ASCII, non-ASCII (Russian 'Привет'), and null character (\u0000 -> 0xC0, 0x80)
        // String: "A\0Б"
        // In Modified UTF-8: 'A' (0x41), '\0' (0xC0, 0x80), 'Б' (0xD0, 0x91) -> 5 bytes total payload
        let mut payload = Vec::new();
        let utflen = 5u16;
        payload.extend_from_slice(&utflen.to_be_bytes());
        payload.push(0x41); // 'A'
        payload.push(0xC0);
        payload.push(0x80); // '\u0000'
        payload.push(0xD0);
        payload.push(0x91); // 'Б'

        let dis1 = make_dis(&payload, &mut heap);
        let res = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readUTF",
            "()Ljava/lang/String;",
            Some(dis1),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(res.is_ok());
        let str_id = res.unwrap().unwrap().as_object().unwrap().unwrap();
        let decoded = heap.get_string_utf8(str_id).unwrap();
        assert_eq!(decoded, "A\0Б");

        // 2. Truncated payload -> throws EOFException
        // Declares 10 bytes, but only 3 bytes provided
        let mut trunc_payload = Vec::new();
        trunc_payload.extend_from_slice(&10u16.to_be_bytes());
        trunc_payload.extend_from_slice(b"abc");
        let dis2 = make_dis(&trunc_payload, &mut heap);
        let res2 = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readUTF",
            "()Ljava/lang/String;",
            Some(dis2),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(res2.is_err());
        assert!(res2.unwrap_err().contains("EXC:java/io/EOFException:"));

        // 3. Malformed Modified UTF-8 bytes -> throws UTFDataFormatException
        let mut bad_payload = Vec::new();
        bad_payload.extend_from_slice(&2u16.to_be_bytes());
        bad_payload.push(0xFF); // Invalid UTF-8 leading byte
        bad_payload.push(0xFF);
        let dis3 = make_dis(&bad_payload, &mut heap);
        let res3 = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readUTF",
            "()Ljava/lang/String;",
            Some(dis3),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(res3.is_err());
        assert!(res3
            .unwrap_err()
            .contains("EXC:java/io/UTFDataFormatException:"));
    }

    #[test]
    fn test_data_input_stream_read_byte_and_read_int_signs_endianness_and_eof() {
        let mut heap = Heap::new();
        let mut cl = ClassLoader::new();
        let mut scheduler = Scheduler::new(5000);
        let mut context = VmContext::new(240, 320, 44100);

        let make_dis = |bytes: &[u8], heap: &mut Heap| -> ObjectId {
            let b_arr = heap.alloc_array(ArrayData::Byte(bytes.iter().map(|&b| b as i8).collect()));
            let bais = heap.alloc_instance("java/io/ByteArrayInputStream");
            heap.set_field(bais, "buf", JavaValue::Object(Some(b_arr)));
            heap.set_field(bais, "pos", JavaValue::Int(0));
            heap.set_field(bais, "count", JavaValue::Int(bytes.len() as i32));

            let dis = heap.alloc_instance("java/io/DataInputStream");
            heap.set_field(dis, "in", JavaValue::Object(Some(bais)));
            dis
        };

        // 1. readByte signedness: 0xFF must be -1, 0x80 must be -128, 0x7F must be 127
        let bytes = vec![0xFF, 0x80, 0x7F];
        let dis = make_dis(&bytes, &mut heap);

        let r1 = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readByte",
            "()B",
            Some(dis),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        )
        .unwrap()
        .unwrap();
        assert_eq!(r1.as_int().unwrap(), -1);

        let r2 = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readByte",
            "()B",
            Some(dis),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        )
        .unwrap()
        .unwrap();
        assert_eq!(r2.as_int().unwrap(), -128);

        let r3 = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readByte",
            "()B",
            Some(dis),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        )
        .unwrap()
        .unwrap();
        assert_eq!(r3.as_int().unwrap(), 127);

        // EOF on readByte
        let r_eof = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readByte",
            "()B",
            Some(dis),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(r_eof.is_err());
        assert!(r_eof.unwrap_err().contains("EXC:java/io/EOFException:"));

        // 2. readInt: Big-endian test [0x12, 0x34, 0x56, 0x78] = 0x12345678 (305419896)
        let int_bytes = vec![0x12, 0x34, 0x56, 0x78];
        let dis_int = make_dis(&int_bytes, &mut heap);
        let ri = NativeBridge::dispatch(
            "java/io/DataInputStream",
            "readInt",
            "()I",
            Some(dis_int),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        )
        .unwrap()
        .unwrap();
        assert_eq!(ri.as_int().unwrap(), 0x12345678);

        // 3. readInt EOF at each byte boundary (length 1, 2, 3 instead of 4)
        for partial_len in 0..4 {
            let partial = vec![0xAA; partial_len];
            let dis_p = make_dis(&partial, &mut heap);
            let r_p = NativeBridge::dispatch(
                "java/io/DataInputStream",
                "readInt",
                "()I",
                Some(dis_p),
                &[],
                &mut heap,
                &mut cl,
                &mut scheduler,
                &mut context,
                0,
            );
            assert!(
                r_p.is_err(),
                "readInt must fail with EOFException on partial byte length {}",
                partial_len
            );
            assert!(r_p.unwrap_err().contains("EXC:java/io/EOFException:"));
        }
    }

    #[test]
    fn test_direct_utils_create_image_overloads_and_validation() {
        let mut heap = Heap::new();
        let mut cl = ClassLoader::new();
        let mut scheduler = Scheduler::new(5000);
        let mut context = VmContext::new(240, 320, 44100);

        // 1. createImage(width, height, ARGBcolor) -> creates mutable image filled with ARGB
        let w = 10;
        let h = 10;
        let argb: i32 = 0x7F112233; // alpha=0x7F, red=0x11, green=0x22, blue=0x33
        let res = NativeBridge::dispatch(
            "com/nokia/mid/ui/DirectUtils",
            "createImage",
            "(III)Ljavax/microedition/lcdui/Image;",
            None,
            &[JavaValue::Int(w), JavaValue::Int(h), JavaValue::Int(argb)],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(res.is_ok());
        let img_id = res.unwrap().unwrap().as_object().unwrap().unwrap();
        let img = context.images.get(&img_id).unwrap();
        assert!(
            img.is_mutable,
            "Image created via DirectUtils(w,h,argb) must be mutable"
        );
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
        // Check pixel format
        let expected_pixel = make_rgba(0x11, 0x22, 0x33, 0x7F);
        assert_eq!(img.pixels[0], expected_pixel);
        assert_eq!(img.pixels[99], expected_pixel);

        // 2. Negative dimensions throw IllegalArgumentException
        let bad_w = NativeBridge::dispatch(
            "com/nokia/mid/ui/DirectUtils",
            "createImage",
            "(III)Ljavax/microedition/lcdui/Image;",
            None,
            &[JavaValue::Int(-5), JavaValue::Int(10), JavaValue::Int(argb)],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(bad_w.is_err());
        assert!(bad_w
            .unwrap_err()
            .contains("EXC:java/lang/IllegalArgumentException:"));

        // 3. Array bounds error in createImage(byte[], offset, len) throws ArrayIndexOutOfBoundsException
        let raw_arr = heap.alloc_array(ArrayData::Byte(vec![0; 10]));
        let oob = NativeBridge::dispatch(
            "com/nokia/mid/ui/DirectUtils",
            "createImage",
            "([BII)Ljavax/microedition/lcdui/Image;",
            None,
            &[
                JavaValue::Object(Some(raw_arr)),
                JavaValue::Int(5),
                JavaValue::Int(10),
            ], // 5 + 10 > 10
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(oob.is_err());
        assert!(oob
            .unwrap_err()
            .contains("EXC:java/lang/ArrayIndexOutOfBoundsException:"));
    }

    fn make_dummy_frame() -> j2me_core::vm::frame::Frame {
        j2me_core::vm::frame::Frame::new(
            "Test".to_string(),
            "run".to_string(),
            "()V".to_string(),
            std::sync::Arc::new(vec![
                j2me_core::vm::instruction::Instruction::Nop,
                j2me_core::vm::instruction::Instruction::Return,
            ]),
            1,
            1,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        )
    }

    #[test]
    fn test_java_monitor_contract_fixtures_via_vm() {
        let mut heap = Heap::new();
        let mut cl = ClassLoader::new();
        let mut scheduler = Scheduler::new(5000);
        let mut context = VmContext::new(240, 320, 44100);

        let lock_obj = heap.alloc_instance("java/lang/Object");

        // Spawn Thread 1 (id=1) and Thread 2 (id=2)
        let f1 = make_dummy_frame();
        let f2 = make_dummy_frame();
        let t1 = scheduler.spawn("Worker-1", f1, None);
        let t2 = scheduler.spawn("Worker-2", f2, None);

        // 1. Thread 1 calls wait() without holding lock -> throws IllegalMonitorStateException!
        let bad_wait = NativeBridge::dispatch(
            "java/lang/Object",
            "wait",
            "()V",
            Some(lock_obj),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(bad_wait.is_err());
        assert!(bad_wait
            .unwrap_err()
            .contains("EXC:java/lang/IllegalMonitorStateException:"));

        // 2. Thread 1 acquires lock
        assert!(scheduler.try_acquire_monitor(t1, lock_obj));

        // 3. Negative timeout throws IllegalArgumentException
        let bad_timeout = NativeBridge::dispatch(
            "java/lang/Object",
            "wait",
            "(J)V",
            Some(lock_obj),
            &[JavaValue::Long(-100)],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(bad_timeout.is_err());
        assert!(bad_timeout
            .unwrap_err()
            .contains("EXC:java/lang/IllegalArgumentException:"));

        // 4. Thread 1 calls wait(100ms) -> atomically enters wait_set and releases monitor
        let ok_wait = NativeBridge::dispatch(
            "java/lang/Object",
            "wait",
            "(J)V",
            Some(lock_obj),
            &[JavaValue::Long(100)],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            0,
        );
        assert!(ok_wait.is_ok());
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, None);
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Waiting { .. }
        ));

        // 5. Thread 2 acquires the released lock
        assert!(scheduler.try_acquire_monitor(t2, lock_obj));

        // 6. Thread 2 calls notify() -> Thread 1 moves to entry_set as Blocked, NOT Ready!
        let ok_notify = NativeBridge::dispatch(
            "java/lang/Object",
            "notify",
            "()V",
            Some(lock_obj),
            &[],
            &mut heap,
            &mut cl,
            &mut scheduler,
            &mut context,
            1,
        );
        assert!(ok_notify.is_ok());
        assert!(matches!(
            scheduler.threads[0].state,
            ThreadState::Blocked { .. }
        ));
        assert_eq!(scheduler.next_ready_thread_index(), Some(1)); // Only Thread 2 is ready

        // 7. Thread 2 releases lock -> Thread 1 re-acquires lock and transitions to Ready!
        assert!(scheduler.release_monitor(t2, lock_obj));
        assert_eq!(scheduler.monitors.get(&lock_obj).unwrap().owner, Some(t1));
        assert_eq!(scheduler.threads[0].state, ThreadState::Ready);
    }
}
