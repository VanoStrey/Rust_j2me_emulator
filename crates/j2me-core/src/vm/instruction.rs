use crate::classfile::constant_pool::{ConstantPool, ConstantPoolEntry};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Nop,
    AConstNull,
    IConst(i32),
    LConst(i64),
    FConst(f32),
    DConst(f64),
    BiPush(i8),
    SiPush(i16),
    LdcInt(i32),
    LdcFloat(f32),
    LdcLong(i64),
    LdcDouble(f64),
    LdcString(String),
    LdcClass(String),

    // Loads
    ILoad(u16),
    LLoad(u16),
    FLoad(u16),
    DLoad(u16),
    ALoad(u16),
    IALoad,
    LALoad,
    FALoad,
    DALoad,
    AALoad,
    BALoad,
    CALoad,
    SALoad,

    // Stores
    IStore(u16),
    LStore(u16),
    FStore(u16),
    DStore(u16),
    AStore(u16),
    IAStore,
    LAStore,
    FAStore,
    DAStore,
    AAStore,
    BAStore,
    CAStore,
    SAStore,

    // Stack ops
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,

    // Arithmetic
    IAdd,
    LAdd,
    FAdd,
    DAdd,
    ISub,
    LSub,
    FSub,
    DSub,
    IMul,
    LMul,
    FMul,
    DMul,
    IDiv,
    LDiv,
    FDiv,
    DDiv,
    IRem,
    LRem,
    FRem,
    DRem,
    INeg,
    LNeg,
    FNeg,
    DNeg,

    // Bitwise / Shifts
    IShl,
    LShl,
    IShr,
    LShr,
    IUShr,
    LUShr,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXor,
    LXor,
    IInc {
        index: u16,
        const_val: i32,
    },

    // Conversions
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,

    // Comparisons
    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,

    // Control flow (targets are pre-resolved instruction indices!)
    IfEq(usize),
    IfNe(usize),
    IfLt(usize),
    IfGe(usize),
    IfGt(usize),
    IfLe(usize),
    IfICmpEq(usize),
    IfICmpNe(usize),
    IfICmpLt(usize),
    IfICmpGe(usize),
    IfICmpGt(usize),
    IfICmpLe(usize),
    IfACmpEq(usize),
    IfACmpNe(usize),
    Goto(usize),
    Jsr(usize),
    Ret(u16),
    TableSwitch {
        default_target: usize,
        low: i32,
        high: i32,
        targets: Vec<usize>,
    },
    LookupSwitch {
        default_target: usize,
        pairs: Vec<(i32, usize)>,
    },

    // Returns
    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,

    // Fields & Methods (pre-resolved symbols!)
    GetStatic {
        class_name: String,
        field_name: String,
        descriptor: String,
    },
    PutStatic {
        class_name: String,
        field_name: String,
        descriptor: String,
    },
    GetField {
        class_name: String,
        field_name: String,
        descriptor: String,
    },
    PutField {
        class_name: String,
        field_name: String,
        descriptor: String,
    },
    InvokeVirtual {
        class_name: String,
        method_name: String,
        descriptor: String,
    },
    InvokeSpecial {
        class_name: String,
        method_name: String,
        descriptor: String,
    },
    InvokeStatic {
        class_name: String,
        method_name: String,
        descriptor: String,
    },
    InvokeInterface {
        class_name: String,
        method_name: String,
        descriptor: String,
        count: u8,
    },

    // Objects & Arrays
    New(String),
    NewArray(u8),
    ANewArray(String),
    ArrayLength,
    AThrow,
    CheckCast(String),
    InstanceOf(String),
    MonitorEnter,
    MonitorExit,
    MultiANewArray {
        class_name: String,
        dimensions: u8,
    },
    IfNull(usize),
    IfNonNull(usize),
}

#[derive(Debug, Clone)]
enum RawTarget {
    Offset(i32),
    TableSwitch {
        default_offset: i32,
        low: i32,
        high: i32,
        offsets: Vec<i32>,
    },
    LookupSwitch {
        default_offset: i32,
        pairs: Vec<(i32, i32)>,
    },
}

