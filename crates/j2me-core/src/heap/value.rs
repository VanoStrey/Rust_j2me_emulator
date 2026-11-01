use super::object::ObjectId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JavaValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(Option<ObjectId>),
}

impl JavaValue {
    pub const NULL: Self = JavaValue::Object(None);

    pub fn default_for_descriptor(desc: &str) -> Self {
        let return_desc = if let Some(idx) = desc.rfind(')') {
            &desc[idx + 1..]
        } else {
            desc
        };
        match return_desc.chars().next() {
            Some('Z') | Some('B') | Some('C') | Some('S') | Some('I') => JavaValue::Int(0),
            Some('J') => JavaValue::Long(0),
            Some('F') => JavaValue::Float(0.0),
            Some('D') => JavaValue::Double(0.0),
            Some('L') | Some('[') => JavaValue::Object(None),
            _ => JavaValue::Int(0),
        }
    }

    #[inline]
    pub fn as_int(&self) -> Result<i32, String> {
        match self {
            JavaValue::Int(v) => Ok(*v),
            JavaValue::Object(None) => Ok(0),
            other => Err(format!("Expected Int, got {:?}", other)),
        }
    }

    #[inline]
    pub fn as_long(&self) -> Result<i64, String> {
        match self {
            JavaValue::Long(v) => Ok(*v),
            other => Err(format!("Expected Long, got {:?}", other)),
        }
    }

    #[inline]
    pub fn as_float(&self) -> Result<f32, String> {
        match self {
            JavaValue::Float(v) => Ok(*v),
            other => Err(format!("Expected Float, got {:?}", other)),
        }
    }

    #[inline]
    pub fn as_double(&self) -> Result<f64, String> {
        match self {
            JavaValue::Double(v) => Ok(*v),
            other => Err(format!("Expected Double, got {:?}", other)),
        }
    }

    #[inline]
    pub fn as_object(&self) -> Result<Option<ObjectId>, String> {
        match self {
            JavaValue::Object(v) => Ok(*v),
            JavaValue::Int(0) => Ok(None),
            other => Err(format!("Expected Object reference, got {:?}", other)),
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, JavaValue::Object(None))
    }
}
