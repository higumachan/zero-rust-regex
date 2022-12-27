use std::fmt::Display;

mod codegen;
pub mod parser;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize),
    Nop,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Char(c) => write!(f, "char {} ", c),
            Instruction::Match => write!(f, "match"),
            Instruction::Jump(i) => write!(f, "jump {:>04}", i),
            Instruction::Split(i, j) => write!(f, "Split {:>04}, {:>04}", i, j),
            Instruction::Nop => write!(f, "nop"),
        }
    }
}
