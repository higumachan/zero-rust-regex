use crate::engine::parser::ParserError::InvalidEscape;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::take;

#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum AST {
    Char(char),
    AnyChar,
    Dollar(Box<AST>),
    Hat(Box<AST>),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

#[derive(Debug)]
pub enum ParserError {
    InvalidEscape(usize, char),
    InvalidHat,
    InvalidDollar,
    NoPrev(usize),
    NoRightParen,
    Empty,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParserError {}

fn parse_escape(pos: usize, c: char) -> Result<AST, ParserError> {
    match c {
        '\\' | '(' | ')' | '|' | '*' | '+' | '?' | '.' | '^' | '$' => Ok(AST::Char(c)),
        _ => Err(InvalidEscape(pos, c)),
    }
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum PSQ {
    Plus,
    Star,
    Question,
}

fn parse_plus_star_question(
    seq: &mut Vec<AST>,
    ast_type: PSQ,
    pos: usize,
) -> Result<(), ParserError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };

        seq.push(ast);
        Ok(())
    } else {
        Err(ParserError::NoPrev(pos))
    }
}

fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();

        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }

        Some(ast)
    } else {
        seq_or.pop()
    }
}

pub fn parse(expr: &str) -> Result<AST, ParserError> {
    enum ParseState {
        Char,
        Escape,
    }

    #[derive(Default)]
    struct Context {
        seq_quantifier: Vec<AST>,
        seq_or: Vec<AST>,
    }

    let mut is_hat = false;
    let mut is_dollar = false;
    let mut context = Context::default();
    let mut stack = vec![];
    let mut state = ParseState::Char;

    for (i, c) in expr.chars().enumerate() {
        match state {
            ParseState::Char => match c {
                '+' => parse_plus_star_question(&mut context.seq_quantifier, PSQ::Plus, i)?,
                '*' => parse_plus_star_question(&mut context.seq_quantifier, PSQ::Star, i)?,
                '?' => parse_plus_star_question(&mut context.seq_quantifier, PSQ::Question, i)?,
                '(' => {
                    let prev = take(&mut context);
                    stack.push(prev);
                }
                ')' => {
                    if let Some(mut prev) = stack.pop() {
                        if !context.seq_quantifier.is_empty() {
                            context.seq_or.push(AST::Seq(context.seq_quantifier))
                        }

                        if let Some(ast) = fold_or(context.seq_or) {
                            prev.seq_quantifier.push(ast);
                        }

                        context = prev;
                    }
                }
                '|' => {
                    if context.seq_quantifier.is_empty() {
                        return Err(ParserError::NoPrev(i));
                    }
                    let prev_quantifier = take(&mut context.seq_quantifier);
                    context.seq_or.push(AST::Seq(prev_quantifier));
                }
                '.' => context.seq_quantifier.push(AST::AnyChar),
                '^' => {
                    if i > 0 {
                        return Err(ParserError::InvalidHat);
                    } else {
                        is_hat = true;
                    }
                }
                '$' => {
                    if i < expr.len() - 1 {
                        return Err(ParserError::InvalidDollar);
                    } else {
                        is_dollar = true;
                    }
                }
                '\\' => {
                    state = ParseState::Escape;
                }
                _ => context.seq_quantifier.push(AST::Char(c)),
            },
            _ => {
                let ast = parse_escape(i, c)?;
                context.seq_quantifier.push(ast);
                state = ParseState::Char;
            }
        }
    }

    if !stack.is_empty() {
        return Err(ParserError::NoRightParen);
    }

    if !context.seq_quantifier.is_empty() {
        context.seq_or.push(AST::Seq(context.seq_quantifier));
    }

    if let Some(ast) = fold_or(context.seq_or) {
        let ast = if is_hat { AST::Hat(Box::new(ast)) } else { ast };
        let ast = if is_dollar {
            AST::Dollar(Box::new(ast))
        } else {
            ast
        };
        Ok(ast)
    } else {
        Err(ParserError::Empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_chars() {
        let expr = "abcd";

        assert_eq!(
            parse(expr).unwrap(),
            AST::Seq(vec![
                AST::Char('a'),
                AST::Char('b'),
                AST::Char('c'),
                AST::Char('d')
            ])
        );
    }

    #[test]
    fn or_case() {
        let expr = "a|b|cd";

        assert_eq!(
            parse(expr).unwrap(),
            AST::Or(
                Box::new(AST::Seq(vec![AST::Char('a')])),
                Box::new(AST::Or(
                    Box::new(AST::Seq(vec![AST::Char('b')])),
                    Box::new(AST::Seq(vec![AST::Char('c'), AST::Char('d')]))
                ))
            )
        );
    }

    #[test]
    fn hat_dollar_case() {
        let expr = "^a";

        assert_eq!(
            parse("^a").unwrap(),
            AST::Hat(Box::new(AST::Seq(vec![AST::Char('a')]))),
        );
        assert_eq!(
            parse("a$").unwrap(),
            AST::Dollar(Box::new(AST::Seq(vec![AST::Char('a')]))),
        );
        assert_eq!(
            parse("^a$").unwrap(),
            AST::Dollar(Box::new(AST::Hat(Box::new(AST::Seq(vec![AST::Char('a')]))))),
        );
    }
}
