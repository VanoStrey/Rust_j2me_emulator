use crate::midp::lcdui::graphics::Graphics;
use crate::midp::lcdui::image::{unpack_rgba, Image};

pub const TRANS_NONE: i32 = 0;
pub const TRANS_MIRROR_ROT180: i32 = 1;
pub const TRANS_MIRROR: i32 = 2;
pub const TRANS_ROT180: i32 = 3;
pub const TRANS_MIRROR_ROT270: i32 = 4;
pub const TRANS_ROT90: i32 = 5;
pub const TRANS_ROT270: i32 = 6;
pub const TRANS_MIRROR_ROT90: i32 = 7;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub image: Image,
    pub frame_w: i32,
    pub frame_h: i32,
    pub cols: i32,
    pub rows: i32,
    pub frames_count: usize,
    pub frame_sequence: Option<Vec<usize>>,
    pub current_frame_idx: usize,
    pub x: i32,
    pub y: i32,
    pub ref_x: i32,
    pub ref_y: i32,
    pub transform: i32,
    pub visible: bool,
}

impl Sprite {
    pub fn new(image: Image, frame_w: i32, frame_h: i32) -> Self {
        let cols = (image.width / frame_w).max(1);
        let rows = (image.height / frame_h).max(1);
        let frames_count = (cols * rows) as usize;

        Self {
            image,
            frame_w,
            frame_h,
            cols,
            rows,
            frames_count,
            frame_sequence: None,
            current_frame_idx: 0,
            x: 0,
            y: 0,
            ref_x: 0,
            ref_y: 0,
            transform: TRANS_NONE,
            visible: true,
        }
    }

    pub fn set_frame(&mut self, sequence_idx: usize) {
        let total = self
            .frame_sequence
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(self.frames_count);
        if sequence_idx < total {
            self.current_frame_idx = sequence_idx;
        }
    }

    pub fn get_raw_frame(&self) -> usize {
        if let Some(ref seq) = self.frame_sequence {
            seq.get(self.current_frame_idx).copied().unwrap_or(0)
        } else {
            self.current_frame_idx
        }
    }

    pub fn define_reference_pixel(&mut self, x: i32, y: i32) {
        self.ref_x = x;
        self.ref_y = y;
    }

    pub fn set_ref_pixel_position(&mut self, target_x: i32, target_y: i32) {
        let fw = self.frame_w;
        let fh = self.frame_h;
        let ref_x = self.ref_x;
        let ref_y = self.ref_y;

        let (dx, dy) = match self.transform {
            TRANS_NONE => (ref_x, ref_y),
            TRANS_MIRROR_ROT180 => (ref_x, fh - 1 - ref_y),
            TRANS_MIRROR => (fw - 1 - ref_x, ref_y),
            TRANS_ROT180 => (fw - 1 - ref_x, fh - 1 - ref_y),
            TRANS_MIRROR_ROT270 => (ref_y, ref_x),
            TRANS_ROT90 => (fh - 1 - ref_y, ref_x),
            TRANS_ROT270 => (ref_y, fw - 1 - ref_x),
            TRANS_MIRROR_ROT90 => (fh - 1 - ref_y, fw - 1 - ref_x),
            _ => (ref_x, ref_y),
        };

        self.x = target_x - dx;
        self.y = target_y - dy;
    }

