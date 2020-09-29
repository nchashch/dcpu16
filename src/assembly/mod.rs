extern crate nom;
mod parser;

use crate::dcpu;
use parser::*;

pub fn parse(s: &str) -> Option<Vec<dcpu::Command>> {
    match parse_program(s) {
        Ok((_, program)) => Some(program),
        _ => None
    }
}

pub fn generate_code(program: Vec<dcpu::Command>) -> Vec<u16> {
    let mut result = vec![];
    for command in program {
        result.push(command.code());
        match command {
            dcpu::Command::Special { op: _, a } => {
                if let Some(word) = dcpu::get_next_word(&a) {
                    result.push(word);
                };
            },
            dcpu::Command::Basic { op: _, b, a } => {
                if let Some(word) = dcpu::get_next_word(&a) {
                    result.push(word);
                };
                if let Some(word) = dcpu::get_next_word(&b) {
                    result.push(word);
                };
            }
        }
    }
    result
}
