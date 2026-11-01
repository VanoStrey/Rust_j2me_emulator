#[derive(Debug, Clone, PartialEq)]
pub enum ConstantPoolEntry {
    Empty, // Placeholder for 0-index and slot after Long/Double
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_index: u16,
    },
    StringRef {
        string_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    pub entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    pub fn parse(reader: &mut &[u8]) -> Result<Self, String> {
        if reader.len() < 2 {
            return Err("Unexpected EOF reading constant pool count".to_string());
        }
        let count = u16::from_be_bytes([reader[0], reader[1]]);
        *reader = &reader[2..];

        let mut entries = Vec::with_capacity(count as usize);
        entries.push(ConstantPoolEntry::Empty); // Entry 0 is unused in JVM spec

        let mut i = 1;
        while i < count {
            if reader.is_empty() {
                return Err("Unexpected EOF reading constant pool tag".to_string());
            }
            let tag = reader[0];
            *reader = &reader[1..];

            match tag {
                1 => {
                    // Utf8
                    if reader.len() < 2 {
                        return Err("Unexpected EOF reading Utf8 length".to_string());
                    }
                    let len = u16::from_be_bytes([reader[0], reader[1]]) as usize;
                    *reader = &reader[2..];
                    if reader.len() < len {
                        return Err("Unexpected EOF reading Utf8 bytes".to_string());
                    }
                    let bytes = &reader[..len];
                    *reader = &reader[len..];
                    // Modified UTF-8 in Java, fallback to lossy utf-8
                    let s = String::from_utf8_lossy(bytes).into_owned();
                    entries.push(ConstantPoolEntry::Utf8(s));
                    i += 1;
                }
                3 => {
                    // Integer
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading Integer".to_string());
                    }
                    let val = i32::from_be_bytes([reader[0], reader[1], reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::Integer(val));
                    i += 1;
                }
                4 => {
                    // Float
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading Float".to_string());
                    }
                    let bits = u32::from_be_bytes([reader[0], reader[1], reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::Float(f32::from_bits(bits)));
                    i += 1;
                }
                5 => {
                    // Long (takes 2 slots)
                    if reader.len() < 8 {
                        return Err("Unexpected EOF reading Long".to_string());
                    }
                    let val = i64::from_be_bytes([
                        reader[0], reader[1], reader[2], reader[3], reader[4], reader[5],
                        reader[6], reader[7],
                    ]);
                    *reader = &reader[8..];
                    entries.push(ConstantPoolEntry::Long(val));
                    entries.push(ConstantPoolEntry::Empty);
                    i += 2;
                }
                6 => {
                    // Double (takes 2 slots)
                    if reader.len() < 8 {
                        return Err("Unexpected EOF reading Double".to_string());
                    }
                    let bits = u64::from_be_bytes([
                        reader[0], reader[1], reader[2], reader[3], reader[4], reader[5],
                        reader[6], reader[7],
                    ]);
                    *reader = &reader[8..];
                    entries.push(ConstantPoolEntry::Double(f64::from_bits(bits)));
                    entries.push(ConstantPoolEntry::Empty);
                    i += 2;
                }
                7 => {
                    // Class
                    if reader.len() < 2 {
                        return Err("Unexpected EOF reading Class index".to_string());
                    }
                    let name_index = u16::from_be_bytes([reader[0], reader[1]]);
                    *reader = &reader[2..];
                    entries.push(ConstantPoolEntry::Class { name_index });
                    i += 1;
                }
                8 => {
                    // String
                    if reader.len() < 2 {
                        return Err("Unexpected EOF reading String index".to_string());
                    }
                    let string_index = u16::from_be_bytes([reader[0], reader[1]]);
                    *reader = &reader[2..];
                    entries.push(ConstantPoolEntry::StringRef { string_index });
                    i += 1;
                }
                9 => {
                    // FieldRef
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading FieldRef".to_string());
                    }
                    let class_index = u16::from_be_bytes([reader[0], reader[1]]);
                    let name_and_type_index = u16::from_be_bytes([reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::FieldRef {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                10 => {
                    // MethodRef
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading MethodRef".to_string());
                    }
                    let class_index = u16::from_be_bytes([reader[0], reader[1]]);
                    let name_and_type_index = u16::from_be_bytes([reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::MethodRef {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                11 => {
                    // InterfaceMethodRef
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading InterfaceMethodRef".to_string());
                    }
                    let class_index = u16::from_be_bytes([reader[0], reader[1]]);
                    let name_and_type_index = u16::from_be_bytes([reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                    });
                    i += 1;
                }
                12 => {
                    // NameAndType
                    if reader.len() < 4 {
                        return Err("Unexpected EOF reading NameAndType".to_string());
                    }
                    let name_index = u16::from_be_bytes([reader[0], reader[1]]);
                    let descriptor_index = u16::from_be_bytes([reader[2], reader[3]]);
                    *reader = &reader[4..];
                    entries.push(ConstantPoolEntry::NameAndType {
                        name_index,
                        descriptor_index,
                    });
                    i += 1;
                }
                other => {
                    return Err(format!("Unsupported constant pool tag: {}", other));
                }
            }
        }

        Ok(Self { entries })
    }

    pub fn get_utf8(&self, index: u16) -> Option<&str> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Utf8(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_class_name(&self, index: u16) -> Option<&str> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Class { name_index }) => self.get_utf8(*name_index),
            _ => None,
        }
    }

    pub fn get_string(&self, index: u16) -> Option<&str> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::StringRef { string_index }) => self.get_utf8(*string_index),
            _ => None,
        }
    }

    pub fn get_name_and_type(&self, index: u16) -> Option<(&str, &str)> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            }) => {
                let name = self.get_utf8(*name_index)?;
                let desc = self.get_utf8(*descriptor_index)?;
                Some((name, desc))
            }
            _ => None,
        }
    }

    pub fn get_field_ref(&self, index: u16) -> Option<(&str, &str, &str)> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            }) => {
                let class_name = self.get_class_name(*class_index)?;
                let (name, desc) = self.get_name_and_type(*name_and_type_index)?;
                Some((class_name, name, desc))
            }
            _ => None,
        }
    }

    pub fn get_method_ref(&self, index: u16) -> Option<(&str, &str, &str)> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            })
            | Some(ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            }) => {
                let class_name = self.get_class_name(*class_index)?;
                let (name, desc) = self.get_name_and_type(*name_and_type_index)?;
                Some((class_name, name, desc))
            }
            _ => None,
        }
    }
}
