use crate::media::manager::{AudioRingBuffer, Player};
use crate::midp::lcdui::graphics::Graphics;
use crate::midp::lcdui::image::Image;
use crate::midp::rms::record_store::RmsManager;
use j2me_core::heap::object::ObjectId;
use j2me_core::jar::jar_loader::JarArchive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsTarget {
    Display,
    Image(ObjectId),
    GameCanvas(ObjectId),
}

#[derive(Debug, Clone)]
pub struct GraphicsContext {
    pub graphics: Graphics,
    pub target: GraphicsTarget,
}

pub struct VmContext {
    pub screen_width: u32,
    pub screen_height: u32,
    pub framebuffer: Vec<u32>, // RGBA8888 flat buffer
    pub screen_dirty: bool,
    pub repaint_requested: bool,
    pub graphics: Graphics,
    pub graphics_contexts: HashMap<ObjectId, GraphicsContext>,
    pub dg_to_graphics: HashMap<ObjectId, ObjectId>,
    pub images: HashMap<ObjectId, Image>,
    pub game_canvas_buffers: HashMap<ObjectId, Vec<u32>>,
    pub game_canvas_suppress_keys: HashMap<ObjectId, bool>,
    pub lists: HashMap<ObjectId, ListState>,
    pub text_boxes: HashMap<ObjectId, TextBoxState>,
    pub forms: HashMap<ObjectId, FormState>,
    pub string_items: HashMap<ObjectId, StringItemState>,
    pub sprites: HashMap<ObjectId, SpriteState>,
    pub tiled_layers: HashMap<ObjectId, TiledLayerState>,
    pub dates: HashMap<ObjectId, i64>,
    pub volume_controls: HashMap<ObjectId, VolumeControlState>,
    pub player_volume_controls: HashMap<ObjectId, ObjectId>,
    pub m3g_children: HashMap<ObjectId, Vec<ObjectId>>,
    pub message_digests: HashMap<ObjectId, MessageDigestState>,
    pub layer_managers: HashMap<ObjectId, LayerManagerState>,
    pub rms: RmsManager,
    pub audio: AudioRingBuffer,
    pub players: HashMap<ObjectId, Player>,
    pub current_displayable: Option<ObjectId>,
    pub key_states: i32,
    pub latched_key_states: i32,
    pub suppress_key_events: bool,
    pub paint_calls_count: u64,
    pub fb_publishes_count: u64,
    pub last_framebuffer_hash: u64,
    pub framebuffer_changes_count: u64,
    pub input_delivered_count: u64,
    pub input_affected_game_state: bool,
    pub rms_accessed_ok: bool,
    pub strict_mode: bool,
    pub repaints_this_tick: u32,
    pub max_repaints_per_tick: u32,
    pub tick_budget_exhausted: bool,
    pub target_fps: u32,
    pub ui_queue: crate::ui::UiEventQueue,
    pub timers: crate::timer::TimerManager,
    pub vibration_active: bool,
    pub lights_level: i32,
    pub jar: Option<JarArchive>,
    pub jad: Option<j2me_core::jar::Jad>,
    pub rng_seed: u64,
    pub displayable_commands: HashMap<ObjectId, DisplayableCommands>,
    pub native_calls_count: u64,
    pub exceptions_count: u64,
    pub exception_telemetry: ExceptionTelemetry,
    pub unhandled_native_calls: u64,
    pub vm_time_us: u64,
    pub scheduler_time_us: u64,
    pub rasterizer_time_us: u64,
    pub audio_time_us: u64,
    pub apis_invoked: std::collections::HashSet<String>,
    pub platform_apis_invoked: std::collections::HashSet<String>,
    pub guest_methods_invoked: std::collections::HashSet<String>,
    pub recent_native_calls: std::collections::VecDeque<String>,
    pub unhandled_natives: std::collections::HashSet<String>,
    pub draw_image_count: u32,
    pub draw_region_count: u32,
    pub fill_rect_count: u32,
    pub draw_string_count: u32,
    pub draw_pixels_count: u32,
    next_player_id: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestExceptionRecord {
    pub exception_class: String,
    pub message: Option<String>,
    pub thread_name: String,
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
    pub pc: usize,
    pub caught: bool,
    pub stack_trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoHealingEvent {
    pub exception_class: String,
    pub message: Option<String>,
    pub thread_name: String,
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
    pub pc: usize,
    pub stack_trace: Vec<String>,
    pub recent_native_calls: Vec<String>,
    pub timestamp_ms: u64,
    pub can_continue: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionTelemetry {
    pub guest_exceptions_thrown: u64,
    pub guest_exceptions_caught: u64,
    pub guest_exceptions_uncaught: u64,
    pub auto_healing_count: u32,
    pub last_auto_healing_signature: Option<String>,
    pub auto_healing_events: Vec<AutoHealingEvent>,
    pub uncaught_exception_threads: Vec<String>,
    pub fatal_guest_exception: Option<GuestExceptionRecord>,
    pub host_panics: u64,
    pub native_errors: u64,
    pub last_uncaught: Option<GuestExceptionRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestVerdict {
    Pass,
    Partial,
    Degraded,
    Fail,
    Timeout,
    NotTested,
}

impl std::fmt::Display for TestVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestVerdict::Pass => write!(f, "PASS"),
            TestVerdict::Partial => write!(f, "PARTIAL"),
            TestVerdict::Degraded => write!(f, "DEGRADED"),
            TestVerdict::Fail => write!(f, "FAIL"),
            TestVerdict::Timeout => write!(f, "TIMEOUT"),
            TestVerdict::NotTested => write!(f, "NOT_TESTED"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservabilityCriteria {
    pub jar_parsed: bool,
    pub midlet_initialized: bool,
    pub thread_running: bool,
    pub thread_sleeping: bool,
    pub thread_terminated: bool,
    pub framebuffer_published: bool,
    pub framebuffer_changed: bool,
    pub input_delivered: bool,
    pub input_affected_game_state: bool,
    pub audio_generated: bool,
    pub rms_write_read_ok: bool,
    pub guest_exception: bool,
    pub host_crash: bool,
    pub auto_heal_occurred: bool,
    pub wall_timeout: bool,
    pub deadlock_suspected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservabilityReport {
    pub verdict: TestVerdict,
    pub strict_mode: bool,
    pub criteria: RuntimeObservabilityCriteria,
    pub instructions_executed: u64,
    pub fb_publishes_count: u64,
    pub fb_changes_count: u64,
    pub input_events_count: u64,
    pub auto_healing_events: Vec<AutoHealingEvent>,
    pub last_uncaught_exception: Option<GuestExceptionRecord>,
    pub host_crash_error: Option<String>,
    pub blocker_summary: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DisplayableCommands {
    pub commands: Vec<ObjectId>,
    pub listener: Option<ObjectId>,
    pub is_full_screen: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ListElement {
    pub string_part: String,
    pub image_part: Option<ObjectId>,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct ListState {
    pub title: String,
    pub list_type: i32, // 1: EXCLUSIVE, 2: MULTIPLE, 3: IMPLICIT
    pub elements: Vec<ListElement>,
    pub selected_index: i32,
    pub fit_policy: i32,
    pub select_command: Option<ObjectId>,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            title: String::new(),
            list_type: 3,
            elements: Vec::new(),
            selected_index: 0,
            fit_policy: 0,
            select_command: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextBoxState {
    pub title: String,
    pub text: String,
    pub max_size: i32,
    pub constraints: i32,
    pub caret_position: i32,
}

#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub title: String,
    pub items: Vec<ObjectId>,
}

#[derive(Debug, Clone, Default)]
pub struct StringItemState {
    pub label: Option<String>,
    pub text: Option<String>,
    pub appearance_mode: i32,
    pub font: Option<ObjectId>,
}

#[derive(Debug, Clone, Default)]
pub struct SpriteState {
    pub ref_x: i32,
    pub ref_y: i32,
    pub frame_sequence: Option<Vec<i32>>,
    pub sequence_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TiledLayerState {
    pub columns: i32,
    pub rows: i32,
    pub tile_width: i32,
    pub tile_height: i32,
    pub image_id: Option<ObjectId>,
    pub cells: Vec<i32>,
    pub animated_tiles: Vec<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct LayerManagerState {
    pub layers: Vec<ObjectId>,
    pub view_x: i32,
    pub view_y: i32,
    pub view_w: i32,
    pub view_h: i32,
}

#[derive(Debug, Clone)]
pub struct VolumeControlState {
    pub level: i32,
    pub muted: bool,
    pub player_obj: Option<ObjectId>,
}

impl Default for VolumeControlState {
    fn default() -> Self {
        Self {
            level: 100,
            muted: false,
            player_obj: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MessageDigestState {
    pub algorithm: String,
    pub buffer: Vec<u8>,
}

impl VmContext {
    pub fn new(width: u32, height: u32, sample_rate: u32) -> Self {
        let size = (width * height) as usize;
        let mut ctx = Self {
            screen_width: width,
            screen_height: height,
            framebuffer: vec![0xFFFFFFFF; size], // Default white background
            screen_dirty: true,
            repaint_requested: true,
            graphics: Graphics::new(width as i32, height as i32),
            graphics_contexts: HashMap::new(),
            dg_to_graphics: HashMap::new(),
            images: HashMap::new(),
            game_canvas_buffers: HashMap::new(),
            game_canvas_suppress_keys: HashMap::new(),
            lists: HashMap::new(),
            text_boxes: HashMap::new(),
            forms: HashMap::new(),
            string_items: HashMap::new(),
            sprites: HashMap::new(),
            tiled_layers: HashMap::new(),
            dates: HashMap::new(),
            volume_controls: HashMap::new(),
            player_volume_controls: HashMap::new(),
            m3g_children: HashMap::new(),
            message_digests: HashMap::new(),
            layer_managers: HashMap::new(),
            rms: RmsManager::new(),
            audio: AudioRingBuffer::new(sample_rate),
            players: HashMap::new(),
            current_displayable: None,
            key_states: 0,
            latched_key_states: 0,
            suppress_key_events: false,
            paint_calls_count: 0,
            fb_publishes_count: 0,
            last_framebuffer_hash: 0,
            framebuffer_changes_count: 0,
            input_delivered_count: 0,
            input_affected_game_state: false,
            rms_accessed_ok: false,
            strict_mode: false,
            repaints_this_tick: 0,
            max_repaints_per_tick: 1,
            tick_budget_exhausted: false,
            target_fps: 30,
            ui_queue: crate::ui::UiEventQueue::new(),
            timers: crate::timer::TimerManager::new(),
            vibration_active: false,
            lights_level: 100,
            jar: None,
            jad: None,
            rng_seed: 0x123456789ABCDEF0,
            displayable_commands: HashMap::new(),
            native_calls_count: 0,
            exceptions_count: 0,
            exception_telemetry: ExceptionTelemetry::default(),
            unhandled_native_calls: 0,
            vm_time_us: 0,
            scheduler_time_us: 0,
            rasterizer_time_us: 0,
            audio_time_us: 0,
            apis_invoked: std::collections::HashSet::new(),
            platform_apis_invoked: std::collections::HashSet::new(),
            guest_methods_invoked: std::collections::HashSet::new(),
            recent_native_calls: std::collections::VecDeque::with_capacity(8),
            unhandled_natives: std::collections::HashSet::new(),
            draw_image_count: 0,
            draw_region_count: 0,
            fill_rect_count: 0,
            draw_string_count: 0,
            draw_pixels_count: 0,
            next_player_id: 1,
        };
        ctx.render_standby();
        ctx
    }

    pub fn add_command(
        &mut self,
        disp_id: ObjectId,
        cmd_id: ObjectId,
        heap: &j2me_core::heap::Heap,
    ) {
        let entry = self.displayable_commands.entry(disp_id).or_default();
        if !entry.commands.contains(&cmd_id) {
            entry.commands.push(cmd_id);
            // Sort by priority following MIDP 2.0 contract
            entry
                .commands
                .sort_by_key(|&c| heap.get_field(c, "priority").as_int().unwrap_or(0));
            self.screen_dirty = true;
        }
    }

    pub fn remove_command(&mut self, disp_id: ObjectId, cmd_id: ObjectId) {
        if let Some(entry) = self.displayable_commands.get_mut(&disp_id) {
            entry.commands.retain(|&c| c != cmd_id);
            self.screen_dirty = true;
        }
    }

    pub fn set_command_listener(&mut self, disp_id: ObjectId, listener: Option<ObjectId>) {
        let entry = self.displayable_commands.entry(disp_id).or_default();
        entry.listener = listener;
    }

    /// Evaluates left and right softkey mapping for given Displayable following MIDP 2.0 rules:
    /// Soft1: OK (4), ITEM (8), SCREEN (1), HELP (5)
    /// Soft2: BACK (7), CANCEL (3), EXIT (6), STOP (2)
    pub fn get_soft_commands(
        &self,
        disp_id: ObjectId,
        heap: &j2me_core::heap::Heap,
    ) -> (Option<ObjectId>, Option<ObjectId>) {
        if let Some(entry) = self.displayable_commands.get(&disp_id) {
            if entry.commands.is_empty() {
                return (None, None);
            }
            if entry.commands.len() == 1 {
                let cmd = entry.commands[0];
                let cmd_type = heap.get_field(cmd, "commandType").as_int().unwrap_or(1);
                // Back/Cancel/Exit/Stop belong to right softkey (Soft2)
                if matches!(cmd_type, 2 | 3 | 6 | 7) {
                    return (None, Some(cmd));
                } else {
                    return (Some(cmd), None);
                }
            }
            let mut soft1 = None;
            let mut soft2 = None;
            for &cmd in &entry.commands {
                let cmd_type = heap.get_field(cmd, "commandType").as_int().unwrap_or(1);
                if matches!(cmd_type, 2 | 3 | 6 | 7) {
                    if soft2.is_none() {
                        soft2 = Some(cmd);
                    } else if soft1.is_none() {
                        soft1 = Some(cmd);
                    }
                } else if soft1.is_none() {
                    soft1 = Some(cmd);
                } else if soft2.is_none() {
                    soft2 = Some(cmd);
                }
            }
            (soft1, soft2)
        } else {
            (None, None)
        }
    }

    /// Resolves an application property following MIDP specification:
    /// JAD descriptor properties take precedence over JAR MANIFEST.MF properties.
    pub fn get_app_property(&self, key: &str) -> Option<&str> {
        j2me_core::jar::Jad::resolve_property(
            self.jad.as_ref(),
            self.jar.as_ref().and_then(|j| j.manifest.as_ref()),
            key,
        )
    }

    pub fn post_call_serially(&mut self, runnable_id: ObjectId) {
        self.ui_queue.post_call_serially(runnable_id);
    }

    pub fn request_repaint(&mut self, disp_id: ObjectId) {
        self.repaint_requested = true;
        self.ui_queue.request_repaint(disp_id);
    }

    pub fn get_or_create_game_canvas_buffer(&mut self, canvas_id: ObjectId) -> &mut Vec<u32> {
        let size = (self.screen_width * self.screen_height) as usize;
        self.game_canvas_buffers
            .entry(canvas_id)
            .or_insert_with(|| vec![0xFFFFFFFF; size])
    }

    pub fn get_or_create_graphics(
        &mut self,
        g_id: ObjectId,
        target: GraphicsTarget,
    ) -> &mut GraphicsContext {
        let (w, h) = match target {
            GraphicsTarget::Display => (self.screen_width as i32, self.screen_height as i32),
            GraphicsTarget::Image(img_id) => {
                if let Some(i) = self.images.get(&img_id) {
                    (i.width, i.height)
                } else {
                    (self.screen_width as i32, self.screen_height as i32)
                }
            }
            GraphicsTarget::GameCanvas(_) => (self.screen_width as i32, self.screen_height as i32),
        };
        self.graphics_contexts
            .entry(g_id)
            .or_insert_with(|| GraphicsContext {
                graphics: Graphics::new(w, h),
                target,
            })
    }

    pub fn render_standby(&mut self) {
        let w = self.screen_width as i32;
        let h = self.screen_height as i32;
        // Background: deep retro blue
        for y in 0..h {
            for x in 0..w {
                let grad = ((y * 40) / h) as u8;
                self.framebuffer[(y * w + x) as usize] =
                    crate::midp::lcdui::image::make_rgba(15, 23 + grad / 2, 42 + grad, 255);
            }
        }

        // Top bar
        self.graphics.set_color_rgb(0x1e293b);
        self.graphics.fill_rect(&mut self.framebuffer, 0, 0, w, 22);

        // Signal icon (green bars)
        self.graphics.set_color_rgb(0x10b981);
        self.graphics.fill_rect(&mut self.framebuffer, 6, 12, 2, 6);
        self.graphics.fill_rect(&mut self.framebuffer, 10, 10, 2, 8);
        self.graphics.fill_rect(&mut self.framebuffer, 14, 8, 2, 10);
        self.graphics.fill_rect(&mut self.framebuffer, 18, 6, 2, 12);

        // Operator
        self.graphics.set_color_rgb(0xffffff);
        self.graphics
            .draw_string(&mut self.framebuffer, "NOKIA", 26, 4, 0);

        // Battery icon
        self.graphics.set_color_rgb(0x64748b);
        self.graphics
            .draw_rect(&mut self.framebuffer, w - 24, 6, 16, 9);
        self.graphics.set_color_rgb(0x10b981);
        self.graphics
            .fill_rect(&mut self.framebuffer, w - 22, 8, 12, 5);

        // Center Clock & Info
        self.graphics.set_color_rgb(0x38bdf8);
        self.graphics
            .draw_string(&mut self.framebuffer, "12:00", w / 2 - 20, 70, 0);

        self.graphics.set_color_rgb(0x94a3b8);
        self.graphics
            .draw_string(&mut self.framebuffer, "23.10.2026", w / 2 - 38, 90, 0);

        self.graphics.set_color_rgb(0xf8fafc);
        self.graphics
            .draw_string(&mut self.framebuffer, "J2ME VM READY", w / 2 - 50, 160, 0);

        self.graphics.set_color_rgb(0x38bdf8);
        self.graphics.draw_string(
            &mut self.framebuffer,
            "DROP .JAR TO PLAY",
            w / 2 - 64,
            185,
            0,
        );

        // Bottom Softkeys
        self.graphics.set_color_rgb(0x1e293b);
        self.graphics
            .fill_rect(&mut self.framebuffer, 0, h - 22, w, 22);

        self.graphics.set_color_rgb(0xffffff);
        self.graphics
            .draw_string(&mut self.framebuffer, "Menu", 8, h - 18, 0);
        self.graphics
            .draw_string(&mut self.framebuffer, "Names", w - 48, h - 18, 0);

        self.screen_dirty = true;
    }

    pub fn set_key(&mut self, key_code: i32, pressed: bool) {
        let bit = match key_code {
            -1 | 50 | 1 => 1 << 1,           // UP (or KEY_NUM2 or Canvas.UP)
            -3 | 52 | 2 => 1 << 2,           // LEFT (or KEY_NUM4 or Canvas.LEFT)
            -4 | 54 | 5 => 1 << 5,           // RIGHT (or KEY_NUM6 or Canvas.RIGHT)
            -2 | 56 | 6 => 1 << 6,           // DOWN (or KEY_NUM8 or Canvas.DOWN)
            -5 | 53 | 8 | 13 | 32 => 1 << 8, // FIRE (or KEY_NUM5, Enter(13), Space(32) or Canvas.FIRE)
            49 | -6 | 9 => 1 << 9,           // GAME_A / SOFT1 / Canvas.GAME_A
            51 | -7 | 10 => 1 << 10,         // GAME_B / SOFT2 / Canvas.GAME_B
            55 | 11 => 1 << 11,              // GAME_C / KEY_NUM7 / Canvas.GAME_C
            57 | 12 => 1 << 12,              // GAME_D / KEY_NUM9 / Canvas.GAME_D
            _ => 0,
        };
        if pressed {
            self.key_states |= bit;
            self.latched_key_states |= bit;
        } else {
            self.key_states &= !bit;
        }
    }

    pub fn get_and_clear_key_states(&mut self) -> i32 {
        let res = self.key_states | self.latched_key_states;
        self.latched_key_states = 0;
        res
    }

    pub fn alloc_player(&mut self, content_type: &str, media_data: Vec<u8>) -> (i32, Player) {
        let id = self.next_player_id;
        self.next_player_id += 1;
        let player = Player::new(id, content_type, media_data, self.audio.sample_rate);
        (id, player)
    }
}

/// Compute a fast 64-bit FNV-1a hash of a framebuffer to detect actual visual mutations.
pub fn hash_framebuffer(fb: &[u32]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i + 1 < fb.len() {
        let p = (fb[i] as u64) | ((fb[i + 1] as u64) << 32);
        h ^= p;
        h = h.wrapping_mul(0x100000001b3);
        i += 2;
    }
    if i < fb.len() {
        h ^= fb[i] as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRE_BIT: i32 = 1 << 8;
    const UP_BIT: i32 = 1 << 1;

    #[test]
    fn key_states_do_not_stick_after_release() {
        let mut ctx = VmContext::new(240, 320, 44100);
        assert_eq!(ctx.get_and_clear_key_states(), 0);

        ctx.set_key(-5, true); // FIRE
        assert_ne!(
            ctx.get_and_clear_key_states() & FIRE_BIT,
            0,
            "latched press"
        );
        // Still physically held: the level bit must remain set.
        assert_ne!(ctx.get_and_clear_key_states() & FIRE_BIT, 0, "held key");

        ctx.set_key(-5, false);
        assert_eq!(ctx.get_and_clear_key_states() & FIRE_BIT, 0, "released");
        assert_eq!(ctx.get_and_clear_key_states(), 0, "no lingering state");
    }

    #[test]
    fn multiple_keys_latch_independently() {
        let mut ctx = VmContext::new(240, 320, 44100);
        ctx.set_key(-1, true); // UP
        ctx.set_key(53, true); // KEY_NUM5 -> FIRE bit

        let states = ctx.get_and_clear_key_states();
        assert_ne!(states & UP_BIT, 0);
        assert_ne!(states & FIRE_BIT, 0);

        ctx.set_key(-1, false);
        let states = ctx.get_and_clear_key_states();
        assert_eq!(states & UP_BIT, 0, "UP released");
        assert_ne!(states & FIRE_BIT, 0, "FIRE still held");
    }

    #[test]
    fn short_press_between_polls_is_latched_and_cleared() {
        let mut ctx = VmContext::new(240, 320, 44100);
        // Key is pressed and released between polls
        ctx.set_key(-5, true);
        ctx.set_key(-5, false);

        // Current physical state is released, but latched bit must capture the transient press
        let states = ctx.get_and_clear_key_states();
        assert_ne!(states & FIRE_BIT, 0, "Pulse was captured");

        // Subsequent poll without new input must return 0
        let next_states = ctx.get_and_clear_key_states();
        assert_eq!(next_states & FIRE_BIT, 0, "Pulse was cleared after poll");
    }

    #[test]
    fn suppress_key_events_flag() {
        let mut ctx = VmContext::new(240, 320, 44100);
        assert!(!ctx.suppress_key_events);
        ctx.suppress_key_events = true;
        assert!(ctx.suppress_key_events);
    }
}
