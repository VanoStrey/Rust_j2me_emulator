use crate::bridge::context::VmContext;
use crate::bridge::native_bridge::NativeBridge;
use crate::input::KeyInput;
use j2me_core::classfile::class_file::ClassFile;
use j2me_core::heap::object::{Heap, ObjectData, ObjectId};
use j2me_core::heap::value::JavaValue;
use j2me_core::jar::jar_loader::JarArchive;
use j2me_core::threads::green_thread::{GreenThread, ThreadState};
use j2me_core::threads::scheduler::Scheduler;
use j2me_core::vm::class_loader::ClassLoader;
use j2me_core::vm::frame::Frame;
use j2me_core::vm::interpreter::{Interpreter, StepResult};
use std::sync::Arc;

use crate::ui::{UiEvent, UI_DISPATCHER_THREAD};

pub struct J2meEmulator {
    pub heap: Heap,
    pub class_loader: ClassLoader,
    pub scheduler: Scheduler,
    pub context: VmContext,
    pub midlet_obj: Option<ObjectId>,
    pub is_running: bool,
    /// Ordered host key queue. Only one handler green thread runs at a time so
    /// press/release ordering is preserved and events are never dropped.
    pub input: KeyInput,
    /// Last displayable the input queue was bound to; a change flushes it.
    last_displayable: Option<ObjectId>,
    pub gc_count: usize,
    pub target_fps: u32,
    pub max_repaints_per_tick: u32,
}

#[derive(Clone, Copy)]
pub struct Timer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
}

impl Timer {
    #[inline]
    pub fn start() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                start: std::time::Instant::now(),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {}
        }
    }

    #[inline]
    pub fn elapsed_micros(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed().as_micros() as u64
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    }
}

