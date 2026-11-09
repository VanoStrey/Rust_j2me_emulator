use super::value::JavaValue;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub u32);

#[derive(Debug, Clone)]
pub enum ArrayData {
    Boolean(Vec<bool>),
    Byte(Vec<i8>),
    Char(Vec<u16>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Reference(Vec<Option<ObjectId>>),
}

impl ArrayData {
    pub fn len(&self) -> usize {
        match self {
            ArrayData::Boolean(v) => v.len(),
            ArrayData::Byte(v) => v.len(),
            ArrayData::Char(v) => v.len(),
            ArrayData::Short(v) => v.len(),
            ArrayData::Int(v) => v.len(),
            ArrayData::Long(v) => v.len(),
            ArrayData::Float(v) => v.len(),
            ArrayData::Double(v) => v.len(),
            ArrayData::Reference(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct InstanceData {
    pub class_name: String,
    pub fields: HashMap<String, JavaValue>,
}

#[derive(Debug, Clone)]
pub enum ObjectData {
    Instance(InstanceData),
    Array(ArrayData),
    String(String),
}

impl ObjectData {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ObjectData::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_instance(&self) -> Option<&InstanceData> {
        match self {
            ObjectData::Instance(inst) => Some(inst),
            _ => None,
        }
    }

    pub fn as_instance_mut(&mut self) -> Option<&mut InstanceData> {
        match self {
            ObjectData::Instance(inst) => Some(inst),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&ArrayData> {
        match self {
            ObjectData::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut ArrayData> {
        match self {
            ObjectData::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Heap {
    slots: Vec<Option<ObjectData>>,
    free_list: Vec<u32>,
    pub total_allocations: u64,
}

impl Heap {
    pub fn new() -> Self {
        // Reserve index 0 as sentinel
        Self {
            slots: vec![None],
            free_list: Vec::new(),
            total_allocations: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn live_objects(&self) -> usize {
        self.slots.len().saturating_sub(self.free_list.len())
    }

    pub fn alloc(&mut self, data: ObjectData) -> ObjectId {
        self.total_allocations += 1;
        if let Some(idx) = self.free_list.pop() {
            self.slots[idx as usize] = Some(data);
            ObjectId(idx)
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Some(data));
            ObjectId(idx)
        }
    }

    pub fn alloc_instance(&mut self, class_name: &str) -> ObjectId {
        self.alloc(ObjectData::Instance(InstanceData {
            class_name: class_name.to_string(),
            fields: HashMap::new(),
        }))
    }

    pub fn alloc_string(&mut self, s: &str) -> ObjectId {
        self.alloc(ObjectData::String(s.to_string()))
    }

    pub fn alloc_array(&mut self, data: ArrayData) -> ObjectId {
        self.alloc(ObjectData::Array(data))
    }

    pub fn get(&self, id: ObjectId) -> Option<&ObjectData> {
        self.slots.get(id.0 as usize).and_then(|slot| slot.as_ref())
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut ObjectData> {
        self.slots
            .get_mut(id.0 as usize)
            .and_then(|slot| slot.as_mut())
    }

    pub fn free(&mut self, id: ObjectId) {
        if let Some(slot) = self.slots.get_mut(id.0 as usize) {
            if slot.is_some() {
                *slot = None;
                self.free_list.push(id.0);
            }
        }
    }

    pub fn get_field(&self, id: ObjectId, field_name: &str) -> JavaValue {
        if let Some(ObjectData::Instance(inst)) = self.get(id) {
            if let Some(val) = inst.fields.get(field_name) {
                return *val;
            }
            let prefix = format!("{}:", field_name);
            for (k, v) in &inst.fields {
                if k.starts_with(&prefix) {
                    return *v;
                }
            }
            let class_field_prefix = format!("/{}:", field_name);
            for (k, v) in &inst.fields {
                if k.contains(&class_field_prefix) {
                    return *v;
                }
            }
            let class_field_end = format!("/{}", field_name);
            for (k, v) in &inst.fields {
                if k.ends_with(&class_field_end) {
                    return *v;
                }
            }
            JavaValue::NULL
        } else {
            JavaValue::NULL
        }
    }

    pub fn set_field(&mut self, id: ObjectId, field_name: &str, val: JavaValue) {
        if let Some(ObjectData::Instance(inst)) = self.get_mut(id) {
            inst.fields.insert(field_name.to_string(), val);
        }
    }

    pub fn get_field_typed(
        &self,
        id: ObjectId,
        owner_class: &str,
        field_name: &str,
        descriptor: &str,
    ) -> JavaValue {
        thread_local! {
            static FIELD_BUF: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(128));
        }
        if let Some(ObjectData::Instance(inst)) = self.get(id) {
            FIELD_BUF.with(|buf| {
                let mut b = buf.borrow_mut();
                b.clear();
                b.push_str(owner_class);
                b.push('/');
                b.push_str(field_name);
                b.push(':');
                b.push_str(descriptor);
                if let Some(val) = inst.fields.get(&**b) {
                    return *val;
                }
                b.clear();
                b.push_str(field_name);
                b.push(':');
                b.push_str(descriptor);
                if let Some(val) = inst.fields.get(&**b) {
                    return *val;
                }
                inst.fields
                    .get(field_name)
                    .copied()
                    .unwrap_or_else(|| JavaValue::default_for_descriptor(descriptor))
            })
        } else {
            JavaValue::default_for_descriptor(descriptor)
        }
    }

    pub fn set_field_typed(
        &mut self,
        id: ObjectId,
        owner_class: &str,
        field_name: &str,
        descriptor: &str,
        val: JavaValue,
    ) {
        if let Some(ObjectData::Instance(inst)) = self.get_mut(id) {
            let class_key = format!("{}/{}:{}", owner_class, field_name, descriptor);
            inst.fields.insert(class_key, val);
        }
    }

    pub fn get_string_utf8(&self, id: ObjectId) -> Option<String> {
        match self.get(id)? {
            ObjectData::String(s) => Some(s.clone()),
            ObjectData::Instance(inst) if inst.class_name == "java/lang/String" => {
                // If it's stored as chars field
                if let Some(JavaValue::Object(Some(char_arr_id))) = inst.fields.get("value") {
                    if let Some(ObjectData::Array(ArrayData::Char(chars))) = self.get(*char_arr_id)
                    {
                        return Some(String::from_utf16_lossy(chars));
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_string_length(&self, id: ObjectId) -> Option<usize> {
        match self.get(id)? {
            ObjectData::String(s) => {
                if s.is_ascii() {
                    Some(s.len())
                } else {
                    Some(s.encode_utf16().count())
                }
            }
            ObjectData::Instance(inst) if inst.class_name == "java/lang/String" => {
                if let Some(JavaValue::Object(Some(char_arr_id))) = inst.fields.get("value") {
                    if let Some(ObjectData::Array(ArrayData::Char(chars))) = self.get(*char_arr_id) {
                        return Some(chars.len());
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_string_char_at(&self, id: ObjectId, idx: usize) -> Option<u16> {
        match self.get(id)? {
            ObjectData::String(s) => {
                if s.is_ascii() {
                    s.as_bytes().get(idx).map(|&b| b as u16)
                } else {
                    s.encode_utf16().nth(idx)
                }
            }
            ObjectData::Instance(inst) if inst.class_name == "java/lang/String" => {
                if let Some(JavaValue::Object(Some(char_arr_id))) = inst.fields.get("value") {
                    if let Some(ObjectData::Array(ArrayData::Char(chars))) = self.get(*char_arr_id) {
                        return chars.get(idx).copied();
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn set_string(&mut self, id: ObjectId, s: &str) {
        if let Some(slot) = self.slots.get_mut(id.0 as usize) {
            *slot = Some(ObjectData::String(s.to_string()));
        }
    }

    /// Mark-and-sweep garbage collection from root ObjectIds
    pub fn collect_garbage(&mut self, roots: &[ObjectId]) -> usize {
        let mut marked = HashSet::new();
        let mut queue: Vec<ObjectId> = roots.to_vec();

        while let Some(id) = queue.pop() {
            if marked.insert(id) {
                if let Some(obj) = self.get(id) {
                    match obj {
                        ObjectData::Instance(inst) => {
                            for val in inst.fields.values() {
                                if let JavaValue::Object(Some(child_id)) = val {
                                    if !marked.contains(child_id) {
                                        queue.push(*child_id);
                                    }
                                }
                            }
                        }
                        ObjectData::Array(ArrayData::Reference(refs)) => {
                            for child in refs.iter().flatten() {
                                if !marked.contains(child) {
                                    queue.push(*child);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut collected = 0;
        for i in 1..self.slots.len() {
            let id = ObjectId(i as u32);
            if self.slots[i].is_some() && !marked.contains(&id) {
                self.slots[i] = None;
                self.free_list.push(id.0);
                collected += 1;
            }
        }

        collected
    }
}
