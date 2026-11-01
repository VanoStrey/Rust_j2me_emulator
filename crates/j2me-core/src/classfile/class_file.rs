use super::attributes::{AttributeInfo, CodeAttribute};
use super::constant_pool::ConstantPool;

pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SYNCHRONIZED: u16 = 0x0020;
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ABSTRACT: u16 = 0x0400;
pub const ACC_STRICT: u16 = 0x0800;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name: String,
    pub descriptor: String,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name: String,
    pub descriptor: String,
    pub attributes: Vec<AttributeInfo>,
}

impl MethodInfo {
    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC != 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags & ACC_NATIVE != 0
    }

    pub fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.access_flags & ACC_SYNCHRONIZED != 0
    }

    pub fn code(&self) -> Option<&CodeAttribute> {
        for attr in &self.attributes {
            if let AttributeInfo::Code(code) = attr {
                return Some(code);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl ClassFile {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = bytes;
        if reader.len() < 8 {
            return Err("Class file too short for header".to_string());
        }
        let magic = u32::from_be_bytes([reader[0], reader[1], reader[2], reader[3]]);
        if magic != 0xCAFEBABE {
            return Err(format!("Invalid magic number: 0x{:08X}", magic));
        }
        let minor_version = u16::from_be_bytes([reader[4], reader[5]]);
        let major_version = u16::from_be_bytes([reader[6], reader[7]]);
        reader = &reader[8..];

        let constant_pool = ConstantPool::parse(&mut reader)?;

        if reader.len() < 6 {
            return Err("Unexpected EOF reading class flags and names".to_string());
        }
        let access_flags = u16::from_be_bytes([reader[0], reader[1]]);
        let this_class_idx = u16::from_be_bytes([reader[2], reader[3]]);
        let super_class_idx = u16::from_be_bytes([reader[4], reader[5]]);
        reader = &reader[6..];

        let this_class = constant_pool
            .get_class_name(this_class_idx)
            .ok_or_else(|| "Failed to resolve this_class".to_string())?
            .to_string();

        let super_class = if super_class_idx == 0 {
            None
        } else {
            Some(
                constant_pool
                    .get_class_name(super_class_idx)
                    .ok_or_else(|| "Failed to resolve super_class".to_string())?
                    .to_string(),
            )
        };

        if reader.len() < 2 {
            return Err("Unexpected EOF reading interfaces count".to_string());
        }
        let interfaces_count = u16::from_be_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];

        let mut interfaces = Vec::with_capacity(interfaces_count);
        for _ in 0..interfaces_count {
            if reader.len() < 2 {
                return Err("Unexpected EOF reading interface index".to_string());
            }
            let iface_idx = u16::from_be_bytes([reader[0], reader[1]]);
            reader = &reader[2..];
            let name = constant_pool
                .get_class_name(iface_idx)
                .ok_or_else(|| "Failed to resolve interface name".to_string())?;
            interfaces.push(name.to_string());
        }

        if reader.len() < 2 {
            return Err("Unexpected EOF reading fields count".to_string());
        }
        let fields_count = u16::from_be_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];

        let mut fields = Vec::with_capacity(fields_count);
        for _ in 0..fields_count {
            if reader.len() < 8 {
                return Err("Unexpected EOF reading field header".to_string());
            }
            let access_flags = u16::from_be_bytes([reader[0], reader[1]]);
            let name_idx = u16::from_be_bytes([reader[2], reader[3]]);
            let desc_idx = u16::from_be_bytes([reader[4], reader[5]]);
            let attr_count = u16::from_be_bytes([reader[6], reader[7]]) as usize;
            reader = &reader[8..];

            let name = constant_pool
                .get_utf8(name_idx)
                .ok_or_else(|| "Invalid field name index".to_string())?
                .to_string();
            let descriptor = constant_pool
                .get_utf8(desc_idx)
                .ok_or_else(|| "Invalid field descriptor index".to_string())?
                .to_string();

            let mut attributes = Vec::with_capacity(attr_count);
            for _ in 0..attr_count {
                attributes.push(AttributeInfo::parse(&mut reader, &constant_pool)?);
            }

            fields.push(FieldInfo {
                access_flags,
                name,
                descriptor,
                attributes,
            });
        }

        if reader.len() < 2 {
            return Err("Unexpected EOF reading methods count".to_string());
        }
        let methods_count = u16::from_be_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];

        let mut methods = Vec::with_capacity(methods_count);
        for _ in 0..methods_count {
            if reader.len() < 8 {
                return Err("Unexpected EOF reading method header".to_string());
            }
            let access_flags = u16::from_be_bytes([reader[0], reader[1]]);
            let name_idx = u16::from_be_bytes([reader[2], reader[3]]);
            let desc_idx = u16::from_be_bytes([reader[4], reader[5]]);
            let attr_count = u16::from_be_bytes([reader[6], reader[7]]) as usize;
            reader = &reader[8..];

            let name = constant_pool
                .get_utf8(name_idx)
                .ok_or_else(|| "Invalid method name index".to_string())?
                .to_string();
            let descriptor = constant_pool
                .get_utf8(desc_idx)
                .ok_or_else(|| "Invalid method descriptor index".to_string())?
                .to_string();

            let mut attributes = Vec::with_capacity(attr_count);
            for _ in 0..attr_count {
                attributes.push(AttributeInfo::parse(&mut reader, &constant_pool)?);
            }

            methods.push(MethodInfo {
                access_flags,
                name,
                descriptor,
                attributes,
            });
        }

        if reader.len() < 2 {
            return Err("Unexpected EOF reading class attributes count".to_string());
        }
        let attr_count = u16::from_be_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];

        let mut attributes = Vec::with_capacity(attr_count);
        for _ in 0..attr_count {
            attributes.push(AttributeInfo::parse(&mut reader, &constant_pool)?);
        }

        Ok(Self {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    pub fn find_method(&self, name: &str, descriptor: &str) -> Option<&MethodInfo> {
        self.methods
            .iter()
            .find(|m| m.name == name && m.descriptor == descriptor)
    }

    pub fn find_field(&self, name: &str, descriptor: &str) -> Option<&FieldInfo> {
        self.fields
            .iter()
            .find(|f| f.name == name && (descriptor.is_empty() || f.descriptor == descriptor))
    }
}