    pub fn set_transform(&mut self, new_transform: i32) {
        if self.transform == new_transform {
            return;
        }
        let fw = self.frame_w;
        let fh = self.frame_h;
        let ref_x = self.ref_x;
        let ref_y = self.ref_y;

        let calc_d = |t: i32| -> (i32, i32) {
            match t {
                TRANS_NONE => (ref_x, ref_y),
                TRANS_MIRROR_ROT180 => (ref_x, fh - 1 - ref_y),
                TRANS_MIRROR => (fw - 1 - ref_x, ref_y),
                TRANS_ROT180 => (fw - 1 - ref_x, fh - 1 - ref_y),
                TRANS_MIRROR_ROT270 => (ref_y, ref_x),
                TRANS_ROT90 => (fh - 1 - ref_y, ref_x),
                TRANS_ROT270 => (ref_y, fw - 1 - ref_x),
                TRANS_MIRROR_ROT90 => (fh - 1 - ref_y, fw - 1 - ref_x),
                _ => (ref_x, ref_y),
            }
        };

        let (dx_old, dy_old) = calc_d(self.transform);
        let (dx_new, dy_new) = calc_d(new_transform);

        self.x += dx_old - dx_new;
        self.y += dy_old - dy_new;
        self.transform = new_transform;
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn get_transformed_bounds(&self) -> (i32, i32, i32, i32) {
        let (w, h) = match self.transform {
            TRANS_ROT90 | TRANS_ROT270 | TRANS_MIRROR_ROT90 | TRANS_MIRROR_ROT270 => {
                (self.frame_h, self.frame_w)
            }
            _ => (self.frame_w, self.frame_h),
        };
        (self.x, self.y, w, h)
    }

    pub fn collides_with_sprite(&self, other: &Sprite, pixel_level: bool) -> bool {
        if !self.visible || !other.visible {
            return false;
        }

        let (x1, y1, w1, h1) = self.get_transformed_bounds();
        let (x2, y2, w2, h2) = other.get_transformed_bounds();

        // Bounding-box test
        let inter_x1 = x1.max(x2);
        let inter_y1 = y1.max(y2);
        let inter_x2 = (x1 + w1).min(x2 + w2);
        let inter_y2 = (y1 + h1).min(y2 + h2);

        if inter_x2 <= inter_x1 || inter_y2 <= inter_y1 {
            return false;
        }

        if !pixel_level {
            return true;
        }

        // Pixel-level collision check in overlap rectangle
        for py in inter_y1..inter_y2 {
            for px in inter_x1..inter_x2 {
                let a1 = self.get_pixel_alpha_at(px, py);
                let a2 = other.get_pixel_alpha_at(px, py);
                if a1 > 0 && a2 > 0 {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_pixel_alpha_at(&self, screen_x: i32, screen_y: i32) -> u8 {
        let local_x = screen_x - self.x;
        let local_y = screen_y - self.y;
        let (_bx, _by, bw, bh) = self.get_transformed_bounds();

        if local_x < 0 || local_x >= bw || local_y < 0 || local_y >= bh {
            return 0;
        }

        // Map (local_x, local_y) in transformed bounding box back to (u_x, u_y) in untransformed frame coordinates
        let (u_x, u_y) = match self.transform {
            TRANS_NONE => (local_x, local_y),
            TRANS_MIRROR_ROT180 => (local_x, self.frame_h - 1 - local_y),
            TRANS_MIRROR => (self.frame_w - 1 - local_x, local_y),
            TRANS_ROT180 => (self.frame_w - 1 - local_x, self.frame_h - 1 - local_y),
            TRANS_MIRROR_ROT270 => (local_y, local_x),
            TRANS_ROT90 => (local_y, self.frame_h - 1 - local_x),
            TRANS_ROT270 => (self.frame_w - 1 - local_y, local_x),
            TRANS_MIRROR_ROT90 => (self.frame_w - 1 - local_y, self.frame_h - 1 - local_x),
            _ => (local_x, local_y),
        };

        let frame_num = self.get_raw_frame();
        let frame_col = (frame_num as i32) % self.cols;
        let frame_row = (frame_num as i32) / self.cols;

        let src_x = frame_col * self.frame_w + u_x;
        let src_y = frame_row * self.frame_h + u_y;

        if src_x >= 0 && src_x < self.image.width && src_y >= 0 && src_y < self.image.height {
            let px = self.image.pixels[(src_y * self.image.width + src_x) as usize];
            unpack_rgba(px).3
        } else {
            0
        }
    }

    pub fn paint(&self, g: &Graphics, pixels: &mut [u32]) {
        if !self.visible {
            return;
        }

        let frame_num = self.get_raw_frame();
        let frame_col = (frame_num as i32) % self.cols;
        let frame_row = (frame_num as i32) / self.cols;

        let src_x = frame_col * self.frame_w;
        let src_y = frame_row * self.frame_h;

        g.draw_region(
            pixels,
            &self.image,
            src_x,
            src_y,
            self.frame_w,
            self.frame_h,
            self.transform,
            self.x,
            self.y,
            0,
        );
    }
}
