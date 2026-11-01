use j2me_stdlib::midp::lcdui::game::sprite::*;
use j2me_stdlib::midp::lcdui::image::Image;

/// Independent mathematical model of MIDP 2.0 Sprite Transformations
/// Derived from Sun Microsystems JSR-118 specification section javax.microedition.lcdui.game.Sprite
struct IndependentSpriteModel {
    src_w: i32,
    src_h: i32,
    ref_x: i32,
    ref_y: i32,
    x: i32,
    y: i32,
}

impl IndependentSpriteModel {
    fn new(w: i32, h: i32, rx: i32, ry: i32, x: i32, y: i32) -> Self {
        Self {
            src_w: w,
            src_h: h,
            ref_x: rx,
            ref_y: ry,
            x,
            y,
        }
    }

    /// Independent reference displacement calculation from first principles:
    /// Transformation maps local (u, v) in [0..W) x [0..H) to transformed (u', v').
    fn expected_ref_displacement(t: i32, w: i32, h: i32, rx: i32, ry: i32) -> (i32, i32) {
        match t {
            TRANS_NONE => (rx, ry),
            TRANS_MIRROR_ROT180 => (rx, h - 1 - ry),
            TRANS_MIRROR => (w - 1 - rx, ry),
            TRANS_ROT180 => (w - 1 - rx, h - 1 - ry),
            TRANS_MIRROR_ROT270 => (ry, rx),
            TRANS_ROT90 => (h - 1 - ry, rx),
            TRANS_ROT270 => (ry, w - 1 - rx),
            TRANS_MIRROR_ROT90 => (h - 1 - ry, w - 1 - rx),
            _ => panic!("Invalid transform {}", t),
        }
    }

    /// Expected display bounds after transformation
    fn expected_dimensions(t: i32, w: i32, h: i32) -> (i32, i32) {
        match t {
            TRANS_NONE | TRANS_MIRROR | TRANS_MIRROR_ROT180 | TRANS_ROT180 => (w, h),
            TRANS_ROT90 | TRANS_ROT270 | TRANS_MIRROR_ROT270 | TRANS_MIRROR_ROT90 => (h, w),
            _ => (w, h),
        }
    }

    /// Expected coordinate in painter space where reference pixel must appear
    fn target_display_ref_point(&self) -> (i32, i32) {
        (self.x + self.ref_x, self.y + self.ref_y)
    }

    /// Independently calculate expected (x, y) when transformed to new_t
    fn expected_position_after_transform(&self, new_t: i32) -> (i32, i32) {
        let (disp_rx, disp_ry) = self.target_display_ref_point();
        let (new_dx, new_dy) = Self::expected_ref_displacement(new_t, self.src_w, self.src_h, self.ref_x, self.ref_y);
        (disp_rx - new_dx, disp_ry - new_dy)
    }
}

#[test]
fn test_table_driven_all_8_transforms_non_square_frame() {
    let w = 16;
    let h = 24;
    let rx = 3;
    let ry = 19;
    let initial_x = 50;
    let initial_y = 70;

    let model = IndependentSpriteModel::new(w, h, rx, ry, initial_x, initial_y);
    let (target_disp_rx, target_disp_ry) = model.target_display_ref_point();
    assert_eq!((target_disp_rx, target_disp_ry), (53, 89));

    let img = Image::new_mutable(w, h);
    let mut spr = Sprite::new(img, w, h);
    spr.set_position(initial_x, initial_y);
    spr.define_reference_pixel(rx, ry);

    let all_transforms = [
        TRANS_NONE,
        TRANS_MIRROR_ROT180,
        TRANS_MIRROR,
        TRANS_ROT180,
        TRANS_MIRROR_ROT270,
        TRANS_ROT90,
        TRANS_ROT270,
        TRANS_MIRROR_ROT90,
    ];

    for &t in &all_transforms {
        // Reset to initial
        spr.set_transform(TRANS_NONE);
        spr.set_position(initial_x, initial_y);

        // Apply transform
        spr.set_transform(t);

        let (exp_x, exp_y) = model.expected_position_after_transform(t);
        assert_eq!(
            (spr.x, spr.y),
            (exp_x, exp_y),
            "Failed position assertion for transform {}",
            t
        );

        // Check reference pixel display location
        let (dx, dy) = IndependentSpriteModel::expected_ref_displacement(t, w, h, rx, ry);
        let actual_disp_rx = spr.x + dx;
        let actual_disp_ry = spr.y + dy;
        assert_eq!(
            (actual_disp_rx, actual_disp_ry),
            (target_disp_rx, target_disp_ry),
            "Reference pixel display coordinate drifted under transform {}",
            t
        );

        // Check dimensions
        let (exp_w, exp_h) = IndependentSpriteModel::expected_dimensions(t, w, h);
        let (_bx, _by, bw, bh) = spr.get_transformed_bounds();
        assert_eq!(
            (bw, bh),
            (exp_w, exp_h),
            "Dimensions incorrect under transform {}",
            t
        );
    }
}

