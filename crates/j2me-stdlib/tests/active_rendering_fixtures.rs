use j2me_stdlib::midp::lcdui::image::make_rgba;
use j2me_stdlib::J2meEmulator;
use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures/active_render/jars")
}

fn run_emulator_ticks(emu: &mut J2meEmulator, ticks: usize) {
    for _ in 0..ticks {
        emu.tick(50.0);
    }
}

#[test]
fn test_fixture_1_draw_no_flush_keeps_changes_in_offscreen_buffer() {
    let jar_path = fixtures_dir().join("Fixture1DrawNoFlush.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture1DrawNoFlush.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    // Red pixel in RGBA: make_rgba(255, 0, 0, 255)
    let expected_red = make_rgba(255, 0, 0, 255);

    // 1. Check offscreen buffer has the red rectangle
    let mut found_red_in_offscreen = false;
    for buf in emu.context.game_canvas_buffers.values() {
        let idx = 15 * 240 + 15;
        if buf.get(idx).copied() == Some(expected_red) {
            found_red_in_offscreen = true;
            break;
        }
    }
    assert!(
        found_red_in_offscreen,
        "GameCanvas offscreen buffer must contain drawn red pixel at (15, 15)"
    );

    // 2. Check display framebuffer DOES NOT have the red rectangle (because flushGraphics was not called)
    let display_pixel = emu.context.framebuffer[15 * 240 + 15];
    assert_ne!(
        display_pixel, expected_red,
        "Display framebuffer must NOT be updated when flushGraphics() is omitted"
    );
}

#[test]
fn test_fixture_2_full_flush_publishes_to_display_framebuffer() {
    let jar_path = fixtures_dir().join("Fixture2FullFlush.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture2FullFlush.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let expected_green = make_rgba(0, 255, 0, 255);
    let display_pixel = emu.context.framebuffer[15 * 240 + 15];
    assert_eq!(
        display_pixel, expected_green,
        "Display framebuffer must contain green pixel at (15, 15) after full flushGraphics()"
    );
    assert!(
        emu.context.fb_publishes_count > 0,
        "fb_publishes_count must be incremented after flushGraphics()"
    );
}

#[test]
fn test_fixture_3_regional_flush_transfers_only_specified_bounds() {
    let jar_path = fixtures_dir().join("Fixture3RegionalFlush.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture3RegionalFlush.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let expected_blue = make_rgba(0, 0, 255, 255);
    let expected_red = make_rgba(255, 0, 0, 255);

    // Box 1 at (15, 15) was within flushed region (10, 10, 20, 20) -> should be BLUE on display
    let disp_box1 = emu.context.framebuffer[15 * 240 + 15];
    assert_eq!(
        disp_box1, expected_blue,
        "Box 1 within flush region must be published to display"
    );

    // Box 2 at (55, 55) was OUTSIDE flushed region -> must NOT be on display!
    let disp_box2 = emu.context.framebuffer[55 * 240 + 55];
    assert_ne!(
        disp_box2, expected_red,
        "Box 2 outside flush region must NOT be published to display"
    );

    // But Box 2 MUST exist in GameCanvas backing buffer!
    let mut found_red_in_offscreen = false;
    for buf in emu.context.game_canvas_buffers.values() {
        let idx = 55 * 240 + 55;
        if buf.get(idx).copied() == Some(expected_red) {
            found_red_in_offscreen = true;
            break;
        }
    }
    assert!(
        found_red_in_offscreen,
        "Box 2 must exist in backing buffer even if un-flushed"
    );
}

#[test]
fn test_fixture_4_repeated_get_graphics_preserves_backing_buffer() {
    let jar_path = fixtures_dir().join("Fixture4RepeatedGetGraphics.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture4RepeatedGetGraphics.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let expected_blue = make_rgba(0, 0, 255, 255);
    let expected_green = make_rgba(0, 255, 0, 255);

    // Both drawings (drawn by g1 and g2 respectively) must be present in the flushed display
    let disp_box1 = emu.context.framebuffer[15 * 240 + 15];
    let disp_box2 = emu.context.framebuffer[45 * 240 + 45];

    assert_eq!(
        disp_box1, expected_blue,
        "First drawing (drawn via g1) must persist across getGraphics() calls"
    );
    assert_eq!(
        disp_box2, expected_green,
        "Second drawing (drawn via g2 after state reset) must be published"
    );
}

#[test]
fn test_fixture_5_two_game_canvases_have_isolated_buffers() {
    let jar_path = fixtures_dir().join("Fixture5TwoGameCanvases.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture5TwoGameCanvases.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    assert!(
        emu.context.game_canvas_buffers.len() >= 2,
        "Emulator must allocate separate backing buffers for each GameCanvas instance"
    );

    let expected_red = make_rgba(255, 0, 0, 255);
    let expected_blue = make_rgba(0, 0, 255, 255);

    // Display was set to CanvasA which drew RED at (15, 15)
    let disp_a = emu.context.framebuffer[15 * 240 + 15];
    let disp_b = emu.context.framebuffer[55 * 240 + 55];

    assert_eq!(
        disp_a, expected_red,
        "Display must show CanvasA's red drawing"
    );
    assert_ne!(
        disp_b, expected_blue,
        "Display must NOT show CanvasB's blue drawing because CanvasB was not flushed to display"
    );
}

#[test]
fn test_fixture_6_switching_displayable_isolates_inactive_flushes() {
    let jar_path = fixtures_dir().join("Fixture6SwitchingDisplayable.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture6SwitchingDisplayable.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let expected_green = make_rgba(0, 255, 0, 255);
    let expected_blue = make_rgba(0, 0, 255, 255);

    let disp_center = emu.context.framebuffer[100 * 240 + 100];
    assert_eq!(
        disp_center, expected_green,
        "Current displayable (Screen 2) must remain green; inactive Screen 1 flush must be rejected"
    );
    assert_ne!(
        disp_center, expected_blue,
        "Inactive Screen 1 must not overwrite active screen buffer"
    );
}

#[test]
fn test_fixture_7_fullscreen_and_ordinary_canvas_heights() {
    let jar_path = fixtures_dir().join("Fixture7FullscreenAndOrdinaryCanvas.jar");
    let jar_bytes =
        fs::read(&jar_path).expect("Fixture7FullscreenAndOrdinaryCanvas.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let norm_h = emu
        .class_loader
        .get_static(
            "fixtures/Fixture7FullscreenAndOrdinaryCanvas",
            "reportedNormalH",
            "I",
        )
        .as_int()
        .unwrap_or(0);
    let full_h = emu
        .class_loader
        .get_static(
            "fixtures/Fixture7FullscreenAndOrdinaryCanvas",
            "reportedFullH",
            "I",
        )
        .as_int()
        .unwrap_or(0);

    assert_eq!(
        norm_h,
        320 - 24,
        "Normal canvas height must reserve 24px softkey bar"
    );
    assert_eq!(
        full_h, 320,
        "Fullscreen canvas height must use entire 320px screen"
    );
}

#[test]
fn test_fixture_8_paint_and_active_render_coexistence() {
    let jar_path = fixtures_dir().join("Fixture8PaintAndActiveRender.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture8PaintAndActiveRender.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let expected_cyan = make_rgba(0, 255, 255, 255);
    let disp_active = emu.context.framebuffer[120 * 240 + 120];
    assert_eq!(
        disp_active, expected_cyan,
        "Active rendering flush must render cyan box without corruption"
    );
}

#[test]
fn test_fixture_9_suppress_key_events_contract() {
    let jar_path = fixtures_dir().join("Fixture9SuppressKeyEvents.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture9SuppressKeyEvents.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 5);

    // Send key press KEY_NUM1 (49)
    emu.key_event(49, true);
    run_emulator_ticks(&mut emu, 5);

    let key_pressed_count = emu
        .class_loader
        .get_static("fixtures/Fixture9SuppressKeyEvents", "keyPressedCount", "I")
        .as_int()
        .unwrap_or(-1);

    assert_eq!(
        key_pressed_count, 0,
        "When suppressKeyEvents=true, keyPressed callback must NOT be invoked"
    );

    // But getKeyStates() must latch the key state
    let key_states = emu.context.get_and_clear_key_states();
    assert_ne!(
        key_states, 0,
        "getKeyStates() must track physical key presses even when callbacks are suppressed"
    );
}

#[test]
fn test_fixture_10_direct_graphics_contracts_and_readback() {
    let jar_path = fixtures_dir().join("Fixture10DirectGraphics.jar");
    let jar_bytes = fs::read(&jar_path).expect("Fixture10DirectGraphics.jar must exist");

    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes).expect("JAR should load");
    run_emulator_ticks(&mut emu, 10);

    let npe = emu
        .class_loader
        .get_static("fixtures/Fixture10DirectGraphics", "npeThrown", "Z")
        .as_int()
        .unwrap_or(0);
    let iae = emu
        .class_loader
        .get_static("fixtures/Fixture10DirectGraphics", "iaeThrown", "Z")
        .as_int()
        .unwrap_or(0);
    let aioobe = emu
        .class_loader
        .get_static("fixtures/Fixture10DirectGraphics", "aioobeThrown", "Z")
        .as_int()
        .unwrap_or(0);
    let readback = emu
        .class_loader
        .get_static("fixtures/Fixture10DirectGraphics", "readbackPixel", "I")
        .as_int()
        .unwrap_or(0);

    assert_eq!(npe, 1, "NullPointerException must be thrown on null array");
    assert_eq!(
        iae, 1,
        "IllegalArgumentException must be thrown on invalid manipulation"
    );
    assert_eq!(
        aioobe, 1,
        "ArrayIndexOutOfBoundsException must be thrown on negative offset"
    );
    assert_eq!(
        readback as u32, 0xFFFF0000,
        "DirectGraphics getPixels readback must match drawn ARGB value 0xFFFF0000"
    );
}