impl J2meEmulator {
    pub fn new(width: u32, height: u32, sample_rate: u32) -> Self {
        let mut heap = Heap::new();
        let mut class_loader = ClassLoader::new();
        let out_id = heap.alloc_instance("java/io/PrintStream");
        class_loader.set_static(
            "java/lang/System",
            "out",
            "Ljava/io/PrintStream;",
            JavaValue::Object(Some(out_id)),
        );
        class_loader.set_static(
            "java/lang/System",
            "err",
            "Ljava/io/PrintStream;",
            JavaValue::Object(Some(out_id)),
        );
        Self::init_standard_statics(&mut class_loader);
        Self {
            heap,
            class_loader,
            scheduler: Scheduler::new(5000),
            context: VmContext::new(width, height, sample_rate),
            midlet_obj: None,
            is_running: false,
            input: KeyInput::new(),
            last_displayable: None,
            gc_count: 0,
            target_fps: 30,
            max_repaints_per_tick: 1,
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        let valid_fps = match fps {
            15 => 15,
            20 => 20,
            60 => 60,
            _ => 30,
        };
        self.target_fps = valid_fps;
        self.context.target_fps = valid_fps;
    }

    pub fn get_target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn init_standard_statics(cl: &mut ClassLoader) {
        // Canvas key & action constants
        let c = "javax/microedition/lcdui/Canvas";
        cl.set_static(c, "UP", "I", JavaValue::Int(1));
        cl.set_static(c, "LEFT", "I", JavaValue::Int(2));
        cl.set_static(c, "RIGHT", "I", JavaValue::Int(5));
        cl.set_static(c, "DOWN", "I", JavaValue::Int(6));
        cl.set_static(c, "FIRE", "I", JavaValue::Int(8));
        cl.set_static(c, "GAME_A", "I", JavaValue::Int(9));
        cl.set_static(c, "GAME_B", "I", JavaValue::Int(10));
        cl.set_static(c, "GAME_C", "I", JavaValue::Int(11));
        cl.set_static(c, "GAME_D", "I", JavaValue::Int(12));
        cl.set_static(c, "KEY_NUM0", "I", JavaValue::Int(48));
        cl.set_static(c, "KEY_NUM1", "I", JavaValue::Int(49));
        cl.set_static(c, "KEY_NUM2", "I", JavaValue::Int(50));
        cl.set_static(c, "KEY_NUM3", "I", JavaValue::Int(51));
        cl.set_static(c, "KEY_NUM4", "I", JavaValue::Int(52));
        cl.set_static(c, "KEY_NUM5", "I", JavaValue::Int(53));
        cl.set_static(c, "KEY_NUM6", "I", JavaValue::Int(54));
        cl.set_static(c, "KEY_NUM7", "I", JavaValue::Int(55));
        cl.set_static(c, "KEY_NUM8", "I", JavaValue::Int(56));
        cl.set_static(c, "KEY_NUM9", "I", JavaValue::Int(57));
        cl.set_static(c, "KEY_STAR", "I", JavaValue::Int(42));
        cl.set_static(c, "KEY_POUND", "I", JavaValue::Int(35));

        // GameCanvas
        let gc = "javax/microedition/lcdui/game/GameCanvas";
        cl.set_static(gc, "UP_PRESSED", "I", JavaValue::Int(1 << 1));
        cl.set_static(gc, "LEFT_PRESSED", "I", JavaValue::Int(1 << 2));
        cl.set_static(gc, "RIGHT_PRESSED", "I", JavaValue::Int(1 << 5));
        cl.set_static(gc, "DOWN_PRESSED", "I", JavaValue::Int(1 << 6));
        cl.set_static(gc, "FIRE_PRESSED", "I", JavaValue::Int(1 << 8));
        cl.set_static(gc, "GAME_A_PRESSED", "I", JavaValue::Int(1 << 9));
        cl.set_static(gc, "GAME_B_PRESSED", "I", JavaValue::Int(1 << 10));
        cl.set_static(gc, "GAME_C_PRESSED", "I", JavaValue::Int(1 << 11));
        cl.set_static(gc, "GAME_D_PRESSED", "I", JavaValue::Int(1 << 12));

        // FullCanvas (Nokia)
        let fc = "com/nokia/mid/ui/FullCanvas";
        cl.set_static(fc, "KEY_SOFTKEY1", "I", JavaValue::Int(-6));
        cl.set_static(fc, "KEY_SOFTKEY2", "I", JavaValue::Int(-7));
        cl.set_static(fc, "KEY_SOFTKEY3", "I", JavaValue::Int(-5));
        cl.set_static(fc, "KEY_UP_ARROW", "I", JavaValue::Int(-1));
        cl.set_static(fc, "KEY_DOWN_ARROW", "I", JavaValue::Int(-2));
        cl.set_static(fc, "KEY_LEFT_ARROW", "I", JavaValue::Int(-3));
        cl.set_static(fc, "KEY_RIGHT_ARROW", "I", JavaValue::Int(-4));
        cl.set_static(fc, "KEY_SEND", "I", JavaValue::Int(-10));
        cl.set_static(fc, "KEY_END", "I", JavaValue::Int(-11));
        cl.set_static(fc, "KEY_CLEAR", "I", JavaValue::Int(-8));

        // Graphics
        let gr = "javax/microedition/lcdui/Graphics";
        cl.set_static(gr, "HCENTER", "I", JavaValue::Int(1));
        cl.set_static(gr, "VCENTER", "I", JavaValue::Int(2));
        cl.set_static(gr, "LEFT", "I", JavaValue::Int(4));
        cl.set_static(gr, "RIGHT", "I", JavaValue::Int(8));
        cl.set_static(gr, "TOP", "I", JavaValue::Int(16));
        cl.set_static(gr, "BOTTOM", "I", JavaValue::Int(32));
        cl.set_static(gr, "BASELINE", "I", JavaValue::Int(64));
        cl.set_static(gr, "SOLID", "I", JavaValue::Int(0));
        cl.set_static(gr, "DOTTED", "I", JavaValue::Int(1));

        // Font
        let font = "javax/microedition/lcdui/Font";
        cl.set_static(font, "STYLE_PLAIN", "I", JavaValue::Int(0));
        cl.set_static(font, "STYLE_BOLD", "I", JavaValue::Int(1));
        cl.set_static(font, "STYLE_ITALIC", "I", JavaValue::Int(2));
        cl.set_static(font, "STYLE_UNDERLINED", "I", JavaValue::Int(4));
        cl.set_static(font, "SIZE_SMALL", "I", JavaValue::Int(8));
        cl.set_static(font, "SIZE_MEDIUM", "I", JavaValue::Int(0));
        cl.set_static(font, "SIZE_LARGE", "I", JavaValue::Int(16));
        cl.set_static(font, "FACE_SYSTEM", "I", JavaValue::Int(0));
        cl.set_static(font, "FACE_MONOSPACE", "I", JavaValue::Int(32));
        cl.set_static(font, "FACE_PROPORTIONAL", "I", JavaValue::Int(64));

        // Sprite
        let spr = "javax/microedition/lcdui/game/Sprite";
        cl.set_static(spr, "TRANS_NONE", "I", JavaValue::Int(0));
        cl.set_static(spr, "TRANS_ROT90", "I", JavaValue::Int(5));
        cl.set_static(spr, "TRANS_ROT180", "I", JavaValue::Int(3));
        cl.set_static(spr, "TRANS_ROT270", "I", JavaValue::Int(6));
        cl.set_static(spr, "TRANS_MIRROR", "I", JavaValue::Int(2));
        cl.set_static(spr, "TRANS_MIRROR_ROT90", "I", JavaValue::Int(7));
        cl.set_static(spr, "TRANS_MIRROR_ROT180", "I", JavaValue::Int(1));
        cl.set_static(spr, "TRANS_MIRROR_ROT270", "I", JavaValue::Int(4));

        // DirectGraphics
        let dg = "com/nokia/mid/ui/DirectGraphics";
        cl.set_static(dg, "TYPE_BYTE_1_GRAY", "I", JavaValue::Int(1));
        cl.set_static(dg, "TYPE_BYTE_1_GRAY_VERTICAL", "I", JavaValue::Int(-1));
        cl.set_static(dg, "TYPE_BYTE_2_GRAY", "I", JavaValue::Int(2));
        cl.set_static(dg, "TYPE_BYTE_4_GRAY", "I", JavaValue::Int(4));
        cl.set_static(dg, "TYPE_BYTE_8_GRAY", "I", JavaValue::Int(8));
        cl.set_static(dg, "TYPE_BYTE_332_RGB", "I", JavaValue::Int(332));
        cl.set_static(dg, "TYPE_USHORT_4444_ARGB", "I", JavaValue::Int(4444));
        cl.set_static(dg, "TYPE_USHORT_444_RGB", "I", JavaValue::Int(444));
        cl.set_static(dg, "TYPE_USHORT_555_RGB", "I", JavaValue::Int(555));
        cl.set_static(dg, "TYPE_USHORT_565_RGB", "I", JavaValue::Int(565));
        cl.set_static(dg, "TYPE_USHORT_1555_ARGB", "I", JavaValue::Int(1555));
        cl.set_static(dg, "TYPE_INT_888_RGB", "I", JavaValue::Int(888));
        cl.set_static(dg, "TYPE_INT_8888_ARGB", "I", JavaValue::Int(8888));
        cl.set_static(dg, "FLIP_HORIZONTAL", "I", JavaValue::Int(8192));
        cl.set_static(dg, "FLIP_VERTICAL", "I", JavaValue::Int(16384));
        cl.set_static(dg, "ROTATE_90", "I", JavaValue::Int(90));
        cl.set_static(dg, "ROTATE_180", "I", JavaValue::Int(180));
        cl.set_static(dg, "ROTATE_270", "I", JavaValue::Int(270));

        // Player
        let pl = "javax/microedition/media/Player";
        cl.set_static(pl, "TIME_UNKNOWN", "J", JavaValue::Long(-1));
        cl.set_static(pl, "UNREALIZED", "I", JavaValue::Int(100));
        cl.set_static(pl, "REALIZED", "I", JavaValue::Int(200));
        cl.set_static(pl, "PREFETCHED", "I", JavaValue::Int(300));
        cl.set_static(pl, "STARTED", "I", JavaValue::Int(400));
        cl.set_static(pl, "CLOSED", "I", JavaValue::Int(0));
    }

    pub fn load_jar(&mut self, jar_bytes: &[u8]) -> Result<String, String> {
        self.load_jar_with_filename(jar_bytes, None)
    }

    pub fn load_jar_with_filename(
        &mut self,
        jar_bytes: &[u8],
        filename: Option<&str>,
    ) -> Result<String, String> {
        let jar = JarArchive::parse(jar_bytes)?;
        let manifest = jar.manifest.clone().unwrap_or_default();

        if let Some(f) = filename {
            self.context.jar_filename = Some(f.to_string());
        }
        let effective_filename = filename.or(self.context.jar_filename.as_deref());

        // 0. Resolution-Agnostic Engine: Auto-detect native resolution from Manifest, JAD, Filename, and Assets
        let detected = j2me_core::jar::ResolutionDetector::detect(
            Some(&manifest),
            self.context.jad.as_ref(),
            effective_filename,
            Some(&jar),
        );
        self.set_screen_size(detected.width, detected.height);
        println!(
            " [RESOLUTION] Auto-detected native display: {}x{} (source: {})",
            detected.width, detected.height, detected.source
        );

        // Load all .class files from JAR into ClassLoader
        let mut _loaded = 0;
        let mut _failed = 0;
        let mut class_names = Vec::new();
        for (path, data) in &jar.entries {
            if path.ends_with(".class") {
                match ClassFile::parse(data) {
                    Ok(class_file) => {
                        class_names.push(class_file.this_class.clone());
                        self.class_loader.define_class(class_file);
                        _loaded += 1;
                    }
                    Err(e) => {
                        eprintln!("[CLASS PARSE FAIL] {}: {}", path, e);
                        _failed += 1;
                    }
                }
            }
        }

        let primary_class = manifest
            .get_primary_midlet_class()
            .or_else(|| {
                class_names
                    .iter()
                    .find(|name| {
                        self.class_loader.find_method(name, "startApp", "()V").is_some()
                    })
                    .cloned()
                    .or_else(|| {
                        class_names
                            .iter()
                            .find(|name| {
                                let l = name.to_lowercase();
                                l.contains("midlet") || l.ends_with("main") || l.ends_with("game")
                            })
                            .cloned()
                    })
                    .or_else(|| class_names.first().cloned())
            })
            .ok_or_else(|| "No runnable class found in JAR archive".to_string())?;

        let app_name = manifest.midlet_name().unwrap_or("J2ME Game").to_string();
        self.context.jar = Some(jar);
        let normalized_cls = primary_class.replace('.', "/");

        // Auto-inject game compatibility properties
        let is_heroes_lore = app_name.to_lowercase().contains("heroes lore")
            || normalized_cls.contains("rpg/GameMIDlet");
        if is_heroes_lore && self.get_app_property("HO-Demo").is_none() {
            self.set_app_property("HO-Demo", "PAID");
        }

        // Instantiate MIDlet class
        let midlet_id = self.heap.alloc_instance(&normalized_cls);
        self.midlet_obj = Some(midlet_id);

        let mut main_thread_frames = Vec::new();

        // 1. startApp frame (pushed first, at bottom of call_stack)
        if let Some(start_method) =
            self.class_loader
                .find_method(&normalized_cls, "startApp", "()V")
        {
            println!(
                "[STARTAPP] Found startApp with {} instructions",
                start_method.instructions.len()
            );
            for (idx, inst) in start_method.instructions.iter().enumerate() {
                println!("  [startApp:{}] {:?}", idx, inst);
            }
            let mut start_frame = Frame::new(
                normalized_cls.clone(),
                "startApp".to_string(),
                "()V".to_string(),
                Arc::clone(&start_method.instructions),
                start_method.max_locals,
                start_method.max_stack,
                start_method.exception_table.clone(),
                Arc::clone(&start_method.pc_map),
            );
            if start_method.is_synchronized() {
                start_frame.sync_monitor = Some(midlet_id);
            }
            start_frame.set_local(0, JavaValue::Object(Some(midlet_id)));
            main_thread_frames.push(start_frame);
        } else {
            eprintln!("[WARN] startApp ()V not found in {}", normalized_cls);
        }

        // 2. <init> frame (pushed second, on top of startApp in call_stack)
        if let Some(init_method) = self
            .class_loader
            .find_method(&normalized_cls, "<init>", "()V")
        {
            println!(
                "[INIT] Found <init> with {} instructions",
                init_method.instructions.len()
            );
            let mut init_frame = Frame::new(
                normalized_cls.clone(),
                "<init>".to_string(),
                "()V".to_string(),
                Arc::clone(&init_method.instructions),
                init_method.max_locals,
                init_method.max_stack,
                init_method.exception_table.clone(),
                Arc::clone(&init_method.pc_map),
            );
            if init_method.is_synchronized() {
                init_frame.sync_monitor = Some(midlet_id);
            }
            init_frame.set_local(0, JavaValue::Object(Some(midlet_id)));
            main_thread_frames.push(init_frame);
        } else {
            eprintln!("[WARN] <init> ()V not found in {}", normalized_cls);
        }

        if !main_thread_frames.is_empty() {
            let initial = main_thread_frames.remove(0);
            let tid = self.scheduler.allocate_thread_id();
            let mut thread = GreenThread::new(tid, "main".to_string(), initial, Some(midlet_id));
            while !main_thread_frames.is_empty() {
                thread.call_stack.push(main_thread_frames.remove(0));
            }
            for f in &thread.call_stack {
                if let Some(m) = f.sync_monitor {
                    if !self.scheduler.try_acquire_monitor(tid, m) {
                        self.scheduler.block_on_monitor(tid, m, 1);
                    }
                }
            }
            thread.strict_mode = self.context.strict_mode;
            self.scheduler.threads.push(thread);
        }

        self.is_running = true;
        Ok(app_name)
    }

    /// Loads and parses an Application Descriptor (JAD) file.
    pub fn load_jad(&mut self, jad_bytes: &[u8]) {
        if let Ok(text) = std::str::from_utf8(jad_bytes) {
            self.load_jad_str(text);
        }
    }

    /// Loads and parses an Application Descriptor (JAD) string.
    pub fn load_jad_str(&mut self, jad_str: &str) {
        self.context.jad = Some(j2me_core::jar::Jad::parse(jad_str));
    }

    /// Sets or overrides an application property directly in the JAD descriptor.
    pub fn set_app_property(&mut self, key: &str, value: &str) {
        if self.context.jad.is_none() {
            self.context.jad = Some(j2me_core::jar::Jad::default());
        }
        if let Some(ref mut jad) = self.context.jad {
            jad.properties.insert(key.to_string(), value.to_string());
        }
    }

    /// Dynamically adjust screen resolution (e.g., 128x128, 176x208, 240x320, 320x240).
    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.context.screen_width = width;
        self.context.screen_height = height;
        self.context.framebuffer = vec![0xFF000000; (width * height) as usize];
        self.context.game_canvas_buffers.clear();
        self.context.graphics.width = width as i32;
        self.context.graphics.height = height as i32;
        self.context.graphics.clip_x = 0;
        self.context.graphics.clip_y = 0;
        self.context.graphics.clip_w = width as i32;
        self.context.graphics.clip_h = height as i32;
        self.context.screen_dirty = true;
    }