#[test]
fn test_round_trip_transformation_preserves_initial_coordinates() {
    let w = 32;
    let h = 48;
    let rx = 7;
    let ry = 29;
    let start_x = 120;
    let start_y = 200;

    let img = Image::new_mutable(w, h);
    let mut spr = Sprite::new(img, w, h);
    spr.set_position(start_x, start_y);
    spr.define_reference_pixel(rx, ry);

    for t in 1..=7 {
        spr.set_transform(t);
        // Position shifted to preserve reference pixel
        assert_ne!((spr.x, spr.y), (start_x, start_y));

        // Return to TRANS_NONE
        spr.set_transform(TRANS_NONE);
        assert_eq!(
            (spr.x, spr.y),
            (start_x, start_y),
            "Round trip T0 -> T{} -> T0 must restore exact initial coordinates",
            t
        );
    }
}

#[test]
fn test_sequential_transformation_walk_preserves_reference_pixel() {
    let w = 20;
    let h = 30;
    let rx = 5;
    let ry = 15;
    let start_x = 100;
    let start_y = 100;

    let model = IndependentSpriteModel::new(w, h, rx, ry, start_x, start_y);
    let (target_rx, target_ry) = model.target_display_ref_point();

    let img = Image::new_mutable(w, h);
    let mut spr = Sprite::new(img, w, h);
    spr.set_position(start_x, start_y);
    spr.define_reference_pixel(rx, ry);

    // Sequential walk through all transforms
    let seq = [2, 5, 1, 6, 3, 7, 4, 0];
    for &t in &seq {
        spr.set_transform(t);
        let (dx, dy) = IndependentSpriteModel::expected_ref_displacement(t, w, h, rx, ry);
        assert_eq!(
            (spr.x + dx, spr.y + dy),
            (target_rx, target_ry),
            "Reference pixel shifted during transform walk at step {}",
            t
        );
    }

    assert_eq!((spr.x, spr.y), (start_x, start_y));
}

#[test]
fn test_pixel_alpha_collision_mapping_under_all_transforms() {
    let w = 8;
    let h = 12;
    let mut img = Image::new_mutable(w, h);
    for p in img.pixels.iter_mut() {
        *p = j2me_stdlib::midp::lcdui::image::make_rgba(0, 0, 0, 0);
    }

    // Paint opaque test pattern: pixel at source (2, 7) has alpha 255
    let test_sx = 2;
    let test_sy = 7;
    img.pixels[(test_sy * w + test_sx) as usize] = j2me_stdlib::midp::lcdui::image::make_rgba(0, 255, 0, 255);

    let mut spr = Sprite::new(img, w, h);
    spr.set_position(0, 0);
    spr.define_reference_pixel(0, 0);

    for t in 0..=7 {
        spr.set_transform(t);
        let (exp_w, exp_h) = IndependentSpriteModel::expected_dimensions(t, w, h);

        // Find the transformed coordinate where the test pixel moved
        let (dx, dy) = IndependentSpriteModel::expected_ref_displacement(t, w, h, test_sx, test_sy);

        // Verify that sampling at (spr.x + dx, spr.y + dy) correctly retrieves the opaque alpha
        assert!(dx >= 0 && dx < exp_w && dy >= 0 && dy < exp_h);
        let alpha = spr.get_pixel_alpha_at(spr.x + dx, spr.y + dy);
        assert_eq!(
            alpha, 255,
            "Pixel alpha at ({}, {}) under transform {} should be 255 but got {}",
            spr.x + dx, spr.y + dy, t, alpha
        );

        // Verify that another coordinate has alpha 0
        if !(test_sx == 0 && test_sy == 0) {
            let zero_alpha = spr.get_pixel_alpha_at(spr.x, spr.y);
            assert_eq!(zero_alpha, 0, "Pixel at (0,0) under transform {} should be 0", t);
        }
    }
}
