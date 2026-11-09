use crate::heap::object::{Heap, ObjectId};
use crate::heap::value::JavaValue;
use crate::threads::green_thread::GreenThread;
use crate::vm::class_loader::ClassLoader;

pub struct GarbageCollector;

impl GarbageCollector {
    pub fn collect(
        heap: &mut Heap,
        threads: &[GreenThread],
        class_loader: &ClassLoader,
        extra_roots: &[ObjectId],
    ) -> usize {
        let mut roots = Vec::new();

        // 1. Thread call stacks (frames, operand stacks, locals)
        for t in threads {
            if !t.is_alive() {
                continue;
            }
            if let Some(id) = t.java_thread_obj {
                roots.push(id);
            }
            for frame in &t.call_stack {
                for val in &frame.stack {
                    if let JavaValue::Object(Some(id)) = val {
                        roots.push(*id);
                    }
                }
                for val in &frame.locals {
                    if let JavaValue::Object(Some(id)) = val {
                        roots.push(*id);
                    }
                }
            }
        }

        // 2. Static class fields
        for class_fields in class_loader.static_fields.values() {
            for val in class_fields.values() {
                if let JavaValue::Object(Some(id)) = val {
                    roots.push(*id);
                }
            }
        }

        // 2b. Class loader class objects (java/lang/Class instances)
        for id in class_loader.class_objects.values() {
            roots.push(*id);
        }

        // 3. Extra native roots
        roots.extend_from_slice(extra_roots);

        // Run mark-and-sweep
        heap.collect_garbage(&roots)
    }
}
