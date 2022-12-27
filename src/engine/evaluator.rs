use crate::engine::{Code, Instruction};
use std::fmt::Display;

pub fn eval(code: &Code, line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(code.instractions(), line, 0, 0)
    } else {
        eval_width(code.instractions(), line, 0, 0)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    InvalidPC,
    InvalidContext,
    AttemptNop,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::PCOverFlow => write!(f, "pc overflow"),
            EvalError::SPOverFlow => write!(f, "sp overflow"),
            EvalError::InvalidPC => write!(f, "invalid pc"),
            EvalError::InvalidContext => write!(f, "invalid context"),
            EvalError::AttemptNop => write!(f, "attempt nop"),
        }
    }
}

impl std::error::Error for EvalError {}

fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    loop {
        let next = inst.get(pc).ok_or(EvalError::InvalidPC)?;

        match next {
            Instruction::Char(c) => {
                let sp_c = line.get(sp).ok_or(EvalError::SPOverFlow)?;
                if *c == *sp_c {
                    pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                    sp = sp.checked_add(1).ok_or(EvalError::SPOverFlow)?;
                } else {
                    return Ok(false);
                }
            }
            Instruction::Match => return Ok(true),
            Instruction::Jump(i) => {
                pc = *i;
            }
            Instruction::Split(branch1, branch2) => {
                return if eval_depth(inst, line, *branch1, sp)? {
                    Ok(true)
                } else {
                    eval_depth(inst, line, *branch2, sp)
                }
            }
            Instruction::Nop => {
                return Err(EvalError::AttemptNop);
            }
        }
    }
}

#[allow(unused_variables, unused_mut)]
fn eval_width(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    todo!()
}