#[derive(Debug, Clone)]
enum IntermediateInstr {
    Resolved(Instruction),
    Branch(u8, RawTarget),
}

pub struct Predecoder;

impl Predecoder {
    pub fn decode_method_code(
        code: &[u8],
        cp: &ConstantPool,
    ) -> (Vec<Instruction>, HashMap<u16, usize>) {
        let mut pc = 0;
        let mut pc_to_instr = HashMap::new();
        let mut intermediate = Vec::new();

        while pc < code.len() {
            let cur_pc = pc as u16;
            let instr_idx = intermediate.len();
            pc_to_instr.insert(cur_pc, instr_idx);

            let opcode = code[pc];
            pc += 1;

            match opcode {
                0x00 => intermediate.push(IntermediateInstr::Resolved(Instruction::Nop)),
                0x01 => intermediate.push(IntermediateInstr::Resolved(Instruction::AConstNull)),
                0x02 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(-1))),
                0x03 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(0))),
                0x04 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(1))),
                0x05 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(2))),
                0x06 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(3))),
                0x07 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(4))),
                0x08 => intermediate.push(IntermediateInstr::Resolved(Instruction::IConst(5))),
                0x09 => intermediate.push(IntermediateInstr::Resolved(Instruction::LConst(0))),
                0x0A => intermediate.push(IntermediateInstr::Resolved(Instruction::LConst(1))),
                0x0B => intermediate.push(IntermediateInstr::Resolved(Instruction::FConst(0.0))),
                0x0C => intermediate.push(IntermediateInstr::Resolved(Instruction::FConst(1.0))),
                0x0D => intermediate.push(IntermediateInstr::Resolved(Instruction::FConst(2.0))),
                0x0E => intermediate.push(IntermediateInstr::Resolved(Instruction::DConst(0.0))),
                0x0F => intermediate.push(IntermediateInstr::Resolved(Instruction::DConst(1.0))),
                0x10 => {
                    let b = code[pc] as i8;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::BiPush(b)));
                }
                0x11 => {
                    let s = i16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::SiPush(s)));
                }
                0x12 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Self::resolve_ldc(idx, cp)));
                }
                0x13 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    intermediate.push(IntermediateInstr::Resolved(Self::resolve_ldc(idx, cp)));
                }
                0x14 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    intermediate.push(IntermediateInstr::Resolved(Self::resolve_ldc2(idx, cp)));
                }
                0x15 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::ILoad(idx)));
                }
                0x16 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::LLoad(idx)));
                }
                0x17 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::FLoad(idx)));
                }
                0x18 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::DLoad(idx)));
                }
                0x19 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::ALoad(idx)));
                }
                0x1A..=0x1D => intermediate.push(IntermediateInstr::Resolved(Instruction::ILoad(
                    (opcode - 0x1A) as u16,
                ))),
                0x1E..=0x21 => intermediate.push(IntermediateInstr::Resolved(Instruction::LLoad(
                    (opcode - 0x1E) as u16,
                ))),
                0x22..=0x25 => intermediate.push(IntermediateInstr::Resolved(Instruction::FLoad(
                    (opcode - 0x22) as u16,
                ))),
                0x26..=0x29 => intermediate.push(IntermediateInstr::Resolved(Instruction::DLoad(
                    (opcode - 0x26) as u16,
                ))),
                0x2A..=0x2D => intermediate.push(IntermediateInstr::Resolved(Instruction::ALoad(
                    (opcode - 0x2A) as u16,
                ))),
                0x2E => intermediate.push(IntermediateInstr::Resolved(Instruction::IALoad)),
                0x2F => intermediate.push(IntermediateInstr::Resolved(Instruction::LALoad)),
                0x30 => intermediate.push(IntermediateInstr::Resolved(Instruction::FALoad)),
                0x31 => intermediate.push(IntermediateInstr::Resolved(Instruction::DALoad)),
                0x32 => intermediate.push(IntermediateInstr::Resolved(Instruction::AALoad)),
                0x33 => intermediate.push(IntermediateInstr::Resolved(Instruction::BALoad)),
                0x34 => intermediate.push(IntermediateInstr::Resolved(Instruction::CALoad)),
                0x35 => intermediate.push(IntermediateInstr::Resolved(Instruction::SALoad)),
                0x36 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::IStore(idx)));
                }
                0x37 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::LStore(idx)));
                }
                0x38 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::FStore(idx)));
                }
                0x39 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::DStore(idx)));
                }
                0x3A => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::AStore(idx)));
                }
                0x3B..=0x3E => intermediate.push(IntermediateInstr::Resolved(Instruction::IStore(
                    (opcode - 0x3B) as u16,
                ))),
                0x3F..=0x42 => intermediate.push(IntermediateInstr::Resolved(Instruction::LStore(
                    (opcode - 0x3F) as u16,
                ))),
                0x43..=0x46 => intermediate.push(IntermediateInstr::Resolved(Instruction::FStore(
                    (opcode - 0x43) as u16,
                ))),
                0x47..=0x4A => intermediate.push(IntermediateInstr::Resolved(Instruction::DStore(
                    (opcode - 0x47) as u16,
                ))),
                0x4B..=0x4E => intermediate.push(IntermediateInstr::Resolved(Instruction::AStore(
                    (opcode - 0x4B) as u16,
                ))),
                0x4F => intermediate.push(IntermediateInstr::Resolved(Instruction::IAStore)),
                0x50 => intermediate.push(IntermediateInstr::Resolved(Instruction::LAStore)),
                0x51 => intermediate.push(IntermediateInstr::Resolved(Instruction::FAStore)),
                0x52 => intermediate.push(IntermediateInstr::Resolved(Instruction::DAStore)),
                0x53 => intermediate.push(IntermediateInstr::Resolved(Instruction::AAStore)),
                0x54 => intermediate.push(IntermediateInstr::Resolved(Instruction::BAStore)),
                0x55 => intermediate.push(IntermediateInstr::Resolved(Instruction::CAStore)),
                0x56 => intermediate.push(IntermediateInstr::Resolved(Instruction::SAStore)),
                0x57 => intermediate.push(IntermediateInstr::Resolved(Instruction::Pop)),
                0x58 => intermediate.push(IntermediateInstr::Resolved(Instruction::Pop2)),
                0x59 => intermediate.push(IntermediateInstr::Resolved(Instruction::Dup)),
                0x5A => intermediate.push(IntermediateInstr::Resolved(Instruction::DupX1)),
                0x5B => intermediate.push(IntermediateInstr::Resolved(Instruction::DupX2)),
                0x5C => intermediate.push(IntermediateInstr::Resolved(Instruction::Dup2)),
                0x5D => intermediate.push(IntermediateInstr::Resolved(Instruction::Dup2X1)),
                0x5E => intermediate.push(IntermediateInstr::Resolved(Instruction::Dup2X2)),
                0x5F => intermediate.push(IntermediateInstr::Resolved(Instruction::Swap)),
                0x60 => intermediate.push(IntermediateInstr::Resolved(Instruction::IAdd)),
                0x61 => intermediate.push(IntermediateInstr::Resolved(Instruction::LAdd)),
                0x62 => intermediate.push(IntermediateInstr::Resolved(Instruction::FAdd)),
                0x63 => intermediate.push(IntermediateInstr::Resolved(Instruction::DAdd)),
                0x64 => intermediate.push(IntermediateInstr::Resolved(Instruction::ISub)),
                0x65 => intermediate.push(IntermediateInstr::Resolved(Instruction::LSub)),
                0x66 => intermediate.push(IntermediateInstr::Resolved(Instruction::FSub)),
                0x67 => intermediate.push(IntermediateInstr::Resolved(Instruction::DSub)),
                0x68 => intermediate.push(IntermediateInstr::Resolved(Instruction::IMul)),
                0x69 => intermediate.push(IntermediateInstr::Resolved(Instruction::LMul)),
                0x6A => intermediate.push(IntermediateInstr::Resolved(Instruction::FMul)),
                0x6B => intermediate.push(IntermediateInstr::Resolved(Instruction::DMul)),
                0x6C => intermediate.push(IntermediateInstr::Resolved(Instruction::IDiv)),
                0x6D => intermediate.push(IntermediateInstr::Resolved(Instruction::LDiv)),
                0x6E => intermediate.push(IntermediateInstr::Resolved(Instruction::FDiv)),
                0x6F => intermediate.push(IntermediateInstr::Resolved(Instruction::DDiv)),
                0x70 => intermediate.push(IntermediateInstr::Resolved(Instruction::IRem)),
                0x71 => intermediate.push(IntermediateInstr::Resolved(Instruction::LRem)),
                0x72 => intermediate.push(IntermediateInstr::Resolved(Instruction::FRem)),
                0x73 => intermediate.push(IntermediateInstr::Resolved(Instruction::DRem)),
                0x74 => intermediate.push(IntermediateInstr::Resolved(Instruction::INeg)),
                0x75 => intermediate.push(IntermediateInstr::Resolved(Instruction::LNeg)),
                0x76 => intermediate.push(IntermediateInstr::Resolved(Instruction::FNeg)),
                0x77 => intermediate.push(IntermediateInstr::Resolved(Instruction::DNeg)),
                0x78 => intermediate.push(IntermediateInstr::Resolved(Instruction::IShl)),
                0x79 => intermediate.push(IntermediateInstr::Resolved(Instruction::LShl)),
                0x7A => intermediate.push(IntermediateInstr::Resolved(Instruction::IShr)),
                0x7B => intermediate.push(IntermediateInstr::Resolved(Instruction::LShr)),
                0x7C => intermediate.push(IntermediateInstr::Resolved(Instruction::IUShr)),
                0x7D => intermediate.push(IntermediateInstr::Resolved(Instruction::LUShr)),
                0x7E => intermediate.push(IntermediateInstr::Resolved(Instruction::IAnd)),
                0x7F => intermediate.push(IntermediateInstr::Resolved(Instruction::LAnd)),
                0x80 => intermediate.push(IntermediateInstr::Resolved(Instruction::IOr)),
                0x81 => intermediate.push(IntermediateInstr::Resolved(Instruction::LOr)),
                0x82 => intermediate.push(IntermediateInstr::Resolved(Instruction::IXor)),
                0x83 => intermediate.push(IntermediateInstr::Resolved(Instruction::LXor)),
                0x84 => {
                    let idx = code[pc] as u16;
                    let c = code[pc + 1] as i8 as i32;
                    pc += 2;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::IInc {
                        index: idx,
                        const_val: c,
                    }));
                }
                0x85 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2L)),
                0x86 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2F)),
                0x87 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2D)),
                0x88 => intermediate.push(IntermediateInstr::Resolved(Instruction::L2I)),
                0x89 => intermediate.push(IntermediateInstr::Resolved(Instruction::L2F)),
                0x8A => intermediate.push(IntermediateInstr::Resolved(Instruction::L2D)),
                0x8B => intermediate.push(IntermediateInstr::Resolved(Instruction::F2I)),
                0x8C => intermediate.push(IntermediateInstr::Resolved(Instruction::F2L)),
                0x8D => intermediate.push(IntermediateInstr::Resolved(Instruction::F2D)),
                0x8E => intermediate.push(IntermediateInstr::Resolved(Instruction::D2I)),
                0x8F => intermediate.push(IntermediateInstr::Resolved(Instruction::D2L)),
                0x90 => intermediate.push(IntermediateInstr::Resolved(Instruction::D2F)),
                0x91 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2B)),
                0x92 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2C)),
                0x93 => intermediate.push(IntermediateInstr::Resolved(Instruction::I2S)),
                0x94 => intermediate.push(IntermediateInstr::Resolved(Instruction::LCmp)),
                0x95 => intermediate.push(IntermediateInstr::Resolved(Instruction::FCmpL)),
                0x96 => intermediate.push(IntermediateInstr::Resolved(Instruction::FCmpG)),
                0x97 => intermediate.push(IntermediateInstr::Resolved(Instruction::DCmpL)),
                0x98 => intermediate.push(IntermediateInstr::Resolved(Instruction::DCmpG)),

                // Conditional and unconditional jumps (branch offset relative to opcode position)
                0x99..=0xA8 | 0xC6..=0xC7 => {
                    let offset = i16::from_be_bytes([code[pc], code[pc + 1]]) as i32;
                    pc += 2;
                    intermediate.push(IntermediateInstr::Branch(
                        opcode,
                        RawTarget::Offset(cur_pc as i32 + offset),
                    ));
                }
                0xA9 => {
                    let idx = code[pc] as u16;
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::Ret(idx)));
                }
                0xAA => {
                    // TableSwitch: 0-3 bytes padding for 4-byte alignment relative to method start
                    let pad = (4 - (pc % 4)) % 4;
                    pc += pad;
                    let default_offset =
                        i32::from_be_bytes([code[pc], code[pc + 1], code[pc + 2], code[pc + 3]]);
                    let low = i32::from_be_bytes([
                        code[pc + 4],
                        code[pc + 5],
                        code[pc + 6],
                        code[pc + 7],
                    ]);
                    let high = i32::from_be_bytes([
                        code[pc + 8],
                        code[pc + 9],
                        code[pc + 10],
                        code[pc + 11],
                    ]);
                    pc += 12;
                    let count = (high - low + 1) as usize;
                    let mut offsets = Vec::with_capacity(count);
                    for _ in 0..count {
                        let off = i32::from_be_bytes([
                            code[pc],
                            code[pc + 1],
                            code[pc + 2],
                            code[pc + 3],
                        ]);
                        offsets.push(cur_pc as i32 + off);
                        pc += 4;
                    }
                    intermediate.push(IntermediateInstr::Branch(
                        opcode,
                        RawTarget::TableSwitch {
                            default_offset: cur_pc as i32 + default_offset,
                            low,
                            high,
                            offsets,
                        },
                    ));
                }
                0xAB => {
                    // LookupSwitch
                    let pad = (4 - (pc % 4)) % 4;
                    pc += pad;
                    let default_offset =
                        i32::from_be_bytes([code[pc], code[pc + 1], code[pc + 2], code[pc + 3]]);
                    let npairs = i32::from_be_bytes([
                        code[pc + 4],
                        code[pc + 5],
                        code[pc + 6],
                        code[pc + 7],
                    ]) as usize;
                    pc += 8;
                    let mut pairs = Vec::with_capacity(npairs);
                    for _ in 0..npairs {
                        let match_val = i32::from_be_bytes([
                            code[pc],
                            code[pc + 1],
                            code[pc + 2],
                            code[pc + 3],
                        ]);
                        let off = i32::from_be_bytes([
                            code[pc + 4],
                            code[pc + 5],
                            code[pc + 6],
                            code[pc + 7],
                        ]);
                        pairs.push((match_val, cur_pc as i32 + off));
                        pc += 8;
                    }
                    intermediate.push(IntermediateInstr::Branch(
                        opcode,
                        RawTarget::LookupSwitch {
                            default_offset: cur_pc as i32 + default_offset,
                            pairs,
                        },
                    ));
                }

                0xAC => intermediate.push(IntermediateInstr::Resolved(Instruction::IReturn)),
                0xAD => intermediate.push(IntermediateInstr::Resolved(Instruction::LReturn)),
                0xAE => intermediate.push(IntermediateInstr::Resolved(Instruction::FReturn)),
                0xAF => intermediate.push(IntermediateInstr::Resolved(Instruction::DReturn)),
                0xB0 => intermediate.push(IntermediateInstr::Resolved(Instruction::AReturn)),
                0xB1 => intermediate.push(IntermediateInstr::Resolved(Instruction::Return)),

                0xB2 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_field_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::GetStatic {
                        class_name: cls.to_string(),
                        field_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB3 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_field_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::PutStatic {
                        class_name: cls.to_string(),
                        field_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB4 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_field_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::GetField {
                        class_name: cls.to_string(),
                        field_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB5 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_field_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::PutField {
                        class_name: cls.to_string(),
                        field_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB6 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_method_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::InvokeVirtual {
                        class_name: cls.to_string(),
                        method_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB7 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_method_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::InvokeSpecial {
                        class_name: cls.to_string(),
                        method_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB8 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let (cls, name, desc) = cp.get_method_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::InvokeStatic {
                        class_name: cls.to_string(),
                        method_name: name.to_string(),
                        descriptor: desc.to_string(),
                    }));
                }
                0xB9 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    let count = code[pc + 2];
                    pc += 4; // count + 1 zero byte
                    let (cls, name, desc) = cp.get_method_ref(idx).unwrap_or(("", "", ""));
                    intermediate.push(IntermediateInstr::Resolved(Instruction::InvokeInterface {
                        class_name: cls.to_string(),
                        method_name: name.to_string(),
                        descriptor: desc.to_string(),
                        count,
                    }));
                }
                0xBB => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let cls = cp.get_class_name(idx).unwrap_or("");
                    intermediate.push(IntermediateInstr::Resolved(Instruction::New(
                        cls.to_string(),
                    )));
                }
                0xBC => {
                    let atype = code[pc];
                    pc += 1;
                    intermediate.push(IntermediateInstr::Resolved(Instruction::NewArray(atype)));
                }
                0xBD => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let cls = cp.get_class_name(idx).unwrap_or("");
                    intermediate.push(IntermediateInstr::Resolved(Instruction::ANewArray(
                        cls.to_string(),
                    )));
                }
                0xBE => intermediate.push(IntermediateInstr::Resolved(Instruction::ArrayLength)),
                0xBF => intermediate.push(IntermediateInstr::Resolved(Instruction::AThrow)),
                0xC0 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let cls = cp.get_class_name(idx).unwrap_or("");
                    intermediate.push(IntermediateInstr::Resolved(Instruction::CheckCast(
                        cls.to_string(),
                    )));
                }
                0xC1 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    let cls = cp.get_class_name(idx).unwrap_or("");
                    intermediate.push(IntermediateInstr::Resolved(Instruction::InstanceOf(
                        cls.to_string(),
                    )));
                }
                0xC2 => intermediate.push(IntermediateInstr::Resolved(Instruction::MonitorEnter)),
                0xC3 => intermediate.push(IntermediateInstr::Resolved(Instruction::MonitorExit)),
                0xC4 => {
                    // Wide instruction
                    let sub_op = code[pc];
                    pc += 1;
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    pc += 2;
                    match sub_op {
                        0x15 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::ILoad(idx)))
                        }
                        0x16 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::LLoad(idx)))
                        }
                        0x17 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::FLoad(idx)))
                        }
                        0x18 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::DLoad(idx)))
                        }
                        0x19 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::ALoad(idx)))
                        }
                        0x36 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::IStore(idx)))
                        }
                        0x37 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::LStore(idx)))
                        }
                        0x38 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::FStore(idx)))
                        }
                        0x39 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::DStore(idx)))
                        }
                        0x3A => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::AStore(idx)))
                        }
                        0x84 => {
                            let const_val = i16::from_be_bytes([code[pc], code[pc + 1]]) as i32;
                            pc += 2;
                            intermediate.push(IntermediateInstr::Resolved(Instruction::IInc {
                                index: idx,
                                const_val,
                            }));
                        }
                        0xA9 => {
                            intermediate.push(IntermediateInstr::Resolved(Instruction::Ret(idx)))
                        }
                        _ => {}
                    }
                }
                0xC5 => {
                    let idx = u16::from_be_bytes([code[pc], code[pc + 1]]);
                    let dimensions = code[pc + 2];
                    pc += 3;
                    let cls = cp.get_class_name(idx).unwrap_or("");
                    intermediate.push(IntermediateInstr::Resolved(Instruction::MultiANewArray {
                        class_name: cls.to_string(),
                        dimensions,
                    }));
                }
                0xC8 | 0xC9 => {
                    // Goto_w or Jsr_w
                    let offset =
                        i32::from_be_bytes([code[pc], code[pc + 1], code[pc + 2], code[pc + 3]]);
                    pc += 4;
                    intermediate.push(IntermediateInstr::Branch(
                        opcode,
                        RawTarget::Offset(cur_pc as i32 + offset),
                    ));
                }
                _ => {
                    // Unknown or unsupported opcode fallback
                    intermediate.push(IntermediateInstr::Resolved(Instruction::Nop));
                }
            }
        }

        // Pass 2: resolve branch target offsets to instruction index
        let mut final_instructions = Vec::with_capacity(intermediate.len());
        for item in intermediate {
            match item {
                IntermediateInstr::Resolved(instr) => final_instructions.push(instr),
                IntermediateInstr::Branch(op, target) => match target {
                    RawTarget::Offset(byte_off) => {
                        let target_idx = *pc_to_instr
                            .get(&(byte_off as u16))
                            .unwrap_or(&final_instructions.len());
                        let resolved = match op {
                            0x99 => Instruction::IfEq(target_idx),
                            0x9A => Instruction::IfNe(target_idx),
                            0x9B => Instruction::IfLt(target_idx),
                            0x9C => Instruction::IfGe(target_idx),
                            0x9D => Instruction::IfGt(target_idx),
                            0x9E => Instruction::IfLe(target_idx),
                            0x9F => Instruction::IfICmpEq(target_idx),
                            0xA0 => Instruction::IfICmpNe(target_idx),
                            0xA1 => Instruction::IfICmpLt(target_idx),
                            0xA2 => Instruction::IfICmpGe(target_idx),
                            0xA3 => Instruction::IfICmpGt(target_idx),
                            0xA4 => Instruction::IfICmpLe(target_idx),
                            0xA5 => Instruction::IfACmpEq(target_idx),
                            0xA6 => Instruction::IfACmpNe(target_idx),
                            0xA7 | 0xC8 => Instruction::Goto(target_idx),
                            0xA8 | 0xC9 => Instruction::Jsr(target_idx),
                            0xC6 => Instruction::IfNull(target_idx),
                            0xC7 => Instruction::IfNonNull(target_idx),
                            _ => Instruction::Nop,
                        };
                        final_instructions.push(resolved);
                    }
                    RawTarget::TableSwitch {
                        default_offset,
                        low,
                        high,
                        offsets,
                    } => {
                        let default_target = *pc_to_instr
                            .get(&(default_offset as u16))
                            .unwrap_or(&final_instructions.len());
                        let targets: Vec<usize> = offsets
                            .iter()
                            .map(|off| {
                                *pc_to_instr
                                    .get(&(*off as u16))
                                    .unwrap_or(&final_instructions.len())
                            })
                            .collect();
                        final_instructions.push(Instruction::TableSwitch {
                            default_target,
                            low,
                            high,
                            targets,
                        });
                    }
                    RawTarget::LookupSwitch {
                        default_offset,
                        pairs,
                    } => {
                        let default_target = *pc_to_instr
                            .get(&(default_offset as u16))
                            .unwrap_or(&final_instructions.len());
                        let resolved_pairs: Vec<(i32, usize)> = pairs
                            .iter()
                            .map(|(val, off)| {
                                (
                                    *val,
                                    *pc_to_instr
                                        .get(&(*off as u16))
                                        .unwrap_or(&final_instructions.len()),
                                )
                            })
                            .collect();
                        final_instructions.push(Instruction::LookupSwitch {
                            default_target,
                            pairs: resolved_pairs,
                        });
                    }
                },
            }
        }

        (final_instructions, pc_to_instr)
    }

    fn resolve_ldc(idx: u16, cp: &ConstantPool) -> Instruction {
        match cp.entries.get(idx as usize) {
            Some(ConstantPoolEntry::Integer(v)) => Instruction::LdcInt(*v),
            Some(ConstantPoolEntry::Float(v)) => Instruction::LdcFloat(*v),
            Some(ConstantPoolEntry::StringRef { string_index }) => {
                let s = cp.get_utf8(*string_index).unwrap_or("");
                Instruction::LdcString(s.to_string())
            }
            Some(ConstantPoolEntry::Class { name_index }) => {
                let s = cp.get_utf8(*name_index).unwrap_or("");
                Instruction::LdcClass(s.to_string())
            }
            _ => Instruction::Nop,
        }
    }

    fn resolve_ldc2(idx: u16, cp: &ConstantPool) -> Instruction {
        match cp.entries.get(idx as usize) {
            Some(ConstantPoolEntry::Long(v)) => Instruction::LdcLong(*v),
            Some(ConstantPoolEntry::Double(v)) => Instruction::LdcDouble(*v),
            _ => Instruction::Nop,
        }
    }
}
