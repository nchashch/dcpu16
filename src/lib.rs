extern crate enum_map;
extern crate either;

use either::{Either};
use enum_map::{EnumMap, Enum, enum_map};
use std::collections::{VecDeque};

#[derive(Debug, Enum, Copy, Clone)]
pub enum Register {
    A,
    B,
    C,
    X,
    Y,
    Z,
    I,
    J
}

impl Register {
    pub fn new(val: u16) -> Option<Register> {
        match val {
            0x00 => Some(Register::A),
            0x01 => Some(Register::B),
            0x02 => Some(Register::C),
            0x03 => Some(Register::X),
            0x04 => Some(Register::Y),
            0x05 => Some(Register::Z),
            0x06 => Some(Register::I),
            0x07 => Some(Register::J),
            _ => None
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            Register::A => 0x00,
            Register::B => 0x01,
            Register::C => 0x02,
            Register::X => 0x03,
            Register::Y => 0x04,
            Register::Z => 0x05,
            Register::I => 0x06,
            Register::J => 0x07
        }
    }
}

pub struct DCPU16 {
    pub reg: EnumMap<Register, u16>,
    pc: u16,
    sp: u16,
    ex: u16,
    ia: u16,
    interrupt_queueing: bool,
    int_queue: VecDeque<(u16, u16)>,
    mem: [u16; 0x10000] // 128 KB of RAM
}

impl Default for DCPU16 {
    fn default() -> DCPU16 {
        DCPU16::new()
    }
}

const MAX_INT_QUEUE_SIZE: usize = 256;

impl DCPU16 {
    pub fn new() -> DCPU16 {
        DCPU16{
            reg: enum_map! {
                Register::A => 0x0000,
                Register::B => 0x0000,
                Register::C => 0x0000,
                Register::X => 0x0000,
                Register::Y => 0x0000,
                Register::Z => 0x0000,
                Register::I => 0x0000,
                Register::J => 0x0000,
            },
            pc: 0x0000,
            sp: 0x0000,
            ex: 0x0000,
            ia: 0x0000,
            interrupt_queueing: false,
            int_queue: VecDeque::with_capacity(MAX_INT_QUEUE_SIZE),
            mem: [0x0000; 0x10000]
        }
    }

    pub fn load(&mut self, rom: [u16; 0x10000]) {
        self.mem = rom;
    }

