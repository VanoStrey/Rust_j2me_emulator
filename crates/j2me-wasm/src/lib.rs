use j2me_core::heap::object::ObjectData;
use j2me_core::threads::green_thread::ThreadState;
use j2me_stdlib::J2meEmulator;
use std::alloc::{alloc, dealloc, Layout};

use std::collections::HashMap;

static mut EMULATOR: Option<J2meEmulator> = None;
static mut EXPORT_BUFFER: Vec<u8> = Vec::new();
static mut PROFILE_BUFFER: Vec<u8> = Vec::new();
static mut PARITY_STATE_BUFFER: Vec<u8> = Vec::new();
static mut AUTO_HEALING_SIG_BUFFER: Vec<u8> = Vec::new();
static mut OBSERVABILITY_BUFFER: Vec<u8> = Vec::new();
static mut ALLOCATIONS: Option<HashMap<usize, usize>> = None;

#[no_mangle]
pub extern "C" fn emu_init(width: u32, height: u32, sample_rate: u32) {
    unsafe {
        EMULATOR = Some(J2meEmulator::new(width, height, sample_rate));
    }
}

/// Allocate memory buffer for JS to write input data (JAR, RMS, etc.)
/// Validates non-zero size, checks maximum size bounds (64MB), and tracks
/// allocations to protect against overflow, layout mismatch, and double-free.
#[no_mangle]
pub extern "C" fn emu_alloc_buffer(size: usize) -> *mut u8 {
    if size == 0 || size > 64 * 1024 * 1024 {
        return std::ptr::null_mut();
    }
    let layout = match Layout::from_size_align(size, 8) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe {
        let ptr = alloc(layout);
        if !ptr.is_null() {
            if let Some(ref mut map) = ALLOCATIONS {
                map.insert(ptr as usize, size);
            } else {
                let mut map = HashMap::new();
                map.insert(ptr as usize, size);
                ALLOCATIONS = Some(map);
            }
        }
        ptr
    }
}

/// Deallocate memory buffer using verified tracked size to prevent
/// double-free corruption, layout mismatch, and wild pointer deallocation.
#[no_mangle]
pub extern "C" fn emu_free_buffer(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    unsafe {
        if let Some(ref mut map) = ALLOCATIONS {
            if let Some(allocated_size) = map.remove(&(ptr as usize)) {
                let actual_size = if allocated_size == size {
                    size
                } else {
                    allocated_size
                };
                if let Ok(layout) = Layout::from_size_align(actual_size, 8) {
                    dealloc(ptr, layout);
                }
            }
        }
    }
}

/// Load JAR from memory buffer
#[no_mangle]
pub extern "C" fn emu_load_jar(jar_ptr: *const u8, jar_len: usize) -> i32 {
    if jar_ptr.is_null() || jar_len == 0 {
        return 0;
    }
    let bytes = unsafe { std::slice::from_raw_parts(jar_ptr, jar_len) };
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            match emu.load_jar(bytes) {
                Ok(_) => 1,
                Err(_) => 0,
            }
        } else {
            0
        }
    }
}

/// Load JAD descriptor from memory buffer
#[no_mangle]
pub extern "C" fn emu_load_jad(jad_ptr: *const u8, jad_len: usize) -> i32 {
    if jad_ptr.is_null() || jad_len == 0 {
        return 0;
    }
    let bytes = unsafe { std::slice::from_raw_parts(jad_ptr, jad_len) };
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.load_jad(bytes);
            1
        } else {
            0
        }
    }
}

/// Set custom application property key/value pair
#[no_mangle]
pub extern "C" fn emu_set_app_property(
    key_ptr: *const u8,
    key_len: usize,
    val_ptr: *const u8,
    val_len: usize,
) -> i32 {
    if key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    let key_bytes = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let val_bytes = if !val_ptr.is_null() && val_len > 0 {
        unsafe { std::slice::from_raw_parts(val_ptr, val_len) }
    } else {
        &[]
    };
    if let (Ok(key), Ok(val)) = (
        std::str::from_utf8(key_bytes),
        std::str::from_utf8(val_bytes),
    ) {
        unsafe {
            if let Some(ref mut emu) = EMULATOR {
                emu.set_app_property(key, val);
                return 1;
            }
        }
    }
    0
}

