use std::fmt::{Display, Formatter};

pub(crate) mod codegen;
pub(crate) mod evaluator;
pub(crate) mod parser;

#[derive(Debug, PartialEq)]
pub struct Code(Vec<Instruction>);

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, inst) in self.0.iter().enumerate() {
            writeln!(f, "{}: {}", i, inst)?;
        }
        Ok(())
    }
}

impl Code {
    pub fn instractions(&self) -> &Vec<Instruction> {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Char(char),
    AnyChar,
    Start,
    IsHead,
    IsTail,
    Match,
    Jump(usize),
    Split(usize, usize),
    Nop,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Char(c) => write!(f, "Char {} ", c),
            Instruction::AnyChar => write!(f, "Anychar"),
            Instruction::Start => write!(f, "Start"),
            Instruction::IsHead => write!(f, "Ishead"),
            Instruction::IsTail => write!(f, "Istail"),
            Instruction::Match => write!(f, "Match"),
            Instruction::Jump(i) => write!(f, "Jump {:>04}", i),
            Instruction::Split(i, j) => write!(f, "Split {:>04}, {:>04}", i, j),
            Instruction::Nop => write!(f, "nop"),
        }
    }
}
