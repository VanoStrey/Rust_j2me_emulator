use super::instruction::Instruction;
use crate::classfile::attributes::ExceptionTableEntry;
use crate::heap::object::ObjectId;
use crate::heap::value::JavaValue;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Frame {
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
    pub instructions: Arc<Vec<Instruction>>,
    pub pc: usize,
    pub locals: Vec<JavaValue>,
    pub stack: Vec<JavaValue>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub pc_map: Arc<HashMap<u16, usize>>,
    pub sync_monitor: Option<ObjectId>,
    pub last_loop_target: usize,
    pub loop_iteration_count: u32,
}

impl Frame {
    pub fn new(
        class_name: String,
        method_name: String,
        descriptor: String,
        instructions: Arc<Vec<Instruction>>,
        max_locals: usize,
        max_stack: usize,
        exception_table: Vec<ExceptionTableEntry>,
        pc_map: Arc<HashMap<u16, usize>>,
    ) -> Self {
        Self {
            class_name,
            method_name,
            descriptor,
            instructions,
            pc: 0,
            locals: vec![JavaValue::Int(0); max_locals],
            stack: Vec::with_capacity(max_stack),
            exception_table,
            pc_map,
            sync_monitor: None,
            last_loop_target: usize::MAX,
            loop_iteration_count: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, val: JavaValue) {
        self.stack.push(val);
    }

    #[inline]
    pub fn pop(&mut self) -> Result<JavaValue, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Operand stack underflow".to_string())
    }

    #[inline]
    pub fn peek(&self) -> Result<JavaValue, String> {
        self.stack
            .last()
            .copied()
            .ok_or_else(|| "Operand stack empty".to_string())
    }

    #[inline]
    pub fn pop_int(&mut self) -> Result<i32, String> {
        match self.pop() {
            Ok(JavaValue::Int(v)) => Ok(v),
            Ok(JavaValue::Object(None)) => Ok(0),
            Ok(JavaValue::Object(Some(id))) => Ok(id.0 as i32),
            Ok(JavaValue::Long(v)) => Ok(v as i32),
            Ok(JavaValue::Float(v)) => Ok(v as i32),
            Ok(JavaValue::Double(v)) => Ok(v as i32),
            Err(_) => Ok(0),
        }
    }

    #[inline]
    pub fn pop_long(&mut self) -> Result<i64, String> {
        match self.pop() {
            Ok(JavaValue::Long(v)) => Ok(v),
            Ok(JavaValue::Int(v)) => Ok(v as i64),
            Ok(JavaValue::Object(None)) => Ok(0),
            Ok(JavaValue::Object(Some(id))) => Ok(id.0 as i64),
            Ok(JavaValue::Float(v)) => Ok(v as i64),
            Ok(JavaValue::Double(v)) => Ok(v as i64),
            Err(_) => Ok(0),
        }
    }

    #[inline]
    pub fn pop_float(&mut self) -> Result<f32, String> {
        match self.pop() {
            Ok(JavaValue::Float(v)) => Ok(v),
            Ok(JavaValue::Int(v)) => Ok(v as f32),
            Ok(JavaValue::Long(v)) => Ok(v as f32),
            Ok(JavaValue::Double(v)) => Ok(v as f32),
            _ => Ok(0.0),
        }
    }

    #[inline]
    pub fn pop_double(&mut self) -> Result<f64, String> {
        match self.pop() {
            Ok(JavaValue::Double(v)) => Ok(v),
            Ok(JavaValue::Int(v)) => Ok(v as f64),
            Ok(JavaValue::Long(v)) => Ok(v as f64),
            Ok(JavaValue::Float(v)) => Ok(v as f64),
            _ => Ok(0.0),
        }
    }

    #[inline]
    pub fn pop_ref(&mut self) -> Result<Option<ObjectId>, String> {
        match self.pop() {
            Ok(JavaValue::Object(v)) => Ok(v),
            Ok(JavaValue::Int(0)) => Ok(None),
            Ok(JavaValue::Int(v)) => Ok(Some(ObjectId(v as u32))),
            Ok(JavaValue::Long(0)) => Ok(None),
            Ok(JavaValue::Long(v)) => Ok(Some(ObjectId(v as u32))),
            _ => Ok(None),
        }
    }

    #[inline]
    pub fn get_local(&self, index: usize) -> JavaValue {
        self.locals.get(index).copied().unwrap_or(JavaValue::Int(0))
    }

    #[inline]
    pub fn set_local(&mut self, index: usize, val: JavaValue) {
        if index >= self.locals.len() {
            self.locals.resize(index + 1, JavaValue::Int(0));
        }
        self.locals[index] = val;
    }
}
