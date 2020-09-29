use crate::dcpu;

use std::str::FromStr;
use nom::{tag, map_res, named, alt, ws, char, preceded, delimited, take_while, separated_pair, tuple, switch, recognize, many0, complete};
use nom::{IResult};
use nom::combinator::{map_res};
use nom::branch::alt;
use nom::character::{is_digit};
use nom::character::complete::{multispace0};

named!(parse_basic_op<&str, dcpu::BasicOp>,
       map_res!(alt!(
           tag!("set") |
           tag!("add") |
           tag!("sub") |
           tag!("mul") |
           tag!("mli") |
           tag!("div") |
           tag!("dvi") |
           tag!("mod") |
           tag!("mdi") |
           tag!("and") |
           tag!("bor") |
           tag!("xor") |
           tag!("shr") |
           tag!("asr") |
           tag!("shl") |
           tag!("ifb") |
           tag!("ifc") |
           tag!("ife") |
           tag!("ifn") |
           tag!("ifg") |
           tag!("ifa") |
           tag!("ifl") |
           tag!("ifu") |
           tag!("adx") |
           tag!("sbx") |
           tag!("sti") |
           tag!("std")
       ), dcpu::BasicOp::from_str)
);

named!(parse_special_op<&str, dcpu::SpecialOp>,
       map_res!(alt!(
           tag!("jsr") |
           tag!("int") |
           tag!("iag") |
           tag!("ias") |
           tag!("rfi") |
           tag!("iaq") |
           tag!("hwn") |
           tag!("hwq") |
           tag!("hwi")
       ), dcpu::SpecialOp::from_str)
);

named!(parse_register<&str, dcpu::Register>,
       map_res!(alt!(
           tag!("a") |
           tag!("b") |
           tag!("c") |
           tag!("x") |
           tag!("y") |
           tag!("z") |
           tag!("i") |
           tag!("j")
       ), dcpu::Register::from_str)
);

named!(parse_value<&str, dcpu::Value>,
       alt!(
           parse_simple_value |
           map_res!(parse_register, wrap_reg) |
           map_res!(parse_number, wrap_next_word) |
           map_res!(delimited!(tuple!(char!('['), multispace0), parse_register, tuple!(multispace0, char!(']'))), wrap_deref_reg) |
           map_res!(delimited!(tuple!(char!('['), multispace0), separated_pair!(parse_register, tuple!(multispace0, char!('+'), multispace0), parse_number), tuple!(multispace0, char!(']'))), wrap_index_reg) |
           map_res!(delimited!(tuple!(char!('['), multispace0), parse_number, tuple!(multispace0, char!(']'))), wrap_deref_next_word)
       )
);

named!(parse_simple_value<&str, dcpu::Value>,
       map_res!(
           alt!(tag!("stack") |
                tag!("peek") |
                tag!("pick") |
                tag!("sp") |
                tag!("pc") |
                tag!("ex")), simple_value)
);

named!(parse_number<&str, u16>,
       map_res!(recognize!(nom::character::complete::digit1), u16::from_str)
);

fn wrap_next_word(num: u16) -> Result<dcpu::Value, ()> {
    Ok(dcpu::Value::NextWord(num))
}

fn wrap_deref_next_word(num: u16) -> Result<dcpu::Value, ()> {
    Ok(dcpu::Value::DerefNextWord(num))
}

fn wrap_index_reg(tuple: (dcpu::Register, u16)) -> Result<dcpu::Value, ()> {
    let (reg, index) = tuple;
    Ok(dcpu::Value::IndexReg(reg, index))
}

fn simple_value(s: &str) -> Result<dcpu::Value, ()> {
    match s {
        "stack" => Ok(dcpu::Value::STACK),
        "peek" => Ok(dcpu::Value::PEEK),
        "pick" => Ok(dcpu::Value::PICK),
        "sp" => Ok(dcpu::Value::SP),
        "pc" => Ok(dcpu::Value::PC),
        "ex" => Ok(dcpu::Value::EX),
        _ => Err(())
    }
}

fn wrap_reg(reg: dcpu::Register) -> Result<dcpu::Value, ()> {
    Ok(dcpu::Value::Reg(reg))
}

fn wrap_deref_reg(reg: dcpu::Register) -> Result<dcpu::Value, ()> {
    Ok(dcpu::Value::DerefReg(reg))
}

named!(parse_command<&str, dcpu::Command>,
       complete!(delimited!(multispace0, alt!(parse_basic_command | parse_special_command), tuple!(multispace0, char!(';'), multispace0)))
);

named!(parse_basic_command<&str, dcpu::Command>,
       map_res!(separated_pair!(parse_basic_op, multispace0, separated_pair!(parse_value, tuple!(multispace0, char!(','), multispace0), parse_value)), wrap_basic)
);

named!(parse_special_command<&str, dcpu::Command>,
       map_res!(separated_pair!(parse_special_op, multispace0, parse_value), wrap_special)
);

named!(pub parse_program<&str, Vec<dcpu::Command>>,
       many0!(parse_command)
);

fn wrap_basic(tuple: (dcpu::BasicOp, (dcpu::Value, dcpu::Value))) -> Result<dcpu::Command, ()> {
    let (op, (b, a)) = tuple;
    Ok(dcpu::Command::Basic{op, b, a})
}

fn wrap_special(tuple: (dcpu::SpecialOp, dcpu::Value)) -> Result<dcpu::Command, ()> {
    let (op, a) = tuple;
    Ok(dcpu::Command::Special{op, a})
}
