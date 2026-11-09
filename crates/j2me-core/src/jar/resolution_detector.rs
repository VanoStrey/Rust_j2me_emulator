//! Resolution Detector for J2ME MIDlets.
//!
//! Provides resolution-agnostic auto-detection prioritizing:
//! 1. Application Manifest / JAD attributes
//! 2. JAR / JAD filename conventions and patterns
//! 3. Embedded graphic assets inspection (PNG IHDR headers of splash/title/bg)
//! 4. Safe fallback default (240x320 QVGA)

use super::jad::Jad;
use super::jar_loader::JarArchive;
use super::manifest::Manifest;

/// Known standard J2ME display resolutions.
pub const STANDARD_RESOLUTIONS: &[(u32, u32)] = &[
    (128, 128), // Nokia Series 40 v1
    (128, 160), // Nokia Series 40 v2 / Motorola
    (176, 208), // Nokia Series 60 v1/v2 / N-Gage
    (176, 220), // Sony Ericsson K700/K750, Motorola E398
    (208, 208), // Nokia 6230 / Series 40 square
    (240, 320), // Standard QVGA (Nokia N73/N95, SE K800i)
    (320, 240), // Landscape QVGA (Nokia E61/E71)
    (240, 400), // WQVGA touch (Samsung Star / LG Cookie)
    (352, 416), // Nokia Series 60 v3 high-res (Nokia N90)
    (360, 640), // Symbian^1 / S60v5 touch (Nokia 5800 / N97)
    (480, 800), // High-end WVGA
    (176, 144), // Sub-QCIF
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    Manifest,
    FileName,
    AssetInspection,
    FallbackDefault,
}

