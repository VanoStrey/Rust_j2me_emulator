use super::instruction::{Instruction, Predecoder};
use crate::classfile::attributes::ExceptionTableEntry;
use crate::classfile::class_file::ClassFile;
use crate::heap::object::{Heap, ObjectId};
use crate::heap::value::JavaValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PredecodedMethod {
    pub class_name: String,
    pub name: String,
    pub descriptor: String,
    pub access_flags: u16,
    pub max_locals: usize,
    pub max_stack: usize,
    pub instructions: Arc<Vec<Instruction>>,
    pub pc_map: Arc<HashMap<u16, usize>>,
    pub exception_table: Vec<ExceptionTableEntry>,
}

impl PredecodedMethod {
    pub fn is_static(&self) -> bool {
        self.access_flags & crate::classfile::class_file::ACC_STATIC != 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags & crate::classfile::class_file::ACC_NATIVE != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.access_flags & crate::classfile::class_file::ACC_SYNCHRONIZED != 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassLoader {
    pub classes: HashMap<String, Arc<ClassFile>>,
    pub predecoded_methods: HashMap<(String, String, String), Arc<PredecodedMethod>>,
    pub static_fields: HashMap<String, HashMap<String, JavaValue>>,
    pub initialized_classes: HashSet<String>,
    pub class_objects: HashMap<String, ObjectId>,
    pub method_cache:
        std::cell::RefCell<HashMap<(String, String, String), Option<Arc<PredecodedMethod>>>>,
    pub field_owner_cache: std::cell::RefCell<HashMap<(String, String, String), String>>,
    pub method_cache_hits: std::cell::Cell<u64>,
    pub method_cache_misses: std::cell::Cell<u64>,
    pub field_cache_hits: std::cell::Cell<u64>,
    pub field_cache_misses: std::cell::Cell<u64>,
    pub cache_invalidations: std::cell::Cell<u64>,
}

#[inline]
pub fn normalize_class_name(name: &str) -> std::borrow::Cow<'_, str> {
    if name.contains('.') {
        std::borrow::Cow::Owned(name.replace('.', "/"))
    } else {
        std::borrow::Cow::Borrowed(name)
    }
}

/// Superclass chain for core platform classes that are never loaded from the
/// JAR (the VM synthesises their instances). Derived from the JVM / CLDC / MIDP
/// specifications, not from any particular application.
const BUILTIN_SUPERS: &[(&str, &str)] = &[
    ("java/lang/Throwable", "java/lang/Object"),
    ("java/lang/Exception", "java/lang/Throwable"),
    ("java/lang/Error", "java/lang/Throwable"),
    ("java/lang/RuntimeException", "java/lang/Exception"),
    (
        "java/lang/NullPointerException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArithmeticException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IndexOutOfBoundsException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArrayIndexOutOfBoundsException",
        "java/lang/IndexOutOfBoundsException",
    ),
    (
        "java/lang/StringIndexOutOfBoundsException",
        "java/lang/IndexOutOfBoundsException",
    ),
    (
        "java/lang/ArrayStoreException",
        "java/lang/RuntimeException",
    ),
    ("java/lang/ClassCastException", "java/lang/RuntimeException"),
    (
        "java/lang/IllegalArgumentException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IllegalStateException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IllegalThreadStateException",
        "java/lang/IllegalArgumentException",
    ),
    (
        "java/lang/NumberFormatException",
        "java/lang/IllegalArgumentException",
    ),
    (
        "java/lang/NegativeArraySizeException",
        "java/lang/RuntimeException",
    ),
    ("java/lang/SecurityException", "java/lang/RuntimeException"),
    ("java/lang/InterruptedException", "java/lang/Exception"),
    ("java/lang/InstantiationException", "java/lang/Exception"),
    ("java/lang/IllegalAccessException", "java/lang/Exception"),
    ("java/lang/ClassNotFoundException", "java/lang/Exception"),
    ("java/lang/OutOfMemoryError", "java/lang/Error"),
    ("java/lang/StackOverflowError", "java/lang/Error"),
    ("java/lang/NoClassDefFoundError", "java/lang/Error"),
    ("java/lang/VirtualMachineError", "java/lang/Error"),
    ("java/io/IOException", "java/lang/Exception"),
    ("java/io/EOFException", "java/io/IOException"),
    ("java/io/InterruptedIOException", "java/io/IOException"),
    (
        "java/io/UnsupportedEncodingException",
        "java/io/IOException",
    ),
    ("java/io/UTFDataFormatException", "java/io/IOException"),
    (
        "java/lang/IllegalMonitorStateException",
        "java/lang/RuntimeException",
    ),
    (
        "javax/microedition/rms/RecordStoreException",
        "java/lang/Exception",
    ),
    (
        "javax/microedition/rms/RecordStoreNotFoundException",
        "javax/microedition/rms/RecordStoreException",
    ),
    (
        "javax/microedition/rms/RecordStoreFullException",
        "javax/microedition/rms/RecordStoreException",
    ),
    (
        "javax/microedition/rms/RecordStoreNotOpenException",
        "javax/microedition/rms/RecordStoreException",
    ),
    (
        "javax/microedition/rms/InvalidRecordIDException",
        "javax/microedition/rms/RecordStoreException",
    ),
    (
        "javax/microedition/io/ConnectionNotFoundException",
        "java/io/IOException",
    ),
    (
        "javax/microedition/media/MediaException",
        "java/lang/Exception",
    ),
    ("java/util/Stack", "java/util/Vector"),
    (
        "java/util/EmptyStackException",
        "java/lang/RuntimeException",
    ),
    (
        "java/util/NoSuchElementException",
        "java/lang/RuntimeException",
    ),
    ("java/io/Reader", "java/lang/Object"),
    ("java/io/InputStreamReader", "java/io/Reader"),
    // M3G (JSR-184) class hierarchy
    ("javax/microedition/m3g/Object3D", "java/lang/Object"),
    (
        "javax/microedition/m3g/Transformable",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/Node",
        "javax/microedition/m3g/Transformable",
    ),
    (
        "javax/microedition/m3g/Group",
        "javax/microedition/m3g/Node",
    ),
    (
        "javax/microedition/m3g/World",
        "javax/microedition/m3g/Group",
    ),
    (
        "javax/microedition/m3g/Camera",
        "javax/microedition/m3g/Node",
    ),
    (
        "javax/microedition/m3g/Light",
        "javax/microedition/m3g/Node",
    ),
    ("javax/microedition/m3g/Mesh", "javax/microedition/m3g/Node"),
    (
        "javax/microedition/m3g/SkinnedMesh",
        "javax/microedition/m3g/Mesh",
    ),
    (
        "javax/microedition/m3g/MorphingMesh",
        "javax/microedition/m3g/Mesh",
    ),
    (
        "javax/microedition/m3g/Sprite3D",
        "javax/microedition/m3g/Node",
    ),
    (
        "javax/microedition/m3g/Texture2D",
        "javax/microedition/m3g/Transformable",
    ),
    (
        "javax/microedition/m3g/Background",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/Appearance",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/Material",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/CompositingMode",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/PolygonMode",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/Fog",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/AnimationTrack",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/AnimationController",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/KeyframeSequence",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/VertexArray",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/VertexBuffer",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/IndexBuffer",
        "javax/microedition/m3g/Object3D",
    ),
    (
        "javax/microedition/m3g/TriangleStripArray",
        "javax/microedition/m3g/IndexBuffer",
    ),
    ("javax/microedition/m3g/RayIntersection", "java/lang/Object"),
    ("javax/microedition/m3g/Graphics3D", "java/lang/Object"),
    ("javax/microedition/m3g/Loader", "java/lang/Object"),
];

/// Superclass of a synthesised platform class, if it is known.
fn builtin_super(class_name: &str) -> Option<&'static str> {
    BUILTIN_SUPERS
        .iter()
        .find(|(name, _)| *name == class_name)
        .map(|(_, sup)| *sup)
}

