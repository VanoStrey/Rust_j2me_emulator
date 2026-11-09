use super::class_loader::{parse_param_count, ClassLoader};
use super::frame::Frame;
use super::instruction::Instruction;
use crate::heap::object::{ArrayData, Heap, ObjectId};
use crate::heap::value::JavaValue;
use crate::threads::green_thread::{ExceptionEvent, GreenThread, ThreadState};
use std::sync::Arc;

#[derive(Debug)]
pub enum StepResult {
    Continue,
    QuantumYield,
    Sleep(u64),
    Yield,
    Wait(Option<ObjectId>),
    MonitorEnter(ObjectId),
    MonitorExit(ObjectId),
    ThreadFinished,
    NativeCall {
        class_name: String,
        method_name: String,
        descriptor: String,
        receiver: Option<ObjectId>,
        args: Vec<JavaValue>,
    },
    UnhandledException(ObjectId),
    Error(String),
}

pub struct Interpreter;

impl Interpreter {
    pub fn step_slice(
        thread: &mut GreenThread,
        class_loader: &mut ClassLoader,
        heap: &mut Heap,
        max_instructions: usize,
    ) -> StepResult {
        let mut executed = 0;

        while executed < max_instructions {
            if thread.call_stack.is_empty() {
                thread.state = ThreadState::Terminated;
                return StepResult::ThreadFinished;
            }

            if thread.call_stack.last().unwrap().pc
                >= thread.call_stack.last().unwrap().instructions.len()
            {
                // Reached end of method without explicit return
                let sync_m = thread.call_stack.last().unwrap().sync_monitor;
                thread.call_stack.pop();
                if let Some(m) = sync_m {
                    return StepResult::MonitorExit(m);
                }
                if thread.call_stack.is_empty() {
                    thread.state = ThreadState::Terminated;
                    return StepResult::ThreadFinished;
                }
                continue;
            }

            let instr = {
                let frame = thread.call_stack.last_mut().unwrap();
                let instr = frame.instructions[frame.pc].clone();
                frame.pc += 1;
                instr
            };
            executed += 1;
            thread.instructions_executed += 1;

            match &instr {
                Instruction::New(cls) => {
                    if Self::check_and_trigger_clinit(thread, class_loader, cls) {
                        continue;
                    }
                }
                Instruction::GetStatic { class_name, .. }
                | Instruction::PutStatic { class_name, .. }
                | Instruction::InvokeStatic { class_name, .. } => {
                    if Self::check_and_trigger_clinit(thread, class_loader, class_name) {
                        continue;
                    }
                }
                _ => {}
            }

            let frame = thread.call_stack.last_mut().unwrap();
            let cur_pc = frame.pc.saturating_sub(1);

            match instr {
                Instruction::Nop => {}
                Instruction::AConstNull => frame.push(JavaValue::NULL),
                Instruction::IConst(val) => frame.push(JavaValue::Int(val)),
                Instruction::LConst(val) => frame.push(JavaValue::Long(val)),
                Instruction::FConst(val) => frame.push(JavaValue::Float(val)),
                Instruction::DConst(val) => frame.push(JavaValue::Double(val)),
                Instruction::BiPush(val) => frame.push(JavaValue::Int(val as i32)),
                Instruction::SiPush(val) => frame.push(JavaValue::Int(val as i32)),
                Instruction::LdcInt(val) => frame.push(JavaValue::Int(val)),
                Instruction::LdcFloat(val) => frame.push(JavaValue::Float(val)),
                Instruction::LdcLong(val) => frame.push(JavaValue::Long(val)),
                Instruction::LdcDouble(val) => frame.push(JavaValue::Double(val)),
                Instruction::LdcString(s) => {
                    let str_id = heap.alloc_string(&s);
                    frame.push(JavaValue::Object(Some(str_id)));
                }
                Instruction::LdcClass(name) => {
                    let str_id = heap.alloc_string(&name);
                    frame.push(JavaValue::Object(Some(str_id)));
                }

                // Loads
                Instruction::ILoad(idx) => frame.push(frame.get_local(idx as usize)),
                Instruction::LLoad(idx) => frame.push(frame.get_local(idx as usize)),
                Instruction::FLoad(idx) => frame.push(frame.get_local(idx as usize)),
                Instruction::DLoad(idx) => frame.push(frame.get_local(idx as usize)),
                Instruction::ALoad(idx) => frame.push(frame.get_local(idx as usize)),

                Instruction::IALoad
                | Instruction::BALoad
                | Instruction::CALoad
                | Instruction::SALoad => {
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    let val = match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => {
                            if (index as usize) >= arr.len() {
                                return Self::throw_aiobe(thread, heap, class_loader);
                            }
                            match arr {
                                ArrayData::Int(v) => v.get(index as usize).copied().map(JavaValue::Int),
                                ArrayData::Byte(v) => v
                                    .get(index as usize)
                                    .copied()
                                    .map(|b| JavaValue::Int(b as i32)),
                                ArrayData::Boolean(v) => v
                                    .get(index as usize)
                                    .copied()
                                    .map(|b| JavaValue::Int(if b { 1 } else { 0 })),
                                ArrayData::Char(v) => v
                                    .get(index as usize)
                                    .copied()
                                    .map(|c| JavaValue::Int(c as i32)),
                                ArrayData::Short(v) => v
                                    .get(index as usize)
                                    .copied()
                                    .map(|s| JavaValue::Int(s as i32)),
                                _ => None,
                            }
                        }
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    };
                    match val {
                        Some(v) => frame.push(v),
                        None => {
                            frame.push(JavaValue::Int(0));
                        }
                    }
                }
                Instruction::LALoad => {
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => {
                            if (index as usize) >= arr.len() {
                                return Self::throw_aiobe(thread, heap, class_loader);
                            }
                            if let ArrayData::Long(v) = arr {
                                frame.push(JavaValue::Long(v[index as usize]));
                            } else {
                                frame.push(JavaValue::Long(0));
                            }
                        }
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    }
                }
                Instruction::FALoad => {
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => {
                            if (index as usize) >= arr.len() {
                                return Self::throw_aiobe(thread, heap, class_loader);
                            }
                            if let ArrayData::Float(v) = arr {
                                frame.push(JavaValue::Float(v[index as usize]));
                            } else {
                                frame.push(JavaValue::Float(0.0));
                            }
                        }
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    }
                }
                Instruction::DALoad => {
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => {
                            if (index as usize) >= arr.len() {
                                return Self::throw_aiobe(thread, heap, class_loader);
                            }
                            if let ArrayData::Double(v) = arr {
                                frame.push(JavaValue::Double(v[index as usize]));
                            } else {
                                frame.push(JavaValue::Double(0.0));
                            }
                        }
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    }
                }
                Instruction::AALoad => {
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => {
                            if (index as usize) >= arr.len() {
                                return Self::throw_aiobe(thread, heap, class_loader);
                            }
                            if let ArrayData::Reference(v) = arr {
                                frame.push(JavaValue::Object(v[index as usize]));
                            } else {
                                frame.push(JavaValue::Object(None));
                            }
                        }
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    }
                }

                // Stores
                Instruction::IStore(idx) => {
                    let v = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.set_local(idx as usize, v);
                }
                Instruction::LStore(idx) => {
                    let v = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.set_local(idx as usize, v);
                }
                Instruction::FStore(idx) => {
                    let v = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.set_local(idx as usize, v);
                }
                Instruction::DStore(idx) => {
                    let v = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.set_local(idx as usize, v);
                }
                Instruction::AStore(idx) => {
                    let v = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.set_local(idx as usize, v);
                }

                Instruction::IAStore
                | Instruction::BAStore
                | Instruction::CAStore
                | Instruction::SAStore => {
                    let val = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    if let Some(crate::heap::object::ObjectData::Array(arr)) = heap.get_mut(arr_ref) {
                        let idx = index as usize;
                        if idx >= arr.len() {
                            return Self::throw_aiobe(thread, heap, class_loader);
                        }
                        match arr {
                            ArrayData::Int(v) => {
                                v[idx] = val;
                            }
                            ArrayData::Byte(v) => {
                                v[idx] = val as i8;
                            }
                            ArrayData::Boolean(v) => {
                                v[idx] = val != 0;
                            }
                            ArrayData::Char(v) => {
                                v[idx] = val as u16;
                            }
                            ArrayData::Short(v) => {
                                v[idx] = val as i16;
                            }
                            _ => {}
                        }
                    } else {
                        return Self::throw_npe(thread, heap, class_loader);
                    }
                }
                Instruction::LAStore => {
                    let val = match frame.pop_long() {
                        Ok(l) => l,
                        Err(e) => return StepResult::Error(e),
                    };
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    if let Some(crate::heap::object::ObjectData::Array(ArrayData::Long(v))) =
                        heap.get_mut(arr_ref)
                    {
                        let idx = index as usize;
                        if idx >= v.len() {
                            return Self::throw_aiobe(thread, heap, class_loader);
                        }
                        v[idx] = val;
                    } else {
                        return Self::throw_npe(thread, heap, class_loader);
                    }
                }
                Instruction::FAStore => {
                    let val = match frame.pop_float() {
                        Ok(f) => f,
                        Err(e) => return StepResult::Error(e),
                    };
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    if let Some(crate::heap::object::ObjectData::Array(ArrayData::Float(v))) =
                        heap.get_mut(arr_ref)
                    {
                        let idx = index as usize;
                        if idx >= v.len() {
                            return Self::throw_aiobe(thread, heap, class_loader);
                        }
                        v[idx] = val;
                    } else {
                        return Self::throw_npe(thread, heap, class_loader);
                    }
                }
                Instruction::DAStore => {
                    let val = match frame.pop_double() {
                        Ok(d) => d,
                        Err(e) => return StepResult::Error(e),
                    };
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    if let Some(crate::heap::object::ObjectData::Array(ArrayData::Double(v))) =
                        heap.get_mut(arr_ref)
                    {
                        let idx = index as usize;
                        if idx >= v.len() {
                            return Self::throw_aiobe(thread, heap, class_loader);
                        }
                        v[idx] = val;
                    } else {
                        return Self::throw_npe(thread, heap, class_loader);
                    }
                }
                Instruction::AAStore => {
                    let val = match frame.pop_ref() {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    let index = match frame.pop_int() {
                        Ok(i) => i,
                        Err(e) => return StepResult::Error(e),
                    };
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    if index < 0 {
                        return Self::throw_aiobe(thread, heap, class_loader);
                    }
                    if let Some(crate::heap::object::ObjectData::Array(ArrayData::Reference(v))) =
                        heap.get_mut(arr_ref)
                    {
                        let idx = index as usize;
                        if idx >= v.len() {
                            return Self::throw_aiobe(thread, heap, class_loader);
                        }
                        v[idx] = val;
                    } else {
                        return Self::throw_npe(thread, heap, class_loader);
                    }
                }

                // Stack ops
                Instruction::Pop => {
                    let _ = frame.pop();
                }
                Instruction::Pop2 => {
                    if let Ok(v1) = frame.pop() {
                        if !matches!(v1, JavaValue::Long(_) | JavaValue::Double(_)) {
                            let _ = frame.pop();
                        }
                    }
                }
                Instruction::Dup => {
                    let val = match frame.peek() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.push(val);
                }
                Instruction::DupX1 => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let v2 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.push(v1);
                    frame.push(v2);
                    frame.push(v1);
                }
                Instruction::DupX2 => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let v2 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if matches!(v2, JavaValue::Long(_) | JavaValue::Double(_)) {
                        // Form 1: v1 is Cat 1, v2 is Cat 2 -> ..., v1, v2, v1
                        frame.push(v1);
                        frame.push(v2);
                        frame.push(v1);
                    } else {
                        // Form 2: v1, v2, v3 are Cat 1 -> ..., v1, v3, v2, v1
                        let v3 = match frame.pop() {
                            Ok(v) => v,
                            Err(e) => return StepResult::Error(e),
                        };
                        frame.push(v1);
                        frame.push(v3);
                        frame.push(v2);
                        frame.push(v1);
                    }
                }
                Instruction::Dup2 => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if matches!(v1, JavaValue::Long(_) | JavaValue::Double(_)) {
                        // Form 1: v1 is Cat 2 -> ..., v1, v1
                        frame.push(v1);
                        frame.push(v1);
                    } else {
                        // Form 2: v1, v2 are Cat 1 -> ..., v2, v1, v2, v1
                        let v2 = match frame.pop() {
                            Ok(v) => v,
                            Err(e) => return StepResult::Error(e),
                        };
                        frame.push(v2);
                        frame.push(v1);
                        frame.push(v2);
                        frame.push(v1);
                    }
                }
                Instruction::Dup2X1 => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let v2 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if matches!(v1, JavaValue::Long(_) | JavaValue::Double(_)) {
                        // Form 1: v1 is Cat 2, v2 is Cat 1 -> ..., v1, v2, v1
                        frame.push(v1);
                        frame.push(v2);
                        frame.push(v1);
                    } else {
                        // Form 2: v1, v2, v3 are Cat 1 -> ..., v2, v1, v3, v2, v1
                        let v3 = match frame.pop() {
                            Ok(v) => v,
                            Err(e) => return StepResult::Error(e),
                        };
                        frame.push(v2);
                        frame.push(v1);
                        frame.push(v3);
                        frame.push(v2);
                        frame.push(v1);
                    }
                }
                Instruction::Dup2X2 => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let v2 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if matches!(v1, JavaValue::Long(_) | JavaValue::Double(_)) {
                        if matches!(v2, JavaValue::Long(_) | JavaValue::Double(_)) {
                            // Form 1: v1 is Cat 2, v2 is Cat 2 -> ..., v1, v2, v1
                            frame.push(v1);
                            frame.push(v2);
                            frame.push(v1);
                        } else {
                            // Form 2: v1 is Cat 2, v2 & v3 are Cat 1 -> ..., v1, v3, v2, v1
                            let v3 = match frame.pop() {
                                Ok(v) => v,
                                Err(e) => return StepResult::Error(e),
                            };
                            frame.push(v1);
                            frame.push(v3);
                            frame.push(v2);
                            frame.push(v1);
                        }
                    } else {
                        if matches!(v2, JavaValue::Long(_) | JavaValue::Double(_)) {
                            frame.push(v1);
                            frame.push(v2);
                            frame.push(v1);
                        } else {
                            let v3 = match frame.pop() {
                                Ok(v) => v,
                                Err(e) => return StepResult::Error(e),
                            };
                            if matches!(v3, JavaValue::Long(_) | JavaValue::Double(_)) {
                                // Form 3: v1 & v2 are Cat 1, v3 is Cat 2 -> ..., v2, v1, v3, v2, v1
                                frame.push(v2);
                                frame.push(v1);
                                frame.push(v3);
                                frame.push(v2);
                                frame.push(v1);
                            } else {
                                // Form 4: v1, v2, v3, v4 are Cat 1 -> ..., v2, v1, v4, v3, v2, v1
                                let v4 = match frame.pop() {
                                    Ok(v) => v,
                                    Err(e) => return StepResult::Error(e),
                                };
                                frame.push(v2);
                                frame.push(v1);
                                frame.push(v4);
                                frame.push(v3);
                                frame.push(v2);
                                frame.push(v1);
                            }
                        }
                    }
                }
                Instruction::Swap => {
                    let v1 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let v2 = match frame.pop() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    frame.push(v1);
                    frame.push(v2);
                }

                // Arithmetic
                Instruction::IAdd => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a.wrapping_add(b)));
                }
                Instruction::LAdd => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a.wrapping_add(b)));
                }
                Instruction::FAdd => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(a + b));
                }
                Instruction::DAdd => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(a + b));
                }

                Instruction::ISub => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a.wrapping_sub(b)));
                }
                Instruction::LSub => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a.wrapping_sub(b)));
                }
                Instruction::FSub => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(a - b));
                }
                Instruction::DSub => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(a - b));
                }

                Instruction::IMul => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a.wrapping_mul(b)));
                }
                Instruction::LMul => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a.wrapping_mul(b)));
                }
                Instruction::FMul => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(a * b));
                }
                Instruction::DMul => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(a * b));
                }

                Instruction::IDiv => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    if b == 0 {
                        frame.push(JavaValue::Int(0));
                        continue;
                    }
                    frame.push(JavaValue::Int(a.wrapping_div(b)));
                }
                Instruction::LDiv => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    if b == 0 {
                        frame.push(JavaValue::Long(0));
                        continue;
                    }
                    frame.push(JavaValue::Long(a.wrapping_div(b)));
                }
                Instruction::FDiv => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(a / b));
                }
                Instruction::DDiv => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(a / b));
                }

                Instruction::IRem => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    if b == 0 {
                        frame.push(JavaValue::Int(0));
                        continue;
                    }
                    frame.push(JavaValue::Int(a.wrapping_rem(b)));
                }
                Instruction::LRem => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    if b == 0 {
                        frame.push(JavaValue::Long(0));
                        continue;
                    }
                    frame.push(JavaValue::Long(a.wrapping_rem(b)));
                }
                Instruction::FRem => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(a % b));
                }
                Instruction::DRem => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(a % b));
                }

                Instruction::INeg => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a.wrapping_neg()));
                }
                Instruction::LNeg => {
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a.wrapping_neg()));
                }
                Instruction::FNeg => {
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Float(-a));
                }
                Instruction::DNeg => {
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Double(-a));
                }

                // Shifts & Bitwise
                Instruction::IShl => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a << (b & 0x1F)));
                }
                Instruction::LShl => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a << (b & 0x3F)));
                }
                Instruction::IShr => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a >> (b & 0x1F)));
                }
                Instruction::LShr => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a >> (b & 0x3F)));
                }
                Instruction::IUShr => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(((a as u32) >> (b & 0x1F)) as i32));
                }
                Instruction::LUShr => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(((a as u64) >> (b & 0x3F)) as i64));
                }
                Instruction::IAnd => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a & b));
                }
                Instruction::LAnd => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a & b));
                }
                Instruction::IOr => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a | b));
                }
                Instruction::LOr => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a | b));
                }
                Instruction::IXor => {
                    let b = frame.pop_int().unwrap();
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a ^ b));
                }
                Instruction::LXor => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Long(a ^ b));
                }
                Instruction::IInc { index, const_val } => {
                    let old = frame.get_local(index as usize).as_int().unwrap_or(0);
                    frame.set_local(index as usize, JavaValue::Int(old.wrapping_add(const_val)));
                }

                // Conversions
                Instruction::I2L => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Long(a as i64));
                }
                Instruction::I2F => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Float(a as f32));
                }
                Instruction::I2D => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Double(a as f64));
                }
                Instruction::L2I => {
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Int(a as i32));
                }
                Instruction::L2F => {
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Float(a as f32));
                }
                Instruction::L2D => {
                    let a = frame.pop_long().unwrap();
                    frame.push(JavaValue::Double(a as f64));
                }
                Instruction::F2I => {
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Int(a as i32));
                }
                Instruction::F2L => {
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Long(a as i64));
                }
                Instruction::F2D => {
                    let a = frame.pop_float().unwrap();
                    frame.push(JavaValue::Double(a as f64));
                }
                Instruction::D2I => {
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Int(a as i32));
                }
                Instruction::D2L => {
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Long(a as i64));
                }
                Instruction::D2F => {
                    let a = frame.pop_double().unwrap();
                    frame.push(JavaValue::Float(a as f32));
                }
                Instruction::I2B => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a as i8 as i32));
                }
                Instruction::I2C => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a as u16 as i32));
                }
                Instruction::I2S => {
                    let a = frame.pop_int().unwrap();
                    frame.push(JavaValue::Int(a as i16 as i32));
                }

                // Comparisons
                Instruction::LCmp => {
                    let b = frame.pop_long().unwrap();
                    let a = frame.pop_long().unwrap();
                    let res = if a > b {
                        1
                    } else if a < b {
                        -1
                    } else {
                        0
                    };
                    frame.push(JavaValue::Int(res));
                }
                Instruction::FCmpL => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    let res = if a.is_nan() || b.is_nan() {
                        -1
                    } else if a > b {
                        1
                    } else if a < b {
                        -1
                    } else {
                        0
                    };
                    frame.push(JavaValue::Int(res));
                }
                Instruction::FCmpG => {
                    let b = frame.pop_float().unwrap();
                    let a = frame.pop_float().unwrap();
                    let res = if a.is_nan() || b.is_nan() {
                        1
                    } else if a > b {
                        1
                    } else if a < b {
                        -1
                    } else {
                        0
                    };
                    frame.push(JavaValue::Int(res));
                }
                Instruction::DCmpL => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    let res = if a.is_nan() || b.is_nan() {
                        -1
                    } else if a > b {
                        1
                    } else if a < b {
                        -1
                    } else {
                        0
                    };
                    frame.push(JavaValue::Int(res));
                }
                Instruction::DCmpG => {
                    let b = frame.pop_double().unwrap();
                    let a = frame.pop_double().unwrap();
                    let res = if a.is_nan() || b.is_nan() {
                        1
                    } else if a > b {
                        1
                    } else if a < b {
                        -1
                    } else {
                        0
                    };
                    frame.push(JavaValue::Int(res));
                }

                // Branches
                Instruction::IfEq(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a == 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfNe(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a != 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfLt(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a < 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfGe(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a >= 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfGt(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a > 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfLe(target) => {
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a <= 0 {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }

                Instruction::IfICmpEq(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a == b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfICmpNe(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a != b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfICmpLt(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a < b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfICmpGe(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a >= b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfICmpGt(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a > b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfICmpLe(target) => {
                    let b = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_int() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a <= b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }

                Instruction::IfACmpEq(target) => {
                    let b = match frame.pop_ref() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_ref() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a == b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfACmpNe(target) => {
                    let b = match frame.pop_ref() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    let a = match frame.pop_ref() {
                        Ok(v) => v,
                        Err(e) => return StepResult::Error(e),
                    };
                    if a != b {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }

                Instruction::Goto(target) => {
                    if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                        return res;
                    }
                }
                Instruction::Jsr(target) => {
                    let ret_pc = frame.pc;
                    frame.push(JavaValue::Int(ret_pc as i32));
                    frame.pc = target;
                }
                Instruction::Ret(idx) => {
                    let ret_pc = frame.get_local(idx as usize).as_int().unwrap_or(0);
                    frame.pc = ret_pc as usize;
                }

                Instruction::TableSwitch {
                    default_target,
                    low,
                    high,
                    ref targets,
                } => {
                    let val = frame.pop_int().unwrap();
                    if val >= low && val <= high {
                        let idx = (val - low) as usize;
                        frame.pc = targets[idx];
                    } else {
                        frame.pc = default_target;
                    }
                }
                Instruction::LookupSwitch {
                    default_target,
                    ref pairs,
                } => {
                    let val = frame.pop_int().unwrap();
                    let mut matched = false;
                    for &(k, target) in pairs {
                        if k == val {
                            frame.pc = target;
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        frame.pc = default_target;
                    }
                }

                // Returns
                Instruction::IReturn
                | Instruction::LReturn
                | Instruction::FReturn
                | Instruction::DReturn
                | Instruction::AReturn => {
                    let ret_val = frame.pop().unwrap();
                    let sync_m = frame.sync_monitor;
                    thread.call_stack.pop();
                    if let Some(caller) = thread.call_stack.last_mut() {
                        caller.push(ret_val);
                    } else {
                        thread.state = ThreadState::Terminated;
                    }
                    if let Some(m) = sync_m {
                        return StepResult::MonitorExit(m);
                    }
                    if thread.call_stack.is_empty() {
                        return StepResult::ThreadFinished;
                    }
                }
                Instruction::Return => {
                    let sync_m = frame.sync_monitor;
                    thread.call_stack.pop();
                    if let Some(m) = sync_m {
                        return StepResult::MonitorExit(m);
                    }
                    if thread.call_stack.is_empty() {
                        thread.state = ThreadState::Terminated;
                        return StepResult::ThreadFinished;
                    }
                }

                // Fields
                Instruction::GetStatic {
                    ref class_name,
                    ref field_name,
                    ref descriptor,
                } => {
                    let val = class_loader.get_static(class_name, field_name, descriptor);
                    frame.push(val);
                }
                Instruction::PutStatic {
                    ref class_name,
                    ref field_name,
                    ref descriptor,
                } => {
                    let val = frame.pop().unwrap();
                    class_loader.set_static(class_name, field_name, descriptor, val);
                }
                Instruction::GetField {
                    ref class_name,
                    ref field_name,
                    ref descriptor,
                } => {
                    let obj_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            frame.push(JavaValue::default_for_descriptor(descriptor));
                            continue;
                        }
                        Err(e) => return StepResult::Error(e),
                    };
                    let owner =
                        class_loader.resolve_field_owner(class_name, field_name, descriptor);
                    let val = heap.get_field_typed(obj_ref, &owner, field_name, descriptor);
                    frame.push(val);
                }
                Instruction::PutField {
                    ref class_name,
                    ref field_name,
                    ref descriptor,
                } => {
                    let val = frame.pop().unwrap();
                    let obj_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            continue;
                        }
                        Err(e) => return StepResult::Error(e),
                    };
                    let owner =
                        class_loader.resolve_field_owner(class_name, field_name, descriptor);
                    heap.set_field_typed(obj_ref, &owner, field_name, descriptor, val);
                }

                // Invocations
                Instruction::InvokeStatic {
                    ref class_name,
                    ref method_name,
                    ref descriptor,
                } => {
                    let param_count = parse_param_count(descriptor);
                    let mut args = Vec::with_capacity(param_count);
                    for _ in 0..param_count {
                        args.push(frame.pop().unwrap_or(JavaValue::Int(0)));
                    }
                    args.reverse();

                    // Check for native / stdlib interceptor
                    if let Some(method) =
                        class_loader.find_method(class_name, method_name, descriptor)
                    {
                        if method.is_native() {
                            return StepResult::NativeCall {
                                class_name: class_name.clone(),
                                method_name: method_name.clone(),
                                descriptor: descriptor.clone(),
                                receiver: None,
                                args,
                            };
                        }
                        // Spawn new Frame on thread call stack
                        let mut new_frame = Frame::new(
                            class_name.clone(),
                            method_name.clone(),
                            descriptor.clone(),
                            Arc::clone(&method.instructions),
                            method.max_locals,
                            method.max_stack,
                            method.exception_table.clone(),
                            Arc::clone(&method.pc_map),
                        );
                        let mut local_idx = 0;
                        for arg in args {
                            let takes_two =
                                matches!(arg, JavaValue::Long(_) | JavaValue::Double(_));
                            new_frame.set_local(local_idx, arg);
                            local_idx += if takes_two { 2 } else { 1 };
                        }
                        let sync_monitor = if method.is_synchronized() {
                            Some(class_loader.get_or_create_class_object(heap, &method.class_name))
                        } else {
                            None
                        };
                        new_frame.sync_monitor = sync_monitor;
                        thread.call_stack.push(new_frame);
                        if let Some(m) = sync_monitor {
                            return StepResult::MonitorEnter(m);
                        }
                    } else {
                        // Native method not loaded via bytecode, dispatch to native bridge
                        return StepResult::NativeCall {
                            class_name: class_name.clone(),
                            method_name: method_name.clone(),
                            descriptor: descriptor.clone(),
                            receiver: None,
                            args,
                        };
                    }
                }
                Instruction::InvokeSpecial {
                    ref class_name,
                    ref method_name,
                    ref descriptor,
                }
                | Instruction::InvokeVirtual {
                    ref class_name,
                    ref method_name,
                    ref descriptor,
                }
                | Instruction::InvokeInterface {
                    ref class_name,
                    ref method_name,
                    ref descriptor,
                    ..
                } => {
                    let param_count = parse_param_count(descriptor);
                    let mut args = Vec::with_capacity(param_count);
                    for _ in 0..param_count {
                        args.push(frame.pop().unwrap_or(JavaValue::Int(0)));
                    }
                    args.reverse();

                    let receiver = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            if class_name == "javax/microedition/lcdui/Display"
                                || class_name.ends_with("/Display")
                            {
                                heap.alloc_instance("javax/microedition/lcdui/Display")
                            } else if class_name == "javax/microedition/lcdui/Image"
                                || class_name.ends_with("/Image")
                            {
                                heap.alloc_instance("javax/microedition/lcdui/Image")
                            } else {
                                let ret_desc = if let Some(idx) = descriptor.rfind(')') {
                                    &descriptor[idx + 1..]
                                } else {
                                    &descriptor[..]
                                };
                                if ret_desc != "V" {
                                    frame.push(JavaValue::default_for_descriptor(ret_desc));
                                }
                                continue;
                            }
                        }
                        Err(e) => return StepResult::Error(e),
                    };

                    // Dynamic dispatch if virtual / interface
                    let actual_class =
                        if let Some(crate::heap::object::ObjectData::Instance(inst)) =
                            heap.get(receiver)
                        {
                            inst.class_name.clone()
                        } else {
                            class_name.clone()
                        };

                    let target_class = if matches!(instr, Instruction::InvokeSpecial { .. }) {
                        class_name.as_str()
                    } else {
                        actual_class.as_str()
                    };

                    if let Some(method) =
                        class_loader.find_method(target_class, method_name, descriptor)
                    {
                        if method.is_native() {
                            return StepResult::NativeCall {
                                class_name: target_class.to_string(),
                                method_name: method_name.clone(),
                                descriptor: descriptor.clone(),
                                receiver: Some(receiver),
                                args,
                            };
                        }
                        let mut new_frame = Frame::new(
                            target_class.to_string(),
                            method_name.clone(),
                            descriptor.clone(),
                            Arc::clone(&method.instructions),
                            method.max_locals,
                            method.max_stack,
                            method.exception_table.clone(),
                            Arc::clone(&method.pc_map),
                        );
                        new_frame.set_local(0, JavaValue::Object(Some(receiver)));
                        let mut local_idx = 1;
                        for arg in args {
                            let takes_two =
                                matches!(arg, JavaValue::Long(_) | JavaValue::Double(_));
                            new_frame.set_local(local_idx, arg);
                            local_idx += if takes_two { 2 } else { 1 };
                        }
                        let sync_monitor = if method.is_synchronized() {
                            if method.is_static() {
                                Some(
                                    class_loader
                                        .get_or_create_class_object(heap, &method.class_name),
                                )
                            } else {
                                Some(receiver)
                            }
                        } else {
                            None
                        };
                        new_frame.sync_monitor = sync_monitor;
                        thread.call_stack.push(new_frame);
                        if let Some(m) = sync_monitor {
                            return StepResult::MonitorEnter(m);
                        }
                    } else {
                        return StepResult::NativeCall {
                            class_name: target_class.to_string(),
                            method_name: method_name.clone(),
                            descriptor: descriptor.clone(),
                            receiver: Some(receiver),
                            args,
                        };
                    }
                }

                // Objects & Arrays
                Instruction::New(ref cls_name) => {
                    let id = heap.alloc_instance(cls_name);
                    frame.push(JavaValue::Object(Some(id)));
                }
                Instruction::NewArray(atype) => {
                    let count = frame.pop_int().unwrap();
                    let count = if count < 0 { 0 } else { count as usize };
                    let arr_data = match atype {
                        4 => ArrayData::Boolean(vec![false; count]),
                        5 => ArrayData::Char(vec![0; count]),
                        6 => ArrayData::Float(vec![0.0; count]),
                        7 => ArrayData::Double(vec![0.0; count]),
                        8 => ArrayData::Byte(vec![0; count]),
                        9 => ArrayData::Short(vec![0; count]),
                        10 => ArrayData::Int(vec![0; count]),
                        11 => ArrayData::Long(vec![0; count]),
                        _ => ArrayData::Int(vec![0; count]),
                    };
                    let id = heap.alloc_array(arr_data);
                    if count == 0 {
                        eprintln!(
                            "[NEWARRAY 0] at {}.{} instr={} atype={} id={:?}",
                            frame.class_name, frame.method_name, frame.pc, atype, id
                        );
                    }
                    frame.push(JavaValue::Object(Some(id)));
                }
                Instruction::ANewArray(_) => {
                    let count = frame.pop_int().unwrap();
                    let count = if count < 0 { 0 } else { count as usize };
                    let id = heap.alloc_array(ArrayData::Reference(vec![None; count]));
                    frame.push(JavaValue::Object(Some(id)));
                }
                Instruction::ArrayLength => {
                    let arr_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    let len = match heap.get(arr_ref) {
                        Some(crate::heap::object::ObjectData::Array(arr)) => arr.len(),
                        _ => return Self::throw_npe(thread, heap, class_loader),
                    };
                    frame.push(JavaValue::Int(len as i32));
                }
                Instruction::AThrow => {
                    let exc_ref = match frame.pop_ref() {
                        Ok(Some(id)) => id,
                        Ok(None) => return Self::throw_npe(thread, heap, class_loader),
                        Err(e) => return StepResult::Error(e),
                    };
                    return Self::handle_athrow(thread, heap, class_loader, exc_ref);
                }
                Instruction::CheckCast(_) => {
                    // CheckCast pops and pushes object (or throws ClassCastException)
                    // In CLDC 1.1 preverified classes this check passes or is a no-op
                }
                Instruction::InstanceOf(ref target_class) => {
                    let obj_ref = frame.pop_ref().unwrap();
                    let is_inst = if let Some(id) = obj_ref {
                        match heap.get(id) {
                            Some(crate::heap::object::ObjectData::Instance(inst)) => {
                                class_loader.is_instance_of(&inst.class_name, target_class)
                            }
                            Some(crate::heap::object::ObjectData::String(_)) => {
                                class_loader.is_instance_of("java/lang/String", target_class)
                            }
                            Some(crate::heap::object::ObjectData::Array(arr)) => {
                                let norm_tgt = target_class.replace('.', "/");
                                if norm_tgt == "java/lang/Object" {
                                    true
                                } else {
                                    match arr {
                                        ArrayData::Boolean(_) => norm_tgt == "[Z",
                                        ArrayData::Byte(_) => norm_tgt == "[B",
                                        ArrayData::Char(_) => norm_tgt == "[C",
                                        ArrayData::Short(_) => norm_tgt == "[S",
                                        ArrayData::Int(_) => norm_tgt == "[I",
                                        ArrayData::Long(_) => norm_tgt == "[J",
                                        ArrayData::Float(_) => norm_tgt == "[F",
                                        ArrayData::Double(_) => norm_tgt == "[D",
                                        ArrayData::Reference(elements) => {
                                            if norm_tgt == "[Ljava/lang/Object;" {
                                                true
                                            } else if norm_tgt.starts_with("[L")
                                                && norm_tgt.ends_with(';')
                                            {
                                                let elem_target = &norm_tgt[2..norm_tgt.len() - 1];
                                                // If all non-null elements are instance of elem_target, or empty
                                                let mut all_match = true;
                                                for elem in elements.iter().flatten() {
                                                    if let Some(
                                                        crate::heap::object::ObjectData::Instance(
                                                            elem_inst,
                                                        ),
                                                    ) = heap.get(*elem)
                                                    {
                                                        if !class_loader.is_instance_of(
                                                            &elem_inst.class_name,
                                                            elem_target,
                                                        ) {
                                                            all_match = false;
                                                            break;
                                                        }
                                                    } else if let Some(
                                                        crate::heap::object::ObjectData::String(_),
                                                    ) = heap.get(*elem)
                                                    {
                                                        if !class_loader.is_instance_of(
                                                            "java/lang/String",
                                                            elem_target,
                                                        ) {
                                                            all_match = false;
                                                            break;
                                                        }
                                                    }
                                                }
                                                all_match
                                            } else {
                                                false
                                            }
                                        }
                                    }
                                }
                            }
                            None => false,
                        }
                    } else {
                        false
                    };
                    frame.push(JavaValue::Int(if is_inst { 1 } else { 0 }));
                }
                Instruction::MonitorEnter => {
                    let r = match frame.pop_ref() {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    if let Some(id) = r {
                        return StepResult::MonitorEnter(id);
                    } else {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Self::handle_athrow(thread, heap, class_loader, exc);
                    }
                }
                Instruction::MonitorExit => {
                    let r = match frame.pop_ref() {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    if let Some(id) = r {
                        return StepResult::MonitorExit(id);
                    } else {
                        let exc = heap.alloc_instance("java/lang/NullPointerException");
                        return Self::handle_athrow(thread, heap, class_loader, exc);
                    }
                }
                Instruction::MultiANewArray {
                    ref class_name,
                    dimensions,
                } => {
                    let mut counts = Vec::with_capacity(dimensions as usize);
                    let mut negative = false;
                    for _ in 0..dimensions {
                        let cnt = match frame.pop_int() {
                            Ok(c) => c,
                            Err(e) => return StepResult::Error(e),
                        };
                        if cnt < 0 {
                            negative = true;
                        }
                        counts.push(cnt.max(0) as usize);
                    }
                    if negative {
                        let exc = heap.alloc_instance("java/lang/NegativeArraySizeException");
                        return Self::handle_athrow(thread, heap, class_loader, exc);
                    }
                    counts.reverse();
                    let id = Self::alloc_multi_array(heap, class_name, &counts);
                    frame.push(JavaValue::Object(Some(id)));
                }
                Instruction::IfNull(target) => {
                    let r = match frame.pop_ref() {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    if r.is_none() {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
                Instruction::IfNonNull(target) => {
                    let r = match frame.pop_ref() {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    if r.is_some() {
                        if let Some(res) = Self::branch_to(frame, cur_pc, target) {
                            return res;
                        }
                    }
                }
            }
        }

        StepResult::QuantumYield
    }

    fn alloc_multi_array(heap: &mut Heap, cls: &str, dims: &[usize]) -> ObjectId {
        if dims.is_empty() {
            return heap.alloc_array(ArrayData::Reference(Vec::new()));
        }
        let total_dims = cls.chars().filter(|&c| c == '[').count();
        Self::alloc_multi_array_inner(heap, cls, total_dims, dims)
    }

    fn alloc_multi_array_inner(
        heap: &mut Heap,
        cls: &str,
        remaining_depth: usize,
        dims: &[usize],
    ) -> ObjectId {
        let len = dims[0];
        if dims.len() == 1 {
            if remaining_depth > 1 {
                // Outer dimension of a multidimensional array whose inner dimensions are not yet allocated
                let refs = vec![None; len];
                heap.alloc_array(ArrayData::Reference(refs))
            } else {
                let elem_char = cls.trim_start_matches('[').chars().next().unwrap_or('L');
                let arr_data = match elem_char {
                    'Z' => ArrayData::Boolean(vec![false; len]),
                    'B' => ArrayData::Byte(vec![0; len]),
                    'C' => ArrayData::Char(vec![0; len]),
                    'S' => ArrayData::Short(vec![0; len]),
                    'I' => ArrayData::Int(vec![0; len]),
                    'J' => ArrayData::Long(vec![0; len]),
                    'F' => ArrayData::Float(vec![0.0; len]),
                    'D' => ArrayData::Double(vec![0.0; len]),
                    _ => ArrayData::Reference(vec![None; len]),
                };
                heap.alloc_array(arr_data)
            }
        } else {
            let mut refs = Vec::with_capacity(len);
            for _ in 0..len {
                let sub = Self::alloc_multi_array_inner(
                    heap,
                    cls,
                    remaining_depth.saturating_sub(1),
                    &dims[1..],
                );
                refs.push(Some(sub));
            }
            heap.alloc_array(ArrayData::Reference(refs))
        }
    }

    fn check_and_trigger_clinit(
        thread: &mut GreenThread,
        class_loader: &mut ClassLoader,
        class_name: &str,
    ) -> bool {
        let normalized = class_name.replace('.', "/");
        if class_loader.is_initialized(&normalized) {
            return false;
        }

        // Build hierarchy chain from base (e.g. Object -> SuperClass -> Class)
        let mut hierarchy = Vec::new();
        let mut curr = Some(normalized.clone());
        while let Some(cls_name) = curr {
            hierarchy.push(cls_name.clone());
            if let Some(cls) = class_loader.get_class(&cls_name) {
                curr = cls.super_class.clone();
            } else {
                curr = None;
            }
        }
        hierarchy.reverse();

        let mut clinit_methods = Vec::new();
        for cls_name in &hierarchy {
            if !class_loader.is_initialized(cls_name) {
                class_loader.mark_initialized(cls_name);
                if let Some(m) = class_loader.predecoded_methods.get(&(
                    cls_name.clone(),
                    "<clinit>".to_string(),
                    "()V".to_string(),
                )) {
                    clinit_methods.push((cls_name.clone(), Arc::clone(m)));
                }
            }
        }

        if clinit_methods.is_empty() {
            return false;
        }

        // Rewind current frame PC
        if let Some(current_frame) = thread.call_stack.last_mut() {
            if current_frame.pc > 0 {
                current_frame.pc -= 1;
            }
        }

        // Push clinit frames in reverse order so base class clinit is at the top of stack
        for (cls_name, method) in clinit_methods.into_iter().rev() {
            let clinit_frame = Frame::new(
                cls_name,
                "<clinit>".to_string(),
                "()V".to_string(),
                Arc::clone(&method.instructions),
                method.max_locals,
                method.max_stack,
                method.exception_table.clone(),
                Arc::clone(&method.pc_map),
            );
            thread.call_stack.push(clinit_frame);
        }

        true
    }

    fn throw_npe(
        thread: &mut GreenThread,
        heap: &mut Heap,
        class_loader: &ClassLoader,
    ) -> StepResult {
        let (loc, cur_inst) = if let Some(f) = thread.call_stack.last() {
            let cur = f.pc.saturating_sub(1);
            let inst_str = f
                .instructions
                .get(cur)
                .map(|i| format!("{:?}", i))
                .unwrap_or_else(|| "unknown".to_string());
            (
                format!("{}.{}{} (pc={})", f.class_name, f.method_name, f.descriptor, cur),
                inst_str,
            )
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };
        let msg = format!("Null pointer dereference during {} at {}", cur_inst, loc);
        let id = heap.alloc_instance("java/lang/NullPointerException");
        let msg_id = heap.alloc_string(&msg);
        heap.set_field(id, "detailMessage", JavaValue::Object(Some(msg_id)));
        Self::handle_athrow(thread, heap, class_loader, id)
    }

    fn throw_aiobe(
        thread: &mut GreenThread,
        heap: &mut Heap,
        class_loader: &ClassLoader,
    ) -> StepResult {
        let (loc, cur_inst) = if let Some(f) = thread.call_stack.last() {
            let cur = f.pc.saturating_sub(1);
            let inst_str = f
                .instructions
                .get(cur)
                .map(|i| format!("{:?}", i))
                .unwrap_or_else(|| "unknown".to_string());
            (
                format!("{}.{}{} (pc={})", f.class_name, f.method_name, f.descriptor, cur),
                inst_str,
            )
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };
        let msg = format!("Array index out of bounds during {} at {}", cur_inst, loc);
        let id = heap.alloc_instance("java/lang/ArrayIndexOutOfBoundsException");
        let msg_id = heap.alloc_string(&msg);
        heap.set_field(id, "detailMessage", JavaValue::Object(Some(msg_id)));
        Self::handle_athrow(thread, heap, class_loader, id)
    }

    #[allow(dead_code)]
    fn throw_arithmetic(
        thread: &mut GreenThread,
        heap: &mut Heap,
        class_loader: &ClassLoader,
    ) -> StepResult {
        let (loc, cur_inst) = if let Some(f) = thread.call_stack.last() {
            let cur = f.pc.saturating_sub(1);
            let inst_str = f
                .instructions
                .get(cur)
                .map(|i| format!("{:?}", i))
                .unwrap_or_else(|| "unknown".to_string());
            (
                format!("{}.{}{} (pc={})", f.class_name, f.method_name, f.descriptor, cur),
                inst_str,
            )
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };
        let msg = format!("/ by zero during {} at {}", cur_inst, loc);
        let id = heap.alloc_instance("java/lang/ArithmeticException");
        let msg_id = heap.alloc_string(&msg);
        heap.set_field(id, "detailMessage", JavaValue::Object(Some(msg_id)));
        Self::handle_athrow(thread, heap, class_loader, id)
    }

    #[inline]
    pub fn branch_to(frame: &mut Frame, cur_pc: usize, target: usize) -> Option<StepResult> {
        if target <= cur_pc {
            if frame.last_loop_target == target {
                frame.loop_iteration_count += 1;
                if frame.loop_iteration_count >= 50_000 {
                    frame.loop_iteration_count = 0;
                    frame.pc = cur_pc + 1;
                    return Some(StepResult::Yield);
                }
            } else {
                frame.last_loop_target = target;
                frame.loop_iteration_count = 1;
            }
        } else {
            frame.loop_iteration_count = 0;
        }
        frame.pc = target;
        None
    }

    pub fn handle_athrow(
        thread: &mut GreenThread,
        heap: &mut Heap,
        class_loader: &ClassLoader,
        exc_id: ObjectId,
    ) -> StepResult {
        let (exc_name, exc_msg) =
            if let Some(crate::heap::object::ObjectData::Instance(i)) = heap.get(exc_id) {
                let msg = i.fields.get("detailMessage").and_then(|v| {
                    if let JavaValue::Object(Some(sid)) = v {
                        heap.get_string_utf8(*sid)
                    } else {
                        None
                    }
                });
                (i.class_name.clone(), msg)
            } else {
                ("UnknownException".to_string(), None)
            };

        let (throw_class, throw_method, throw_desc, throw_pc) =
            if let Some(f) = thread.call_stack.last() {
                (
                    f.class_name.clone(),
                    f.method_name.clone(),
                    f.descriptor.clone(),
                    f.pc.saturating_sub(1),
                )
            } else {
                (String::new(), String::new(), String::new(), 0)
            };

        thread.exceptions_thrown += 1;

        let stack_trace: Vec<String> = thread
            .call_stack
            .iter()
            .rev()
            .map(|f| {
                let cur = f.pc.saturating_sub(1);
                let inst = f
                    .instructions
                    .get(cur)
                    .map(|i| format!("{:?}", i))
                    .unwrap_or_else(|| "none".to_string());
                format!(
                    "  at {}.{}{} (pc={}, instr={})",
                    f.class_name, f.method_name, f.descriptor, cur, inst
                )
            })
            .collect();

        // Phase 1: Search for an exception handler across the call stack
        let mut target_frame_idx = None;
        let mut target_handler_idx = None;

        for (frame_idx, frame) in thread.call_stack.iter().enumerate().rev() {
            let cur_instr = frame.pc.saturating_sub(1);
            for entry in &frame.exception_table {
                let start_idx = frame.pc_map.get(&entry.start_pc).copied();
                let end_idx = frame
                    .pc_map
                    .get(&entry.end_pc)
                    .copied()
                    .unwrap_or(frame.instructions.len());

                if let Some(s_idx) = start_idx {
                    if cur_instr < s_idx || cur_instr >= end_idx {
                        continue;
                    }
                } else {
                    continue;
                }

                let type_matches = match &entry.catch_class {
                    None => true, // catch_type == 0 catches everything
                    Some(name) => class_loader.is_instance_of(&exc_name, name),
                };
                if type_matches {
                    if let Some(&h_idx) = frame.pc_map.get(&entry.handler_pc) {
                        target_frame_idx = Some(frame_idx);
                        target_handler_idx = Some(h_idx);
                        break;
                    }
                }
            }
            if target_frame_idx.is_some() {
                break;
            }
        }

        // Phase 2: If a catch block was found, unwind directly to it
        if let (Some(target_f_idx), Some(handler_idx)) = (target_frame_idx, target_handler_idx) {
            while thread.call_stack.len() > target_f_idx + 1 {
                if let Some(frame) = thread.call_stack.pop() {
                    if let Some(m) = frame.sync_monitor {
                        thread.pending_monitor_releases.push(m);
                    }
                }
            }
            if let Some(frame) = thread.call_stack.last_mut() {
                thread.exceptions_caught += 1;
                thread.caught_exception_classes.insert(exc_name.clone());
                let is_critical = exc_name == "java/lang/NullPointerException"
                    || exc_name == "java/lang/NoClassDefFoundError"
                    || exc_name == "java/lang/ClassNotFoundException"
                    || exc_name == "java/lang/NoSuchMethodError"
                    || exc_name == "java/lang/NoSuchFieldError"
                    || exc_name == "java/lang/ArrayIndexOutOfBoundsException"
                    || exc_name == "java/lang/ArrayStoreException"
                    || exc_name == "java/lang/IllegalStateException";
                if is_critical {
                    thread.critical_exceptions_caught += 1;
                    if std::env::var_os("J2ME_LOG_CRIT_EXC").is_some() {
                        eprintln!("[CRIT_EXC] Caught {} msg={:?} at {}.{}:{} in thread {}", exc_name, exc_msg, throw_class, throw_method, throw_pc, thread.name);
                    }
                }
                thread.last_exception_event = Some(ExceptionEvent {
                    exception_class: exc_name,
                    message: exc_msg,
                    thread_name: thread.name.clone(),
                    class_name: throw_class,
                    method_name: throw_method,
                    descriptor: throw_desc,
                    pc: throw_pc,
                    caught: true,
                    stack_trace,
                });
                frame.stack.clear();
                frame.push(JavaValue::Object(Some(exc_id)));
                frame.pc = handler_idx;
                if !thread.pending_monitor_releases.is_empty() {
                    return StepResult::Yield;
                }
                return StepResult::Continue;
            }
        }

        // Phase 3: No catch handler in entire call stack - Auto-Healing Uncaught Handler
        thread.exceptions_uncaught += 1;
        let signature = format!("{}.{}{} pc={} [{}]", throw_class, throw_method, throw_desc, throw_pc, exc_name);
        thread.last_exception_event = Some(ExceptionEvent {
            exception_class: exc_name.clone(),
            message: exc_msg,
            thread_name: thread.name.clone(),
            class_name: throw_class,
            method_name: throw_method,
            descriptor: throw_desc,
            pc: throw_pc,
            caught: false,
            stack_trace: stack_trace.clone(),
        });

        if !thread.strict_mode && thread.call_stack.len() > 1 {
            thread.auto_healing_count += 1;
            thread.last_auto_healing_signature = Some(signature);
            if let Some(ref ev) = thread.last_exception_event {
                thread.auto_healing_events.push(ev.clone());
            }
            let failing_frame = thread.call_stack.pop().unwrap();
            if let Some(m) = failing_frame.sync_monitor {
                thread.pending_monitor_releases.push(m);
            }
            let ret_desc = if let Some(idx) = failing_frame.descriptor.rfind(')') {
                &failing_frame.descriptor[idx + 1..]
            } else {
                &failing_frame.descriptor[..]
            };
            if ret_desc != "V" {
                if let Some(parent_frame) = thread.call_stack.last_mut() {
                    parent_frame.push(JavaValue::default_for_descriptor(ret_desc));
                }
            }
            if !thread.pending_monitor_releases.is_empty() {
                return StepResult::Yield;
            }
            return StepResult::Continue;
        } else {
            return StepResult::UnhandledException(exc_id);
        }
    }
}
