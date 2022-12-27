use crate::engine::evaluator::eval;
use crate::engine::{codegen, parser};
use crate::helper::DynError;

mod engine;
mod helper;

pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool, DynError> {
    let ast = parser::parse(expr)?;
    let code = codegen::get_code(&ast)?;

    let line: Vec<char> = line.chars().collect();

    Ok(eval(&code, &line, is_depth)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(do_matching("a|b", "a", true).unwrap(), true);
        assert_eq!(do_matching("a|b", "b", true).unwrap(), true);
        assert_eq!(do_matching("a|b", "c", true).unwrap(), false);
        assert_eq!(do_matching("a|b|c", "c", true).unwrap(), true);
    }

    #[test]
    fn test_matching() {
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());

        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acd", true).unwrap());
    }

    #[test]
    fn fail_case_001() {
        assert!(do_matching("abc?", "ab", true).unwrap());
    }
}