impl ClassLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define_class(&mut self, class_file: ClassFile) -> Arc<ClassFile> {
        let name = class_file.this_class.clone();

        // Pre-decode all methods with bytecode
        for m in &class_file.methods {
            if let Some(code_attr) = m.code() {
                let (instructions, pc_map) =
                    Predecoder::decode_method_code(&code_attr.code, &class_file.constant_pool);
                let predec = PredecodedMethod {
                    class_name: name.clone(),
                    name: m.name.clone(),
                    descriptor: m.descriptor.clone(),
                    access_flags: m.access_flags,
                    max_locals: code_attr.max_locals as usize,
                    max_stack: code_attr.max_stack as usize,
                    instructions: Arc::new(instructions),
                    pc_map: Arc::new(pc_map),
                    exception_table: code_attr.exception_table.clone(),
                };
                self.predecoded_methods.insert(
                    (name.clone(), m.name.clone(), m.descriptor.clone()),
                    Arc::new(predec),
                );
            }
        }

        // Initialize static fields to defaults or ConstantValue
        let mut statics = HashMap::new();
        for f in &class_file.fields {
            if f.access_flags & crate::classfile::class_file::ACC_STATIC != 0 {
                let mut initial_val = JavaValue::default_for_descriptor(&f.descriptor);
                for attr in &f.attributes {
                    if let crate::classfile::attributes::AttributeInfo::ConstantValue(idx) = attr {
                        match class_file.constant_pool.entries.get(*idx as usize) {
                            Some(crate::classfile::constant_pool::ConstantPoolEntry::Integer(
                                v,
                            )) => initial_val = JavaValue::Int(*v),
                            Some(crate::classfile::constant_pool::ConstantPoolEntry::Long(v)) => {
                                initial_val = JavaValue::Long(*v)
                            }
                            Some(crate::classfile::constant_pool::ConstantPoolEntry::Float(v)) => {
                                initial_val = JavaValue::Float(*v)
                            }
                            Some(crate::classfile::constant_pool::ConstantPoolEntry::Double(v)) => {
                                initial_val = JavaValue::Double(*v)
                            }
                            _ => {}
                        }
                    }
                }
                let typed_key = format!("{}:{}", f.name, f.descriptor);
                statics.insert(typed_key, initial_val);
                statics.insert(f.name.clone(), initial_val);
            }
        }
        self.static_fields.insert(name.clone(), statics);