/// Execute one frame tick
#[no_mangle]
pub extern "C" fn emu_tick(delta_ms: f64) -> i32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            if emu.tick(delta_ms) {
                1
            } else {
                0
            }
        } else {
            0
        }
    }
}

/// Set target FPS profile (15, 20, 30, 60)
#[no_mangle]
pub extern "C" fn emu_set_target_fps(fps: u32) -> u32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.set_target_fps(fps);
            emu.target_fps
        } else {
            fps
        }
    }
}

/// Get current target FPS profile
#[no_mangle]
pub extern "C" fn emu_get_target_fps() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.target_fps
        } else {
            30
        }
    }
}

/// Direct pointer to flat RGBA8888 framebuffer (zero-copy shared with JS)
#[no_mangle]
pub extern "C" fn emu_get_framebuffer_ptr() -> *const u8 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.framebuffer_ptr()
        } else {
            std::ptr::null()
        }
    }
}

/// Length of RGBA8888 framebuffer in bytes
#[no_mangle]
pub extern "C" fn emu_get_framebuffer_len() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.framebuffer_len()
        } else {
            0
        }
    }
}

/// Key input event handler
#[no_mangle]
pub extern "C" fn emu_key_event(key_code: i32, pressed: i32) {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.key_event(key_code, pressed != 0);
        }
    }
}

/// Read audio samples (f32 PCM) directly into buffer allocated by JS
#[no_mangle]
pub extern "C" fn emu_read_audio_samples(dest_ptr: *mut f32, max_samples: usize) -> usize {
    if dest_ptr.is_null() || max_samples == 0 {
        return 0;
    }
    let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr, max_samples) };
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.read_audio_samples(dest)
        } else {
            0
        }
    }
}

/// Telemetry / Streaming: Get current number of unconsumed samples queued in audio ring buffer
#[no_mangle]
pub extern "C" fn emu_get_audio_queue_len() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.audio.len()
        } else {
            0
        }
    }
}

/// Export RMS stores to binary buffer for IndexedDB / localStorage
#[no_mangle]
pub extern "C" fn emu_export_rms() -> *const u8 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            EXPORT_BUFFER = emu.export_rms();
            (*std::ptr::addr_of!(EXPORT_BUFFER)).as_ptr()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_export_rms_len() -> usize {
    unsafe { (*std::ptr::addr_of!(EXPORT_BUFFER)).len() }
}

/// Import RMS stores from binary buffer
#[no_mangle]
pub extern "C" fn emu_import_rms(data_ptr: *const u8, len: usize) -> i32 {
    if data_ptr.is_null() || len == 0 {
        return 0;
    }
    let data = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            if emu.import_rms(data).is_ok() {
                1
            } else {
                0
            }
        } else {
            0
        }
    }
}

/// Live debug: active threads count
#[no_mangle]
pub extern "C" fn emu_get_active_threads_count() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.scheduler.active_thread_count()
        } else {
            0
        }
    }
}

/// Live memory read for cheat inspection and debugging
#[no_mangle]
pub extern "C" fn emu_read_memory_u32(ptr: *const u32) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe { *ptr }
}

/// Live memory write for cheat injection
#[no_mangle]
pub extern "C" fn emu_write_memory_u32(ptr: *mut u32, val: u32) {
    if !ptr.is_null() {
        unsafe {
            *ptr = val;
        }
    }
}