    pub fn step(&mut self) -> Result<u16, &'static str> {
        let cmd_code = self.mem[self.pc as usize];
        self.pc += 1;
        let mut cycles = 0;
        match Command::new(cmd_code) {
            Some(cmd) => match cmd {
                Command::Basic { op, b, a } => {
                    cycles += op.cycles() + b.cycles() + a.cycles();
                    // Get a copy of immutable operand A
                    let a = self.value(a);
                    // old_ex is copied here to prevent use of borrowed value error
                    let old_ex = self.ex;
                    // Get a mutable reference to mutable operand B
                    let b = self.mut_value(&b);
                    match op {
                        BasicOp::SET => {
                            if let Either::Right(b) = b {
                                *b = a;
                            }
                        },
                        BasicOp::ADD => {
                            if let Either::Right(b) = b {
                                let (result, overflow) = b.overflowing_add(a);
                                *b = result;
                                self.ex = if overflow { 0x0001 } else { 0x0000 };
                            }
                        },
                        BasicOp::SUB => {
                            if let Either::Right(b) = b {
                                let (result, underflow) = b.overflowing_sub(a);
                                *b = result;
                                self.ex = if underflow { 0xffff } else { 0x0000 };
                            }
                        },
                        BasicOp::MUL => {
                            if let Either::Right(b) = b {
                                let b32 = *b as u32;
                                let a32 = a as u32;
                                let result = b32 * a32;
                                *b = (result & 0xffff) as u16;
                                self.ex = ((result >> 16) & 0xffff) as u16;
                            }
                        },
                        BasicOp::MLI => {
                            if let Either::Right(b) = b {
                                let b32 = *b as i32;
                                let a32 = a as i32;
                                let result = b32 * a32;
                                *b = (result & 0xffff) as u16;
                                self.ex = ((result >> 16) & 0xffff) as u16;
                            }
                        },
                        BasicOp::DIV => {
                            if let Either::Right(b) = b {
                                if a == 0 {
                                    *b = 0;
                                    self.ex = 0;
                                } else {
                                    *b = b.wrapping_div(a);
                                    let b32 = *b as u32;
                                    let a32 = a as u32;
                                    self.ex = (((b32 << 16) / a32) & 0xffff) as u16;
                                }
                            }
                        },
                        BasicOp::DVI => {
                            if let Either::Right(b) = b {
                                if a == 0 {
                                    *b = 0;
                                    self.ex = 0;
                                } else {
                                    let result = (*b as i16).wrapping_div(a as i16);
                                    *b = result as u16;
                                    let b32 = *b as i32;
                                    let a32 = a as i32;
                                    self.ex = (((b32 << 16) / a32) & 0xffff) as u16;
                                }
                            }
                        },
                        BasicOp::MOD => {
                            if let Either::Right(b) = b {
                                if a == 0 {
                                    *b = 0;
                                } else {
                                    *b %= a;
                                }
                            }
                        },
                        BasicOp::MDI => {
                            if let Either::Right(b) = b {
                                if a == 0 {
                                    *b = 0;
                                } else {
                                    let result = *b as i16 % a as i16;
                                    *b = result as u16;
                                }
                            }
                        },
                        BasicOp::AND => {
                            if let Either::Right(b) = b {
                                *b &= a;
                            }
                        },
                        BasicOp::BOR => {
                            if let Either::Right(b) = b {
                                *b |= a;
                            }
                        },
                        BasicOp::XOR => {
                            if let Either::Right(b) = b {
                                *b ^= a;
                            }
                        },
                        BasicOp::SHR => {
                            if let Either::Right(b) = b {
                                let b32 = *b as u32;
                                let a32 = a as u32;
                                *b >>= a;
                                self.ex = (((b32 << 16) >> a32) & 0xffff) as u16;
                            }
                        },
                        BasicOp::ASR => {
                            if let Either::Right(b) = b {
                                let b32 = *b as i32;
                                let a32 = a as i32;
                                let result = *b as i16 >> a as i16;
                                *b = result as u16;
                                self.ex = (((b32 << 16) >> a32) & 0xffff) as u16;
                            }
                        },
                        BasicOp::SHL => {
                            if let Either::Right(b) = b {
                                let b32 = *b as u32;
                                let a32 = a as u32;
                                *b <<= a;
                                self.ex = (((b32 << a32) >> 16) & 0xffff) as u16;
                            }
                        },
                        BasicOp::IFB => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b & a != 0x0000 {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFC => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b & a == 0x0000 {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFE => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b == a {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFN => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b != a {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFG => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b > a {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFA => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b as i16 > a as i16 {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFL => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if b < a {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },
                        BasicOp::IFU => {
                            let b = match b {
                                Either::Right(b) => *b,
                                Either::Left(b) => b
                            };
                            if (b as i16) < (a as i16) {
                                // Do nothing
                            } else {
                                // Skip next instruction
                                self.pc += 1;
                            }
                        },

                        BasicOp::ADX => {
                            if let Either::Right(b) = b {
                                let (result, overflow_1) = b.overflowing_add(a);
                                let (result, overflow_2) = result.overflowing_add(old_ex);
                                let overflow = overflow_1 || overflow_2;
                                *b = result;
                                self.ex = if overflow { 0x0001 } else { 0x0000 };
                            }
                        },
                        BasicOp::SBX => {
                            if let Either::Right(b) = b {
                                let (result, underflow) = b.overflowing_sub(a);
                                let (result, overflow) = result.overflowing_add(old_ex);
                                let trouble = underflow || overflow;
                                *b = result;
                                self.ex = if trouble { 0xffff } else { 0x0000 };
                            }
                        },
                        BasicOp::STI => {
                            if let Either::Right(b) = b {
                                *b = a;
                                self.reg[Register::I] += 1;
                                self.reg[Register::J] += 1;
                            }
                        },
                        BasicOp::STD => {
                            if let Either::Right(b) = b {
                                *b = a;
                                self.reg[Register::I] -= 1;
                                self.reg[Register::J] -= 1;
                            }
                        }
                        _ => {}
                    }
                    Ok(self.pc)
                },
                Command::Special { op, a } => {
                    let old_ia = self.ia;
                    let a = self.mut_value(&a);
                    match op {
                        SpecialOp::JSR => {
                            let a = match a {
                                Either::Right(a) => *a,
                                Either::Left(a) => a
                            };
                            self.sp += 1;
                            self.mem[self.sp as usize] = self.pc;
                            self.pc = a;
                        },
                        SpecialOp::INT => {
                            let a = match a {
                                Either::Right(a) => *a,
                                Either::Left(a) => a
                            };
                            if self.ia != 0 {
                                if !self.interrupt_queueing {
                                    self.interrupt_queueing = true;
                                    self.sp += 1;
                                    self.mem[self.sp as usize] = self.pc;
                                    self.sp += 1;
                                    self.mem[self.sp as usize] = self.reg[Register::A];
                                    self.pc = self.ia;
                                    self.reg[Register::A] = a;
                                } else {
                                    // It is not clear if it is possible to cause
                                    // software interrupts to queue.
                                    self.int_queue.push_back((self.pc, a));
                                }
                            }
                        },
                        SpecialOp::IAG => {
                            if let Either::Right(a) = a {
                                *a = old_ia;
                            }
                        },
                        SpecialOp::IAS => {
                            let a = match a {
                                Either::Right(a) => *a,
                                Either::Left(a) => a
                            };
                            self.ia = a;
                        },
                        SpecialOp::RFI => {
                            self.reg[Register::A] = self.mem[self.sp as usize];
                            self.sp -= 1;
                            self.pc = self.mem[self.sp as usize];
                            self.sp -= 1;
                            self.interrupt_queueing = false;
                        },
                        SpecialOp::IAQ => {
                            let a = match a {
                                Either::Right(a) => *a,
                                Either::Left(a) => a
                            };
                            self.interrupt_queueing = a != 0;
                        },
                        SpecialOp::HWN => {
                            unimplemented!();
                        },
                        SpecialOp::HWQ => {
                            unimplemented!();
                        },
                        SpecialOp::HWI => {
                            unimplemented!();
                        }
                    }
                    Ok(self.pc)
                }
            },
            None => Err("couldn't decode command")
        }
    }

    pub fn next_word(&mut self) -> u16 {
        let word = self.mem[self.pc as usize];
        self.pc += 1;
        word
    }

    pub fn value(&mut self, val: Value) -> u16 {
        match val {
            Value::Reg(reg) => {
                self.reg[reg]
            },
            Value::DerefReg(reg) => {
                self.mem[self.reg[reg] as usize]
            },
            Value::IndexReg(reg) => {
                let address = self.reg[reg] + self.next_word();
                self.mem[address as usize]
            },
            Value::STACK => {
                let pop = self.mem[self.sp as usize];
                self.sp += 1;
                pop
            },
            Value::PEEK => {
                self.mem[self.sp as usize]
            },
            Value::PICK => {
                let address = self.sp + self.next_word();
                self.mem[address as usize]
            },
            Value::SP => {
                self.sp
            },
            Value::PC => {
                self.pc
            },
            Value::EX => {
                self.ex
            },
            Value::DerefNextWord => {
                self.mem[self.next_word() as usize]
            },
            Value::NextWord => {
                self.next_word()
            },
            Value::Literal(literal) => {
                literal
            }
        }
    }

    pub fn mut_value(&mut self, val: &Value) -> Either<u16, &mut u16> {
        match val {
            Value::Reg(reg) => {
                Either::Right(&mut self.reg[*reg])
            },
            Value::DerefReg(reg) => {
                Either::Right(&mut self.mem[self.reg[*reg] as usize])
            },
            Value::IndexReg(reg) => {
                let address = self.reg[*reg] + self.next_word();
                Either::Right(&mut self.mem[address as usize])
            },
            Value::STACK => {
                self.sp -= 1;
                Either::Right(&mut self.mem[self.sp as usize])
            },
            Value::PEEK => {
                Either::Right(&mut self.mem[self.sp as usize])
            },
            Value::PICK => {
                let address = self.sp + self.next_word();
                Either::Right(&mut self.mem[address as usize])
            },
            Value::SP => {
                Either::Right(&mut self.sp)
            },
            Value::PC => {
                Either::Right(&mut self.pc)
            },
            Value::EX => {
                Either::Right(&mut self.ex)
            },
            Value::DerefNextWord => {
                // self.pc increment is handled in self.next_word()
                Either::Right(&mut self.mem[self.next_word() as usize])
            },
            Value::NextWord => {
                let result = &mut self.mem[self.pc as usize];
                self.pc += 1;
                Either::Right(result)
            },
            Value::Literal(literal) => {
                Either::Left(*literal)
            }
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Reg(Register), // register (A, B, C, X, Y, Z, I or J, in that order)
    DerefReg(Register), // [register]
    IndexReg(Register), // [register + next word]
    STACK, // (PUSH / [--SP]) if in b, or (POP / [SP++]) if in a
    PEEK, // [SP] / PEEK
    PICK, // [SP + next word] / PICK n
    SP,
    PC,
    EX,
    DerefNextWord, // [next word]
    NextWord, // next word (literal)
    Literal(u16) // literal value 0xffff-0x1e (-1..30) (literal) (only for a)
}

impl Value {
    pub fn new(val: u16) -> Option<Value> {
        if val <= 0x07 {
            if let Some(reg) = Register::new(val) {
                return Some(Value::Reg(reg));
            }
        } else if 0x08 <= val && val <= 0x0f {
            if let Some(reg) = Register::new(val - 0x08) {
                return Some(Value::DerefReg(reg));
            }
        } else if 0x10 <= val && val <= 0x17 {
            if let Some(reg) = Register::new(val - 0x10) {
                return Some(Value::IndexReg(reg));
            }
        } else if 0x20 <= val && val <= 0x3f {
            let literal = val
                .wrapping_add(0xffff)
                .wrapping_sub(0x20);
            return Some(Value::Literal(literal));
        }
        match val {
            0x18 => Some(Value::STACK),
            0x19 => Some(Value::PEEK),
            0x1a => Some(Value::PICK),
            0x1b => Some(Value::SP),
            0x1c => Some(Value::PC),
            0x1d => Some(Value::EX),
            0x1e => Some(Value::DerefNextWord),
            0x1f => Some(Value::NextWord),
            _ => None
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            Value::Reg(reg) => reg.code(),
            Value::DerefReg(reg) => 0x08 + reg.code(),
            Value::IndexReg(reg) => 0x10 + reg.code(),
            Value::STACK => 0x18,
            Value::PEEK => 0x19,
            Value::PICK => 0x1a,
            Value::SP => 0x1b,
            Value::PC => 0x1c,
            Value::EX => 0x1d,
            Value::DerefNextWord => 0x1e,
            Value::NextWord => 0x1f,
            Value::Literal(literal) => {
                literal
                    .wrapping_add(0x20 )
                    .wrapping_sub(0xffff)
            }
        }
    }

    pub fn cycles(&self) -> usize {
        match self {
            Value::Reg(_) => 0,
            Value::DerefReg(_) => 0,
            Value::IndexReg(_) => 1,
            Value::STACK => 0,
            Value::PEEK => 0,
            Value::PICK => 1,
            Value::SP => 0,
            Value::PC => 0,
            Value::EX => 0,
            Value::DerefNextWord => 1,
            Value::NextWord => 0,
            Value::Literal(_) => 0
        }
    }
}

#[derive(Debug)]
pub enum BasicOp {
    SET,
    ADD,
    SUB,
    MUL,
    MLI,
    DIV,
    DVI,
    MOD,
    MDI,
    AND,
    BOR,
    XOR,
    SHR,
    ASR,
    SHL,
    IFB,
    IFC,
    IFE,
    IFN,
    IFG,
    IFA,
    IFL,
    IFU,
    ADX,
    SBX,
    STI,
    STD
}

impl BasicOp {
    pub fn new(val: u16) -> Option<BasicOp> {
        match val {
            0x01 => Some(BasicOp::SET),
            0x02 => Some(BasicOp::ADD),
            0x03 => Some(BasicOp::SUB),
            0x04 => Some(BasicOp::MUL),
            0x05 => Some(BasicOp::MLI),
            0x06 => Some(BasicOp::DIV),
            0x07 => Some(BasicOp::DVI),
            0x08 => Some(BasicOp::MOD),
            0x09 => Some(BasicOp::MDI),
            0x0a => Some(BasicOp::AND),
            0x0b => Some(BasicOp::BOR),
            0x0c => Some(BasicOp::XOR),
            0x0d => Some(BasicOp::SHR),
            0x0e => Some(BasicOp::ASR),
            0x0f => Some(BasicOp::SHL),
            0x10 => Some(BasicOp::IFB),
            0x11 => Some(BasicOp::IFC),
            0x12 => Some(BasicOp::IFE),
            0x13 => Some(BasicOp::IFN),
            0x14 => Some(BasicOp::IFG),
            0x15 => Some(BasicOp::IFA),
            0x16 => Some(BasicOp::IFL),
            0x17 => Some(BasicOp::IFU),

            0x1a => Some(BasicOp::ADX),
            0x1b => Some(BasicOp::SBX),

            0x1e => Some(BasicOp::STI),
            0x1f => Some(BasicOp::STD),
            _ => None
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            BasicOp::SET => 0x01,
            BasicOp::ADD => 0x02,
            BasicOp::SUB => 0x03,
            BasicOp::MUL => 0x04,
            BasicOp::MLI => 0x05,
            BasicOp::DIV => 0x06,
            BasicOp::DVI => 0x07,
            BasicOp::MOD => 0x08,
            BasicOp::MDI => 0x09,
            BasicOp::AND => 0x0a,
            BasicOp::BOR => 0x0b,
            BasicOp::XOR => 0x0c,
            BasicOp::SHR => 0x0d,
            BasicOp::ASR => 0x0e,
            BasicOp::SHL => 0x0f,
            BasicOp::IFB => 0x10,
            BasicOp::IFC => 0x11,
            BasicOp::IFE => 0x12,
            BasicOp::IFN => 0x13,
            BasicOp::IFG => 0x14,
            BasicOp::IFA => 0x15,
            BasicOp::IFL => 0x16,
            BasicOp::IFU => 0x17,

            BasicOp::ADX => 0x1a,
            BasicOp::SBX => 0x1b,

            BasicOp::STI => 0x1e,
            BasicOp::STD => 0x1f
        }
    }

    pub fn cycles(&self) -> usize {
        match self {
            BasicOp::SET => 1,
            BasicOp::ADD => 2,
            BasicOp::SUB => 2,
            BasicOp::MUL => 2,
            BasicOp::MLI => 2,

            BasicOp::DIV => 3,
            BasicOp::DVI => 3,
            BasicOp::MOD => 3,
            BasicOp::MDI => 3,

            BasicOp::AND => 1,
            BasicOp::BOR => 1,
            BasicOp::XOR => 1,
            BasicOp::SHR => 1,
            BasicOp::ASR => 1,
            BasicOp::SHL => 1,

            BasicOp::IFB => 2,
            BasicOp::IFC => 2,
            BasicOp::IFE => 2,
            BasicOp::IFN => 2,
            BasicOp::IFG => 2,
            BasicOp::IFA => 2,
            BasicOp::IFL => 2,
            BasicOp::IFU => 2,

            BasicOp::ADX => 3,
            BasicOp::SBX => 3,

            BasicOp::STI => 2,
            BasicOp::STD => 2
        }
    }
}

#[derive(Debug)]
pub enum SpecialOp {
    JSR,
    INT,
    IAG,
    IAS,
    RFI,
    IAQ,
    HWN,
    HWQ,
    HWI
}

impl SpecialOp {
    pub fn new(val: u16) -> Option<SpecialOp> {
        match val {
            0x01 => Some(SpecialOp::JSR),
            0x08 => Some(SpecialOp::INT),
            0x09 => Some(SpecialOp::IAG),
            0x0a => Some(SpecialOp::IAS),
            0x0b => Some(SpecialOp::RFI),
            0x0c => Some(SpecialOp::IAQ),
            0x10 => Some(SpecialOp::HWN),
            0x11 => Some(SpecialOp::HWQ),
            0x12 => Some(SpecialOp::HWI),
            _ => None
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            SpecialOp::JSR => 0x01,

            SpecialOp::INT => 0x08,
            SpecialOp::IAG => 0x09,
            SpecialOp::IAS => 0x0a,
            SpecialOp::RFI => 0x0b,
            SpecialOp::IAQ => 0x0c,

            SpecialOp::HWN => 0x10,
            SpecialOp::HWQ => 0x11,
            SpecialOp::HWI => 0x12,
        }
    }

    pub fn cycles(&self) -> usize {
        match self {
            SpecialOp::JSR => 3,
            SpecialOp::INT => 4,
            SpecialOp::IAG => 1,
            SpecialOp::IAS => 1,
            SpecialOp::RFI => 3,
            SpecialOp::IAQ => 2,
            SpecialOp::HWN => 2,
            SpecialOp::HWQ => 4,
            SpecialOp::HWI => 4
        }
    }
}

#[derive(Debug)]

pub enum Command {
    Basic {
        op: BasicOp,
        b: Value,
        a: Value
    },
    Special {
        op: SpecialOp,
        a: Value
    }
}

const B_SHIFT: usize = 5;
const A_SHIFT: usize = 10;


impl Command {
    pub fn new(val: u16) -> Option<Command> {
        const OP_MASK: u16 = 0b11111;
        const B_MASK: u16  = 0b11111 << B_SHIFT;
        const A_MASK: u16  = 0b111111 << A_SHIFT;

        let op_code = val & OP_MASK;


        if op_code == 0x0000 {
            let special_op_code = (val & B_MASK) >> B_SHIFT;
            let a_code = (val & A_MASK) >> A_SHIFT;

            let op = SpecialOp::new(special_op_code).unwrap();
            let a = Value::new(a_code).unwrap();

            Some(Command::Special{
                op,
                a
            })
        } else {
            let b_code = (val & B_MASK) >> B_SHIFT;
            let a_code = (val & A_MASK) >> A_SHIFT;

            let op = BasicOp::new(op_code).unwrap();
            let b = Value::new(b_code).unwrap();
            let a = Value::new(a_code).unwrap();

            Some(Command::Basic{
                op,
                b,
                a
            })
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            Command::Basic { op, b, a } => {
                let op_code = op.code();
                let b_code = b.code();
                let a_code = a.code();
                (a_code << A_SHIFT) | (b_code << B_SHIFT) | op_code
            },
            Command::Special { op, a } => {
                let op_code = 0x00;
                let a_code = a.code();
                let special_op_code = op.code();
                (a_code << A_SHIFT) | (special_op_code << A_SHIFT) | op_code
            }
        }
    }
}