        let arc = Arc::new(class_file);
        self.classes.insert(name, Arc::clone(&arc));
        self.method_cache.borrow_mut().clear();
        self.field_owner_cache.borrow_mut().clear();
        self.cache_invalidations
            .set(self.cache_invalidations.get() + 1);
        arc
    }

    pub fn is_initialized(&self, class_name: &str) -> bool {
        let normalized = normalize_class_name(class_name);
        self.initialized_classes.contains(normalized.as_ref())
    }

    pub fn mark_initialized(&mut self, class_name: &str) {
        let normalized = normalize_class_name(class_name);
        self.initialized_classes.insert(normalized.into_owned());
    }

    pub fn get_class(&self, name: &str) -> Option<Arc<ClassFile>> {
        let normalized = normalize_class_name(name);
        self.classes.get(normalized.as_ref()).cloned()
    }

    pub fn get_or_create_class_object(&mut self, heap: &mut Heap, class_name: &str) -> ObjectId {
        let norm = normalize_class_name(class_name).into_owned();
        if let Some(&id) = self.class_objects.get(&norm) {
            return id;
        }
        let cls_obj = heap.alloc_instance("java/lang/Class");
        let (binary_name, internal_name) = if norm.starts_with('[') {
            (norm.replace('/', "."), norm.clone())
        } else {
            (norm.replace('/', "."), norm.clone())
        };
        let name_id = heap.alloc_string(&binary_name);
        let target_id = heap.alloc_string(&internal_name);
        heap.set_field(cls_obj, "name", JavaValue::Object(Some(name_id)));
        heap.set_field(cls_obj, "target_class", JavaValue::Object(Some(target_id)));
        self.class_objects.insert(norm, cls_obj);
        cls_obj
    }

    pub fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
    ) -> Option<Arc<PredecodedMethod>> {
        let normalized = normalize_class_name(class_name);
        let key = (
            normalized.to_string(),
            method_name.to_string(),
            descriptor.to_string(),
        );

        if let Some(cached) = self.method_cache.borrow().get(&key) {
            self.method_cache_hits.set(self.method_cache_hits.get() + 1);
            return cached.clone();
        }
        self.method_cache_misses
            .set(self.method_cache_misses.get() + 1);

        // 1. Direct match on class
        if let Some(m) = self.predecoded_methods.get(&key) {
            let res = Some(Arc::clone(m));
            self.method_cache.borrow_mut().insert(key, res.clone());
            return res;
        }

        // 2. Hierarchy traversal (super classes)
        let mut curr_class = self.get_class(&normalized);
        while let Some(cls) = curr_class {
            if let Some(super_name) = &cls.super_class {
                let super_key = (
                    super_name.clone(),
                    method_name.to_string(),
                    descriptor.to_string(),
                );
                if let Some(m) = self.predecoded_methods.get(&super_key) {
                    let res = Some(Arc::clone(m));
                    self.method_cache.borrow_mut().insert(key, res.clone());
                    return res;
                }
                curr_class = self.get_class(super_name);
            } else {
                break;
            }
        }

        self.method_cache.borrow_mut().insert(key, None);
        None
    }

    pub fn resolve_field_owner(
        &self,
        class_name: &str,
        field_name: &str,
        descriptor: &str,
    ) -> String {
        let normalized = normalize_class_name(class_name);
        let key = (
            normalized.to_string(),
            field_name.to_string(),
            descriptor.to_string(),
        );

        if let Some(cached) = self.field_owner_cache.borrow().get(&key) {
            self.field_cache_hits.set(self.field_cache_hits.get() + 1);
            return cached.clone();
        }
        self.field_cache_misses
            .set(self.field_cache_misses.get() + 1);

        let typed_key = format!("{}:{}", field_name, descriptor);
        let mut curr_name = Some(normalized.to_string());
        while let Some(cls_name) = curr_name {
            if let Some(cls) = self.get_class(&cls_name) {
                if cls.find_field(field_name, descriptor).is_some() {
                    self.field_owner_cache
                        .borrow_mut()
                        .insert(key, cls_name.clone());
                    return cls_name;
                }
                curr_name = cls.super_class.clone();
            } else {
                if let Some(fields) = self.static_fields.get(&cls_name) {
                    if fields.contains_key(&typed_key)
                        || (descriptor.is_empty() && fields.contains_key(field_name))
                    {
                        self.field_owner_cache
                            .borrow_mut()
                            .insert(key, cls_name.clone());
                        return cls_name;
                    }
                }
                curr_name = builtin_super(&cls_name).map(|s| s.to_string());
            }
        }

        let fallback = normalized.to_string();
        self.field_owner_cache
            .borrow_mut()
            .insert(key, fallback.clone());
        fallback
    }

    pub fn get_static(&self, class_name: &str, field_name: &str, descriptor: &str) -> JavaValue {
        let owner = self.resolve_field_owner(class_name, field_name, descriptor);
        let typed_key = format!("{}:{}", field_name, descriptor);
        if let Some(fields) = self.static_fields.get(&owner) {
            if let Some(val) = fields.get(&typed_key) {
                return *val;
            }
            if descriptor.is_empty() {
                if let Some(val) = fields.get(field_name) {
                    return *val;
                }
            }
        }
        JavaValue::default_for_descriptor(descriptor)
    }

    pub fn set_static(
        &mut self,
        class_name: &str,
        field_name: &str,
        descriptor: &str,
        val: JavaValue,
    ) {
        let owner = self.resolve_field_owner(class_name, field_name, descriptor);
        let typed_key = format!("{}:{}", field_name, descriptor);
        self.static_fields
            .entry(owner)
            .or_default()
            .insert(typed_key, val);
    }

    pub fn is_instance_of(&self, actual_class: &str, target_class: &str) -> bool {
        let act = actual_class.replace('.', "/");
        let tgt = target_class.replace('.', "/");
        if act == tgt || tgt == "java/lang/Object" {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(act);

        while let Some(cls_name) = queue.pop_front() {
            if !visited.insert(cls_name.clone()) {
                continue;
            }
            if cls_name == tgt {
                return true;
            }
            if let Some(cls) = self.get_class(&cls_name) {
                if let Some(ref sup) = cls.super_class {
                    queue.push_back(sup.clone());
                }
                for iface in &cls.interfaces {
                    queue.push_back(iface.clone());
                }
            } else if let Some(sup) = builtin_super(&cls_name) {
                queue.push_back(sup.to_string());
            } else {
                let builtin_sup = match cls_name.as_str() {
                    "com/nokia/mid/ui/FullCanvas" => Some("javax/microedition/lcdui/Canvas"),
                    "javax/microedition/lcdui/game/GameCanvas" => {
                        Some("javax/microedition/lcdui/Canvas")
                    }
                    "javax/microedition/lcdui/Canvas" => {
                        Some("javax/microedition/lcdui/Displayable")
                    }
                    "javax/microedition/lcdui/Screen" => {
                        Some("javax/microedition/lcdui/Displayable")
                    }
                    "javax/microedition/lcdui/Form" => Some("javax/microedition/lcdui/Screen"),
                    "javax/microedition/lcdui/List" => Some("javax/microedition/lcdui/Screen"),
                    "javax/microedition/lcdui/TextBox" => Some("javax/microedition/lcdui/Screen"),
                    "javax/microedition/lcdui/Alert" => Some("javax/microedition/lcdui/Screen"),
                    "javax/microedition/lcdui/Displayable" => Some("java/lang/Object"),
                    "javax/microedition/midlet/MIDlet" => Some("java/lang/Object"),
                    "java/io/ByteArrayInputStream" | "java/io/DataInputStream" => {
                        Some("java/io/InputStream")
                    }
                    "java/io/ByteArrayOutputStream" | "java/io/DataOutputStream" => {
                        Some("java/io/OutputStream")
                    }
                    "java/io/InputStream" | "java/io/OutputStream" => Some("java/lang/Object"),
                    _ => None,
                };
                if let Some(sup) = builtin_sup {
                    queue.push_back(sup.to_string());
                }
            }
        }
        false
    }
}

