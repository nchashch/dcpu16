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
