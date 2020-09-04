use crate::dcpu::{Register};

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