/// Parse parameter descriptors: returns count of parameters
pub fn parse_param_count(descriptor: &str) -> usize {
    let mut count = 0;
    if let Some(start) = descriptor.find('(') {
        if let Some(end) = descriptor.find(')') {
            let params = &descriptor[start + 1..end];
            let mut chars = params.chars();
            while let Some(c) = chars.next() {
                match c {
                    'Z' | 'B' | 'C' | 'S' | 'I' | 'J' | 'F' | 'D' => count += 1,
                    'L' => {
                        count += 1;
                        for ch in chars.by_ref() {
                            if ch == ';' {
                                break;
                            }
                        }
                    }
                    '[' => {
                        // Skip array dimensions
                        while let Some(&next_c) = chars.clone().peekable().peek() {
                            if next_c == '[' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(next_c) = chars.next() {
                            if next_c == 'L' {
                                for ch in chars.by_ref() {
                                    if ch == ';' {
                                        break;
                                    }
                                }
                            }
                        }
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod exception_hierarchy_tests {
    use super::*;

    #[test]
    fn catch_by_superclass_matches_subclass() {
        let loader = ClassLoader::new();
        // Every thrown runtime exception must be catchable as Exception, which
        // is what almost every J2ME `catch (Exception e)` block relies on.
        for leaf in [
            "java/lang/NullPointerException",
            "java/lang/ArithmeticException",
            "java/lang/ArrayIndexOutOfBoundsException",
            "java/lang/NumberFormatException",
            "java/lang/ClassCastException",
        ] {
            assert!(
                loader.is_instance_of(leaf, "java/lang/Exception"),
                "{leaf} must be catchable as java/lang/Exception"
            );
            assert!(
                loader.is_instance_of(leaf, "java/lang/Throwable"),
                "{leaf} must be catchable as java/lang/Throwable"
            );
        }
    }

    #[test]
    fn unrelated_exception_is_not_caught() {
        let loader = ClassLoader::new();
        assert!(!loader.is_instance_of("java/lang/NullPointerException", "java/io/IOException"));
        assert!(!loader.is_instance_of("java/io/IOException", "java/lang/RuntimeException"));
    }

    #[test]
    fn rms_exceptions_are_catchable_as_exception() {
        let loader = ClassLoader::new();
        assert!(loader.is_instance_of(
            "javax/microedition/rms/RecordStoreNotFoundException",
            "java/lang/Exception"
        ));
        assert!(loader.is_instance_of(
            "javax/microedition/rms/RecordStoreNotFoundException",
            "javax/microedition/rms/RecordStoreException"
        ));
    }
}

#[cfg(test)]
mod static_fields_resolution_tests {
    use super::*;
    use crate::classfile::class_file::{ClassFile, FieldInfo};

    fn make_test_class(name: &str, super_name: Option<&str>, fields: &[(&str, &str)]) -> ClassFile {
        let mut cf = ClassFile {
            magic: 0xCAFEBABE,
            minor_version: 0,
            major_version: 45,
            constant_pool: crate::classfile::constant_pool::ConstantPool {
                entries: Vec::new(),
            },
            access_flags: 0x0001,
            this_class: name.to_string(),
            super_class: super_name.map(|s| s.to_string()),
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
        };
        for &(fname, fdesc) in fields {
            cf.fields.push(FieldInfo {
                access_flags: 0x0008, // ACC_STATIC
                name: fname.to_string(),
                descriptor: fdesc.to_string(),
                attributes: Vec::new(),
            });
        }
        cf
    }

    #[test]
    fn static_field_inheritance() {
        let mut loader = ClassLoader::new();
        // Base class has static field "SHARED_FLAG:I"
        let base = make_test_class(
            "com/test/Base",
            Some("java/lang/Object"),
            &[("SHARED_FLAG", "I")],
        );
        // Derived class inherits without redeclaring
        let derived = make_test_class("com/test/Derived", Some("com/test/Base"), &[]);
        loader.define_class(base);
        loader.define_class(derived);

        // Put static via Base
        loader.set_static("com/test/Base", "SHARED_FLAG", "I", JavaValue::Int(42));
        // Get static via Derived resolves to Base
        assert_eq!(
            loader.get_static("com/test/Derived", "SHARED_FLAG", "I"),
            JavaValue::Int(42)
        );
        // Put static via Derived resolves to Base
        loader.set_static("com/test/Derived", "SHARED_FLAG", "I", JavaValue::Int(100));
        assert_eq!(
            loader.get_static("com/test/Base", "SHARED_FLAG", "I"),
            JavaValue::Int(100)
        );
    }

    #[test]
    fn static_field_hiding_same_signature() {
        let mut loader = ClassLoader::new();
        // Base has "ID:I"
        let base = make_test_class("com/test/Base", Some("java/lang/Object"), &[("ID", "I")]);
        // Child also defines "ID:I" (hiding)
        let child = make_test_class("com/test/Child", Some("com/test/Base"), &[("ID", "I")]);
        loader.define_class(base);
        loader.define_class(child);

        loader.set_static("com/test/Base", "ID", "I", JavaValue::Int(10));
        loader.set_static("com/test/Child", "ID", "I", JavaValue::Int(20));

        // Child.ID must not overwrite Base.ID!
        assert_eq!(
            loader.get_static("com/test/Base", "ID", "I"),
            JavaValue::Int(10)
        );
        assert_eq!(
            loader.get_static("com/test/Child", "ID", "I"),
            JavaValue::Int(20)
        );
    }

    #[test]
    fn static_fields_same_name_different_descriptors() {
        let mut loader = ClassLoader::new();
        // A single class declaring "val:I" and "val:Ljava/lang/String;"
        let cls = make_test_class(
            "com/test/Multi",
            Some("java/lang/Object"),
            &[("val", "I"), ("val", "Ljava/lang/String;")],
        );
        loader.define_class(cls);

        loader.set_static("com/test/Multi", "val", "I", JavaValue::Int(777));
        loader.set_static(
            "com/test/Multi",
            "val",
            "Ljava/lang/String;",
            JavaValue::Object(Some(crate::heap::object::ObjectId(5))),
        );

        assert_eq!(
            loader.get_static("com/test/Multi", "val", "I"),
            JavaValue::Int(777)
        );
        assert_eq!(
            loader.get_static("com/test/Multi", "val", "Ljava/lang/String;"),
            JavaValue::Object(Some(crate::heap::object::ObjectId(5)))
        );
    }

    #[test]
    fn cache_hit_and_miss_metrics() {
        let mut loader = ClassLoader::new();
        let cls = make_test_class(
            "com/test/CacheSubject",
            Some("java/lang/Object"),
            &[("fieldA", "I")],
        );
        loader.define_class(cls);

        assert_eq!(loader.field_cache_hits.get(), 0);
        assert_eq!(loader.field_cache_misses.get(), 0);

        // First resolution: cache miss
        let owner1 = loader.resolve_field_owner("com/test/CacheSubject", "fieldA", "I");
        assert_eq!(owner1, "com/test/CacheSubject");
        assert_eq!(loader.field_cache_misses.get(), 1);
        assert_eq!(loader.field_cache_hits.get(), 0);

        // Second resolution: cache hit
        let owner2 = loader.resolve_field_owner("com/test/CacheSubject", "fieldA", "I");
        assert_eq!(owner2, "com/test/CacheSubject");
        assert_eq!(loader.field_cache_hits.get(), 1);
        assert_eq!(loader.field_cache_misses.get(), 1);

        // Define class invalidates cache
        let inv_before = loader.cache_invalidations.get();
        let another = make_test_class("com/test/Another", Some("java/lang/Object"), &[]);
        loader.define_class(another);
        assert_eq!(loader.cache_invalidations.get(), inv_before + 1);

        // Next resolution is a miss again
        loader.resolve_field_owner("com/test/CacheSubject", "fieldA", "I");
        assert_eq!(loader.field_cache_misses.get(), 2);
    }

    #[test]
    fn clinit_tracking_exactly_once() {
        let mut loader = ClassLoader::new();
        assert!(!loader.is_initialized("com.test.ClinitCls"));
        loader.mark_initialized("com.test.ClinitCls");
        assert!(loader.is_initialized("com/test/ClinitCls"));
        assert!(loader.is_initialized("com.test.ClinitCls"));
    }
}