/// Diagnostic pattern generator: Pure Red, Green, Blue, and Transparent/White checkerboard
#[no_mangle]
pub extern "C" fn emu_fill_diagnostic_pattern() {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            let w = emu.context.screen_width as usize;
            let h = emu.context.screen_height as usize;
            for y in 0..h {
                for x in 0..w {
                    // RGBA little-endian in memory: byte 0=R, 1=G, 2=B, 3=A
                    let pixel = if x < w / 2 && y < h / 2 {
                        // Top-left: Pure Red [255, 0, 0, 255]
                        0xFF0000FF
                    } else if x >= w / 2 && y < h / 2 {
                        // Top-right: Pure Green [0, 255, 0, 255]
                        0xFF00FF00
                    } else if x < w / 2 && y >= h / 2 {
                        // Bottom-left: Pure Blue [0, 0, 255, 255]
                        0xFFFF0000
                    } else {
                        // Bottom-right: Checkerboard White / Transparent
                        if ((x / 8) + (y / 8)) % 2 == 0 {
                            0xFFFFFFFF
                        } else {
                            0x00000000
                        }
                    };
                    let idx = y * w + x;
                    if idx < emu.context.framebuffer.len() {
                        emu.context.framebuffer[idx] = pixel;
                    }
                }
            }
            emu.context.screen_dirty = true;
        }
    }
}

/// Teardown and reset current emulator instance, releasing all tracked buffers
#[no_mangle]
pub extern "C" fn emu_destroy() {
    unsafe {
        EMULATOR = None;
        if let Some(ref mut map) = ALLOCATIONS {
            for (&ptr_addr, &size) in map.iter() {
                if let Ok(layout) = Layout::from_size_align(size, 8) {
                    dealloc(ptr_addr as *mut u8, layout);
                }
            }
            map.clear();
        }
        ALLOCATIONS = None;
    }
}

/// Telemetry: Total virtual instructions executed
#[no_mangle]
pub extern "C" fn emu_get_instructions_executed() -> u64 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.scheduler
                .threads
                .iter()
                .map(|t| t.instructions_executed)
                .sum()
        } else {
            0
        }
    }
}

/// Telemetry: VM execution time in microseconds
#[no_mangle]
pub extern "C" fn emu_get_vm_time_us() -> u64 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.vm_time_us
        } else {
            0
        }
    }
}

/// Telemetry: Exceptions thrown count
#[no_mangle]
pub extern "C" fn emu_get_exceptions_count() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.exceptions_count as u32
        } else {
            0
        }
    }
}

/// Telemetry: Uncaught exceptions count (fatal exceptions causing thread termination)
#[no_mangle]
pub extern "C" fn emu_get_uncaught_exceptions_count() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.exception_telemetry.guest_exceptions_uncaught as u32
        } else {
            0
        }
    }
}

/// Telemetry: Caught exceptions count
#[no_mangle]
pub extern "C" fn emu_get_caught_exceptions_count() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.exception_telemetry.guest_exceptions_caught as u32
        } else {
            0
        }
    }
}

/// Telemetry: Unhandled native calls count
#[no_mangle]
pub extern "C" fn emu_get_unhandled_native_count() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.unhandled_native_calls as u32
        } else {
            0
        }
    }
}

/// Telemetry: Auto-healing count (how many times unhandled exceptions were caught and healed by core)
#[no_mangle]
pub extern "C" fn emu_get_auto_healing_count() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.exception_telemetry.auto_healing_count
        } else {
            0
        }
    }
}

/// Telemetry: Last auto-healing signature pointer
#[no_mangle]
pub extern "C" fn emu_get_last_auto_healing_signature_ptr() -> *const u8 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            if let Some(ref sig) = emu.context.exception_telemetry.last_auto_healing_signature {
                AUTO_HEALING_SIG_BUFFER = sig.as_bytes().to_vec();
                (*std::ptr::addr_of!(AUTO_HEALING_SIG_BUFFER)).as_ptr()
            } else {
                std::ptr::null()
            }
        } else {
            std::ptr::null()
        }
    }
}

/// Telemetry: Last auto-healing signature length in bytes
#[no_mangle]
pub extern "C" fn emu_get_last_auto_healing_signature_len() -> usize {
    unsafe { (*std::ptr::addr_of!(AUTO_HEALING_SIG_BUFFER)).len() }
}

/// Telemetry: Alias for emu_get_last_auto_healing_signature_ptr
#[no_mangle]
pub extern "C" fn emu_get_last_auto_healing_signature() -> *const u8 {
    emu_get_last_auto_healing_signature_ptr()
}

