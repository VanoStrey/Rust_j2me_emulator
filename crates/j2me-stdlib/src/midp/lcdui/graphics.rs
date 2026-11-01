use super::font::FontSpec;
use super::image::{make_rgba, unpack_rgba, Image};

// MIDP 2.0 Anchor points
pub const HCENTER: i32 = 1;
pub const VCENTER: i32 = 2;
pub const LEFT: i32 = 4;
pub const RIGHT: i32 = 8;
pub const TOP: i32 = 16;
pub const BOTTOM: i32 = 32;
pub const BASELINE: i32 = 64;

#[derive(Debug, Clone)]
pub struct Graphics {
    pub width: i32,
    pub height: i32,
    pub trans_x: i32,
    pub trans_y: i32,
    pub clip_x: i32,
    pub clip_y: i32,
    pub clip_w: i32,
    pub clip_h: i32,
    pub current_color: u32, // RGBA8888
    pub font: FontSpec,
}

impl Graphics {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            trans_x: 0,
            trans_y: 0,
            clip_x: 0,
            clip_y: 0,
            clip_w: width,
            clip_h: height,
            current_color: make_rgba(0, 0, 0, 255), // Default black
            font: FontSpec::default(),
        }
    }

    pub fn reset_clip(&mut self) {
        self.clip_x = 0;
        self.clip_y = 0;
        self.clip_w = self.width;
        self.clip_h = self.height;
    }

    pub fn reset_state(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.trans_x = 0;
        self.trans_y = 0;
        self.clip_x = 0;
        self.clip_y = 0;
        self.clip_w = width;
        self.clip_h = height;
        self.current_color = make_rgba(0, 0, 0, 255);
    }

    pub fn translate(&mut self, x: i32, y: i32) {
        self.trans_x += x;
        self.trans_y += y;
    }

    pub fn set_clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        let max_w = self.width.max(0);
        let max_h = self.height.max(0);
        let cx1 = abs_x.clamp(0, max_w);
        let cy1 = abs_y.clamp(0, max_h);
        let cx2 = (abs_x + w.max(0)).clamp(0, max_w);
        let cy2 = (abs_y + h.max(0)).clamp(0, max_h);

        self.clip_x = cx1;
        self.clip_y = cy1;
        self.clip_w = cx2 - cx1;
        self.clip_h = cy2 - cy1;
    }

    pub fn clip_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        let new_x1 = self.clip_x.max(abs_x);
        let new_y1 = self.clip_y.max(abs_y);
        let new_x2 = (self.clip_x + self.clip_w).min(abs_x + w.max(0));
        let new_y2 = (self.clip_y + self.clip_h).min(abs_y + h.max(0));

        if new_x2 > new_x1 && new_y2 > new_y1 {
            self.clip_x = new_x1;
            self.clip_y = new_y1;
            self.clip_w = new_x2 - new_x1;
            self.clip_h = new_y2 - new_y1;
        } else {
            self.clip_w = 0;
            self.clip_h = 0;
        }
    }

    pub fn set_color_rgb(&mut self, rgb: i32) {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        self.current_color = make_rgba(r, g, b, 255);
    }

    pub fn set_color_argb(&mut self, argb: i32) {
        let a = ((argb >> 24) & 0xFF) as u8;
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        self.current_color = make_rgba(r, g, b, a);
    }

    pub fn fill_rect(&self, pixels: &mut [u32], x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            return;
        }
        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        let x1 = abs_x.max(self.clip_x);
        let y1 = abs_y.max(self.clip_y);
        let x2 = (abs_x + w).min(self.clip_x + self.clip_w);
        let y2 = (abs_y + h).min(self.clip_y + self.clip_h);

        let color = self.current_color;
        let (_, _, _, a) = unpack_rgba(color);

        for py in y1..y2 {
            let row_start = (py * self.width) as usize;
            for px in x1..x2 {
                let idx = row_start + px as usize;
                if a == 255 {
                    pixels[idx] = color;
                } else if a > 0 {
                    pixels[idx] = blend_pixels(pixels[idx], color);
                }
            }
        }
    }

    pub fn draw_rect(&self, pixels: &mut [u32], x: i32, y: i32, w: i32, h: i32) {
        self.draw_line(pixels, x, y, x + w, y);
        self.draw_line(pixels, x + w, y, x + w, y + h);
        self.draw_line(pixels, x + w, y + h, x, y + h);
        self.draw_line(pixels, x, y + h, x, y);
    }

    pub fn draw_line(&self, pixels: &mut [u32], x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1 + self.trans_x;
        let mut y = y1 + self.trans_y;
        let x_end = x2 + self.trans_x;
        let y_end = y2 + self.trans_y;

        let dx = (x_end - x).abs();
        let dy = -(y_end - y).abs();
        let sx = if x < x_end { 1 } else { -1 };
        let sy = if y < y_end { 1 } else { -1 };
        let mut err = dx + dy;

        let color = self.current_color;
        let (_, _, _, a) = unpack_rgba(color);

        loop {
            if x >= self.clip_x
                && x < self.clip_x + self.clip_w
                && y >= self.clip_y
                && y < self.clip_y + self.clip_h
            {
                let idx = (y * self.width + x) as usize;
                if idx < pixels.len() {
                    pixels[idx] = if a == 255 {
                        color
                    } else {
                        blend_pixels(pixels[idx], color)
                    };
                }
            }
            if x == x_end && y == y_end {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn fill_round_rect(
        &self,
        pixels: &mut [u32],
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        arc_w: i32,
        arc_h: i32,
    ) {
        if w <= 0 || h <= 0 {
            return;
        }
        if w <= 1 || h <= 1 || arc_w <= 0 || arc_h <= 0 {
            self.fill_rect(pixels, x, y, w, h);
            return;
        }
        let max_rx = (w / 2).max(1);
        let max_ry = (h / 2).max(1);
        let rx = (arc_w / 2).clamp(1, max_rx);
        let ry = (arc_h / 2).clamp(1, max_ry);

        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        let x1 = abs_x.max(self.clip_x);
        let y1 = abs_y.max(self.clip_y);
        let x2 = (abs_x + w).min(self.clip_x + self.clip_w);
        let y2 = (abs_y + h).min(self.clip_y + self.clip_h);

        let color = self.current_color;
        let (_, _, _, a) = unpack_rgba(color);

        let rx_sq = (rx * rx) as f32;
        let ry_sq = (ry * ry) as f32;

        for py in y1..y2 {
            let dy = if py < abs_y + ry {
                (abs_y + ry) - py
            } else if py >= abs_y + h - ry {
                py - (abs_y + h - ry - 1)
            } else {
                0
            };

            let row_start = (py * self.width) as usize;
            for px in x1..x2 {
                let dx = if px < abs_x + rx {
                    (abs_x + rx) - px
                } else if px >= abs_x + w - rx {
                    px - (abs_x + w - rx - 1)
                } else {
                    0
                };

                if dx > 0 && dy > 0 {
                    let ex = (dx * dx) as f32 / rx_sq;
                    let ey = (dy * dy) as f32 / ry_sq;
                    if ex + ey > 1.0 {
                        continue;
                    }
                }

                let idx = row_start + px as usize;
                if idx < pixels.len() {
                    if a == 255 {
                        pixels[idx] = color;
                    } else if a > 0 {
                        pixels[idx] = blend_pixels(pixels[idx], color);
                    }
                }
            }
        }
    }

    pub fn draw_round_rect(
        &self,
        pixels: &mut [u32],
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        arc_w: i32,
        arc_h: i32,
    ) {
        if w <= 0 || h <= 0 {
            return;
        }
        if w <= 1 || h <= 1 || arc_w <= 0 || arc_h <= 0 {
            self.draw_rect(pixels, x, y, w, h);
            return;
        }
        let max_rx = (w / 2).max(1);
        let max_ry = (h / 2).max(1);
        let rx = (arc_w / 2).clamp(1, max_rx);
        let ry = (arc_h / 2).clamp(1, max_ry);

        self.draw_line(pixels, x + rx, y, x + w - rx, y);
        self.draw_line(pixels, x + rx, y + h, x + w - rx, y + h);
        self.draw_line(pixels, x, y + ry, x, y + h - ry);
        self.draw_line(pixels, x + w, y + ry, x + w, y + h - ry);

        self.draw_arc(pixels, x, y, rx * 2, ry * 2, 90, 90);
        self.draw_arc(pixels, x + w - rx * 2, y, rx * 2, ry * 2, 0, 90);
        self.draw_arc(pixels, x + w - rx * 2, y + h - ry * 2, rx * 2, ry * 2, 270, 90);
        self.draw_arc(pixels, x, y + h - ry * 2, rx * 2, ry * 2, 180, 90);
    }

    pub fn draw_arc(
        &self,
        pixels: &mut [u32],
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        start_angle: i32,
        arc_angle: i32,
    ) {
        if w <= 0 || h <= 0 || arc_angle == 0 {
            return;
        }
        let cx = x + self.trans_x + w / 2;
        let cy = y + self.trans_y + h / 2;
        let rx = (w as f32) / 2.0;
        let ry = (h as f32) / 2.0;

        let steps = (w.max(h) * 2).clamp(16, 360);
        let color = self.current_color;
        let (_, _, _, a) = unpack_rgba(color);

        let start_rad = (start_angle as f32).to_radians();
        let sweep_rad = (arc_angle as f32).to_radians();

        for i in 0..=steps {
            let theta = start_rad + sweep_rad * (i as f32 / steps as f32);
            let px = (cx as f32 + rx * theta.cos()).round() as i32;
            let py = (cy as f32 - ry * theta.sin()).round() as i32;

            if px >= self.clip_x
                && px < self.clip_x + self.clip_w
                && py >= self.clip_y
                && py < self.clip_y + self.clip_h
            {
                let idx = (py * self.width + px) as usize;
                if idx < pixels.len() {
                    pixels[idx] = if a == 255 {
                        color
                    } else {
                        blend_pixels(pixels[idx], color)
                    };
                }
            }
        }
    }

    pub fn fill_arc(
        &self,
        pixels: &mut [u32],
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        start_angle: i32,
        arc_angle: i32,
    ) {
        if w <= 0 || h <= 0 || arc_angle == 0 {
            return;
        }
        let cx = x + self.trans_x + w / 2;
        let cy = y + self.trans_y + h / 2;
        let rx = (w as f32) / 2.0;
        let ry = (h as f32) / 2.0;
        let rx_sq = rx * rx;
        let ry_sq = ry * ry;

        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        let x1 = abs_x.max(self.clip_x);
        let y1 = abs_y.max(self.clip_y);
        let x2 = (abs_x + w).min(self.clip_x + self.clip_w);
        let y2 = (abs_y + h).min(self.clip_y + self.clip_h);

        let color = self.current_color;
        let (_, _, _, a) = unpack_rgba(color);

        let norm_start = ((start_angle % 360) + 360) % 360;
        let norm_end = norm_start + arc_angle;

        for py in y1..y2 {
            let dy = (cy - py) as f32;
            let dy_term = (dy * dy) / ry_sq;
            if dy_term > 1.0 {
                continue;
            }

            let row_start = (py * self.width) as usize;
            for px in x1..x2 {
                let dx = (px - cx) as f32;
                if (dx * dx) / rx_sq + dy_term <= 1.0 {
                    let angle_deg = dx.atan2(dy).to_degrees();
                    let deg = (angle_deg as i32 + 360) % 360;
                    if arc_angle.abs() < 360 {
                        if norm_end <= 360 {
                            if deg < norm_start || deg > norm_end {
                                continue;
                            }
                        } else if deg < norm_start && deg > (norm_end % 360) {
                            continue;
                        }
                    }

                    let idx = row_start + px as usize;
                    if idx < pixels.len() {
                        if a == 255 {
                            pixels[idx] = color;
                        } else if a > 0 {
                            pixels[idx] = blend_pixels(pixels[idx], color);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_triangle(
        &self,
        pixels: &mut [u32],
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
    ) {
        self.draw_line(pixels, x1, y1, x2, y2);
        self.draw_line(pixels, x2, y2, x3, y3);
        self.draw_line(pixels, x3, y3, x1, y1);
    }

    pub fn fill_triangle(
        &self,
        pixels: &mut [u32],
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
    ) {
        let mut pts = [(x1, y1), (x2, y2), (x3, y3)];
        pts.sort_by_key(|p| p.1);
        let (p1, p2, p3) = (pts[0], pts[1], pts[2]);

        let interpolate = |y: i32, pa: (i32, i32), pb: (i32, i32)| -> i32 {
            if pb.1 == pa.1 {
                pa.0
            } else {
                pa.0 + (pb.0 - pa.0) * (y - pa.1) / (pb.1 - pa.1)
            }
        };

        for y in p1.1..=p3.1 {
            let xa = interpolate(y, p1, p3);
            let xb = if y < p2.1 {
                interpolate(y, p1, p2)
            } else {
                interpolate(y, p2, p3)
            };
            let (start_x, end_x) = if xa < xb { (xa, xb) } else { (xb, xa) };
            self.draw_line(pixels, start_x, y, end_x, y);
        }
    }

    pub fn draw_image(&self, pixels: &mut [u32], img: &Image, x: i32, y: i32, anchor: i32) {
        self.draw_region(pixels, img, 0, 0, img.width, img.height, 0, x, y, anchor);
    }

    pub fn draw_region(
        &self,
        pixels: &mut [u32],
        img: &Image,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        transform: i32,
        dst_x: i32,
        dst_y: i32,
        anchor: i32,
    ) {
        if src_w <= 0 || src_h <= 0 {
            return;
        }

        let (target_w, target_h) = match transform {
            4 | 5 | 6 | 7 => (src_h, src_w),
            _ => (src_w, src_h),
        };

        // Anchor adjustment
        let mut draw_x = dst_x + self.trans_x;
        let mut draw_y = dst_y + self.trans_y;

        if anchor & HCENTER != 0 {
            draw_x -= target_w / 2;
        } else if anchor & RIGHT != 0 {
            draw_x -= target_w;
        }

        if anchor & VCENTER != 0 {
            draw_y -= target_h / 2;
        } else if anchor & BOTTOM != 0 {
            draw_y -= target_h;
        }

        // Render transformed region with clipping
        for row in 0..src_h {
            for col in 0..src_w {
                let (dst_col, dst_row) = match transform {
                    0 => (col, row),                               // TRANS_NONE
                    1 => (col, target_h - 1 - row),                // TRANS_MIRROR_ROT180 (FLIP_Y)
                    2 => (target_w - 1 - col, row),                // TRANS_MIRROR (FLIP_X)
                    3 => (target_w - 1 - col, target_h - 1 - row), // TRANS_ROT180
                    4 => (row, col),                               // TRANS_MIRROR_ROT270 (Transpose)
                    5 => (target_w - 1 - row, col),                // TRANS_ROT90 (90 deg CW)
                    6 => (row, target_h - 1 - col),                // TRANS_ROT270 (270 deg CW)
                    7 => (target_w - 1 - row, target_h - 1 - col), // TRANS_MIRROR_ROT90 (Anti-transpose)
                    _ => (col, row),
                };

                let screen_x = draw_x + dst_col;
                let screen_y = draw_y + dst_row;

                if screen_x >= self.clip_x
                    && screen_x < self.clip_x + self.clip_w
                    && screen_y >= self.clip_y
                    && screen_y < self.clip_y + self.clip_h
                {
                    let src_idx = ((src_y + row) * img.width + (src_x + col)) as usize;
                    if let Some(&src_color) = img.pixels.get(src_idx) {
                        let (_, _, _, a) = unpack_rgba(src_color);
                        if a > 0 {
                            let dst_idx = (screen_y * self.width + screen_x) as usize;
                            if dst_idx < pixels.len() {
                                pixels[dst_idx] = if a == 255 {
                                    src_color
                                } else {
                                    blend_pixels(pixels[dst_idx], src_color)
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn draw_rgb(
        &self,
        pixels: &mut [u32],
        rgb_data: &[i32],
        offset: usize,
        scanlength: usize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        process_alpha: bool,
    ) {
        let abs_x = x + self.trans_x;
        let abs_y = y + self.trans_y;

        for row in 0..height {
            let screen_y = abs_y + row;
            if screen_y < self.clip_y || screen_y >= self.clip_y + self.clip_h {
                continue;
            }

            for col in 0..width {
                let screen_x = abs_x + col;
                if screen_x < self.clip_x || screen_x >= self.clip_x + self.clip_w {
                    continue;
                }

                let src_idx = offset + (row as usize) * scanlength + col as usize;
                if src_idx >= rgb_data.len() {
                    continue;
                }

                let argb = rgb_data[src_idx];
                let a = if process_alpha {
                    ((argb >> 24) & 0xFF) as u8
                } else {
                    255
                };
                let r = ((argb >> 16) & 0xFF) as u8;
                let g = ((argb >> 8) & 0xFF) as u8;
                let b = (argb & 0xFF) as u8;

                if a > 0 {
                    let color = make_rgba(r, g, b, a);
                    let dst_idx = (screen_y * self.width + screen_x) as usize;
                    if dst_idx < pixels.len() {
                        pixels[dst_idx] = if a == 255 {
                            color
                        } else {
                            blend_pixels(pixels[dst_idx], color)
                        };
                    }
                }
            }
        }
    }

    pub fn draw_string(&self, pixels: &mut [u32], text: &str, x: i32, y: i32, anchor: i32) {
        let mut draw_x = x + self.trans_x;
        let mut draw_y = y + self.trans_y;

        let str_w = self.font.string_width(text);
        let str_h = self.font.get_height();

        if anchor & HCENTER != 0 {
            draw_x -= str_w / 2;
        } else if anchor & RIGHT != 0 {
            draw_x -= str_w;
        }

        if anchor & VCENTER != 0 {
            draw_y -= str_h / 2;
        } else if anchor & BOTTOM != 0 {
            draw_y -= str_h;
        } else if anchor & BASELINE != 0 {
            draw_y -= self.font.get_baseline_position();
        }

        self.font.render_string(
            pixels,
            self.width,
            self.height,
            (self.clip_x, self.clip_y, self.clip_w, self.clip_h),
            text,
            draw_x,
            draw_y,
            self.current_color,
        );
    }

    pub fn copy_area(
        &self,
        pixels: &mut [u32],
        x_src: i32,
        y_src: i32,
        width: i32,
        height: i32,
        x_dest: i32,
        y_dest: i32,
        anchor: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }

        let mut draw_x = x_dest + self.trans_x;
        let mut draw_y = y_dest + self.trans_y;

        if anchor & HCENTER != 0 {
            draw_x -= width / 2;
        } else if anchor & RIGHT != 0 {
            draw_x -= width;
        }

        if anchor & VCENTER != 0 {
            draw_y -= height / 2;
        } else if anchor & BOTTOM != 0 {
            draw_y -= height;
        }

        let abs_src_x = x_src + self.trans_x;
        let abs_src_y = y_src + self.trans_y;

        let mut temp = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            for col in 0..width {
                let sx = abs_src_x + col;
                let sy = abs_src_y + row;
                if sx >= 0 && sx < self.width && sy >= 0 && sy < self.height {
                    let idx = (sy * self.width + sx) as usize;
                    temp.push(pixels.get(idx).copied().unwrap_or(0));
                } else {
                    temp.push(0);
                }
            }
        }

        for row in 0..height {
            let dy = draw_y + row;
            if dy < self.clip_y || dy >= self.clip_y + self.clip_h {
                continue;
            }
            for col in 0..width {
                let dx = draw_x + col;
                if dx < self.clip_x || dx >= self.clip_x + self.clip_w {
                    continue;
                }
                let src_px = temp[(row * width + col) as usize];
                let dst_idx = (dy * self.width + dx) as usize;
                if dst_idx < pixels.len() {
                    pixels[dst_idx] = src_px;
                }
            }
        }
    }
}

#[inline(always)]
fn blend_pixels(bg: u32, fg: u32) -> u32 {
    let (br, bg_g, bb, _) = unpack_rgba(bg);
    let (fr, fg_g, fb, fa) = unpack_rgba(fg);
    let alpha = fa as u32;
    let inv_alpha = 255 - alpha;

    let r = ((fr as u32 * alpha + br as u32 * inv_alpha) / 255) as u8;
    let g = ((fg_g as u32 * alpha + bg_g as u32 * inv_alpha) / 255) as u8;
    let b = ((fb as u32 * alpha + bb as u32 * inv_alpha) / 255) as u8;
    make_rgba(r, g, b, 255)
}
