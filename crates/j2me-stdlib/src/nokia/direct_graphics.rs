use crate::midp::lcdui::graphics::Graphics;
use crate::midp::lcdui::image::{make_rgba, unpack_rgba, Image};

// Official Nokia UI API DirectGraphics format constants (exact decimal values in bytecode)
pub const TYPE_BYTE_1_GRAY: i32 = 1;
pub const TYPE_BYTE_1_GRAY_VERTICAL: i32 = -1;
pub const TYPE_BYTE_2_GRAY: i32 = 2;
pub const TYPE_BYTE_4_GRAY: i32 = 4;
pub const TYPE_BYTE_8_GRAY: i32 = 8;
pub const TYPE_BYTE_332_RGB: i32 = 332;

pub const TYPE_USHORT_4444_ARGB: i32 = 4444;
pub const TYPE_USHORT_444_RGB: i32 = 444;
pub const TYPE_USHORT_555_RGB: i32 = 555;
pub const TYPE_USHORT_565_RGB: i32 = 565;
pub const TYPE_USHORT_1555_ARGB: i32 = 1555;
pub const TYPE_INT_888_RGB: i32 = 888;
pub const TYPE_INT_8888_ARGB: i32 = 8888;

pub const FLIP_HORIZONTAL: i32 = 8192; // 0x2000
pub const FLIP_VERTICAL: i32 = 16384; // 0x4000
pub const ROTATE_90: i32 = 90;
pub const ROTATE_180: i32 = 180;
pub const ROTATE_270: i32 = 270;

pub struct DirectGraphics;

impl DirectGraphics {
    pub const TYPE_BYTE_1_GRAY: i32 = TYPE_BYTE_1_GRAY;
    pub const TYPE_BYTE_1_GRAY_VERTICAL: i32 = TYPE_BYTE_1_GRAY_VERTICAL;
    pub const TYPE_BYTE_2_GRAY: i32 = TYPE_BYTE_2_GRAY;
    pub const TYPE_BYTE_4_GRAY: i32 = TYPE_BYTE_4_GRAY;
    pub const TYPE_BYTE_8_GRAY: i32 = TYPE_BYTE_8_GRAY;
    pub const TYPE_BYTE_332_RGB: i32 = TYPE_BYTE_332_RGB;
    pub const TYPE_USHORT_4444_ARGB: i32 = TYPE_USHORT_4444_ARGB;
    pub const TYPE_USHORT_444_RGB: i32 = TYPE_USHORT_444_RGB;
    pub const TYPE_USHORT_555_RGB: i32 = TYPE_USHORT_555_RGB;
    pub const TYPE_USHORT_565_RGB: i32 = TYPE_USHORT_565_RGB;
    pub const TYPE_USHORT_1555_ARGB: i32 = TYPE_USHORT_1555_ARGB;
    pub const TYPE_INT_888_RGB: i32 = TYPE_INT_888_RGB;
    pub const TYPE_INT_8888_ARGB: i32 = TYPE_INT_8888_ARGB;
    /// Nokia polygon drawing (scans lines between vertices)
    pub fn draw_polygon(
        g: &Graphics,
        pixels: &mut [u32],
        x_points: &[i32],
        y_points: &[i32],
        n_points: usize,
        argb_color: i32,
    ) {
        if n_points < 2 {
            return;
        }
        let mut g_copy = g.clone();
        g_copy.set_color_argb(argb_color);

        for i in 0..n_points {
            let next = (i + 1) % n_points;
            g_copy.draw_line(
                pixels,
                x_points[i],
                y_points[i],
                x_points[next],
                y_points[next],
            );
        }
    }