/// Telemetry: Paint calls count
#[no_mangle]
pub extern "C" fn emu_get_paint_calls() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.paint_calls_count as u32
        } else {
            0
        }
    }
}

/// Telemetry: Framebuffer publications count
#[no_mangle]
pub extern "C" fn emu_get_framebuffer_publications() -> u32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.fb_publishes_count as u32
        } else {
            0
        }
    }
}

/// Telemetry: Live objects in heap
#[no_mangle]
pub extern "C" fn emu_get_live_objects() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.heap.live_objects()
        } else {
            0
        }
    }
}

/// Explicit Garbage Collection: trigger mark-and-sweep, returning freed object count
#[no_mangle]
pub extern "C" fn emu_force_gc() -> usize {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.force_gc()
        } else {
            0
        }
    }
}

/// Telemetry: Write audio ring buffer statistics: [generated, consumed, dropped, underrun, overflow]
#[no_mangle]
pub extern "C" fn emu_get_audio_stats(out_stats: *mut u32) {
    if out_stats.is_null() {
        return;
    }
    unsafe {
        if let Some(ref emu) = EMULATOR {
            *out_stats.add(0) = emu.context.audio.generated_samples as u32;
            *out_stats.add(1) = emu.context.audio.consumed_samples as u32;
            *out_stats.add(2) = emu.context.audio.dropped_samples as u32;
            *out_stats.add(3) = emu.context.audio.underrun_count as u32;
            *out_stats.add(4) = emu.context.audio.overflow_count as u32;
        }
    }
}

/// Telemetry: Get guest VM heap live object count
#[no_mangle]
pub extern "C" fn emu_get_heap_object_count() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.heap.live_objects()
        } else {
            0
        }
    }
}

/// Telemetry: Get total garbage collection passes performed
#[no_mangle]
pub extern "C" fn emu_get_gc_count() -> usize {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.gc_count
        } else {
            0
        }
    }
}

/// Telemetry: Get instantaneous audio peak amplitude [0.0 .. 1.0+] with decay
#[no_mangle]
pub extern "C" fn emu_get_audio_peak_level() -> f32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.context.audio.get_and_decay_peak()
        } else {
            0.0
        }
    }
}

/// Telemetry: Get count of clipped audio samples (>= 0.99)
#[no_mangle]
pub extern "C" fn emu_get_clipped_samples() -> u64 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.audio.clipped_samples
        } else {
            0
        }
    }
}

/// Dynamically adjust screen size (width, height)
#[no_mangle]
pub extern "C" fn emu_set_screen_size(width: u32, height: u32) {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.set_screen_size(width, height);
        }
    }
}

