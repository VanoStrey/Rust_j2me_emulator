/// Windows-1251 (CP1251) to Unicode conversion table for bytes 0x80..=0xFF
static CP1251_TABLE: [u16; 128] = [
    0x0402, 0x0403, 0x201A, 0x0453, 0x201E, 0x2026, 0x2020, 0x2021, // 0x80 - 0x87
    0x20AC, 0x2030, 0x0409, 0x2039, 0x040A, 0x040C, 0x040B, 0x040F, // 0x88 - 0x8F
    0x0452, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, // 0x90 - 0x97
    0x0098, 0x2122, 0x0459, 0x203A, 0x045A, 0x045C, 0x045B, 0x045F, // 0x98 - 0x9F
    0x00A0, 0x040E, 0x045E, 0x0408, 0x00A4, 0x0490, 0x00A6, 0x00A7, // 0xA0 - 0xA7
    0x0401, 0x00A9, 0x0404, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x0407, // 0xA8 - 0xAF
    0x00B0, 0x00B1, 0x0406, 0x0456, 0x0491, 0x00B5, 0x00B6, 0x00B7, // 0xB0 - 0xB7
    0x0451, 0x2116, 0x0454, 0x00BB, 0x0458, 0x0405, 0x0455, 0x0457, // 0xB8 - 0xBF
    // 0xC0..0xFF: А-Я, а-я
    0x0410, 0x0411, 0x0412, 0x0413, 0x0414, 0x0415, 0x0416, 0x0417, 0x0418, 0x0419, 0x041A, 0x041B,
    0x041C, 0x041D, 0x041E, 0x041F, 0x0420, 0x0421, 0x0422, 0x0423, 0x0424, 0x0425, 0x0426, 0x0427,
    0x0428, 0x0429, 0x042A, 0x042B, 0x042C, 0x042D, 0x042E, 0x042F, 0x0430, 0x0431, 0x0432, 0x0433,
    0x0434, 0x0435, 0x0436, 0x0437, 0x0438, 0x0439, 0x043A, 0x043B, 0x043C, 0x043D, 0x043E, 0x043F,
    0x0440, 0x0441, 0x0442, 0x0443, 0x0444, 0x0445, 0x0446, 0x0447, 0x0448, 0x0449, 0x044A, 0x044B,
    0x044C, 0x044D, 0x044E, 0x044F,
];

pub fn decode_bytes(bytes: &[u8], encoding: &str) -> String {
    let enc = encoding.to_ascii_lowercase();
    match enc.as_str() {
        "windows-1251" | "cp1251" | "win-1251" => {
            let mut u16_chars = Vec::with_capacity(bytes.len());
            for &b in bytes {
                if b < 0x80 {
                    u16_chars.push(b as u16);
                } else {
                    u16_chars.push(CP1251_TABLE[(b - 0x80) as usize]);
                }
            }
            String::from_utf16_lossy(&u16_chars)
        }
        "iso-8859-1" | "latin1" | "iso_8859_1" => bytes.iter().map(|&b| b as char).collect(),
        _ => {
            // Default to UTF-8 lossy
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
}

pub fn encode_string(s: &str, encoding: &str) -> Vec<u8> {
    let enc = encoding.to_ascii_lowercase();
    match enc.as_str() {
        "windows-1251" | "cp1251" | "win-1251" => {
            let mut res = Vec::with_capacity(s.len());
            for ch in s.chars() {
                let code = ch as u32;
                if code < 0x80 {
                    res.push(code as u8);
                } else {
                    let mut found = false;
                    for (i, &u) in CP1251_TABLE.iter().enumerate() {
                        if u as u32 == code {
                            res.push((0x80 + i) as u8);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        res.push(b'?');
                    }
                }
            }
            res
        }
        "iso-8859-1" | "latin1" => s
            .chars()
            .map(|c| if (c as u32) < 256 { c as u8 } else { b'?' })
            .collect(),
        _ => s.as_bytes().to_vec(),
    }
}
