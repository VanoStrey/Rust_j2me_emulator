use super::manifest::Manifest;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct JarEntry {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct JarArchive {
    pub manifest: Option<Manifest>,
    pub entries: HashMap<String, Vec<u8>>,
}

impl JarArchive {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut entries = HashMap::new();

        // Scan backwards for End of Central Directory (EOCD) signature: 0x06054b50
        if bytes.len() < 22 {
            return Err("File too small to be a valid ZIP/JAR".to_string());
        }

        let search_start = if bytes.len() > 65557 {
            bytes.len() - 65557
        } else {
            0
        };
        let mut eocd_pos = None;
        for i in (search_start..=bytes.len() - 22).rev() {
            if bytes[i] == 0x50
                && bytes[i + 1] == 0x4b
                && bytes[i + 2] == 0x05
                && bytes[i + 3] == 0x06
            {
                eocd_pos = Some(i);
                break;
            }
        }

        let eocd_pos =
            eocd_pos.ok_or_else(|| "ZIP End of Central Directory not found".to_string())?;
        let total_entries =
            u16::from_le_bytes([bytes[eocd_pos + 10], bytes[eocd_pos + 11]]) as usize;
        let cd_offset = u32::from_le_bytes([
            bytes[eocd_pos + 16],
            bytes[eocd_pos + 17],
            bytes[eocd_pos + 18],
            bytes[eocd_pos + 19],
        ]) as usize;

        // Detect prepended / SFX / DoCoMo headers shifting central directory
        let mut shift = 0isize;
        if cd_offset + 4 > bytes.len() || &bytes[cd_offset..cd_offset + 4] != [0x50, 0x4b, 0x01, 0x02] {
            for i in 0..bytes.len().saturating_sub(4) {
                if bytes[i] == 0x50
                    && bytes[i + 1] == 0x4b
                    && bytes[i + 2] == 0x01
                    && bytes[i + 3] == 0x02
                {
                    shift = i as isize - cd_offset as isize;
                    break;
                }
            }
        }

        let mut pos = (cd_offset as isize + shift).max(0) as usize;
        for _ in 0..total_entries {
            if pos + 46 > bytes.len() {
                break;
            }
            let sig =
                u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
            if sig != 0x02014b50 {
                break;
            }
            let method = u16::from_le_bytes([bytes[pos + 10], bytes[pos + 11]]);
            let compressed_size = u32::from_le_bytes([
                bytes[pos + 20],
                bytes[pos + 21],
                bytes[pos + 22],
                bytes[pos + 23],
            ]) as usize;
            let _uncompressed_size = u32::from_le_bytes([
                bytes[pos + 24],
                bytes[pos + 25],
                bytes[pos + 26],
                bytes[pos + 27],
            ]) as usize;
            let fname_len = u16::from_le_bytes([bytes[pos + 28], bytes[pos + 29]]) as usize;
            let extra_len = u16::from_le_bytes([bytes[pos + 30], bytes[pos + 31]]) as usize;
            let comment_len = u16::from_le_bytes([bytes[pos + 32], bytes[pos + 33]]) as usize;
            let raw_local_offset = u32::from_le_bytes([
                bytes[pos + 42],
                bytes[pos + 43],
                bytes[pos + 44],
                bytes[pos + 45],
            ]) as usize;
            let local_header_offset = (raw_local_offset as isize + shift).max(0) as usize;

            let fname_start = pos + 46;
            if fname_start + fname_len > bytes.len() {
                break;
            }
            let name_bytes = &bytes[fname_start..fname_start + fname_len];
            let name = String::from_utf8_lossy(name_bytes).to_string();

            // Next entry in Central Directory
            pos = fname_start + fname_len + extra_len + comment_len;

            // Extract file content from local header
            if local_header_offset + 30 <= bytes.len() {
                let loc_fname_len = u16::from_le_bytes([
                    bytes[local_header_offset + 26],
                    bytes[local_header_offset + 27],
                ]) as usize;
                let loc_extra_len = u16::from_le_bytes([
                    bytes[local_header_offset + 28],
                    bytes[local_header_offset + 29],
                ]) as usize;
                let data_start = local_header_offset + 30 + loc_fname_len + loc_extra_len;

                if data_start + compressed_size <= bytes.len() {
                    let comp_data = &bytes[data_start..data_start + compressed_size];
                    let uncomp_data = match method {
                        0 => comp_data.to_vec(),
                        8 => match miniz_oxide::inflate::decompress_to_vec(comp_data) {
                            Ok(decomp) => decomp,
                            Err(e) => {
                                eprintln!("Failed to decompress entry {}: {:?}", name, e);
                                Vec::new()
                            }
                        },
                        _ => Vec::new(),
                    };

                    // Normalize path (leading slash removed)
                    let clean_name = name.trim_start_matches('/').to_string();
                    entries.insert(clean_name, uncomp_data);
                }
            }
        }

        // Resilient fallback: scan local file headers directly if central directory yielded 0 entries
        if entries.is_empty() {
            let mut i = 0;
            while i + 30 <= bytes.len() {
                if bytes[i] == 0x50 && bytes[i + 1] == 0x4b && bytes[i + 2] == 0x03 && bytes[i + 3] == 0x04 {
                    let method = u16::from_le_bytes([bytes[i + 8], bytes[i + 9]]);
                    let comp_size = u32::from_le_bytes([bytes[i + 18], bytes[i + 19], bytes[i + 20], bytes[i + 21]]) as usize;
                    let fname_len = u16::from_le_bytes([bytes[i + 26], bytes[i + 27]]) as usize;
                    let extra_len = u16::from_le_bytes([bytes[i + 28], bytes[i + 29]]) as usize;
                    let name_start = i + 30;
                    if name_start + fname_len <= bytes.len() {
                        let name = String::from_utf8_lossy(&bytes[name_start..name_start + fname_len]).to_string();
                        let data_start = name_start + fname_len + extra_len;
                        if comp_size > 0 && data_start + comp_size <= bytes.len() {
                            let comp_data = &bytes[data_start..data_start + comp_size];
                            let uncomp = match method {
                                0 => comp_data.to_vec(),
                                8 => miniz_oxide::inflate::decompress_to_vec(comp_data).unwrap_or_default(),
                                _ => Vec::new(),
                            };
                            let clean_name = name.trim_start_matches('/').to_string();
                            entries.insert(clean_name, uncomp);
                        }
                    }
                }
                i += 1;
            }
        }

        let mut manifest = entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("META-INF/MANIFEST.MF"))
            .map(|(_, data)| String::from_utf8_lossy(data))
            .map(|text| Manifest::parse(&text));

        // Auto-heal missing or incomplete manifest
        let needs_synthesis = match &manifest {
            None => true,
            Some(m) => m.get_primary_midlet_class().is_none(),
        };

        if needs_synthesis {
            let mut candidate_class = None;
            for (name, data) in &entries {
                if name.ends_with(".class") && data.len() >= 8 {
                    if let Ok(cf) = crate::classfile::class_file::ClassFile::parse(data) {
                        let has_start_app = cf.methods.iter().any(|m| m.name == "startApp");
                        let is_midlet = cf.super_class.as_deref() == Some("javax/microedition/midlet/MIDlet");
                        if has_start_app || is_midlet {
                            candidate_class = Some(cf.this_class.replace('/', "."));
                            break;
                        }
                    }
                }
            }
            if let Some(cls) = candidate_class {
                let mut m = manifest.unwrap_or_default();
                m.properties.entry("MIDlet-1".to_string()).or_insert_with(|| format!("App, , {}", cls));
                m.properties.entry("MIDlet-Name".to_string()).or_insert_with(|| cls.clone());
                m.properties.entry("MIDlet-Version".to_string()).or_insert_with(|| "1.0.0".to_string());
                m.properties.entry("MIDlet-Vendor".to_string()).or_insert_with(|| "AutoGenerated".to_string());
                m.properties.entry("MicroEdition-Configuration".to_string()).or_insert_with(|| "CLDC-1.1".to_string());
                m.properties.entry("MicroEdition-Profile".to_string()).or_insert_with(|| "MIDP-2.0".to_string());
                manifest = Some(m);
            }
        }

        Ok(Self { manifest, entries })
    }

    pub fn get_entry(&self, path: &str) -> Option<&[u8]> {
        let mut clean = path.trim();
        while clean.starts_with('/') || clean.starts_with("./") {
            if let Some(stripped) = clean.strip_prefix('/') {
                clean = stripped;
            } else if let Some(stripped) = clean.strip_prefix("./") {
                clean = stripped;
            }
        }

        // 1. Exact match
        if let Some(data) = self.entries.get(clean) {
            return Some(data.as_slice());
        }

        // 2. Case-insensitive match
        for (key, val) in &self.entries {
            if key.eq_ignore_ascii_case(clean) {
                return Some(val.as_slice());
            }
        }

        // 3. Basename/suffix search
        let filename = if let Some(idx) = clean.rfind('/') {
            &clean[idx + 1..]
        } else {
            clean
        };

        if !filename.is_empty() {
            for (key, val) in &self.entries {
                let entry_fname = if let Some(idx) = key.rfind('/') {
                    &key[idx + 1..]
                } else {
                    key.as_str()
                };
                if entry_fname.eq_ignore_ascii_case(filename) {
                    return Some(val.as_slice());
                }
            }
        }

        None
    }

    pub fn class_names(&self) -> Vec<String> {
        self.entries
            .keys()
            .filter(|k| k.ends_with(".class"))
            .map(|k| k.trim_end_matches(".class").replace('/', "."))
            .collect()
    }
}
