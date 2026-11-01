use super::constant_pool::ConstantPool;

#[derive(Debug, Clone)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16, // 0 means catches any exception (finally)
    /// Resolved class name of `catch_type` (`None` for catch-all handlers).
    pub catch_class: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub enum AttributeInfo {
    Code(CodeAttribute),
    ConstantValue(u16),
    Exceptions(Vec<u16>),
    StackMap(Vec<u8>), // CLDC 1.1 preverification data
    SourceFile(u16),
    LineNumberTable(Vec<(u16, u16)>),
    Other { name: String, data: Vec<u8> },
}

impl AttributeInfo {
    pub fn parse(reader: &mut &[u8], cp: &ConstantPool) -> Result<Self, String> {
        if reader.len() < 6 {
            return Err("Unexpected EOF reading attribute header".to_string());
        }
        let name_index = u16::from_be_bytes([reader[0], reader[1]]);
        let length = u32::from_be_bytes([reader[2], reader[3], reader[4], reader[5]]) as usize;
        *reader = &reader[6..];

        if reader.len() < length {
            return Err(format!(
                "Attribute data truncated: expected {}, got {}",
                length,
                reader.len()
            ));
        }
        let mut attr_bytes = &reader[..length];
        *reader = &reader[length..];

        let name = cp.get_utf8(name_index).unwrap_or("");
        match name {
            "Code" => {
                if attr_bytes.len() < 8 {
                    return Err("Invalid Code attribute length".to_string());
                }
                let max_stack = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]);
                let max_locals = u16::from_be_bytes([attr_bytes[2], attr_bytes[3]]);
                let code_length = u32::from_be_bytes([
                    attr_bytes[4],
                    attr_bytes[5],
                    attr_bytes[6],
                    attr_bytes[7],
                ]) as usize;
                attr_bytes = &attr_bytes[8..];

                if attr_bytes.len() < code_length {
                    return Err("Code attribute bytecode truncated".to_string());
                }
                let code = attr_bytes[..code_length].to_vec();
                attr_bytes = &attr_bytes[code_length..];

                if attr_bytes.len() < 2 {
                    return Err("Truncated exception table length".to_string());
                }
                let exception_table_length =
                    u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]) as usize;
                attr_bytes = &attr_bytes[2..];

                let mut exception_table = Vec::with_capacity(exception_table_length);
                for _ in 0..exception_table_length {
                    if attr_bytes.len() < 8 {
                        return Err("Truncated exception table entry".to_string());
                    }
                    let start_pc = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]);
                    let end_pc = u16::from_be_bytes([attr_bytes[2], attr_bytes[3]]);
                    let handler_pc = u16::from_be_bytes([attr_bytes[4], attr_bytes[5]]);
                    let catch_type = u16::from_be_bytes([attr_bytes[6], attr_bytes[7]]);
                    attr_bytes = &attr_bytes[8..];
                    exception_table.push(ExceptionTableEntry {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                        catch_class: if catch_type == 0 {
                            None
                        } else {
                            cp.get_class_name(catch_type).map(|n| n.to_string())
                        },
                    });
                }

                if attr_bytes.len() < 2 {
                    return Err("Truncated code sub-attributes length".to_string());
                }
                let sub_attr_count = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]) as usize;
                attr_bytes = &attr_bytes[2..];

                let mut sub_attrs = Vec::with_capacity(sub_attr_count);
                for _ in 0..sub_attr_count {
                    sub_attrs.push(AttributeInfo::parse(&mut attr_bytes, cp)?);
                }

                Ok(AttributeInfo::Code(CodeAttribute {
                    max_stack,
                    max_locals,
                    code,
                    exception_table,
                    attributes: sub_attrs,
                }))
            }
            "ConstantValue" => {
                if attr_bytes.len() < 2 {
                    return Err("Invalid ConstantValue length".to_string());
                }
                let cv_idx = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]);
                Ok(AttributeInfo::ConstantValue(cv_idx))
            }
            "Exceptions" => {
                if attr_bytes.len() < 2 {
                    return Err("Invalid Exceptions length".to_string());
                }
                let count = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]) as usize;
                attr_bytes = &attr_bytes[2..];
                let mut throws = Vec::with_capacity(count);
                for _ in 0..count {
                    if attr_bytes.len() < 2 {
                        break;
                    }
                    throws.push(u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]));
                    attr_bytes = &attr_bytes[2..];
                }
                Ok(AttributeInfo::Exceptions(throws))
            }
            "StackMap" | "StackMapTable" => Ok(AttributeInfo::StackMap(attr_bytes.to_vec())),
            "SourceFile" => {
                if attr_bytes.len() >= 2 {
                    Ok(AttributeInfo::SourceFile(u16::from_be_bytes([
                        attr_bytes[0],
                        attr_bytes[1],
                    ])))
                } else {
                    Ok(AttributeInfo::Other {
                        name: name.to_string(),
                        data: attr_bytes.to_vec(),
                    })
                }
            }
            "LineNumberTable" => {
                if attr_bytes.len() >= 2 {
                    let count = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]) as usize;
                    attr_bytes = &attr_bytes[2..];
                    let mut lines = Vec::with_capacity(count);
                    for _ in 0..count {
                        if attr_bytes.len() < 4 {
                            break;
                        }
                        let start_pc = u16::from_be_bytes([attr_bytes[0], attr_bytes[1]]);
                        let line_no = u16::from_be_bytes([attr_bytes[2], attr_bytes[3]]);
                        lines.push((start_pc, line_no));
                        attr_bytes = &attr_bytes[4..];
                    }
                    Ok(AttributeInfo::LineNumberTable(lines))
                } else {
                    Ok(AttributeInfo::Other {
                        name: name.to_string(),
                        data: attr_bytes.to_vec(),
                    })
                }
            }
            other => Ok(AttributeInfo::Other {
                name: other.to_string(),
                data: attr_bytes.to_vec(),
            }),
        }
    }
}
