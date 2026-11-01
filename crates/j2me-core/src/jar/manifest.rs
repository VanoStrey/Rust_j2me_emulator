use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub properties: HashMap<String, String>,
}

impl Manifest {
    pub fn parse(text: &str) -> Self {
        let mut properties = HashMap::new();
        let mut current_key: Option<String> = None;
        let mut current_val = String::new();

        for line in text.lines() {
            if line.is_empty() {
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

    pub fn midlet_name(&self) -> Option<&str> {
        self.get("MIDlet-Name")
    }

    pub fn midlet_version(&self) -> Option<&str> {
        self.get("MIDlet-Version")
    }

    pub fn midlet_vendor(&self) -> Option<&str> {
        self.get("MIDlet-Vendor")
    }

    pub fn get_midlet_info(&self, index: usize) -> Option<(String, String, String)> {
        let key = format!("MIDlet-{}", index);
        let val = self.get(&key)?;
        let parts: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else if parts.len() == 1 {
            Some((parts[0].to_string(), String::new(), parts[0].to_string()))
        } else {
            None
        }
    }

    pub fn get_primary_midlet_class(&self) -> Option<String> {
        if let Some((_, _, class_name)) = self.get_midlet_info(1) {
            return Some(class_name);
        }
        if let Some(c) = self.get("MIDlet-Class") {
            return Some(c.to_string());
        }
        None
    }
}
