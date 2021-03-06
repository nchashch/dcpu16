use crate::dcpu::{Value, BasicOp, SpecialOp};

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

            let op = match SpecialOp::new(special_op_code) {
                Some(op) => op,
                None => return None
            };
            let a = match Value::new(a_code) {
                Some(a) => a,
                None => return None
            };

            Some(Command::Special{
                op,
                a
            })
        } else {
            let b_code = (val & B_MASK) >> B_SHIFT;
            let a_code = (val & A_MASK) >> A_SHIFT;

            let op = match BasicOp::new(op_code) {
                Some(op) => op,
                None => return None
            };
            let b = match Value::new(b_code) {
                Some(b) => b,
                None => return None
            };
            let a = match Value::new(a_code) {
                Some(a) => a,
                None => return None
            };

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
                dbg!(op_code, a_code, special_op_code);
                (a_code << A_SHIFT) | (special_op_code << B_SHIFT) | op_code
            }
        }
    }

    pub fn get_size(&self) -> u16 {
        let command_size = 1;
        let next_words_size = match self {
            Command::Special { op: _, a } => {
                match get_next_word(a) {
                    Some(_) => 1,
                    None => 0
                }
            },
            Command::Basic { op: _, b, a } => {
                let b_size = match get_next_word(b) {
                    Some(_) => 1,
                    None => 0
                };
                let a_size = match get_next_word(a) {
                    Some(_) => 1,
                    None => 0
                };
                b_size + a_size
            }
        };
        command_size + next_words_size
    }
}

pub fn get_next_word(value: &Value) -> Option<u16> {
    match value {
        Value::IndexReg(_, word) => Some(*word),
        Value::DerefNextWord(word) => Some(*word),
        Value::NextWord(word) => Some(*word),
        _ => None
    }
}
