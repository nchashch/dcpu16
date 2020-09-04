extern crate enum_map;
extern crate either;

mod register;
mod value;
mod basic_op;
mod special_op;
mod command;

pub use register::*;
pub use value::*;
pub use basic_op::*;
pub use special_op::*;
pub use command::*;

use either::{Either};
use enum_map::{EnumMap, enum_map};
use std::collections::{VecDeque};

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
