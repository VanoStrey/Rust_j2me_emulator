use super::manifest::Manifest;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Jad {
    pub properties: HashMap<String, String>,
}

impl Jad {
    /// Parses a Java Application Descriptor (JAD) file.
    /// Handles UTF-8 with optional BOM, CRLF/LF line endings,
    /// and continuation lines starting with space or tab.
    pub fn parse(text: &str) -> Self {
        let clean_text = text.strip_prefix("\u{FEFF}").unwrap_or(text);
        let mut properties = HashMap::new();
        let mut current_key: Option<String> = None;
        let mut current_val = String::new();

        for line in clean_text.lines() {
            let trimmed_end = line.trim_end();
            if trimmed_end.is_empty() {
                continue;
            }

            if line.starts_with(' ') || line.starts_with('\t') {
                // Continuation line
                if current_key.is_some() {
                    current_val.push_str(line.trim_start());
                }
            } else {
                if let Some(key) = current_key.take() {
                    properties.insert(key, current_val.trim().to_string());
                    current_val.clear();
                }
                if let Some((k, v)) = line.split_once(':') {
                    current_key = Some(k.trim().to_string());
                    current_val.push_str(v.trim());
                }
            }
        }

        if let Some(key) = current_key {
            properties.insert(key, current_val.trim().to_string());
        }

        Self { properties }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    /// Resolves an application property following MIDP specification:
    /// JAD descriptor properties take precedence over JAR MANIFEST.MF properties.
    pub fn resolve_property<'a>(
        jad: Option<&'a Jad>,
        manifest: Option<&'a Manifest>,
        key: &str,
    ) -> Option<&'a str> {
        let clean_key = key.trim();
        if let Some(j) = jad {
            if let Some(val) = j.get(clean_key) {
                return Some(val);
            }
            if let Some((_, val)) = j
                .properties
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(clean_key))
            {
                return Some(val.as_str());
            }
        }
        if let Some(m) = manifest {
            if let Some(val) = m.get(clean_key) {
                return Some(val);
            }
            if let Some((_, val)) = m
                .properties
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(clean_key))
            {
                return Some(val.as_str());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jad_with_continuation_and_bom() {
        let jad_text = concat!(
            "\u{FEFF}MIDlet-Name: Super Game\r\n",
            "MIDlet-Version: 2.0.1\r\n",
            "MIDlet-Vendor: Gameloft\r\n",
            "Custom-Property: Very Long \r\n",
            " Value Continued Here\r\n",
            "MIDlet-Jar-URL: game.jar\r\n",
        );

        let jad = Jad::parse(jad_text);
        assert_eq!(jad.get("MIDlet-Name"), Some("Super Game"));
        assert_eq!(jad.get("MIDlet-Version"), Some("2.0.1"));
        assert_eq!(jad.get("MIDlet-Vendor"), Some("Gameloft"));
        assert_eq!(
            jad.get("Custom-Property"),
            Some("Very LongValue Continued Here")
        );
        assert_eq!(jad.get("MIDlet-Jar-URL"), Some("game.jar"));
    }

    #[test]
    fn jad_takes_precedence_over_manifest() {
        let manifest_text = "MIDlet-Name: Manifest Title\r\n\
MIDlet-Vendor: Vendor A\r\n\
Manifest-Only: 12345\r\n";
        let jad_text = "MIDlet-Name: JAD Overridden Title\r\n\
Jad-Only: 67890\r\n";

        let manifest = Manifest::parse(manifest_text);
        let jad = Jad::parse(jad_text);

        // JAD overrides manifest
        assert_eq!(
            Jad::resolve_property(Some(&jad), Some(&manifest), "MIDlet-Name"),
            Some("JAD Overridden Title")
        );
        // Falls back to manifest if missing in JAD
        assert_eq!(
            Jad::resolve_property(Some(&jad), Some(&manifest), "MIDlet-Vendor"),
            Some("Vendor A")
        );
        assert_eq!(
            Jad::resolve_property(Some(&jad), Some(&manifest), "Manifest-Only"),
            Some("12345")
        );
        // JAD only property
        assert_eq!(
            Jad::resolve_property(Some(&jad), Some(&manifest), "Jad-Only"),
            Some("67890")
        );
        // Missing in both
        assert_eq!(
            Jad::resolve_property(Some(&jad), Some(&manifest), "Non-Existent"),
            None
        );
    }
}
