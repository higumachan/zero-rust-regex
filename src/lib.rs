use crate::engine::evaluator::{eval, eval_with_pattern};
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

pub fn do_matching_with_pattern(
    expr: &str,
    line: &str,
    is_depth: bool,
) -> Result<Option<Vec<char>>, DynError> {
    let ast = parser::parse(expr)?;
    let code = codegen::get_code(&ast)?;

    let line: Vec<char> = line.chars().collect();

    let pattern_range = eval_with_pattern(&code, &line, is_depth)?;

    Ok(pattern_range.map(move |r| line[r].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("a|b", "a", true)]
    #[case("a|b", "b", true)]
    #[case("a|b", "c", false)]
    #[case("a|b|c", "c", true)]
    #[case(".", "c", true)]
    #[case(".d", "cd", true)]
    fn it_works(
        #[case] expr: &str,
        #[case] line: &str,
        #[case] expect: bool,
        #[values(true, false)] is_depth: bool,
    ) {
        assert_eq!(do_matching(expr, line, is_depth).unwrap(), expect);
    }

    #[rstest]
    #[case("abc|def", "def", true)]
    #[case("(ab|cd)+", "abcdcd", true)]
    #[case("abc?", "ab", true)]
    #[case("^abc", "abcdef", true)]
    #[case("def$", "abcdef", true)]
    fn matched_case(
        #[case] expr: &str,
        #[case] line: &str,
        #[case] expect: bool,
        #[values(true, false)] is_depth: bool,
    ) {
        assert!(do_matching(expr, line, is_depth).unwrap());
    }

    #[rstest]
    #[case("abc|def", "efa", true)]
    #[case("(ab|cd)+", "", true)]
    #[case("abc?", "acd", true)]
    #[case("abc$", "abcdef", true)]
    #[case("^def", "abcdef", true)]
    fn unmatched_case(
        #[case] expr: &str,
        #[case] line: &str,
        #[case] expect: bool,
        #[values(true, false)] is_depth: bool,
    ) {
        assert!(!do_matching(expr, line, is_depth).unwrap());
    }

    #[rstest]
    #[case("*b", "bbb", true)]
    #[case("+b", "bbb", true)]
    #[case("|b", "bbb", true)]
    #[case("?b", "bbb", true)]
    fn parse_error_case(#[case] expr: &str, #[case] line: &str, #[case] expect: bool) {
        assert!(do_matching(expr, line, true).is_err());
    }

    #[test]
    fn test_matching() {
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());
    }

    #[test]
    fn fail_case_001() {
        assert!(do_matching("abc?", "ab", true).unwrap());
    }

    #[test]
    fn fail_case_002() {
        assert!(do_matching("def$", "abcdef", true).unwrap());
    }

    #[test]
    fn benchmark_small_case() {
        let expr = "a?a?aa";
        let ast = parser::parse(expr).unwrap();
        let code = codegen::get_code(&ast).unwrap();
        println!("{}", code);
        assert!(do_matching(expr, "aa", true).unwrap());
    }

    #[test]
    fn benchmark_big_case() {
        let expr = "a?a?a?a?a?a?a?a?a?a?aaaaaaaaaa";
        let ast = parser::parse(expr).unwrap();
        let code = codegen::get_code(&ast).unwrap();
        println!("{}", code);
        assert!(do_matching(expr, "aaaaaaaaaa", false).unwrap());
    }

    #[test]
    fn with_pattern() {
        assert_eq!(
            do_matching_with_pattern("ab.", "aabcabd", true).unwrap(),
            Some(vec!['a', 'b', 'c'])
        );

        let expr = "(ab)*";
        let ast = parser::parse(expr).unwrap();
        let code = codegen::get_code(&ast).unwrap();
        println!("{}", code);
        assert_eq!(
            do_matching_with_pattern("(ab)*", "cababc", true).unwrap(),
            Some(vec!['a', 'b', 'a', 'b'])
        );

        assert_eq!(
            do_matching_with_pattern("(ab)*c?", "cababc", true).unwrap(),
            Some(vec!['a', 'b', 'a', 'b', 'c'])
        );
    }
}