    /// Nokia polygon fill using standard scanline algorithm
    pub fn fill_polygon(
        g: &Graphics,
        pixels: &mut [u32],
        x_points: &[i32],
        y_points: &[i32],
        n_points: usize,
        argb_color: i32,
    ) {
        if n_points < 3 {
            return;
        }

        let a = ((argb_color >> 24) & 0xFF) as u8;
        let r = ((argb_color >> 16) & 0xFF) as u8;
        let g_col = ((argb_color >> 8) & 0xFF) as u8;
        let b = (argb_color & 0xFF) as u8;
        let color = make_rgba(r, g_col, b, a);

        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for i in 0..n_points {
            min_y = min_y.min(y_points[i]);
            max_y = max_y.max(y_points[i]);
        }

        let abs_min_y = (min_y + g.trans_y).max(g.clip_y);
        let abs_max_y = (max_y + g.trans_y).min(g.clip_y + g.clip_h - 1);

        let mut node_x = Vec::with_capacity(n_points);

        for y in abs_min_y..=abs_max_y {
            let local_y = y - g.trans_y;
            node_x.clear();

            let mut j = n_points - 1;
            for i in 0..n_points {
                let yi = y_points[i];
                let yj = y_points[j];
                let xi = x_points[i];
                let xj = x_points[j];

                if (yi < local_y && yj >= local_y) || (yj < local_y && yi >= local_y) {
                    let inter_x = xi + (local_y - yi) * (xj - xi) / (yj - yi);
                    node_x.push(inter_x);
                }
                j = i;
            }

            node_x.sort_unstable();

            let mut idx = 0;
            while idx + 1 < node_x.len() {
                let x1 = (node_x[idx] + g.trans_x).max(g.clip_x);
                let x2 = (node_x[idx + 1] + g.trans_x).min(g.clip_x + g.clip_w - 1);

                for px in x1..=x2 {
                    let pixel_idx = (y * g.width + px) as usize;
                    if pixel_idx < pixels.len() {
                        pixels[pixel_idx] = color;
                    }
                }
                idx += 2;
            }
        }
    }