/// Retrieve comprehensive parity state telemetry as a JSON string
#[no_mangle]
pub extern "C" fn emu_get_parity_state_ptr() -> *const u8 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            let active_screen_name = if let Some(id) = emu.context.current_displayable {
                if let Some(ObjectData::Instance(inst)) = emu.heap.get(id) {
                    inst.class_name.clone()
                } else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            };

            let is_full = if let Some(id) = emu.context.current_displayable {
                emu.context
                    .displayable_commands
                    .get(&id)
                    .map(|e| e.is_full_screen)
                    .unwrap_or(false)
            } else {
                false
            };

            let canvas_w = emu.context.screen_width;
            let canvas_h = if is_full {
                emu.context.screen_height
            } else {
                emu.context.screen_height.saturating_sub(24)
            };

            let gc = &emu.context.graphics;
            let clip_x = gc.clip_x;
            let clip_y = gc.clip_y;
            let clip_w = gc.clip_w;
            let clip_h = gc.clip_h;
            let trans_x = gc.trans_x;
            let trans_y = gc.trans_y;

            let active_threads = emu
                .scheduler
                .threads
                .iter()
                .filter(|t| !matches!(t.state, ThreadState::Terminated))
                .count();
            let active_timers = emu.context.timers.active_tasks_count();

            let recent_calls: Vec<String> = emu
                .context
                .recent_native_calls
                .iter()
                .take(10)
                .map(|s| format!("{:?}", s))
                .collect();

            let last_exc_json = if let Some(ref exc) =
                emu.context.exception_telemetry.fatal_guest_exception
            {
                format!(
                    "{{\"class\":{:?},\"message\":{:?},\"location\":{:?}}}",
                    exc.exception_class,
                    exc.message.as_deref().unwrap_or(""),
                    format!(
                        "{}.{}{} (pc={})",
                        exc.class_name, exc.method_name, exc.descriptor, exc.pc
                    )
                )
            } else {
                "null".to_string()
            };

            let json = format!(
                concat!(
                    "{{",
                    "\"active_screen\":{:?},",
                    "\"screen_width\":{},",
                    "\"screen_height\":{},",
                    "\"canvas_width\":{},",
                    "\"canvas_height\":{},",
                    "\"is_fullscreen\":{},",
                    "\"clip_rect\":{{\"x\":{},\"y\":{},\"w\":{},\"h\":{}}},",
                    "\"translate\":{{\"tx\":{},\"ty\":{}}},",
                    "\"draw_calls\":{{",
                    "\"draw_image\":{},",
                    "\"draw_region\":{},",
                    "\"fill_rect\":{},",
                    "\"draw_string\":{},",
                    "\"draw_pixels\":{},",
                    "\"total\":{}",
                    "}},",
                    "\"active_threads\":{},",
                    "\"active_timers\":{},",
                    "\"recent_api_calls\":[{}],",
                    "\"last_exception\":{}",
                    "}}"
                ),
                active_screen_name,
                emu.context.screen_width,
                emu.context.screen_height,
                canvas_w,
                canvas_h,
                is_full,
                clip_x,
                clip_y,
                clip_w,
                clip_h,
                trans_x,
                trans_y,
                emu.context.draw_image_count,
                emu.context.draw_region_count,
                emu.context.fill_rect_count,
                emu.context.draw_string_count,
                emu.context.draw_pixels_count,
                emu.context.draw_image_count
                    + emu.context.draw_region_count
                    + emu.context.fill_rect_count
                    + emu.context.draw_string_count
                    + emu.context.draw_pixels_count,
                active_threads,
                active_timers,
                recent_calls.join(","),
                last_exc_json
            );

            PARITY_STATE_BUFFER = json.into_bytes();
            (*std::ptr::addr_of!(PARITY_STATE_BUFFER)).as_ptr()
        } else {
            std::ptr::null()
        }
    }
}

/// Length of JSON parity state buffer
#[no_mangle]
pub extern "C" fn emu_get_parity_state_len() -> usize {
    unsafe { (*std::ptr::addr_of!(PARITY_STATE_BUFFER)).len() }
}

/// Enable or disable strict execution mode (disables auto-healing exception swallowing)
#[no_mangle]
pub extern "C" fn emu_set_strict_mode(strict: i32) {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.set_strict_mode(strict != 0);
        }
    }
}

/// Query whether strict execution mode is enabled
#[no_mangle]
pub extern "C" fn emu_is_strict_mode() -> i32 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            if emu.is_strict_mode() {
                1
            } else {
                0
            }
        } else {
            0
        }
    }
}

/// Get total number of distinct framebuffer mutations (FNV-1a hash changes)
#[no_mangle]
pub extern "C" fn emu_get_framebuffer_changes_count() -> u64 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            emu.context.framebuffer_changes_count
        } else {
            0
        }
    }
}

/// Evaluate runtime observability criteria and serialize RuntimeObservabilityReport to JSON buffer
#[no_mangle]
pub extern "C" fn emu_evaluate_observability(wall_timeout: i32) -> *const u8 {
    unsafe {
        if let Some(ref emu) = EMULATOR {
            let report = emu.evaluate_observability(wall_timeout != 0);
            let json = serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string());
            OBSERVABILITY_BUFFER = json.into_bytes();
            (*std::ptr::addr_of!(OBSERVABILITY_BUFFER)).as_ptr()
        } else {
            std::ptr::null()
        }
    }
}

/// Length of JSON observability report buffer
#[no_mangle]
pub extern "C" fn emu_get_observability_report_len() -> usize {
    unsafe { (*std::ptr::addr_of!(OBSERVABILITY_BUFFER)).len() }
}