    pub fn get_screen_resolution(&self) -> (u32, u32) {
        (self.context.screen_width, self.context.screen_height)
    }

    pub fn set_jar_filename(&mut self, filename: &str) {
        self.context.jar_filename = Some(filename.to_string());
    }

    /// Retrieves an application property following MIDP precedence (JAD overrides Manifest).
    pub fn get_app_property(&self, key: &str) -> Option<&str> {
        self.context.get_app_property(key)
    }

    pub fn record_uncaught_exception(&mut self, thread_idx: usize, eid: ObjectId) {
        let th_name = self.scheduler.threads[thread_idx].name.clone();
        let exc_name = if let Some(ObjectData::Instance(inst)) = self.heap.get(eid) {
            inst.class_name.clone()
        } else {
            "UnknownException".to_string()
        };
        eprintln!(
            "[VM ERROR] Thread '{}' unhandled exception: {} (ObjectId {:?})",
            th_name, exc_name, eid
        );
        let rec = self.scheduler.threads[thread_idx]
            .last_exception_event
            .as_ref()
            .map(|ev| crate::bridge::context::GuestExceptionRecord {
                exception_class: ev.exception_class.clone(),
                message: ev.message.clone(),
                thread_name: ev.thread_name.clone(),
                class_name: ev.class_name.clone(),
                method_name: ev.method_name.clone(),
                descriptor: ev.descriptor.clone(),
                pc: ev.pc,
                caught: false,
                stack_trace: ev.stack_trace.clone(),
            });
        self.context.exception_telemetry.last_uncaught = rec.clone();
        self.context
            .exception_telemetry
            .uncaught_exception_threads
            .push(th_name.clone());
        let other_alive = self
            .scheduler
            .threads
            .iter()
            .enumerate()
            .filter(|(i, t)| *i != thread_idx && t.is_alive())
            .count();
        if th_name == "main" || other_alive == 0 {
            self.context.exception_telemetry.fatal_guest_exception = rec;
        }
        self.scheduler.threads[thread_idx].state = ThreadState::Terminated;
    }

