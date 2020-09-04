use enum_map::{Enum};

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