fn parse_resolution_str(s: &str) -> Option<(u32, u32)> {
    let clean = s.trim();
    let parts: Vec<&str> = if clean.contains('x') || clean.contains('X') {
        clean.split(|c| c == 'x' || c == 'X').collect()
    } else if clean.contains(',') {
        clean.split(',').collect()
    } else if clean.contains('*') {
        clean.split('*').collect()
    } else {
        return None;
    };
    if parts.len() >= 2 {
        let w = parts[0].trim().parse::<u32>().ok()?;
        let h = parts[1].trim().parse::<u32>().ok()?;
        if (64..=1920).contains(&w) && (64..=1920).contains(&h) {
            return Some((w, h));
        }
    }
    None
}

fn detect_screen_resolution_from_assets(
    jar: &j2me_core::jar::JarArchive,
) -> Option<(u32, u32, &'static str)> {
    let standard_resolutions: &[(u32, u32)] = &[
        (128, 128),
        (128, 160),
        (176, 208),
        (176, 220),
        (208, 208),
        (240, 320),
        (320, 240),
        (240, 400),
        (352, 416),
        (480, 800),
    ];

    let mut found_dimensions: Vec<(u32, u32)> = Vec::new();

    for (_name, data) in &jar.entries {
        // Find all PNG headers in data
        let mut idx = 0;
        while idx + 24 <= data.len() {
            if &data[idx..idx + 8] == b"\x89PNG\r\n\x1a\n" && &data[idx + 12..idx + 16] == b"IHDR" {
                let w = u32::from_be_bytes([
                    data[idx + 16],
                    data[idx + 17],
                    data[idx + 18],
                    data[idx + 19],
                ]);
                let h = u32::from_be_bytes([
                    data[idx + 20],
                    data[idx + 21],
                    data[idx + 22],
                    data[idx + 23],
                ]);
                if (16..=1920).contains(&w) && (16..=1920).contains(&h) {
                    found_dimensions.push((w, h));
                }
                idx += 24;
            } else {
                idx += 1;
            }
        }

        // Check LMB format (Mayhem Studio games, e.g. Cyberpunk)
        if data.len() >= 4 && (1..=3).contains(&data[0]) {
            let fmt = data[0];
            let count = u16::from_be_bytes([data[1], data[2]]) as usize;
            let pal_len = if fmt == 2 {
                16
            } else if fmt == 3 {
                4
            } else {
                0
            };
            let mut off = 3 + pal_len;
            for _ in 0..count {
                if off + 4 <= data.len() {
                    let w = u16::from_be_bytes([data[off], data[off + 1]]) as u32;
                    let h = u16::from_be_bytes([data[off + 2], data[off + 3]]) as u32;
                    if (16..=1920).contains(&w) && (16..=1920).contains(&h) {
                        found_dimensions.push((w, h));
                    }
                    off += 4;
                } else {
                    break;
                }
            }
        }
    }

    // 1. Direct match with standard full-screen resolutions (e.g. splash/background)
    for &(sw, sh) in standard_resolutions {
        if found_dimensions.iter().any(|&(w, h)| w == sw && h == sh) {
            return Some((sw, sh, "image_exact_match"));
        }
    }

    // 2. Max width heuristics
    let max_w = found_dimensions.iter().map(|&(w, _)| w).max().unwrap_or(0);
    if max_w == 176 {
        return Some((176, 208, "image_width_heuristic"));
    } else if max_w == 128 {
        let max_h = found_dimensions.iter().map(|&(_, h)| h).max().unwrap_or(0);
        if max_h <= 128 {
            return Some((128, 128, "image_width_heuristic"));
        } else {
            return Some((128, 160, "image_width_heuristic"));
        }
    } else if max_w == 208 {
        return Some((208, 208, "image_width_heuristic"));
    } else if max_w == 240 {
        return Some((240, 320, "image_width_heuristic"));
    } else if max_w == 320 {
        return Some((320, 240, "image_width_heuristic"));
    }

    None
}