    /// Nokia drawPixels for int array in ARGB8888 or other formats
    pub fn draw_pixels_int(
        g: &Graphics,
        pixels: &mut [u32],
        src_pixels: &[i32],
        transparency: bool,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        for row in 0..height {
            for col in 0..width {
                let src_idx = match (row as i64)
                    .checked_mul(scanlength as i64)
                    .and_then(|r| r.checked_add(col as i64))
                    .and_then(|r| r.checked_add(offset as i64))
                {
                    Some(idx) if idx >= 0 && (idx as usize) < src_pixels.len() => idx as usize,
                    _ => continue,
                };
                let val = src_pixels[src_idx] as u32;
                let (r, g_val, b, mut a) = decode_int_pixel(val, format);
                if !transparency {
                    a = 255;
                }
                if a == 0 {
                    continue;
                }

                let (rc, rr, _, _) = transform_coords(col, row, width, height, manipulation);
                let screen_x = abs_x + rc;
                let screen_y = abs_y + rr;

                if screen_x >= g.clip_x
                    && screen_x < g.clip_x + g.clip_w
                    && screen_y >= g.clip_y
                    && screen_y < g.clip_y + g.clip_h
                {
                    let dst_idx = (screen_y * g.width + screen_x) as usize;
                    if dst_idx < pixels.len() {
                        let color = make_rgba(r, g_val, b, a);
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

    /// Nokia drawPixels for short array in 4444, 565, 444, 1555 formats
    pub fn draw_pixels_short(
        g: &Graphics,
        pixels: &mut [u32],
        src_pixels: &[i16],
        transparency: bool,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        for row in 0..height {
            for col in 0..width {
                let src_idx = match (row as i64)
                    .checked_mul(scanlength as i64)
                    .and_then(|r| r.checked_add(col as i64))
                    .and_then(|r| r.checked_add(offset as i64))
                {
                    Some(idx) if idx >= 0 && (idx as usize) < src_pixels.len() => idx as usize,
                    _ => continue,
                };
                let val = src_pixels[src_idx] as u16;
                let (r, g_val, b, mut a) = decode_short_pixel(val, format);
                if !transparency {
                    a = 255;
                }
                if a == 0 {
                    continue;
                }

                let (rc, rr, _, _) = transform_coords(col, row, width, height, manipulation);
                let screen_x = abs_x + rc;
                let screen_y = abs_y + rr;

                if screen_x >= g.clip_x
                    && screen_x < g.clip_x + g.clip_w
                    && screen_y >= g.clip_y
                    && screen_y < g.clip_y + g.clip_h
                {
                    let dst_idx = (screen_y * g.width + screen_x) as usize;
                    if dst_idx < pixels.len() {
                        let color = make_rgba(r, g_val, b, a);
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

    /// Nokia drawPixels for byte array in gray and RGB332 formats
    pub fn draw_pixels_byte(
        g: &Graphics,
        pixels: &mut [u32],
        src_pixels: &[i8],
        transparency_mask: Option<&[i8]>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        let is_unpacked = match (height as i64 - 1)
            .checked_mul(scanlength as i64)
            .and_then(|r| r.checked_add(width as i64 - 1))
            .and_then(|r| r.checked_add(offset as i64))
        {
            Some(max_idx) => max_idx >= 0 && (max_idx as usize) < src_pixels.len(),
            None => false,
        };

        for row in 0..height {
            for col in 0..width {
                let (val, mask_bit): (u8, u8) = if is_unpacked
                    && format != TYPE_BYTE_1_GRAY_VERTICAL
                    && format != TYPE_BYTE_1_GRAY
                {
                    let idx = match (row as i64)
                        .checked_mul(scanlength as i64)
                        .and_then(|r| r.checked_add(col as i64))
                        .and_then(|r| r.checked_add(offset as i64))
                    {
                        Some(i) if i >= 0 && (i as usize) < src_pixels.len() => i as usize,
                        _ => continue,
                    };
                    let b = src_pixels[idx] as u8;
                    let m = transparency_mask
                        .and_then(|mask| mask.get(idx))
                        .map(|&v| v as u8)
                        .unwrap_or(255);
                    (b, m)
                } else {
                    match format {
                        TYPE_BYTE_1_GRAY => {
                            let (byte_idx, bit_pos) =
                                if scanlength <= (width + 7) / 8 && scanlength > 0 {
                                    (
                                        (offset + row * scanlength + col / 8) as usize,
                                        7 - (col % 8),
                                    )
                                } else {
                                    (
                                        (offset + (row * scanlength + col) / 8) as usize,
                                        7 - (col % 8),
                                    )
                                };
                            if byte_idx >= src_pixels.len() {
                                continue;
                            }
                            let bit = ((src_pixels[byte_idx] as u8) >> bit_pos) & 1;
                            let m_bit = transparency_mask
                                .map(|mask| {
                                    if byte_idx < mask.len() {
                                        ((mask[byte_idx] as u8) >> bit_pos) & 1
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(1);
                            (bit, m_bit)
                        }
                        TYPE_BYTE_1_GRAY_VERTICAL => {
                            let byte_idx = (offset + (row / 8) * scanlength + col) as usize;
                            if byte_idx >= src_pixels.len() {
                                continue;
                            }
                            let bit_pos = row % 8;
                            let bit = ((src_pixels[byte_idx] as u8) >> bit_pos) & 1;
                            let m_bit = transparency_mask
                                .map(|mask| {
                                    if byte_idx < mask.len() {
                                        ((mask[byte_idx] as u8) >> bit_pos) & 1
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(1);
                            (bit, m_bit)
                        }
                        TYPE_BYTE_2_GRAY => {
                            let byte_idx = (offset + (row * scanlength + col) / 4) as usize;
                            if byte_idx >= src_pixels.len() {
                                continue;
                            }
                            let shift = (3 - (col % 4)) * 2;
                            let bits = ((src_pixels[byte_idx] as u8) >> shift) & 0x03;
                            let m_bit = transparency_mask
                                .map(|mask| {
                                    if byte_idx < mask.len() {
                                        ((mask[byte_idx] as u8) >> shift) & 0x03
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(1);
                            (bits, m_bit)
                        }
                        TYPE_BYTE_4_GRAY => {
                            let byte_idx = (offset + (row * scanlength + col) / 2) as usize;
                            if byte_idx >= src_pixels.len() {
                                continue;
                            }
                            let shift = (1 - (col % 2)) * 4;
                            let bits = ((src_pixels[byte_idx] as u8) >> shift) & 0x0F;
                            let m_bit = transparency_mask
                                .map(|mask| {
                                    if byte_idx < mask.len() {
                                        ((mask[byte_idx] as u8) >> shift) & 0x0F
                                    } else {
                                        0
                                    }
                                })
                                .unwrap_or(1);
                            (bits, m_bit)
                        }
                        _ => {
                            let idx = (offset + row * scanlength + col) as usize;
                            if idx >= src_pixels.len() {
                                continue;
                            }
                            let b = src_pixels[idx] as u8;
                            let m = transparency_mask
                                .and_then(|mask| mask.get(idx))
                                .map(|&v| v as u8)
                                .unwrap_or(255);
                            (b, m)
                        }
                    }
                };

                let (r, g_val, b, mut a) = decode_byte_pixel(val, format);
                if transparency_mask.is_some() && mask_bit == 0 {
                    a = 0;
                }
                if a == 0 {
                    continue;
                }

                let (rc, rr, _, _) = transform_coords(col, row, width, height, manipulation);
                let screen_x = abs_x + rc;
                let screen_y = abs_y + rr;

                if screen_x >= g.clip_x
                    && screen_x < g.clip_x + g.clip_w
                    && screen_y >= g.clip_y
                    && screen_y < g.clip_y + g.clip_h
                {
                    let dst_idx = (screen_y * g.width + screen_x) as usize;
                    if dst_idx < pixels.len() {
                        let color = make_rgba(r, g_val, b, a);
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

    /// Nokia getPixels for int array
    pub fn get_pixels_int(
        g: &Graphics,
        pixels: &[u32],
        dst_pixels: &mut [i32],
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        for row in 0..height {
            let sy = abs_y + row;
            for col in 0..width {
                let sx = abs_x + col;
                let dst_idx = match (row as i64)
                    .checked_mul(scanlength as i64)
                    .and_then(|r| r.checked_add(col as i64))
                    .and_then(|r| r.checked_add(offset as i64))
                {
                    Some(idx) if idx >= 0 && (idx as usize) < dst_pixels.len() => idx as usize,
                    _ => continue,
                };

                let px = if sx >= 0 && sx < g.width && sy >= 0 && sy < g.height {
                    let idx = (sy * g.width + sx) as usize;
                    pixels.get(idx).copied().unwrap_or(0)
                } else {
                    0
                };

                let (r, g_val, b, a) = unpack_rgba(px);
                let val = match format {
                    TYPE_INT_888_RGB => ((r as i32) << 16) | ((g_val as i32) << 8) | (b as i32),
                    _ => {
                        ((a as i32) << 24) | ((r as i32) << 16) | ((g_val as i32) << 8) | (b as i32)
                    }
                };
                dst_pixels[dst_idx] = val;
            }
        }
    }

    /// Nokia getPixels for short array
    pub fn get_pixels_short(
        g: &Graphics,
        pixels: &[u32],
        dst_pixels: &mut [i16],
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        for row in 0..height {
            let sy = abs_y + row;
            for col in 0..width {
                let sx = abs_x + col;
                let dst_idx = match (row as i64)
                    .checked_mul(scanlength as i64)
                    .and_then(|r| r.checked_add(col as i64))
                    .and_then(|r| r.checked_add(offset as i64))
                {
                    Some(idx) if idx >= 0 && (idx as usize) < dst_pixels.len() => idx as usize,
                    _ => continue,
                };

                let px = if sx >= 0 && sx < g.width && sy >= 0 && sy < g.height {
                    let idx = (sy * g.width + sx) as usize;
                    pixels.get(idx).copied().unwrap_or(0)
                } else {
                    0
                };

                let (r, g_val, b, a) = unpack_rgba(px);
                let s_val = encode_short_pixel(r, g_val, b, a, format);
                dst_pixels[dst_idx] = s_val as i16;
            }
        }
    }

    /// Nokia getPixels for byte array
    pub fn get_pixels_byte(
        g: &Graphics,
        pixels: &[u32],
        dst_pixels: &mut [i8],
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let abs_x = x + g.trans_x;
        let abs_y = y + g.trans_y;

        for row in 0..height {
            let sy = abs_y + row;
            for col in 0..width {
                let sx = abs_x + col;
                let dst_idx = match (row as i64)
                    .checked_mul(scanlength as i64)
                    .and_then(|r| r.checked_add(col as i64))
                    .and_then(|r| r.checked_add(offset as i64))
                {
                    Some(idx) if idx >= 0 && (idx as usize) < dst_pixels.len() => idx as usize,
                    _ => continue,
                };

                let px = if sx >= 0 && sx < g.width && sy >= 0 && sy < g.height {
                    let idx = (sy * g.width + sx) as usize;
                    pixels.get(idx).copied().unwrap_or(0)
                } else {
                    0
                };

                let (r, g_val, b, a) = unpack_rgba(px);
                let b_val = encode_byte_pixel(r, g_val, b, a, format);
                dst_pixels[dst_idx] = b_val as i8;
            }
        }
    }

    /// Nokia drawImage with manipulation
    pub fn draw_image(
        g: &Graphics,
        pixels: &mut [u32],
        img: &Image,
        x: i32,
        y: i32,
        anchor: i32,
        manipulation: i32,
    ) {
        if img.width <= 0 || img.height <= 0 {
            return;
        }

        let rot = manipulation & 0xFF;
        let (target_w, target_h) = match rot {
            90 | 270 => (img.height, img.width),
            _ => (img.width, img.height),
        };

        let mut draw_x = x + g.trans_x;
        let mut draw_y = y + g.trans_y;

        if anchor & crate::midp::lcdui::graphics::HCENTER != 0 {
            draw_x -= target_w / 2;
        } else if anchor & crate::midp::lcdui::graphics::RIGHT != 0 {
            draw_x -= target_w;
        }

        if anchor & crate::midp::lcdui::graphics::VCENTER != 0 {
            draw_y -= target_h / 2;
        } else if anchor & crate::midp::lcdui::graphics::BOTTOM != 0 {
            draw_y -= target_h;
        }

        for row in 0..img.height {
            for col in 0..img.width {
                let src_idx = (row * img.width + col) as usize;
                if let Some(&src_color) = img.pixels.get(src_idx) {
                    let (_, _, _, a) = unpack_rgba(src_color);
                    if a == 0 {
                        continue;
                    }

                    let (rc, rr, _, _) =
                        transform_coords(col, row, img.width, img.height, manipulation);
                    let screen_x = draw_x + rc;
                    let screen_y = draw_y + rr;

                    if screen_x >= g.clip_x
                        && screen_x < g.clip_x + g.clip_w
                        && screen_y >= g.clip_y
                        && screen_y < g.clip_y + g.clip_h
                    {
                        let dst_idx = (screen_y * g.width + screen_x) as usize;
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

pub fn transform_coords(
    col: i32,
    row: i32,
    w: i32,
    h: i32,
    manipulation: i32,
) -> (i32, i32, i32, i32) {
    let rot = manipulation & 0x1FFF & !(FLIP_HORIZONTAL | FLIP_VERTICAL); // Rotation angle: 0, 90, 180, 270 clockwise
    let (mut rc, mut rr, target_w, target_h) = match rot {
        90 => (h - 1 - row, col, h, w),
        180 => (w - 1 - col, h - 1 - row, w, h),
        270 => (row, w - 1 - col, h, w),
        _ => (col, row, w, h),
    };
    if manipulation & FLIP_HORIZONTAL != 0 {
        rc = target_w - 1 - rc;
    }
    if manipulation & FLIP_VERTICAL != 0 {
        rr = target_h - 1 - rr;
    }
    (rc, rr, target_w, target_h)
}

#[inline(always)]
pub fn decode_short_pixel(val: u16, format: i32) -> (u8, u8, u8, u8) {
    match format {
        TYPE_USHORT_4444_ARGB | 2 => {
            let a = (((val >> 12) & 0xF) as u8) * 17;
            let r = (((val >> 8) & 0xF) as u8) * 17;
            let g = (((val >> 4) & 0xF) as u8) * 17;
            let b = ((val & 0xF) as u8) * 17;
            (r, g, b, a)
        }
        TYPE_USHORT_565_RGB | 3 => {
            let r = (((val >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((val >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            let b = ((val & 0x1F) as u32 * 255 / 31) as u8;
            (r, g, b, 255)
        }
        TYPE_USHORT_444_RGB | 4 => {
            let r = (((val >> 8) & 0xF) as u8) * 17;
            let g = (((val >> 4) & 0xF) as u8) * 17;
            let b = ((val & 0xF) as u8) * 17;
            (r, g, b, 255)
        }
        TYPE_USHORT_1555_ARGB | 5 => {
            let a = if (val & 0x8000) != 0 { 255 } else { 0 };
            let r = (((val >> 10) & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((val >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((val & 0x1F) as u32 * 255 / 31) as u8;
            (r, g, b, a)
        }
        TYPE_USHORT_555_RGB => {
            let r = (((val >> 10) & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((val >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((val & 0x1F) as u32 * 255 / 31) as u8;
            (r, g, b, 255)
        }
        _ => {
            let r = (((val >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((val >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            let b = ((val & 0x1F) as u32 * 255 / 31) as u8;
            (r, g, b, 255)
        }
    }
}

#[inline(always)]
pub fn decode_int_pixel(val: u32, format: i32) -> (u8, u8, u8, u8) {
    match format {
        TYPE_INT_8888_ARGB | 1 => {
            let a = ((val >> 24) & 0xFF) as u8;
            let r = ((val >> 16) & 0xFF) as u8;
            let g = ((val >> 8) & 0xFF) as u8;
            let b = (val & 0xFF) as u8;
            (r, g, b, a)
        }
        TYPE_INT_888_RGB | 6 => {
            let r = ((val >> 16) & 0xFF) as u8;
            let g = ((val >> 8) & 0xFF) as u8;
            let b = (val & 0xFF) as u8;
            (r, g, b, 255)
        }
        _ => {
            let a = ((val >> 24) & 0xFF) as u8;
            let r = ((val >> 16) & 0xFF) as u8;
            let g = ((val >> 8) & 0xFF) as u8;
            let b = (val & 0xFF) as u8;
            (r, g, b, a)
        }
    }
}

#[inline(always)]
pub fn encode_short_pixel(r: u8, g: u8, b: u8, a: u8, format: i32) -> u16 {
    match format {
        TYPE_USHORT_4444_ARGB | 2 => {
            (((a as u16 >> 4) & 0xF) << 12)
                | (((r as u16 >> 4) & 0xF) << 8)
                | (((g as u16 >> 4) & 0xF) << 4)
                | ((b as u16 >> 4) & 0xF)
        }
        TYPE_USHORT_565_RGB | 3 => {
            (((r as u16 >> 3) & 0x1F) << 11)
                | (((g as u16 >> 2) & 0x3F) << 5)
                | ((b as u16 >> 3) & 0x1F)
        }
        TYPE_USHORT_444_RGB | 4 => {
            (((r as u16 >> 4) & 0xF) << 8)
                | (((g as u16 >> 4) & 0xF) << 4)
                | ((b as u16 >> 4) & 0xF)
        }
        TYPE_USHORT_1555_ARGB | 5 => {
            let a_bit = if a >= 128 { 0x8000 } else { 0 };
            a_bit
                | (((r as u16 >> 3) & 0x1F) << 10)
                | (((g as u16 >> 3) & 0x1F) << 5)
                | ((b as u16 >> 3) & 0x1F)
        }
        TYPE_USHORT_555_RGB => {
            (((r as u16 >> 3) & 0x1F) << 10)
                | (((g as u16 >> 3) & 0x1F) << 5)
                | ((b as u16 >> 3) & 0x1F)
        }
        _ => {
            (((r as u16 >> 3) & 0x1F) << 11)
                | (((g as u16 >> 2) & 0x3F) << 5)
                | ((b as u16 >> 3) & 0x1F)
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

#[inline(always)]
pub fn decode_byte_pixel(val: u8, format: i32) -> (u8, u8, u8, u8) {
    match format {
        TYPE_BYTE_8_GRAY => (val, val, val, 255),
        TYPE_BYTE_4_GRAY => {
            let gray = (val & 0x0F) * 17;
            (gray, gray, gray, 255)
        }
        TYPE_BYTE_2_GRAY => {
            let gray = (val & 0x03) * 85;
            (gray, gray, gray, 255)
        }
        TYPE_BYTE_1_GRAY | TYPE_BYTE_1_GRAY_VERTICAL => {
            let gray = if (val & 1) != 0 { 255 } else { 0 };
            (gray, gray, gray, 255)
        }
        TYPE_BYTE_332_RGB => {
            let r = (((val >> 5) & 0x07) as u32 * 255 / 7) as u8;
            let g = (((val >> 2) & 0x07) as u32 * 255 / 7) as u8;
            let b = ((val & 0x03) as u32 * 255 / 3) as u8;
            (r, g, b, 255)
        }
        _ => (val, val, val, 255),
    }
}

#[inline(always)]
pub fn encode_byte_pixel(r: u8, g: u8, b: u8, _a: u8, format: i32) -> u8 {
    match format {
        TYPE_BYTE_8_GRAY => ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8,
        TYPE_BYTE_4_GRAY => {
            let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8;
            gray / 17
        }
        TYPE_BYTE_2_GRAY => {
            let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8;
            gray / 85
        }
        TYPE_BYTE_1_GRAY | TYPE_BYTE_1_GRAY_VERTICAL => {
            let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8;
            if gray >= 128 {
                1
            } else {
                0
            }
        }
        TYPE_BYTE_332_RGB => {
            (((r as u8 >> 5) & 0x07) << 5)
                | (((g as u8 >> 5) & 0x07) << 2)
                | ((b as u8 >> 6) & 0x03)
        }
        _ => ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8,
    }
}