impl std::fmt::Display for ResolutionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionSource::Manifest => write!(f, "MANIFEST"),
            ResolutionSource::FileName => write!(f, "FILENAME"),
            ResolutionSource::AssetInspection => write!(f, "ASSET_INSPECTION"),
            ResolutionSource::FallbackDefault => write!(f, "FALLBACK_DEFAULT"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedResolution {
    pub width: u32,
    pub height: u32,
    pub source: ResolutionSource,
}

pub struct ResolutionDetector;

impl ResolutionDetector {
    /// Detect screen resolution using the full priority hierarchy.
    pub fn detect(
        manifest: Option<&Manifest>,
        jad: Option<&Jad>,
        file_name: Option<&str>,
        jar: Option<&JarArchive>,
    ) -> DetectedResolution {
        // Priority 1: Manifest / JAD Attributes
        if let Some((w, h)) = Self::detect_from_manifest_or_jad(manifest, jad) {
            return DetectedResolution {
                width: w,
                height: h,
                source: ResolutionSource::Manifest,
            };
        }

        // Priority 2: File Name Heuristics
        if let Some(name) = file_name {
            if let Some((w, h)) = Self::detect_from_file_name(name) {
                return DetectedResolution {
                    width: w,
                    height: h,
                    source: ResolutionSource::FileName,
                };
            }
        }

        // Priority 3: Inspection of Graphical Resources (PNG IHDR in splash/bg/title)
        if let Some(archive) = jar {
            if let Some((w, h)) = Self::detect_from_assets(archive) {
                return DetectedResolution {
                    width: w,
                    height: h,
                    source: ResolutionSource::AssetInspection,
                };
            }
        }

        // Priority 4: Fallback Default (240x320 QVGA)
        DetectedResolution {
            width: 240,
            height: 320,
            source: ResolutionSource::FallbackDefault,
        }
    }

    /// Helper to detect directly from raw JAR bytes and optional file/JAD context.
    pub fn detect_from_jar_bytes(
        jar_bytes: &[u8],
        file_name: Option<&str>,
        jad_str: Option<&str>,
    ) -> DetectedResolution {
        let jad = jad_str.map(Jad::parse);
        match JarArchive::parse(jar_bytes) {
            Ok(jar) => Self::detect(
                jar.manifest.as_ref(),
                jad.as_ref(),
                file_name,
                Some(&jar),
            ),
            Err(_) => {
                // If JAR parse fails, still try filename and jad
                Self::detect(None, jad.as_ref(), file_name, None)
            }
        }
    }

    /// Priority 1: Check Manifest & JAD properties.
    pub fn detect_from_manifest_or_jad(
        manifest: Option<&Manifest>,
        jad: Option<&Jad>,
    ) -> Option<(u32, u32)> {
        // Properties with combined WxH or W,H
        const COMBINED_KEYS: &[&str] = &[
            "Nokia-MIDlet-Original-Display-Size",
            "Nokia-MIDlet-Target-Display-Size",
            "MIDlet-ScreenSize",
            "SE-MIDlet-Original-Display-Size",
            "SE-MIDlet-Target-Display-Size",
            "Resolution",
            "Screen-Size",
            "ScreenSize",
            "MIDlet-Display-Size",
            "MIDlet-Resolution",
        ];

        for &key in COMBINED_KEYS {
            if let Some(val) = Jad::resolve_property(jad, manifest, key) {
                if let Some((w, h)) = Self::parse_dimension_str(val) {
                    return Some((w, h));
                }
            }
        }

        // Check separate Width & Height keys
        const WIDTH_KEYS: &[&str] = &[
            "MIDlet-Screen-Width",
            "MIDlet-Target-Width",
            "Nokia-MIDlet-Original-Display-Width",
            "Display-Width",
        ];
        const HEIGHT_KEYS: &[&str] = &[
            "MIDlet-Screen-Height",
            "MIDlet-Target-Height",
            "Nokia-MIDlet-Original-Display-Height",
            "Display-Height",
        ];

        for &w_key in WIDTH_KEYS {
            if let Some(w_str) = Jad::resolve_property(jad, manifest, w_key) {
                if let Ok(w) = w_str.trim().parse::<u32>() {
                    for &h_key in HEIGHT_KEYS {
                        if let Some(h_str) = Jad::resolve_property(jad, manifest, h_key) {
                            if let Ok(h) = h_str.trim().parse::<u32>() {
                                if (32..=1920).contains(&w) && (32..=1920).contains(&h) {
                                    return Some((w, h));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Priority 2: Extract resolution from file name.
    pub fn detect_from_file_name(file_name: &str) -> Option<(u32, u32)> {
        let base_name = std::path::Path::new(file_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        // Strip known extensions
        let clean_name = base_name
            .trim_end_matches(".jar")
            .trim_end_matches(".jad")
            .trim_end_matches(".JAR")
            .trim_end_matches(".JAD");

        // 1. Search for 2D patterns: WxH (e.g. 240x320, 176_208, 176-220, 320*240, 240×320)
        if let Some((w, h)) = Self::find_2d_resolution_in_string(clean_name) {
            return Some((w, h));
        }

        // 2. Search for common single-dimension shorthand suffixes:
        // e.g. "Game-208", "Game_208", "Game-128", "Game_128", "Game-160", "Game-176", "Game-220", "Game-240"
        Self::find_shorthand_suffix(clean_name)
    }

    /// Search for WxH patterns inside an arbitrary string.
    fn find_2d_resolution_in_string(s: &str) -> Option<(u32, u32)> {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Find start of a number (width)
            if chars[i].is_ascii_digit() {
                let start_w = i;
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let end_w = i;
                let w_digits = end_w - start_w;

                // Typical width is 3 or 4 digits (e.g. 128, 176, 240, 320, 480, 800)
                if (3..=4).contains(&w_digits) {
                    if let Ok(w) = s[start_w..end_w].parse::<u32>() {
                        // Check separator: 'x', 'X', '×', '*', '_', '-'
                        if i < len {
                            let sep = chars[i];
                            let is_sep = sep == 'x' || sep == 'X' || sep == '×' || sep == '*' || sep == '_' || sep == '-';
                            if is_sep {
                                i += 1; // skip separator
                                let start_h = i;
                                while i < len && chars[i].is_ascii_digit() {
                                    i += 1;
                                }
                                let end_h = i;
                                let h_digits = end_h - start_h;
                                if (3..=4).contains(&h_digits) {
                                    if let Ok(h) = s[start_h..end_h].parse::<u32>() {
                                        // Validate dimensions: either in standard table or valid aspect range
                                        if Self::is_plausible_resolution(w, h) {
                                            return Some((w, h));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        None
    }

    /// Detect common single-dimension shorthand markers in filenames.
    fn find_shorthand_suffix(s: &str) -> Option<(u32, u32)> {
        let lower = s.to_lowercase();

        // Shorthand rules ordered by specificity
        let shorthand_rules: &[(&[&str], u32, u32)] = &[
            (&["-208", "_208", " 208"], 176, 208), // Nokia Series 60 v1/v2
            (&["-128", "_128", " 128"], 128, 128), // Nokia Series 40 v1
            (&["-160", "_160", " 160"], 128, 160), // Nokia Series 40 v2
            (&["-220", "_220", " 220"], 176, 220), // Sony Ericsson / Motorola
            (&["-176", "_176", " 176"], 176, 220), // Mid-range default
            (&["-320", "_320", " 320"], 240, 320), // QVGA Portrait
            (&["-240", "_240", " 240"], 240, 320), // QVGA
            (&["-416", "_416", " 416"], 352, 416), // High-res Nokia
            (&["-640", "_640", " 640"], 360, 640), // Symbian Touch
            (&["-800", "_800", " 800"], 480, 800), // WVGA
        ];

        for (markers, w, h) in shorthand_rules {
            for marker in *markers {
                if let Some(pos) = lower.find(marker) {
                    // Check boundary: end of string or non-digit follows
                    let end_pos = pos + marker.len();
                    let boundary = end_pos == lower.len()
                        || !lower.as_bytes()[end_pos].is_ascii_digit();
                    if boundary {
                        return Some((*w, *h));
                    }
                }
            }
        }

        None
    }

    /// Priority 3: Scan archive for PNG headers in splash / background / menu / title images.
    pub fn detect_from_assets(jar: &JarArchive) -> Option<(u32, u32)> {
        // Collect PNG dimensions, prioritizing background/splash images
        let mut prioritized_dims: Vec<(u32, u32)> = Vec::new();
        let mut general_dims: Vec<(u32, u32)> = Vec::new();

        for (name, data) in &jar.entries {
            let l_name = name.to_lowercase();
            let is_priority_name = l_name.contains("splash")
                || l_name.contains("bg")
                || l_name.contains("title")
                || l_name.contains("menu")
                || l_name.contains("intro")
                || l_name.contains("background")
                || l_name.contains("screen")
                || l_name.contains("main");

            // Inspect PNG IHDR
            if let Some((w, h)) = Self::extract_png_ihdr_dimensions(data) {
                if Self::is_plausible_resolution(w, h) {
                    if is_priority_name {
                        prioritized_dims.push((w, h));
                    } else {
                        general_dims.push((w, h));
                    }
                }
            }
        }

        // 1. Check exact match in prioritized images with standard resolutions
        for &(sw, sh) in STANDARD_RESOLUTIONS {
            if prioritized_dims.iter().any(|&(w, h)| w == sw && h == sh) {
                return Some((sw, sh));
            }
        }

        // 2. Check exact match in general images with standard resolutions
        for &(sw, sh) in STANDARD_RESOLUTIONS {
            if general_dims.iter().any(|&(w, h)| w == sw && h == sh) {
                return Some((sw, sh));
            }
        }

        // 3. If prioritized has at least one plausible resolution, use the most frequent or maximum
        if let Some(&(w, h)) = prioritized_dims.iter().max_by_key(|(w, h)| w * h) {
            return Some((w, h));
        }

        None
    }

    /// Parse dimension string like "240x320", "240,320", "240*320", "240×320".
    fn parse_dimension_str(s: &str) -> Option<(u32, u32)> {
        let clean = s.trim();
        let parts: Vec<&str> = if clean.contains('x') || clean.contains('X') {
            clean.split(|c| c == 'x' || c == 'X').collect()
        } else if clean.contains('×') {
            clean.split('×').collect()
        } else if clean.contains(',') {
            clean.split(',').collect()
        } else if clean.contains('*') {
            clean.split('*').collect()
        } else {
            return None;
        };

        if parts.len() >= 2 {
            let w = parts[0].trim().parse::<u32>().ok()?;
            let h = parts[1].trim().parse::<u32>().ok()?;
            if (32..=1920).contains(&w) && (32..=1920).contains(&h) {
                return Some((w, h));
            }
        }
        None
    }

    /// Extract width and height from PNG IHDR chunk without fully decoding image.
    fn extract_png_ihdr_dimensions(data: &[u8]) -> Option<(u32, u32)> {
        let mut idx = 0;
        while idx + 24 <= data.len() {
            if &data[idx..idx + 8] == b"\x89PNG\r\n\x1a\n" && &data[idx + 12..idx + 16] == b"IHDR" {
                let w = u32::from_be_bytes([
                    data[idx + 16],
                    data[idx + 17],
                    data[idx + 18],
                    data[idx + 19],
                ]);
                let h = u32::from_be_bytes([
                    data[idx + 20],
                    data[idx + 21],
                    data[idx + 22],
                    data[idx + 23],
                ]);
                return Some((w, h));
            }
            idx += 1;
        }
        None
    }

    fn is_plausible_resolution(w: u32, h: u32) -> bool {
        // Direct standard check
        if STANDARD_RESOLUTIONS.iter().any(|&(sw, sh)| sw == w && sh == h) {
            return true;
        }

        // Acceptable J2ME mobile screen bounds: min 64px, max 1920px
        if !(64..=1920).contains(&w) || !(64..=1920).contains(&h) {
            return false;
        }

        // Check reasonable aspect ratio between 1:3 and 3:1
        let ratio = w as f64 / h as f64;
        (0.33..=3.0).contains(&ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_nokia_original_display_size() {
        let m = Manifest::parse("Nokia-MIDlet-Original-Display-Size: 240,320\r\n");
        let res = ResolutionDetector::detect(Some(&m), None, None, None);
        assert_eq!(res.width, 240);
        assert_eq!(res.height, 320);
        assert_eq!(res.source, ResolutionSource::Manifest);
    }

    #[test]
    fn test_manifest_resolution_key() {
        let m = Manifest::parse("Resolution: 176x220\r\n");
        let res = ResolutionDetector::detect(Some(&m), None, None, None);
        assert_eq!(res.width, 176);
        assert_eq!(res.height, 220);
        assert_eq!(res.source, ResolutionSource::Manifest);
    }

    #[test]
    fn test_manifest_separate_width_and_height() {
        let m = Manifest::parse("MIDlet-Screen-Width: 128\r\nMIDlet-Screen-Height: 160\r\n");
        let res = ResolutionDetector::detect(Some(&m), None, None, None);
        assert_eq!(res.width, 128);
        assert_eq!(res.height, 160);
        assert_eq!(res.source, ResolutionSource::Manifest);
    }

    #[test]
    fn test_jad_overrides_manifest() {
        let m = Manifest::parse("Resolution: 128x128\r\n");
        let j = Jad::parse("Nokia-MIDlet-Original-Display-Size: 240,320\r\n");
        let res = ResolutionDetector::detect(Some(&m), Some(&j), None, None);
        assert_eq!(res.width, 240);
        assert_eq!(res.height, 320);
        assert_eq!(res.source, ResolutionSource::Manifest);
    }

    #[test]
    fn test_filename_exact_2d_formats() {
        let test_cases = [
            ("RoboBombo128x128.jar", 128, 128),
            ("SaveTheSheriff_176x208.jar", 176, 208),
            ("WALL-E-176x220.jar", 176, 220),
            ("Sonic-240x320-mod.jar", 240, 320),
            ("DoomIIRPG-320x240.jar", 320, 240),
            ("CrazyPenguinParty-5-320x240.jar", 320, 240),
            ("ActionGame_352x416.jar", 352, 416),
            ("TouchRPG-360x640.jar", 360, 640),
            ("Game_480x800.jar", 480, 800),
            ("Game_208x208.jar", 208, 208),
            ("Special_176-220_SE.jar", 176, 220),
        ];

        for (filename, exp_w, exp_h) in test_cases {
            let res = ResolutionDetector::detect(None, None, Some(filename), None);
            assert_eq!(
                (res.width, res.height),
                (exp_w, exp_h),
                "Failed on filename: {}",
                filename
            );
            assert_eq!(res.source, ResolutionSource::FileName);
        }
    }

    #[test]
    fn test_filename_shorthand_suffixes() {
        let test_cases = [
            ("BoulderDash-128.jar", 128, 128),
            ("MotoGP-208.jar", 176, 208),
            ("Castlevania_160.jar", 128, 160),
            ("PrinceOfPersia_220.jar", 176, 220),
            ("Racing_320.jar", 240, 320),
        ];

        for (filename, exp_w, exp_h) in test_cases {
            let res = ResolutionDetector::detect(None, None, Some(filename), None);
            assert_eq!(
                (res.width, res.height),
                (exp_w, exp_h),
                "Failed on shorthand: {}",
                filename
            );
            assert_eq!(res.source, ResolutionSource::FileName);
        }
    }

    #[test]
    fn test_fallback_default_qvga() {
        let res = ResolutionDetector::detect(None, None, Some("UnknownGame.jar"), None);
        assert_eq!(res.width, 240);
        assert_eq!(res.height, 320);
        assert_eq!(res.source, ResolutionSource::FallbackDefault);
    }
}