fn detect_jar_profile_impl(bytes: &[u8]) -> String {
    let jar = match j2me_core::jar::JarArchive::parse(bytes) {
        Ok(j) => j,
        Err(e) => return format!("{{\"error\":\"Failed to parse JAR: {}\"}}", e),
    };

    let mut name = "Unknown MIDlet".to_string();
    let mut vendor = "Unknown Vendor".to_string();
    let mut version = "1.0.0".to_string();
    let mut midlets_json = Vec::new();
    let mut suggested_w = 240u32;
    let mut suggested_h = 320u32;
    let mut res_source = "default";

    if let Some(ref m) = jar.manifest {
        if let Some(n) = m.midlet_name() {
            name = n.to_string();
        }
        if let Some(v) = m.midlet_vendor() {
            vendor = v.to_string();
        }
        if let Some(ver) = m.midlet_version() {
            version = ver.to_string();
        }

        for i in 1..=16 {
            if let Some((m_name, m_icon, m_cls)) = m.get_midlet_info(i) {
                midlets_json.push(format!(
                    "{{\"index\":{},\"name\":{:?},\"icon\":{:?},\"class\":{:?}}}",
                    i, m_name, m_icon, m_cls
                ));
            }
        }

        // Check manifest resolution keys
        let res_keys = [
            "Nokia-MIDlet-Original-Display-Size",
            "Nokia-MIDlet-Target-Display-Size",
            "MIDlet-ScreenSize",
            "SE-MIDlet-Original-Display-Size",
            "Resolution",
            "Screen-Size",
            "ScreenSize",
        ];
        for k in &res_keys {
            if let Some(val) = m.get(k) {
                if let Some((w, h)) = parse_resolution_str(val) {
                    suggested_w = w;
                    suggested_h = h;
                    res_source = "manifest";
                    break;
                }
            }
        }
    }

    if res_source == "default" {
        // Heuristics based on entry paths and class names
        let known_resolutions = [
            ("128x128", 128, 128),
            ("128x160", 128, 160),
            ("176x208", 176, 208),
            ("176x220", 176, 220),
            ("208x208", 208, 208),
            ("240x320", 240, 320),
            ("320x240", 320, 240),
            ("352x416", 352, 416),
            ("480x800", 480, 800),
        ];

        for (res_str, w, h) in &known_resolutions {
            let found = jar
                .entries
                .keys()
                .any(|k| k.to_lowercase().contains(res_str));
            if found {
                suggested_w = *w;
                suggested_h = *h;
                res_source = "entry_heuristic";
                break;
            }
        }

        if res_source == "default" {
            // Check for FullCanvas
            if jar.entries.keys().any(|k| k.contains("FullCanvas")) {
                suggested_w = 176;
                suggested_h = 208;
                res_source = "nokia_heuristic";
            }
        }

        if res_source == "default" {
            if let Some((w, h, src)) = detect_screen_resolution_from_assets(&jar) {
                suggested_w = w;
                suggested_h = h;
                res_source = src;
            }
        }
    }

    format!(
        "{{\"name\":{:?},\"vendor\":{:?},\"version\":{:?},\"midlets\":[{}],\"suggested_width\":{},\"suggested_height\":{},\"resolution_source\":{:?}}}",
        name, vendor, version, midlets_json.join(","), suggested_w, suggested_h, res_source
    )
}

/// Analyze raw JAR byte buffer and extract manifest metadata and recommended resolution
#[no_mangle]
pub extern "C" fn detect_jar_profile(jar_ptr: *const u8, jar_len: usize) -> *const u8 {
    if jar_ptr.is_null() || jar_len == 0 {
        return std::ptr::null();
    }
    let bytes = unsafe { std::slice::from_raw_parts(jar_ptr, jar_len) };
    let json = detect_jar_profile_impl(bytes);
    unsafe {
        PROFILE_BUFFER = json.into_bytes();
        (*std::ptr::addr_of!(PROFILE_BUFFER)).as_ptr()
    }
}

/// Length of JSON profile buffer returned by `detect_jar_profile`
#[no_mangle]
pub extern "C" fn detect_jar_profile_len() -> usize {
    unsafe { (*std::ptr::addr_of!(PROFILE_BUFFER)).len() }
}
