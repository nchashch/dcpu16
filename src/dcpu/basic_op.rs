use std::str::FromStr;

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

impl FromStr for BasicOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(BasicOp::SET),
            "add" => Ok(BasicOp::ADD),
            "sub" => Ok(BasicOp::SUB),
            "mul" => Ok(BasicOp::MUL),
            "mli" => Ok(BasicOp::MLI),
            "div" => Ok(BasicOp::DIV),
            "dvi" => Ok(BasicOp::DVI),
            "mod" => Ok(BasicOp::MOD),
            "mdi" => Ok(BasicOp::MDI),
            "and" => Ok(BasicOp::AND),
            "bor" => Ok(BasicOp::BOR),
            "xor" => Ok(BasicOp::XOR),
            "shr" => Ok(BasicOp::SHR),
            "asr" => Ok(BasicOp::ASR),
            "shl" => Ok(BasicOp::SHL),
            "ifb" => Ok(BasicOp::IFB),
            "ifc" => Ok(BasicOp::IFC),
            "ife" => Ok(BasicOp::IFE),
            "ifn" => Ok(BasicOp::IFN),
            "ifg" => Ok(BasicOp::IFG),
            "ifa" => Ok(BasicOp::IFA),
            "ifl" => Ok(BasicOp::IFL),
            "ifu" => Ok(BasicOp::IFU),
            "adx" => Ok(BasicOp::ADX),
            "sbx" => Ok(BasicOp::SBX),
            "sti" => Ok(BasicOp::STI),
            "std" => Ok(BasicOp::STD),
            _ => Err(())
        }
    }
}
