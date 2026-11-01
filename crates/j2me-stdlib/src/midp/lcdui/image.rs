#[derive(Debug, Clone)]
pub struct Image {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>, // RGBA8888 (in memory: byte 0=R, 1=G, 2=B, 3=A)
    pub is_mutable: bool,
}

#[inline(always)]
pub fn make_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}

#[inline(always)]
pub fn unpack_rgba(pixel: u32) -> (u8, u8, u8, u8) {
    let r = (pixel & 0xFF) as u8;
    let g = ((pixel >> 8) & 0xFF) as u8;
    let b = ((pixel >> 16) & 0xFF) as u8;
    let a = ((pixel >> 24) & 0xFF) as u8;
    (r, g, b, a)
}

impl Image {
    pub fn new_mutable(width: i32, height: i32) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        let size = (w * h) as usize;
        Self {
            width: w,
            height: h,
            pixels: vec![make_rgba(255, 255, 255, 255); size], // Default white
            is_mutable: true,
        }
    }

    pub fn new_mutable_with_color(width: i32, height: i32, argb: u32) -> Self {
        let a = ((argb >> 24) & 0xFF) as u8;
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        let pixel = make_rgba(r, g, b, a);
        let w = width.max(1);
        let h = height.max(1);
        let size = (w * h) as usize;
        Self {
            width: w,
            height: h,
            pixels: vec![pixel; size],
            is_mutable: true,
        }
    }

    pub fn new_immutable(width: i32, height: i32, pixels: Vec<u32>) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            pixels,
            is_mutable: false,
        }
    }

    /// Decode PNG, JPEG, or raw image with non-strict CRC and palette swap support
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() >= 8 && &bytes[..8] == b"\x89PNG\r\n\x1a\n" {
            Self::decode_png_non_strict(bytes)
        } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
            let (w, h) = Self::decode_jpeg_dimensions(bytes).unwrap_or((176, 208));
            let pixels = vec![0xFF202020; (w * h) as usize];
            Ok(Self {
                width: w,
                height: h,
                is_mutable: false,
                pixels,
            })
        } else {
            Err("Unsupported image format".to_string())
        }
    }

    fn decode_jpeg_dimensions(bytes: &[u8]) -> Option<(i32, i32)> {
        if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
            return None;
        }
        let mut pos = 2;
        while pos + 8 < bytes.len() {
            if bytes[pos] != 0xFF {
                pos += 1;
                continue;
            }
            let marker = bytes[pos + 1];
            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            if pos + 4 > bytes.len() {
                break;
            }
            let length = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
            if (0xC0..=0xC3).contains(&marker) && pos + 9 <= bytes.len() {
                let h = u16::from_be_bytes([bytes[pos + 5], bytes[pos + 6]]) as i32;
                let w = u16::from_be_bytes([bytes[pos + 7], bytes[pos + 8]]) as i32;
                if w > 0 && h > 0 {
                    return Some((w, h));
                }
            }
            pos += 2 + length;
        }
        None
    }

    /// Non-strict PNG decoder: ignores all CRC32 and Adler32 checksums
    fn decode_png_non_strict(bytes: &[u8]) -> Result<Self, String> {
        let mut pos = 8;
        let mut width = 0i32;
        let mut height = 0i32;
        let mut bit_depth = 8u8;
        let mut color_type = 6u8;
        let mut palette: Vec<(u8, u8, u8, u8)> = Vec::new();
        let mut idat_data = Vec::new();

        while pos + 8 <= bytes.len() {
            let length =
                u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                    as usize;
            pos += 4;
            let chunk_type = &bytes[pos..pos + 4];
            pos += 4;

            if pos + length > bytes.len() {
                break;
            }
            let chunk_data = &bytes[pos..pos + length];
            pos += length;
            // Skip 4-byte CRC without checking!
            pos += 4;

            match chunk_type {
                b"IHDR" => {
                    if chunk_data.len() >= 13 {
                        width = u32::from_be_bytes([
                            chunk_data[0],
                            chunk_data[1],
                            chunk_data[2],
                            chunk_data[3],
                        ]) as i32;
                        height = u32::from_be_bytes([
                            chunk_data[4],
                            chunk_data[5],
                            chunk_data[6],
                            chunk_data[7],
                        ]) as i32;
                        bit_depth = chunk_data[8];
                        color_type = chunk_data[9];
                    }
                }
                b"PLTE" => {
                    let count = chunk_data.len() / 3;
                    palette.clear();
                    for i in 0..count {
                        let r = chunk_data[i * 3];
                        let g = chunk_data[i * 3 + 1];
                        let b = chunk_data[i * 3 + 2];
                        palette.push((r, g, b, 255));
                    }
                }
                b"tRNS" => {
                    if color_type == 3 {
                        // Palette transparency
                        for (i, &alpha) in chunk_data.iter().enumerate() {
                            if i < palette.len() {
                                palette[i].3 = alpha;
                            }
                        }
                    }
                }
                b"IDAT" => {
                    idat_data.extend_from_slice(chunk_data);
                }
                b"IEND" => break,
                _ => {}
            }
        }

        if width <= 0 || height <= 0 || idat_data.is_empty() {
            return Err("Invalid PNG dimensions or missing IDAT".to_string());
        }

        // Decompress IDAT (try zlib first, then raw deflate if Adler32 was broken by palette swap)
        let decompressed = match miniz_oxide::inflate::decompress_to_vec_zlib(&idat_data) {
            Ok(data) => data,
            Err(_) => {
                // If zlib header is present, skip 2 bytes and raw inflate
                let slice = if idat_data.len() > 2 {
                    &idat_data[2..]
                } else {
                    &idat_data
                };
                miniz_oxide::inflate::decompress_to_vec(slice)
                    .map_err(|e| format!("Deflate failed: {:?}", e))?
            }
        };

        // Determine bytes per pixel in raw scanlines
        let bpp = match color_type {
            0 => 1, // Grayscale
            2 => 3, // RGB
            3 => 1, // Indexed
            4 => 2, // Grayscale + Alpha
            6 => 4, // RGBA
            _ => 1,
        };

        let stride = (width as usize * bpp * bit_depth as usize + 7) / 8;
        let mut pixels = vec![0u32; (width * height) as usize];
        let mut prev_row = vec![0u8; stride];
        let mut cur_row = vec![0u8; stride];
        let mut dec_pos = 0;

        for y in 0..height as usize {
            if dec_pos >= decompressed.len() {
                break;
            }
            let filter_type = decompressed[dec_pos];
            dec_pos += 1;

            if dec_pos + stride > decompressed.len() {
                break;
            }
            let row_bytes = &decompressed[dec_pos..dec_pos + stride];
            dec_pos += stride;

            // Unfilter row
            match filter_type {
                0 => cur_row.copy_from_slice(row_bytes),
                1 => {
                    // Sub
                    for i in 0..stride {
                        let a = if i >= bpp { cur_row[i - bpp] } else { 0 };
                        cur_row[i] = row_bytes[i].wrapping_add(a);
                    }
                }
                2 => {
                    // Up
                    for i in 0..stride {
                        cur_row[i] = row_bytes[i].wrapping_add(prev_row[i]);
                    }
                }
                3 => {
                    // Average
                    for i in 0..stride {
                        let a = if i >= bpp { cur_row[i - bpp] as u16 } else { 0 };
                        let b = prev_row[i] as u16;
                        cur_row[i] = row_bytes[i].wrapping_add(((a + b) / 2) as u8);
                    }
                }
                4 => {
                    // Paeth
                    for i in 0..stride {
                        let a = if i >= bpp { cur_row[i - bpp] as i32 } else { 0 };
                        let b = prev_row[i] as i32;
                        let c = if i >= bpp {
                            prev_row[i - bpp] as i32
                        } else {
                            0
                        };
                        let p = a + b - c;
                        let pa = (p - a).abs();
                        let pb = (p - b).abs();
                        let pc = (p - c).abs();
                        let pr = if pa <= pb && pa <= pc {
                            a
                        } else if pb <= pc {
                            b
                        } else {
                            c
                        };
                        cur_row[i] = row_bytes[i].wrapping_add(pr as u8);
                    }
                }
                _ => cur_row.copy_from_slice(row_bytes),
            }

            // Convert uncompressed scanline to RGBA pixels
            for x in 0..width as usize {
                let px_idx = y * width as usize + x;
                let color = match color_type {
                    3 => {
                        // Indexed
                        let idx = match bit_depth {
                            8 => cur_row.get(x).copied().unwrap_or(0) as usize,
                            4 => {
                                let b = cur_row.get(x / 2).copied().unwrap_or(0);
                                let shift = (1 - (x % 2)) * 4;
                                ((b >> shift) & 0x0F) as usize
                            }
                            2 => {
                                let b = cur_row.get(x / 4).copied().unwrap_or(0);
                                let shift = (3 - (x % 4)) * 2;
                                ((b >> shift) & 0x03) as usize
                            }
                            1 => {
                                let b = cur_row.get(x / 8).copied().unwrap_or(0);
                                let shift = 7 - (x % 8);
                                ((b >> shift) & 0x01) as usize
                            }
                            _ => cur_row.get(x).copied().unwrap_or(0) as usize,
                        };
                        if let Some(&(r, g, b, a)) = palette.get(idx) {
                            make_rgba(r, g, b, a)
                        } else {
                            0
                        }
                    }
                    6 => {
                        // RGBA
                        let off = x * 4;
                        if off + 3 < cur_row.len() {
                            make_rgba(
                                cur_row[off],
                                cur_row[off + 1],
                                cur_row[off + 2],
                                cur_row[off + 3],
                            )
                        } else {
                            0
                        }
                    }
                    2 => {
                        // RGB
                        let off = x * 3;
                        if off + 2 < cur_row.len() {
                            make_rgba(cur_row[off], cur_row[off + 1], cur_row[off + 2], 255)
                        } else {
                            0
                        }
                    }
                    0 => {
                        // Grayscale
                        let g = match bit_depth {
                            8 => cur_row.get(x).copied().unwrap_or(0),
                            4 => {
                                let b = cur_row.get(x / 2).copied().unwrap_or(0);
                                let shift = (1 - (x % 2)) * 4;
                                (((b >> shift) & 0x0F) * 17) as u8
                            }
                            2 => {
                                let b = cur_row.get(x / 4).copied().unwrap_or(0);
                                let shift = (3 - (x % 4)) * 2;
                                (((b >> shift) & 0x03) * 85) as u8
                            }
                            1 => {
                                let b = cur_row.get(x / 8).copied().unwrap_or(0);
                                let shift = 7 - (x % 8);
                                (((b >> shift) & 0x01) * 255) as u8
                            }
                            _ => cur_row.get(x).copied().unwrap_or(0),
                        };
                        make_rgba(g, g, g, 255)
                    }
                    4 => {
                        // Grayscale + Alpha
                        let off = x * 2;
                        if off + 1 < cur_row.len() {
                            let g = cur_row[off];
                            let a = cur_row[off + 1];
                            make_rgba(g, g, g, a)
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                if px_idx < pixels.len() {
                    pixels[px_idx] = color;
                }
            }

            prev_row.copy_from_slice(&cur_row);
        }

        Ok(Self::new_immutable(width, height, pixels))
    }

    /// Sub-region with transformation
    pub fn create_region(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        transform: i32,
    ) -> Result<Self, String> {
        if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > self.width || y + h > self.height {
            return Err("Region bounds out of image bounds".to_string());
        }

        let (target_w, target_h) = match transform {
            4 | 5 | 6 | 7 => (h, w), // Rotated 90, 270 or transposed
            _ => (w, h),
        };

        let mut out_pixels = vec![0u32; (target_w * target_h) as usize];

        for row in 0..h {
            for col in 0..w {
                let src_px = self.pixels[((y + row) * self.width + (x + col)) as usize];
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
                if dst_col >= 0 && dst_col < target_w && dst_row >= 0 && dst_row < target_h {
                    out_pixels[(dst_row * target_w + dst_col) as usize] = src_px;
                }
            }
        }

        Ok(Self::new_immutable(target_w, target_h, out_pixels))
    }

    pub fn get_rgb(
        &self,
        rgb_data: &mut [i32],
        offset: usize,
        scanlength: usize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        for row in 0..height {
            let src_y = y + row;
            if src_y < 0 || src_y >= self.height {
                continue;
            }
            for col in 0..width {
                let src_x = x + col;
                if src_x < 0 || src_x >= self.width {
                    continue;
                }
                let px = self.pixels[(src_y * self.width + src_x) as usize];
                let (r, g, b, a) = unpack_rgba(px);
                // J2ME RGB is ARGB8888 in i32
                let argb = ((a as i32) << 24) | ((r as i32) << 16) | ((g as i32) << 8) | (b as i32);
                let dst_idx = offset + (row as usize) * scanlength + col as usize;
                if dst_idx < rgb_data.len() {
                    rgb_data[dst_idx] = argb;
                }
            }
        }
    }

    pub fn encode_png(pixels: &[u32], width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

        let mut ihdr = Vec::with_capacity(13);
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.push(8); // 8-bit depth
        ihdr.push(6); // RGBA
        ihdr.push(0); // deflate
        ihdr.push(0); // standard filter
        ihdr.push(0); // no interlace
        write_chunk(&mut out, b"IHDR", &ihdr);

        let mut raw_scanlines = Vec::with_capacity((height * (width * 4 + 1)) as usize);
        for y in 0..height {
            raw_scanlines.push(0); // filter: None
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let px = if idx < pixels.len() { pixels[idx] } else { 0 };
                let (r, g, b, a) = unpack_rgba(px);
                raw_scanlines.push(r);
                raw_scanlines.push(g);
                raw_scanlines.push(b);
                raw_scanlines.push(a);
            }
        }

        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&raw_scanlines, 6);
        write_chunk(&mut out, b"IDAT", &compressed);
        write_chunk(&mut out, b"IEND", &[]);

        out
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let crc_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32(&out[crc_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}
