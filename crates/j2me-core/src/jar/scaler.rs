//! Letterbox and Integer Scaler for Resolution-Agnostic J2ME Rendering.
//!
//! Provides aspect-ratio preservation, pixel-perfect integer scaling,
//! and centered letterbox/pillarbox padding onto arbitrary target surfaces.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    /// Integer-only scaling (1x, 2x, 3x, etc.) centered with black letterbox/pillarbox borders.
    Integer,
    /// Preserves exact aspect ratio filling maximum available dimension, centered with letterbox/pillarbox.
    FitAspectRatio,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledViewport {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub scale_x: f64,
    pub scale_y: f64,
}

/// Calculate the scaled and centered viewport rectangle for a source image inside a target area.
pub fn calculate_viewport(
    src_w: usize,
    src_h: usize,
    target_w: usize,
    target_h: usize,
    mode: ScaleMode,
) -> ScaledViewport {
    if src_w == 0 || src_h == 0 || target_w == 0 || target_h == 0 {
        return ScaledViewport {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            scale_x: 1.0,
            scale_y: 1.0,
        };
    }

    match mode {
        ScaleMode::Integer => {
            let scale = ((target_w / src_w).min(target_h / src_h)).max(1);
            let scaled_w = src_w * scale;
            let scaled_h = src_h * scale;
            let x = (target_w.saturating_sub(scaled_w)) / 2;
            let y = (target_h.saturating_sub(scaled_h)) / 2;
            ScaledViewport {
                x,
                y,
                width: scaled_w,
                height: scaled_h,
                scale_x: scale as f64,
                scale_y: scale as f64,
            }
        }
        ScaleMode::FitAspectRatio => {
            let scale_x = target_w as f64 / src_w as f64;
            let scale_y = target_h as f64 / src_h as f64;
            let scale = scale_x.min(scale_y);
            let scaled_w = ((src_w as f64 * scale).round() as usize).min(target_w).max(1);
            let scaled_h = ((src_h as f64 * scale).round() as usize).min(target_h).max(1);
            let x = (target_w.saturating_sub(scaled_w)) / 2;
            let y = (target_h.saturating_sub(scaled_h)) / 2;
            ScaledViewport {
                x,
                y,
                width: scaled_w,
                height: scaled_h,
                scale_x: scale,
                scale_y: scale,
            }
        }
    }
}

/// Scale and render `src` pixels into `target` preserving aspect ratio with letterboxing.
pub fn render_letterboxed(
    src: &[u32],
    src_w: usize,
    src_h: usize,
    target: &mut [u32],
    target_w: usize,
    target_h: usize,
    mode: ScaleMode,
    border_color: u32,
) -> ScaledViewport {
    if target.len() < target_w * target_h {
        return ScaledViewport {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            scale_x: 1.0,
            scale_y: 1.0,
        };
    }

    // Fill target with border color
    target[..target_w * target_h].fill(border_color);

    let vp = calculate_viewport(src_w, src_h, target_w, target_h, mode);
    if vp.width == 0 || vp.height == 0 {
        return vp;
    }

    for row in 0..vp.height {
        let dst_y = vp.y + row;
        if dst_y >= target_h {
            break;
        }
        let src_y = ((row as f64 / vp.height as f64) * src_h as f64) as usize;
        let src_y = src_y.min(src_h - 1);

        let dst_row_start = dst_y * target_w;
        let src_row_start = src_y * src_w;

        for col in 0..vp.width {
            let dst_x = vp.x + col;
            if dst_x >= target_w {
                break;
            }
            let src_x = ((col as f64 / vp.width as f64) * src_w as f64) as usize;
            let src_x = src_x.min(src_w - 1);

            let src_pixel = src[src_row_start + src_x];
            target[dst_row_start + dst_x] = src_pixel;
        }
    }

    vp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_scaling_176x208_into_720x1080() {
        // 176 * 3 = 528 (<= 720), 208 * 3 = 624 (<= 1080)
        // 176 * 4 = 704 (<= 720), 208 * 4 = 832 (<= 1080)
        // 176 * 5 = 880 (> 720)
        // So integer scale = 4x
        let vp = calculate_viewport(176, 208, 720, 1080, ScaleMode::Integer);
        assert_eq!(vp.width, 176 * 4);
        assert_eq!(vp.height, 208 * 4);
        assert_eq!(vp.scale_x, 4.0);
        // Centered horizontally: (720 - 704) / 2 = 8
        assert_eq!(vp.x, 8);
        // Centered vertically: (1080 - 832) / 2 = 124
        assert_eq!(vp.y, 124);
    }

    #[test]
    fn test_fit_aspect_ratio_320x240_landscape_into_240x320_portrait() {
        let vp = calculate_viewport(320, 240, 240, 320, ScaleMode::FitAspectRatio);
        // Scale = 240 / 320 = 0.75
        // scaled_w = 240, scaled_h = 180
        assert_eq!(vp.width, 240);
        assert_eq!(vp.height, 180);
        assert_eq!(vp.x, 0); // No horizontal padding
        assert_eq!(vp.y, (320 - 180) / 2); // 70px top & bottom letterbox
    }

    #[test]
    fn test_render_letterboxed_places_pixels_correctly() {
        let src = vec![0xFFFFFFFF; 2 * 2]; // 2x2 white
        let mut target = vec![0; 4 * 4]; // 4x4 black

        let vp = render_letterboxed(&src, 2, 2, &mut target, 4, 4, ScaleMode::Integer, 0xFF000000);
        assert_eq!(vp.width, 4);
        assert_eq!(vp.height, 4);
        assert_eq!(vp.x, 0);
        assert_eq!(vp.y, 0);
        for pixel in target {
            assert_eq!(pixel, 0xFFFFFFFF);
        }
    }
}
