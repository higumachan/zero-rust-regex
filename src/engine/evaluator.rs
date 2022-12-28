use crate::engine::{Code, Instruction};
use std::collections::VecDeque;
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
                if let Some(sp_c) = line.get(sp) {
                    if *c == *sp_c {
                        pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                        sp = sp.checked_add(1).ok_or(EvalError::SPOverFlow)?;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Instruction::AnyChar => {
                if let Some(_) = line.get(sp) {
                    pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                    sp = sp.checked_add(1).ok_or(EvalError::SPOverFlow)?;
                } else {
                    return Ok(false);
                }
            }
            Instruction::IsHead => {
                if sp == 0 {
                    pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                } else {
                    return Ok(false);
                }
            }
            Instruction::IsTail => {
                if sp == line.len() {
                    pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
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
    pc: usize,
    sp: usize,
) -> Result<bool, EvalError> {
    let mut queue = VecDeque::new();
    queue.push_back((pc, sp));

    loop {
        if let Some((pc, sp)) = queue.pop_front() {
            let next = inst.get(pc).ok_or(EvalError::InvalidPC)?;

            match next {
                Instruction::Char(c) => {
                    if let Some(sp_c) = line.get(sp) {
                        if *c == *sp_c {
                            let next_pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                            let next_sp = sp.checked_add(1).ok_or(EvalError::SPOverFlow)?;

                            queue.push_back((next_pc, next_sp));
                        }
                    }
                }
                Instruction::AnyChar => {
                    if let Some(_) = line.get(sp) {
                        let next_pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                        let next_sp = sp.checked_add(1).ok_or(EvalError::SPOverFlow)?;

                        queue.push_back((next_pc, next_sp));
                    }
                }
                Instruction::IsHead => {
                    if sp == 0 {
                        let next_pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                        queue.push_back((next_pc, sp));
                    }
                }
                Instruction::IsTail => {
                    if sp == line.len() {
                        let next_pc = pc.checked_add(1).ok_or(EvalError::PCOverFlow)?;
                        queue.push_back((next_pc, sp));
                    }
                }
                Instruction::Match => return Ok(true),
                Instruction::Jump(i) => {
                    queue.push_back((*i, sp));
                }
                Instruction::Split(branch1, branch2) => {
                    queue.push_back((*branch1, sp));
                    queue.push_back((*branch2, sp));
                }
                Instruction::Nop => {
                    return Err(EvalError::AttemptNop);
                }
            }
        } else {
            return Ok(false);
        }
    }
}