    /// Advance emulator by delta_ms
    pub fn tick(&mut self, delta_ms: f64) -> bool {
        if !self.is_running {
            return false;
        }

        let actual_delta = if delta_ms > 0.0 {
            delta_ms
        } else {
            1000.0 / (self.target_fps as f64)
        };

        let t_sched_start = Timer::start();
        self.scheduler.tick_time_rational(actual_delta);
        self.context.scheduler_time_us += t_sched_start.elapsed_micros();

        self.context.repaints_this_tick = 0;
        self.context.tick_budget_exhausted = false;

        // Drain pending input before executing VM instruction slice and render
        if self.last_displayable != self.context.current_displayable {
            if self.last_displayable.is_some() {
                self.input.canvas_changed();
            }
            self.last_displayable = self.context.current_displayable;
        }
        self.input.advance(self.scheduler.current_time_ms);
        self.dispatch_ui();

        // Reset per-frame yield count for all active threads
        for t in &mut self.scheduler.threads {
            t.yield_count_in_frame = 0;
        }

        let max_frame_instructions = if self.context.repaints_this_tick > 0 {
            300_000
        } else {
            500_000
        };
        let mut total_executed = 0;

        while total_executed < max_frame_instructions {
            let t_sched = Timer::start();
            let ready_idx = match self.scheduler.next_ready_thread_index() {
                Some(idx) => idx,
                None => {
                    if !self.context.ui_queue.is_empty() {
                        self.dispatch_ui();
                        if let Some(idx) = self.scheduler.next_ready_thread_index() {
                            idx
                        } else {
                            break;
                        }
                    } else if self.scheduler.threads.iter().filter(|t| t.is_alive()).all(|t| matches!(t.state, ThreadState::Blocked { .. })) && self.scheduler.active_thread_count() > 0 {
                        if self.scheduler.break_deadlocks() {
                            if let Some(idx) = self.scheduler.next_ready_thread_index() {
                                idx
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            };
            self.context.scheduler_time_us += t_sched.elapsed_micros();

            let t_vm = Timer::start();
            let prev_inst = self.scheduler.threads[ready_idx].instructions_executed;
            let step_result = {
                let thread = &mut self.scheduler.threads[ready_idx];
                Interpreter::step_slice(thread, &mut self.class_loader, &mut self.heap, 5000)
            };
            let executed = (self.scheduler.threads[ready_idx]
                .instructions_executed
                .saturating_sub(prev_inst)) as usize;
            total_executed += executed.max(1);
            self.context.vm_time_us += t_vm.elapsed_micros();

            let tid = self.scheduler.threads[ready_idx].id;
            while let Some(m) = self.scheduler.threads[ready_idx]
                .pending_monitor_releases
                .pop()
            {
                self.scheduler.release_monitor(tid, m);
            }

            match step_result {
                StepResult::Continue | StepResult::QuantumYield => {}
                StepResult::Sleep(ms) => {
                    self.scheduler.threads[ready_idx].state = ThreadState::Sleeping {
                        wake_time_ms: self.scheduler.current_time_ms + ms,
                    };
                    if ms <= 15 && self.context.repaints_this_tick >= self.max_repaints_per_tick {
                        self.context.tick_budget_exhausted = true;
                    }
                }
                StepResult::Yield => {
                    self.scheduler.threads[ready_idx].state = ThreadState::Ready;
                    if self.context.repaints_this_tick >= self.max_repaints_per_tick {
                        self.context.tick_budget_exhausted = true;
                    }
                }
                StepResult::Wait(monitor) => {
                    self.scheduler.threads[ready_idx].state = ThreadState::Waiting {
                        monitor,
                        timeout_wake_ms: None,
                        saved_recursion: 1,
                    };
                }
                StepResult::MonitorEnter(monitor) => {
                    let tid = self.scheduler.threads[ready_idx].id;
                    if !self.scheduler.try_acquire_monitor(tid, monitor) {
                        self.scheduler.block_on_monitor(tid, monitor, 1);
                    }
                }
                StepResult::MonitorExit(monitor) => {
                    let tid = self.scheduler.threads[ready_idx].id;
                    if !self.scheduler.release_monitor(tid, monitor) {
                        let owned_by_other = self
                            .scheduler
                            .monitors
                            .get(&monitor)
                            .and_then(|m| m.owner)
                            .map(|o| o != tid)
                            .unwrap_or(false);
                        if owned_by_other {
                            eprintln!("[EXC] IllegalMonitorStateException on monitorexit! tid={:?} m={:?} owner={:?}", tid, monitor, self.scheduler.get_monitor_owner(monitor));
                            let exc = self
                                .heap
                                .alloc_instance("java/lang/IllegalMonitorStateException");
                            if let StepResult::UnhandledException(eid) = Interpreter::handle_athrow(
                                &mut self.scheduler.threads[ready_idx],
                                &mut self.heap,
                                &self.class_loader,
                                exc,
                            ) {
                                self.record_uncaught_exception(ready_idx, eid);
                            }
                        }
                    }
                }
                StepResult::ThreadFinished => {
                    let java_th_obj = self.scheduler.threads[ready_idx].java_thread_obj;
                    let is_ui_dispatcher = self.scheduler.threads[ready_idx].name == UI_DISPATCHER_THREAD;
                    self.scheduler.threads[ready_idx].state = ThreadState::Terminated;
                    if let Some(target_obj) = java_th_obj {
                        self.scheduler.wake_threads_waiting_for(target_obj);
                    }
                    if is_ui_dispatcher {
                        self.scheduler.clean_terminated();
                        if !self.context.ui_queue.is_empty() {
                            self.dispatch_ui();
                        }
                    }
                }
                StepResult::NativeCall {
                    class_name,
                    method_name,
                    descriptor,
                    receiver,
                    args,
                } => {
                    total_executed += 5;
                    thread_local! {
                        static METHOD_KEY_BUF: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(128));
                    }
                    let already_recorded = METHOD_KEY_BUF.with(|buf| {
                        let mut b = buf.borrow_mut();
                        b.clear();
                        b.push_str(&class_name);
                        b.push('.');
                        b.push_str(&method_name);
                        self.context.apis_invoked.contains(&**b)
                    });
                    if !already_recorded {
                        let method_key = format!("{}.{}", class_name, method_name);
                        self.context.apis_invoked.insert(method_key.clone());
                        self.context.platform_apis_invoked.insert(method_key);
                    }
                    let res = NativeBridge::dispatch(
                        &class_name,
                        &method_name,
                        &descriptor,
                        receiver,
                        &args,
                        &mut self.heap,
                        &mut self.class_loader,
                        &mut self.scheduler,
                        &mut self.context,
                        ready_idx,
                    );
                    self.context.native_calls_count += 1;
                    if self.context.recent_native_calls.len() >= 8 {
                        self.context.recent_native_calls.pop_front();
                    }
                    if res.is_err() {
                        self.context.recent_native_calls.push_back(format!(
                            "{}.{}{} -> Err",
                            class_name, method_name, descriptor
                        ));
                    } else {
                        self.context.recent_native_calls.push_back(format!(
                            "{}.{}",
                            class_name, method_name
                        ));
                    }

                    let is_void = descriptor.ends_with(")V");
                    match res {
                        Ok(Some(return_val)) => {
                            if !is_void {
                                if let Some(top_frame) =
                                    self.scheduler.threads[ready_idx].current_frame_mut()
                                {
                                    top_frame.push(return_val);
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            if e.starts_with("EXC:") {
                                let parts: Vec<&str> = e.split(':').collect();
                                if parts.len() >= 2 {
                                    let exc_cls = parts[1].trim();
                                    let exc_id = if parts.len() >= 3 {
                                        if let Ok(raw_id) = parts[2].trim().parse::<u32>() {
                                            j2me_core::heap::object::ObjectId(raw_id)
                                        } else {
                                            let inst = self.heap.alloc_instance(exc_cls);
                                            let msg_sid = self.heap.alloc_string(parts[2].trim());
                                            self.heap.set_field(
                                                inst,
                                                "detailMessage",
                                                JavaValue::Object(Some(msg_sid)),
                                            );
                                            inst
                                        }
                                    } else {
                                        self.heap.alloc_instance(exc_cls)
                                    };
                                    if let StepResult::UnhandledException(eid) =
                                        Interpreter::handle_athrow(
                                            &mut self.scheduler.threads[ready_idx],
                                            &mut self.heap,
                                            &self.class_loader,
                                            exc_id,
                                        )
                                    {
                                        self.record_uncaught_exception(ready_idx, eid);
                                    }
                                    continue;
                                }
                            }
                            self.context.exception_telemetry.native_errors += 1;
                            self.context.unhandled_native_calls += 1;
                            eprintln!(
                                "[NATIVE ERROR] {}.{}{} : {}",
                                class_name, method_name, descriptor, e
                            );
                            let ret_desc = if let Some(idx) = descriptor.rfind(')') {
                                &descriptor[idx + 1..]
                            } else {
                                &descriptor[..]
                            };
                            if ret_desc != "V" {
                                if let Some(top_frame) =
                                    self.scheduler.threads[ready_idx].current_frame_mut()
                                {
                                    let fallback_val = if method_name == "read" && ret_desc == "I" {
                                        JavaValue::Int(-1)
                                    } else {
                                        JavaValue::default_for_descriptor(ret_desc)
                                    };
                                    top_frame.push(fallback_val);
                                }
                            }
                        }
                    }
                }
                StepResult::UnhandledException(exc_id) => {
                    self.record_uncaught_exception(ready_idx, exc_id);
                }
                StepResult::Error(err) => {
                    let frame_info = self.scheduler.threads[ready_idx]
                        .current_frame()
                        .map(|f| {
                            let inst_info = f
                                .instructions
                                .get(f.pc.saturating_sub(1))
                                .map(|i| format!("{:?}", i))
                                .unwrap_or_else(|| "unknown".to_string());
                            format!(
                                "{}.{}{} at pc {} (prev instr: {:?})",
                                f.class_name, f.method_name, f.descriptor, f.pc, inst_info
                            )
                        })
                        .unwrap_or_default();
                    eprintln!(
                        "[VM ERROR] Thread '{}' execution error: {} in {}",
                        self.scheduler.threads[ready_idx].name, err, frame_info
                    );
                    self.scheduler.threads[ready_idx].state = ThreadState::Terminated;
                }
            }

            if self.context.tick_budget_exhausted {
                self.context.tick_budget_exhausted = false;
                break;
            }
        }

        for thread in &mut self.scheduler.threads {
            if let Some(ref ev) = thread.last_exception_event {
                if !ev.caught && self.context.exception_telemetry.last_uncaught.is_none() {
                    self.context.exception_telemetry.last_uncaught = Some(crate::bridge::context::GuestExceptionRecord {
                        exception_class: ev.exception_class.clone(),
                        message: ev.message.clone(),
                        thread_name: ev.thread_name.clone(),
                        class_name: ev.class_name.clone(),
                        method_name: ev.method_name.clone(),
                        descriptor: ev.descriptor.clone(),
                        pc: ev.pc,
                        caught: false,
                        stack_trace: ev.stack_trace.clone(),
                    });
                }
            }
            while let Some(ev) = thread.auto_healing_events.pop() {
                let recent_natives: Vec<String> = self.context.recent_native_calls.iter().cloned().collect();
                self.context.exception_telemetry.auto_healing_events.push(crate::bridge::context::AutoHealingEvent {
                    exception_class: ev.exception_class,
                    message: ev.message,
                    thread_name: ev.thread_name,
                    class_name: ev.class_name,
                    method_name: ev.method_name,
                    descriptor: ev.descriptor,
                    pc: ev.pc,
                    stack_trace: ev.stack_trace,
                    recent_native_calls: recent_natives,
                    timestamp_ms: self.scheduler.current_time_ms,
                    can_continue: true,
                });
            }
        }

        self.scheduler.clean_terminated();

        self.context.exception_telemetry.guest_exceptions_thrown =
            self.scheduler.total_exceptions_thrown();
        self.context.exception_telemetry.guest_exceptions_caught =
            self.scheduler.total_exceptions_caught();
        self.context.exception_telemetry.critical_exceptions_caught =
            self.scheduler.total_critical_exceptions_caught();
        self.context.exception_telemetry.caught_exception_classes =
            self.scheduler.unique_caught_exception_classes();
        self.context.exception_telemetry.guest_exceptions_uncaught =
            self.scheduler.total_exceptions_uncaught();
        self.context.exception_telemetry.auto_healing_count =
            self.scheduler.total_auto_healings();
        self.context.exception_telemetry.last_auto_healing_signature =
            self.scheduler.last_auto_healing_signature();
        self.context.exceptions_count = self.context.exception_telemetry.guest_exceptions_thrown;

        // Advance timers and schedule tasks
        let ready_timer_tasks = self.context.timers.advance(self.scheduler.current_time_ms);
        for task_id in ready_timer_tasks {
            if let Some(ObjectData::Instance(inst)) = self.heap.get(task_id) {
                let task_cls = inst.class_name.clone();
                let th_name = format!("{}-TimerTask", task_cls);
                // In Java, Timer tasks run sequentially on the timer's background thread.
                // Do not spawn duplicate concurrent threads for an already-running timer task.
                if self.scheduler.threads.iter().any(|t| t.name == th_name && t.is_alive()) {
                    continue;
                }
                if let Some(run_m) = self.class_loader.find_method(&task_cls, "run", "()V") {
                    let mut task_frame = Frame::new(
                        task_cls.clone(),
                        "run".to_string(),
                        "()V".to_string(),
                        Arc::clone(&run_m.instructions),
                        run_m.max_locals,
                        run_m.max_stack,
                        run_m.exception_table.clone(),
                        Arc::clone(&run_m.pc_map),
                    );
                    if run_m.is_synchronized() {
                        task_frame.sync_monitor = Some(task_id);
                    }
                    task_frame.set_local(0, JavaValue::Object(Some(task_id)));
                    self.scheduler.spawn(
                        &th_name,
                        task_frame,
                        None,
                    );
                }
            }
        }

        // Input & UI event loop
        if self.last_displayable != self.context.current_displayable {
            if self.last_displayable.is_some() {
                self.input.canvas_changed();
            }
            self.last_displayable = self.context.current_displayable;
        }
        self.input.advance(self.scheduler.current_time_ms);
        self.dispatch_ui();

        if std::env::var_os("J2ME_DEBUG_STATE").is_some() {
            let states: Vec<String> = self
                .scheduler
                .threads
                .iter()
                .map(|t| {
                    format!(
                        "{}={:?}@{}.{}@{}",
                        t.name,
                        t.state,
                        t.call_stack.last().map(|f| f.class_name.as_str()).unwrap_or(""),
                        t.call_stack.last().map(|f| f.method_name.as_str()).unwrap_or(""),
                        t.call_stack.last().map(|f| f.pc).unwrap_or(0)
                    )
                })
                .collect();
            eprintln!("[STATE] t={} {:?}", self.scheduler.current_time_ms, states);
        }

        // Mark-and-Sweep Garbage Collection (periodic or on heap threshold)
        if (self.scheduler.current_time_ms % 4000 < 20 && self.heap.live_objects() > 2000)
            || self.heap.live_objects() > 25000
        {
            let mut roots = Vec::new();
            if let Some(id) = self.context.current_displayable {
                roots.push(id);
            }
            if let Some(id) = self.midlet_obj {
                roots.push(id);
            }
            for &p_id in self.context.players.keys() {
                roots.push(p_id);
            }
            for (&g_id, gc) in &self.context.graphics_contexts {
                if self.heap.get(g_id).is_some() {
                    match gc.target {
                        crate::bridge::context::GraphicsTarget::Image(img_id) => {
                            roots.push(img_id);
                        }
                        crate::bridge::context::GraphicsTarget::GameCanvas(cid) => {
                            roots.push(cid);
                        }
                        _ => {}
                    }
                }
            }
            let collected = j2me_core::gc::GarbageCollector::collect(
                &mut self.heap,
                &self.scheduler.threads,
                &self.class_loader,
                &roots,
            );
            if collected > 0 {
                self.context
                    .images
                    .retain(|id, _| self.heap.get(*id).is_some());
                self.context
                    .graphics_contexts
                    .retain(|id, _| self.heap.get(*id).is_some());
                self.context
                    .dg_to_graphics
                    .retain(|id, _| self.heap.get(*id).is_some());

                let active_gc_targets: std::collections::HashSet<
                    j2me_core::heap::object::ObjectId,
                > = self
                    .context
                    .graphics_contexts
                    .values()
                    .filter_map(|gc| match gc.target {
                        crate::bridge::context::GraphicsTarget::GameCanvas(cid) => Some(cid),
                        _ => None,
                    })
                    .collect();
                self.context
                    .game_canvas_buffers
                    .retain(|id, _| self.heap.get(*id).is_some() || active_gc_targets.contains(id));
                self.context
                    .game_canvas_suppress_keys
                    .retain(|id, _| self.heap.get(*id).is_some());
            }
        }

        // Render audio from active MMAPI players (e.g. ~735 samples per 60fps frame @ 44100 Hz)
        let t_audio = Timer::start();
        let audio_samples_per_frame =
            ((self.context.audio.sample_rate as f64) * (delta_ms / 1000.0)).round() as usize;
        let active_player_keys: Vec<j2me_core::heap::object::ObjectId> =
            self.context.players.keys().copied().collect();

        // Honest synchronous component-wise mixing across all active players
        let mut mixed_frame = vec![0.0f32; audio_samples_per_frame];
        let mut any_player_rendered = false;
        let mut player_buf = Vec::with_capacity(audio_samples_per_frame);

        for key in active_player_keys {
            if let Some(player) = self.context.players.get_mut(&key) {
                if player.state == crate::media::manager::STATE_STARTED {
                    player_buf.clear();
                    player.render_audio_samples(audio_samples_per_frame, &mut player_buf);
                    if !player_buf.is_empty() {
                        any_player_rendered = true;
                        for (i, &s) in player_buf.iter().enumerate() {
                            if i < mixed_frame.len() {
                                mixed_frame[i] += s;
                            }
                        }
                    }
                }
            }
        }

        if any_player_rendered {
            for s in mixed_frame {
                self.context.audio.write_sample(s.clamp(-1.0, 1.0));
            }
        }
        self.context.audio_time_us += t_audio.elapsed_micros();

        let t_rast = Timer::start();
        if let Some(disp_id) = self.context.current_displayable {
            if self.context.lists.contains_key(&disp_id) {
                self.render_list(disp_id);
            } else if self.context.text_boxes.contains_key(&disp_id) {
                self.render_text_box(disp_id);
            } else if self.context.forms.contains_key(&disp_id) {
                self.render_form(disp_id);
            }
        }
        self.render_softkey_bar();
        self.context.rasterizer_time_us += t_rast.elapsed_micros();

        if self.context.screen_dirty {
            self.context.fb_publishes_count += 1;
            let new_hash = crate::bridge::context::hash_framebuffer(&self.context.framebuffer);
            if new_hash != self.context.last_framebuffer_hash {
                self.context.last_framebuffer_hash = new_hash;
                self.context.framebuffer_changes_count += 1;
            }
        }
        let was_dirty = self.context.screen_dirty;
        self.context.screen_dirty = false;
        was_dirty
    }

    /// Perform explicit mark-and-sweep garbage collection, returning count of collected objects.
    pub fn force_gc(&mut self) -> usize {
        self.gc_count += 1;
        let mut roots = Vec::new();
        if let Some(id) = self.context.current_displayable {
            roots.push(id);
        }
        if let Some(id) = self.midlet_obj {
            roots.push(id);
        }
        for &p_id in self.context.players.keys() {
            roots.push(p_id);
        }
        for (&g_id, gc) in &self.context.graphics_contexts {
            if self.heap.get(g_id).is_some() {
                match gc.target {
                    crate::bridge::context::GraphicsTarget::Image(img_id) => {
                        roots.push(img_id);
                    }
                    crate::bridge::context::GraphicsTarget::GameCanvas(cid) => {
                        roots.push(cid);
                    }
                    _ => {}
                }
            }
        }
        for (&l_id, list) in &self.context.lists {
            roots.push(l_id);
            if let Some(cmd) = list.select_command {
                roots.push(cmd);
            }
            for elem in &list.elements {
                if let Some(img) = elem.image_part {
                    roots.push(img);
                }
            }
        }
        for &tb_id in self.context.text_boxes.keys() {
            roots.push(tb_id);
        }
        for (&f_id, form) in &self.context.forms {
            roots.push(f_id);
            for &item_id in &form.items {
                roots.push(item_id);
            }
        }
        for &si_id in self.context.string_items.keys() {
            roots.push(si_id);
        }
        for (&tl_id, tl) in &self.context.tiled_layers {
            roots.push(tl_id);
            if let Some(iid) = tl.image_id {
                roots.push(iid);
            }
        }
        for &spr_id in self.context.sprites.keys() {
            roots.push(spr_id);
        }
        for &d_id in self.context.dates.keys() {
            roots.push(d_id);
        }
        for &vc_id in self.context.volume_controls.keys() {
            roots.push(vc_id);
        }
        for (&p_id, &vc_id) in &self.context.player_volume_controls {
            roots.push(p_id);
            roots.push(vc_id);
        }
        for (&parent_id, children) in &self.context.m3g_children {
            roots.push(parent_id);
            for &c in children {
                roots.push(c);
            }
        }
        for &md_id in self.context.message_digests.keys() {
            roots.push(md_id);
        }
        for (&lm_id, lm) in &self.context.layer_managers {
            roots.push(lm_id);
            for &layer_id in &lm.layers {
                roots.push(layer_id);
            }
        }
        let collected = j2me_core::gc::GarbageCollector::collect(
            &mut self.heap,
            &self.scheduler.threads,
            &self.class_loader,
            &roots,
        );
        self.context
            .images
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .graphics_contexts
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .dg_to_graphics
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .lists
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .text_boxes
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .forms
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .string_items
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .tiled_layers
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .sprites
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .dates
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context
            .layer_managers
            .retain(|id, _| self.heap.get(*id).is_some());
        self.context.timers.purge_dead_objects(&self.heap);

        let active_gc_targets: std::collections::HashSet<j2me_core::heap::object::ObjectId> = self
            .context
            .graphics_contexts
            .values()
            .filter_map(|gc| match gc.target {
                crate::bridge::context::GraphicsTarget::GameCanvas(cid) => Some(cid),
                _ => None,
            })
            .collect();
        self.context
            .game_canvas_buffers
            .retain(|id, _| self.heap.get(*id).is_some() || active_gc_targets.contains(id));
        self.context
            .game_canvas_suppress_keys
            .retain(|id, _| self.heap.get(*id).is_some());

        collected
    }

    /// Renders standard LCDUI softkey bar for non-FullCanvas displayables.
    pub fn render_softkey_bar(&mut self) {
        let disp_id = match self.context.current_displayable {
            Some(id) => id,
            None => return,
        };

        // Check if FullCanvas or full screen mode
        let is_full = if let Some(ObjectData::Instance(inst)) = self.heap.get(disp_id) {
            let cls = inst.class_name.clone();
            self.class_loader
                .is_instance_of(&cls, "com/nokia/mid/ui/FullCanvas")
                || self
                    .context
                    .displayable_commands
                    .get(&disp_id)
                    .map(|e| e.is_full_screen)
                    .unwrap_or(false)
        } else {
            return;
        };

        if is_full {
            return;
        }

        let (soft1_cmd, soft2_cmd) = self.context.get_soft_commands(disp_id, &self.heap);
        if soft1_cmd.is_none() && soft2_cmd.is_none() {
            return;
        }

        let w = self.context.screen_width as usize;
        let h = self.context.screen_height as usize;
        let bar_h = 20usize;
        if h <= bar_h {
            return;
        }

        let bar_start = h - bar_h;
        // Dark bar background (0xFF1A2634)
        for y in bar_start..h {
            for x in 0..w {
                self.context.framebuffer[y * w + x] = 0xFF1A2634;
            }
        }
        // Top border line (0xFF3A4E66)
        for x in 0..w {
            self.context.framebuffer[bar_start * w + x] = 0xFF3A4E66;
        }

        let prev_color = self.context.graphics.current_color;
        self.context.graphics.current_color = 0xFFFFFFFF;

        // Render soft1 label on the left
        if let Some(cmd) = soft1_cmd {
            let label = self.heap.get_field(cmd, "label");
            if let Ok(Some(str_id)) = label.as_object() {
                if let Some(s) = self.heap.get_string_utf8(str_id) {
                    self.context.graphics.draw_string(
                        &mut self.context.framebuffer,
                        &s,
                        4,
                        (bar_start + 4) as i32,
                        0,
                    );
                }
            }
        }

        // Render soft2 label on the right
        if let Some(cmd) = soft2_cmd {
            let label = self.heap.get_field(cmd, "label");
            if let Ok(Some(str_id)) = label.as_object() {
                if let Some(s) = self.heap.get_string_utf8(str_id) {
                    let text_w = self.context.graphics.font.string_width(&s);
                    let x = (w as i32) - text_w - 4;
                    self.context.graphics.draw_string(
                        &mut self.context.framebuffer,
                        &s,
                        x.max(4),
                        (bar_start + 4) as i32,
                        0,
                    );
                }
            }
        }

        self.context.graphics.current_color = prev_color;
    }

    pub fn dispatch_command(&mut self, displayable: ObjectId, cmd_id: ObjectId) -> bool {
        if let Some(listener_id) = self
            .context
            .displayable_commands
            .get(&displayable)
            .and_then(|e| e.listener)
        {
            if let Some(ObjectData::Instance(l_inst)) = self.heap.get(listener_id) {
                let l_cls = l_inst.class_name.clone();
                if let Some(ca_m) = self.class_loader.find_method(
                    &l_cls,
                    "commandAction",
                    "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                ) {
                    let mut ca_frame = Frame::new(
                        l_cls,
                        "commandAction".to_string(),
                        "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V"
                            .to_string(),
                        Arc::clone(&ca_m.instructions),
                        ca_m.max_locals,
                        ca_m.max_stack,
                        ca_m.exception_table.clone(),
                        Arc::clone(&ca_m.pc_map),
                    );
                    if ca_m.is_synchronized() {
                        ca_frame.sync_monitor = Some(listener_id);
                    }
                    ca_frame.set_local(0, JavaValue::Object(Some(listener_id)));
                    ca_frame.set_local(1, JavaValue::Object(Some(cmd_id)));
                    ca_frame.set_local(2, JavaValue::Object(Some(displayable)));
                    self.scheduler
                        .spawn(UI_DISPATCHER_THREAD, ca_frame, None);
                    self.context.input_affected_game_state = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn render_list(&mut self, disp_id: ObjectId) {
        let list = match self.context.lists.get(&disp_id) {
            Some(l) => l.clone(),
            None => return,
        };
        let w = self.context.screen_width as i32;
        let h = self.context.screen_height as i32;
        if w <= 0 || h <= 0 {
            return;
        }

        // Dark background 0xFF1E293B
        for px in self.context.framebuffer.iter_mut() {
            *px = 0xFF1E293B;
        }

        let title_h = 24i32;
        let orig_color = self.context.graphics.current_color;
        self.context.graphics.current_color = 0xFF0F172A;
        self.context
            .graphics
            .fill_rect(&mut self.context.framebuffer, 0, 0, w, title_h);

        self.context.graphics.current_color = 0xFFFFFFFF;
        let title_str = if list.title.is_empty() {
            "Select"
        } else {
            &list.title
        };
        self.context
            .graphics
            .draw_string(&mut self.context.framebuffer, title_str, 8, 4, 0);

        self.context.graphics.current_color = 0xFF334155;
        self.context.graphics.draw_line(
            &mut self.context.framebuffer,
            0,
            title_h - 1,
            w,
            title_h - 1,
        );

        let item_h = 24i32;
        let start_y = title_h + 4;
        let visible_items = ((h - title_h - 24) / item_h).max(1);

        let scroll_top = if list.selected_index >= visible_items {
            list.selected_index - visible_items + 1
        } else {
            0
        };

        for i in 0..visible_items {
            let item_idx = (scroll_top + i) as usize;
            if item_idx >= list.elements.len() {
                break;
            }
            let item = &list.elements[item_idx];
            let item_y = start_y + i * item_h;
            let is_sel = item_idx as i32 == list.selected_index;

            if is_sel {
                self.context.graphics.current_color = 0xFF2563EB;
                self.context.graphics.fill_round_rect(
                    &mut self.context.framebuffer,
                    4,
                    item_y,
                    w - 8,
                    item_h - 2,
                    4,
                    4,
                );
                self.context.graphics.current_color = 0xFFFFFFFF;
            } else {
                self.context.graphics.current_color = 0xFFE2E8F0;
            }

            let prefix = match list.list_type {
                1 => {
                    if item.selected {
                        "(*) "
                    } else {
                        "( ) "
                    }
                }
                2 => {
                    if item.selected {
                        "[X] "
                    } else {
                        "[ ] "
                    }
                }
                _ => {
                    if is_sel {
                        "> "
                    } else {
                        "  "
                    }
                }
            };
            let label = format!("{}{}", prefix, item.string_part);
            self.context.graphics.draw_string(
                &mut self.context.framebuffer,
                &label,
                10,
                item_y + 4,
                0,
            );
        }

        self.context.graphics.current_color = orig_color;
        self.context.screen_dirty = true;
    }

    pub fn render_text_box(&mut self, disp_id: ObjectId) {
        let tb = match self.context.text_boxes.get(&disp_id) {
            Some(t) => t.clone(),
            None => return,
        };
        let w = self.context.screen_width as i32;
        let h = self.context.screen_height as i32;
        if w <= 0 || h <= 0 {
            return;
        }

        for px in self.context.framebuffer.iter_mut() {
            *px = 0xFF1E293B;
        }

        let title_h = 24i32;
        let orig_color = self.context.graphics.current_color;
        self.context.graphics.current_color = 0xFF0F172A;
        self.context
            .graphics
            .fill_rect(&mut self.context.framebuffer, 0, 0, w, title_h);

        self.context.graphics.current_color = 0xFFFFFFFF;
        let title_str = if tb.title.is_empty() {
            "Edit"
        } else {
            &tb.title
        };
        self.context
            .graphics
            .draw_string(&mut self.context.framebuffer, title_str, 8, 4, 0);

        self.context.graphics.current_color = 0xFF334155;
        self.context.graphics.draw_line(
            &mut self.context.framebuffer,
            0,
            title_h - 1,
            w,
            title_h - 1,
        );

        let box_y = title_h + 8;
        let box_h = (h - box_y - 28).max(40);
        self.context.graphics.current_color = 0xFF0F172A;
        self.context.graphics.fill_round_rect(
            &mut self.context.framebuffer,
            8,
            box_y,
            w - 16,
            box_h,
            6,
            6,
        );
        self.context.graphics.current_color = 0xFF475569;
        self.context.graphics.draw_round_rect(
            &mut self.context.framebuffer,
            8,
            box_y,
            w - 16,
            box_h,
            6,
            6,
        );

        self.context.graphics.current_color = 0xFFFFFFFF;
        let display_text = if tb.text.is_empty() {
            "|"
        } else {
            &tb.text
        };
        self.context.graphics.draw_string(
            &mut self.context.framebuffer,
            display_text,
            14,
            box_y + 8,
            0,
        );

        self.context.graphics.current_color = orig_color;
        self.context.screen_dirty = true;
    }

    pub fn render_form(&mut self, disp_id: ObjectId) {
        let form = match self.context.forms.get(&disp_id) {
            Some(f) => f.clone(),
            None => return,
        };
        let w = self.context.screen_width as i32;
        let h = self.context.screen_height as i32;
        if w <= 0 || h <= 0 {
            return;
        }

        // Dark background 0xFF1E293B
        for px in self.context.framebuffer.iter_mut() {
            *px = 0xFF1E293B;
        }

        let title_h = 24i32;
        self.context.graphics.current_color = 0xFF0F172A;
        self.context
            .graphics
            .fill_rect(&mut self.context.framebuffer, 0, 0, w, title_h);

        self.context.graphics.current_color = 0xFFFFFFFF;
        let title_str = if form.title.is_empty() {
            "Form"
        } else {
            &form.title
        };
        self.context
            .graphics
            .draw_string(&mut self.context.framebuffer, title_str, 8, 4, 0);

        self.context.graphics.current_color = 0xFF334155;
        self.context.graphics.draw_line(
            &mut self.context.framebuffer,
            0,
            title_h - 1,
            w,
            title_h - 1,
        );

        let mut cur_y = title_h + 8;
        for &item_id in &form.items {
            if cur_y >= h - 28 {
                break;
            }
            if let Some(si) = self.context.string_items.get(&item_id).cloned() {
                if let Some(ref label) = si.label {
                    self.context.graphics.current_color = 0xFF38BDF8; // cyan accent
                    self.context
                        .graphics
                        .draw_string(&mut self.context.framebuffer, label, 8, cur_y, 0);
                    cur_y += 16;
                }
                if let Some(ref text) = si.text {
                    self.context.graphics.current_color = 0xFFE2E8F0; // light gray
                    self.context
                        .graphics
                        .draw_string(&mut self.context.framebuffer, text, 8, cur_y, 0);
                    cur_y += 18;
                }
                cur_y += 4;
            }
        }

        self.context.screen_dirty = true;
    }

    pub fn key_event(&mut self, key_code: i32, pressed: bool) {
        self.context.input_delivered_count += 1;
        self.context.set_key(key_code, pressed);
        let now = self.scheduler.current_time_ms;
        if pressed {
            self.input.press(key_code, now);
        } else {
            self.input.release(key_code, now);
        }
        self.dispatch_ui();
    }

    /// Strictly serialized UI event dispatcher according to MIDP specification.
    /// Only one UI event thread executes at a time; all paint(), callSerially(),
    /// keyPressed(), keyReleased(), showNotify(), and hideNotify() handlers run
    /// sequentially on this dispatch loop.
    pub fn dispatch_ui(&mut self) {
        // Enqueue newly arrived key events if a Displayable is active
        while let Some(event) = self.input.next_event() {
            self.context
                .set_key(event.key, event.action == crate::input::KeyAction::Pressed);
            if let Some(disp_id) = self.context.current_displayable {
                self.context
                    .ui_queue
                    .post_key(event.key, event.action, disp_id);
            }
        }

        // If a UI event handler is currently active, wait for it to finish
        if self
            .scheduler
            .threads
            .iter()
            .any(|t| t.name == UI_DISPATCHER_THREAD && !matches!(t.state, ThreadState::Terminated))
        {
            return;
        }

        // Pop next event from FIFO queue
        while let Some(event) = self.context.ui_queue.pop_next() {
            match event {
                UiEvent::CallSerially(r_id) => {
                    if let Some(ObjectData::Instance(inst)) = self.heap.get(r_id) {
                        let r_cls = inst.class_name.clone();
                        if let Some(run_m) = self.class_loader.find_method(&r_cls, "run", "()V") {
                            let mut run_frame = Frame::new(
                                r_cls,
                                "run".to_string(),
                                "()V".to_string(),
                                Arc::clone(&run_m.instructions),
                                run_m.max_locals,
                                run_m.max_stack,
                                run_m.exception_table.clone(),
                                Arc::clone(&run_m.pc_map),
                            );
                            if run_m.is_synchronized() {
                                run_frame.sync_monitor = Some(r_id);
                            }
                            run_frame.set_local(0, JavaValue::Object(Some(r_id)));
                            self.scheduler
                                .spawn(UI_DISPATCHER_THREAD, run_frame, None);
                            return;
                        }
                    }
                }
                UiEvent::Paint(disp_id) => {
                    // Prevent concurrent re-entrant paint dispatch across threads
                    if self
                        .scheduler
                        .threads
                        .iter()
                        .any(|t| t.call_stack.iter().any(|f| f.method_name == "paint"))
                    {
                        self.context.ui_queue.coalesced += 1;
                        self.context.ui_queue.coalesced_repaints += 1;
                        return;
                    }
                    // Throttle asynchronous repaints to max_repaints_per_tick per display tick
                    if self.context.repaints_this_tick >= self.max_repaints_per_tick {
                        self.context.ui_queue.defer_paint(disp_id);
                        return;
                    }
                    if let Some(ObjectData::Instance(inst)) = self.heap.get(disp_id) {
                        let cls = inst.class_name.clone();
                        if let Some(paint_m) = self.class_loader.find_method(
                            &cls,
                            "paint",
                            "(Ljavax/microedition/lcdui/Graphics;)V",
                        ) {
                            let g_id = self
                                .heap
                                .alloc_instance("javax/microedition/lcdui/Graphics");
                            self.context.get_or_create_graphics(
                                g_id,
                                crate::bridge::context::GraphicsTarget::Display,
                            );
                            let mut paint_frame = Frame::new(
                                cls,
                                "paint".to_string(),
                                "(Ljavax/microedition/lcdui/Graphics;)V".to_string(),
                                Arc::clone(&paint_m.instructions),
                                paint_m.max_locals,
                                paint_m.max_stack,
                                paint_m.exception_table.clone(),
                                Arc::clone(&paint_m.pc_map),
                            );
                            if paint_m.is_synchronized() {
                                paint_frame.sync_monitor = Some(disp_id);
                            }
                            paint_frame.set_local(0, JavaValue::Object(Some(disp_id)));
                            paint_frame.set_local(1, JavaValue::Object(Some(g_id)));
                            self.context.paint_calls_count += 1;
                            self.context.repaints_this_tick += 1;
                            self.scheduler
                                .spawn(UI_DISPATCHER_THREAD, paint_frame, None);
                            return;
                        }
                    }
                }
                UiEvent::Key {
                    key,
                    action,
                    displayable,
                } => {
                    self.context
                        .set_key(key, action == crate::input::KeyAction::Pressed);

                    // MIDP 2.0 Command action dispatch for Soft1 (-6, -21) and Soft2 (-7, -22)
                    if (key == -6 || key == -7 || key == -21 || key == -22)
                        && action == crate::input::KeyAction::Pressed
                    {
                        let (soft1, soft2) =
                            self.context.get_soft_commands(displayable, &self.heap);
                        let target_cmd = if key == -6 || key == -21 { soft1 } else { soft2 };
                        if let Some(cmd_id) = target_cmd {
                            if self.dispatch_command(displayable, cmd_id) {
                                return;
                            }
                        }
                    }

                    // Keypad handling for active List / TextBox
                    if action == crate::input::KeyAction::Pressed {
                        if let Some(list) = self.context.lists.get_mut(&displayable) {
                            if key == -1 {
                                // UP
                                list.selected_index = (list.selected_index - 1).max(0);
                                self.context.screen_dirty = true;
                                self.context.input_affected_game_state = true;
                                return;
                            } else if key == -2 {
                                // DOWN
                                list.selected_index = (list.selected_index + 1)
                                    .min((list.elements.len() as i32 - 1).max(0));
                                self.context.screen_dirty = true;
                                self.context.input_affected_game_state = true;
                                return;
                            } else if key == -5 || key == 53 {
                                // FIRE or 5
                                self.context.input_affected_game_state = true;
                                let idx = list.selected_index as usize;
                                if list.list_type == 3 {
                                    // IMPLICIT: dispatch select_command or List.SELECT_COMMAND
                                    let select_cmd = list.select_command.unwrap_or_else(|| {
                                        let c = self
                                            .heap
                                            .alloc_instance("javax/microedition/lcdui/Command");
                                        let label_id = self.heap.alloc_string("Select");
                                        self.heap.set_field(
                                            c,
                                            "label",
                                            JavaValue::Object(Some(label_id)),
                                        );
                                        self.heap.set_field(c, "commandType", JavaValue::Int(1));
                                        self.heap.set_field(c, "priority", JavaValue::Int(0));
                                        c
                                    });
                                    if self.dispatch_command(displayable, select_cmd) {
                                        return;
                                    }
                                } else if list.list_type == 2 {
                                    // MULTIPLE
                                    if idx < list.elements.len() {
                                        list.elements[idx].selected = !list.elements[idx].selected;
                                        self.context.screen_dirty = true;
                                        return;
                                    }
                                } else if list.list_type == 1 {
                                    // EXCLUSIVE
                                    for (i, elem) in list.elements.iter_mut().enumerate() {
                                        elem.selected = i == idx;
                                    }
                                    self.context.screen_dirty = true;
                                    return;
                                }
                            }
                        }
                    }

                    let is_suppressed = self
                        .context
                        .game_canvas_suppress_keys
                        .get(&displayable)
                        .copied()
                        .unwrap_or(self.context.suppress_key_events);

                    // MIDP 2.0: suppressKeyEvents only suppresses keys mapped to game actions.
                    // Non-game keys (unmapped digits, star, pound, etc.) still dispatch.
                    let is_game_key = Self::key_to_game_action(key) != 0;
                    let should_suppress = is_suppressed && is_game_key;

                    if !should_suppress {
                        if let Some(ObjectData::Instance(inst)) = self.heap.get(displayable) {
                            let cls = inst.class_name.clone();
                            let method_name = action.method_name();
                            if let Some(key_m) =
                                self.class_loader.find_method(&cls, method_name, "(I)V")
                            {
                                let mut key_frame = Frame::new(
                                    cls.clone(),
                                    method_name.to_string(),
                                    "(I)V".to_string(),
                                    Arc::clone(&key_m.instructions),
                                    key_m.max_locals,
                                    key_m.max_stack,
                                    key_m.exception_table.clone(),
                                    Arc::clone(&key_m.pc_map),
                                );
                                if key_m.is_synchronized() {
                                    key_frame.sync_monitor = Some(displayable);
                                }
                                key_frame.set_local(0, JavaValue::Object(Some(displayable)));
                                key_frame.set_local(1, JavaValue::Int(key));
                                self.scheduler.spawn(
                                    UI_DISPATCHER_THREAD,
                                    key_frame,
                                    None,
                                );
                                self.context.input_affected_game_state = true;
                                return;
                            }
                        }
                    }
                }
                UiEvent::ShowNotify(disp_id) => {
                    if let Some(ObjectData::Instance(inst)) = self.heap.get(disp_id) {
                        let cls = inst.class_name.clone();
                        if let Some(show_m) =
                            self.class_loader.find_method(&cls, "showNotify", "()V")
                        {
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
                            if show_m.is_synchronized() {
                                show_frame.sync_monitor = Some(disp_id);
                            }
                            show_frame.set_local(0, JavaValue::Object(Some(disp_id)));
                            self.scheduler
                                .spawn(UI_DISPATCHER_THREAD, show_frame, None);
                            return;
                        }
                    }
                }
                UiEvent::HideNotify(disp_id) => {
                    if let Some(ObjectData::Instance(inst)) = self.heap.get(disp_id) {
                        let cls = inst.class_name.clone();
                        if let Some(hide_m) =
                            self.class_loader.find_method(&cls, "hideNotify", "()V")
                        {
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
                            if hide_m.is_synchronized() {
                                hide_frame.sync_monitor = Some(disp_id);
                            }
                            hide_frame.set_local(0, JavaValue::Object(Some(disp_id)));
                            self.scheduler
                                .spawn(UI_DISPATCHER_THREAD, hide_frame, None);
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn framebuffer_ptr(&self) -> *const u8 {
        self.context.framebuffer.as_ptr() as *const u8
    }

    pub fn framebuffer_len(&self) -> usize {
        self.context.framebuffer.len() * 4
    }

    pub fn read_audio_samples(&mut self, dest: &mut [f32]) -> usize {
        self.context.audio.read_samples(dest)
    }

    pub fn export_rms(&self) -> Vec<u8> {
        self.context.rms.serialize()
    }

    pub fn import_rms(&mut self, data: &[u8]) -> Result<(), String> {
        self.context.rms.deserialize(data)
    }

    pub fn key_to_game_action(key: i32) -> i32 {
        match key {
            -1 | 50 | 1 => 1,                 // UP (or KEY_NUM2 or Canvas.UP)
            -3 | 52 | 2 => 2,                 // LEFT (or KEY_NUM4 or Canvas.LEFT)
            -4 | 54 | 5 => 5,                 // RIGHT (or KEY_NUM6 or Canvas.RIGHT)
            -2 | 56 | 6 => 6,                 // DOWN (or KEY_NUM8 or Canvas.DOWN)
            -5 | 53 | 13 | 32 | 8 => 8,       // FIRE (or KEY_NUM5, Enter, Space or Canvas.FIRE)
            49 | -6 | -21 | 9 => 9,           // GAME_A / SOFT1 / Motorola -21
            51 | -7 | -22 | 10 => 10,         // GAME_B / SOFT2 / Motorola -22
            55 | 11 => 11,                    // GAME_C (KEY_NUM7)
            57 | 12 => 12,                    // GAME_D (KEY_NUM9)
            _ => 0,
        }
    }

    pub fn pause_app(&mut self) {
        if self.context.is_app_paused {
            return;
        }
        self.context.is_app_paused = true;
        if let Some(disp_id) = self.context.current_displayable {
            self.context.ui_queue.post_hide_notify(disp_id);
        }
        if let Some(midlet_id) = self.midlet_obj {
            if let Some(ObjectData::Instance(inst)) = self.heap.get(midlet_id) {
                let cls = inst.class_name.clone();
                if let Some(m) = self.class_loader.find_method(&cls, "pauseApp", "()V") {
                    let mut frame = Frame::new(
                        cls,
                        "pauseApp".to_string(),
                        "()V".to_string(),
                        Arc::clone(&m.instructions),
                        m.max_locals,
                        m.max_stack,
                        m.exception_table.clone(),
                        Arc::clone(&m.pc_map),
                    );
                    if m.is_synchronized() {
                        frame.sync_monitor = Some(midlet_id);
                    }
                    frame.set_local(0, JavaValue::Object(Some(midlet_id)));
                    self.scheduler.spawn("Lifecycle-pauseApp", frame, None);
                }
            }
        }
    }

    pub fn resume_app(&mut self) {
        if !self.context.is_app_paused {
            return;
        }
        self.context.is_app_paused = false;
        if let Some(disp_id) = self.context.current_displayable {
            self.context.ui_queue.post_show_notify(disp_id);
            self.context.request_repaint(disp_id);
        }
        if self.scheduler.threads.iter().any(|t| (t.name == "Lifecycle-startApp" || t.name == "main") && t.is_alive()) {
            return;
        }
        if let Some(midlet_id) = self.midlet_obj {
            if let Some(ObjectData::Instance(inst)) = self.heap.get(midlet_id) {
                let cls = inst.class_name.clone();
                if let Some(m) = self.class_loader.find_method(&cls, "startApp", "()V") {
                    let mut frame = Frame::new(
                        cls,
                        "startApp".to_string(),
                        "()V".to_string(),
                        Arc::clone(&m.instructions),
                        m.max_locals,
                        m.max_stack,
                        m.exception_table.clone(),
                        Arc::clone(&m.pc_map),
                    );
                    if m.is_synchronized() {
                        frame.sync_monitor = Some(midlet_id);
                    }
                    frame.set_local(0, JavaValue::Object(Some(midlet_id)));
                    self.scheduler.spawn("Lifecycle-startApp", frame, None);
                }
            }
        }
    }

    pub fn destroy_app(&mut self, unconditional: bool) {
        if let Some(midlet_id) = self.midlet_obj {
            if let Some(ObjectData::Instance(inst)) = self.heap.get(midlet_id) {
                let cls = inst.class_name.clone();
                if let Some(m) = self.class_loader.find_method(&cls, "destroyApp", "(Z)V") {
                    let mut frame = Frame::new(
                        cls,
                        "destroyApp".to_string(),
                        "(Z)V".to_string(),
                        Arc::clone(&m.instructions),
                        m.max_locals,
                        m.max_stack,
                        m.exception_table.clone(),
                        Arc::clone(&m.pc_map),
                    );
                    if m.is_synchronized() {
                        frame.sync_monitor = Some(midlet_id);
                    }
                    frame.set_local(0, JavaValue::Object(Some(midlet_id)));
                    frame.set_local(1, JavaValue::Int(if unconditional { 1 } else { 0 }));
                    self.scheduler.spawn("Lifecycle-destroyApp", frame, None);
                }
            }
        }
    }

    pub fn set_strict_mode(&mut self, strict: bool) {
        self.context.strict_mode = strict;
        self.scheduler.strict_mode = strict;
        for t in &mut self.scheduler.threads {
            t.strict_mode = strict;
        }
    }

    pub fn is_strict_mode(&self) -> bool {
        self.context.strict_mode
    }

    pub fn evaluate_observability(&self, wall_timeout: bool) -> crate::bridge::context::RuntimeObservabilityReport {
        let midlet_initialized = self.midlet_obj.is_some() || self.context.current_displayable.is_some();
        let jar_parsed = self.context.jar.is_some() || !self.class_loader.classes.is_empty() || midlet_initialized;
        let thread_running = self.scheduler.threads.iter().any(|t| t.instructions_executed > 0)
            || self.scheduler.total_instructions() > 0;
        let thread_sleeping = self.scheduler.threads.iter().any(|t| matches!(t.state, ThreadState::Sleeping { .. }));
        let thread_terminated = self.scheduler.threads.iter().any(|t| matches!(t.state, ThreadState::Terminated));
        let framebuffer_published = self.context.fb_publishes_count > 0;
        let framebuffer_changed = self.context.framebuffer_changes_count > 0;
        let input_delivered = self.context.input_delivered_count > 0;
        let input_affected_game_state = self.context.input_affected_game_state;
        let audio_generated = self.context.audio.generated_samples > 0;
        let rms_write_read_ok = self.context.rms_accessed_ok
            || self.context.rms.stores.values().any(|s| !s.records.is_empty());
        let host_crash = self.context.exception_telemetry.host_panics > 0;
        let fatal_guest_exception = self.context.exception_telemetry.fatal_guest_exception.is_some();
        let auto_heal_occurred = self.context.exception_telemetry.auto_healing_count > 0
            || !self.context.exception_telemetry.auto_healing_events.is_empty();
        let guest_exception = fatal_guest_exception
            || (self.context.exception_telemetry.last_uncaught.is_some() && !auto_heal_occurred);

        let active_threads = self.scheduler.active_thread_count();
        let deadlock_suspected = active_threads > 0 && self.scheduler.threads.iter().all(|t| {
            matches!(t.state, ThreadState::Blocked { .. } | ThreadState::Waiting { timeout_wake_ms: None, .. })
        });

        let criteria = crate::bridge::context::RuntimeObservabilityCriteria {
            jar_parsed,
            midlet_initialized,
            thread_running,
            thread_sleeping,
            thread_terminated,
            framebuffer_published,
            framebuffer_changed,
            input_delivered,
            input_affected_game_state,
            audio_generated,
            rms_write_read_ok,
            guest_exception,
            host_crash,
            auto_heal_occurred,
            wall_timeout,
            deadlock_suspected,
        };

        let mut blocker_summary = Vec::new();
        if host_crash {
            blocker_summary.push("Host crash / panic occurred during VM execution".to_string());
        }
        if fatal_guest_exception || guest_exception {
            if let Some(ref exc) = self.context.exception_telemetry.last_uncaught {
                blocker_summary.push(format!(
                    "Guest Java unhandled exception: {} in {}.{}{}:{}",
                    exc.exception_class, exc.class_name, exc.method_name, exc.descriptor, exc.pc
                ));
            } else {
                blocker_summary.push("Guest Java unhandled exception terminated thread".to_string());
            }
        }
        if auto_heal_occurred {
            blocker_summary.push(format!(
                "Auto-Healing intervened {} time(s) to suppress uncaught exceptions",
                self.context.exception_telemetry.auto_healing_count
            ));
        }
        if deadlock_suspected {
            blocker_summary.push("Suspected deadlock: all active threads are blocked or waiting indefinitely".to_string());
        }
        if wall_timeout {
            blocker_summary.push("Wall timeout reached".to_string());
        }

        // Verdict determination:
        // In strict mode, or standard mode: Auto-Healing MUST NEVER be masked as PASS!
        let verdict = if host_crash {
            crate::bridge::context::TestVerdict::Fail
        } else if fatal_guest_exception {
            crate::bridge::context::TestVerdict::Fail
        } else if wall_timeout {
            crate::bridge::context::TestVerdict::Timeout
        } else if deadlock_suspected {
            crate::bridge::context::TestVerdict::Fail
        } else if !jar_parsed || !midlet_initialized {
            crate::bridge::context::TestVerdict::Fail
        } else if auto_heal_occurred {
            // Auto-Healing occurred -> degraded verdict, preserving error details
            crate::bridge::context::TestVerdict::Degraded
        } else if guest_exception {
            crate::bridge::context::TestVerdict::Degraded
        } else if framebuffer_changed && thread_running {
            crate::bridge::context::TestVerdict::Pass
        } else if framebuffer_published || thread_running {
            crate::bridge::context::TestVerdict::Partial
        } else {
            crate::bridge::context::TestVerdict::Fail
        };

        crate::bridge::context::RuntimeObservabilityReport {
            verdict,
            strict_mode: self.context.strict_mode,
            criteria,
            instructions_executed: self.scheduler.total_instructions(),
            fb_publishes_count: self.context.fb_publishes_count,
            fb_changes_count: self.context.framebuffer_changes_count,
            input_events_count: self.context.input_delivered_count,
            swallowed_exceptions_count: self.context.exception_telemetry.critical_exceptions_caught as usize,
            unique_swallowed_classes: self.context.exception_telemetry.caught_exception_classes.clone(),
            auto_healing_events: self.context.exception_telemetry.auto_healing_events.clone(),
            last_uncaught_exception: self.context.exception_telemetry.last_uncaught.clone(),
            host_crash_error: None,
            blocker_summary,
        }
    }
}
