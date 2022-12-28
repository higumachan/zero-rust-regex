use crate::engine::parser::AST;
use crate::engine::{Code, Instruction};
use std::fmt::Display;

pub fn get_code(ast: &AST) -> Result<Code, CodeGenError> {
    let mut generator = Generator::new();

    generator.gen_code(ast)?;

    Ok(Code(generator.instructions))
}

#[derive(Debug)]
#[allow(dead_code)]
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
        self.pc = self.pc.checked_add(1).ok_or(CodeGenError::PcOverFlow)?;

        Ok(())
    }

    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::AnyChar => self.gen_anychar()?,
            AST::Dollar(ast) => self.gen_dollar(ast)?,
            AST::Hat(ast) => self.gen_hat(ast)?,
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

    fn gen_anychar(&mut self) -> Result<(), CodeGenError> {
        self.instructions.push(Instruction::AnyChar);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_hat(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        self.instructions.push(Instruction::IsHead);
        self.inc_pc()?;
        self.gen_expr(expr)?;
        Ok(())
    }

    fn gen_dollar(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(expr)?;
        self.instructions.push(Instruction::IsTail);
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

    fn gen_skip_head(&mut self) -> Result<(), CodeGenError> {
        assert_eq!(self.pc, 0);
        self.instructions.push(Instruction::Split(3, 1));
        self.inc_pc()?;
        self.instructions.push(Instruction::AnyChar);
        self.inc_pc()?;
        self.instructions.push(Instruction::Jump(0));
        self.inc_pc()?;
        self.instructions.push(Instruction::Start);
        self.inc_pc()?;

        Ok(())
    }

    fn gen_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_skip_head()?;
        self.gen_expr(ast)?;
        self.instructions.push(Instruction::Match);
        self.inc_pc()?;

        Ok(())
    }
}
