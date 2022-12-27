use crate::engine::parser::AST;
use crate::engine::Instruction::{Char, Split};
use crate::engine::{Code, Instruction};
use std::fmt::Display;

pub fn get_code(ast: &AST) -> Result<Code, CodeGenError> {
    let mut generator = Generator::new();

    generator.gen_code(ast)?;

    Ok(Code(generator.instructions))
}

#[derive(Debug)]
pub enum CodeGenError {
    PcOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeGenError::PcOverFlow => write!(f, "pc overflow"),
            CodeGenError::FailStar => write!(f, "fail star"),
            CodeGenError::FailOr => write!(f, "fail or"),
            CodeGenError::FailQuestion => write!(f, "fail question"),
        }
    }
}

impl std::error::Error for CodeGenError {}

struct Generator {
    pc: usize,
    instructions: Vec<Instruction>,
}

impl Generator {
    fn new() -> Self {
        Self {
            pc: 0,
            instructions: vec![],
        }
    }

    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        self.pc = self
            .pc
            .checked_add(1)
            .ok_or_else(|| CodeGenError::PcOverFlow)?;

        Ok(())
    }

    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Or(ast1, ast2) => self.gen_or(ast1, ast2)?,
            AST::Plus(ast) => self.gen_plus(ast)?,
            AST::Star(ast) => self.gen_star(ast)?,
            AST::Question(ast) => self.gen_question(ast)?,
            AST::Seq(asts) => self.gen_seq(asts)?,
        }

        Ok(())
    }

    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        self.instructions.push(Instruction::Char(c));
        self.inc_pc()?;
        Ok(())
    }

    fn gen_or(&mut self, expr_left: &AST, expr_right: &AST) -> Result<(), CodeGenError> {
        self.instructions.push(Instruction::Nop);
        let split_inst_pc = self.pc;
        self.inc_pc()?;
        let split_branch1_pc = self.pc;
        self.gen_expr(expr_left)?;
        self.instructions.push(Instruction::Nop);
        let jump_pc = self.pc;
        self.inc_pc()?;
        let split_branch2_pc = self.pc;
        self.gen_expr(expr_right)?;

        self.instructions[split_inst_pc] = Instruction::Split(split_branch1_pc, split_branch2_pc);
        self.instructions[jump_pc] = Instruction::Jump(self.pc);

        Ok(())
    }

    fn gen_question(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        self.instructions.push(Instruction::Nop);
        let split_inst_pc = self.pc;
        self.inc_pc()?;
        let split_branch1_pc = self.pc;

        self.gen_expr(expr)?;

        self.instructions[split_inst_pc] = Instruction::Split(split_branch1_pc, self.pc);

        Ok(())
    }

    fn gen_plus(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        let jump_pc = self.pc;

        self.gen_expr(expr)?;
        let split_pc = self.pc;
        self.instructions.push(Instruction::Nop);
        self.inc_pc()?;

        self.instructions[split_pc] = Instruction::Split(jump_pc, self.pc);

        Ok(())
    }

    fn gen_star(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        let split_pc = self.pc;
        self.instructions.push(Instruction::Nop);
        self.inc_pc()?;

        let branch1_pc = self.pc;
        self.gen_expr(expr)?;
        self.instructions.push(Instruction::Jump(split_pc));
        self.inc_pc()?;

        self.instructions[split_pc] = Instruction::Split(branch1_pc, self.pc);

        Ok(())
    }

    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), CodeGenError> {
        for expr in exprs {
            self.gen_expr(expr)?;
        }

        Ok(())
    }

    fn gen_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        self.instructions.push(Instruction::Match);
        self.inc_pc()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Instruction::Match;

    #[test]
    fn simple_code_gen() {
        let ast = AST::Seq(vec![AST::Char('a'), AST::Char('b')]);
        let mut gen = Generator::new();
        gen.gen_code(&ast).unwrap();
        assert_eq!(gen.instructions, vec![Char('a'), Char('b'), Match]);
    }
}
