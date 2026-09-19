mod tests {
    use crate::frontend::parser::*;

    #[test]
    fn test_top_level_parser() {
        let result = create_top_level_parser().parse("def foo(x, y) x + y").into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_def_parser() {
        let result = create_def_parser().parse("def foo(x, y) x + y").into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_def() {
        let result = create_top_level_parser()
            .parse(" def    foo(  x  ,  y  )    x   +   y  ")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_extern() {
        let result = create_top_level_parser().parse(" extern   sin ( a )  ").into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_expr() {
        let result = create_top_level_parser().parse(" 1 +  (  2    * 3   )  ").into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    fn fast_parses(input: &str) -> bool { create_top_level_parser().parse(input).into_result().is_ok() }

    #[test]
    fn test_valid_inputs() {
        assert!(fast_parses("def foo(x, y) x + y"));
        assert!(fast_parses("extern sin(a)"));
        assert!(fast_parses("1 + 2 * 3"));
        assert!(fast_parses("foo(1, 2, 3)"));
        assert!(fast_parses("(1 + 2) * 3"));
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(!fast_parses("def foo(x"));
        assert!(!fast_parses("1 +"));
        assert!(!fast_parses(")"));
        assert!(!fast_parses("def"));
    }

    #[test]
    fn test_whitespace_variants() {
        assert!(fast_parses("def    foo(  x  ,  y  )    x   +   y"));
        assert!(fast_parses("1+2"));
        assert!(fast_parses("  1  +  2  "));
    }

    /// lightweight fuzz
    #[test]
    fn test_no_panic_on_garbage() {
        let garbage = [
            "",
            " ",
            "\n",
            "?",
            "@#$",
            "def def",
            "((((",
            "))))",
            "1 + + 2",
            "foo()",
            "def foo()",
            "extern",
            "\0\0\0",
            "💥",
            "def foo(x) x + ",
            "4.",
            "4.5.6",
            ".5",
            "99999999999999999999999999999999999",
        ];

        for input in garbage {
            let _ = create_top_level_parser().parse(input).into_result();
        }
    }

    #[test]
    fn test_keywords_not_identifiers() {
        assert!(!fast_parses("def"));
        assert!(!fast_parses("extern"));
        assert!(!fast_parses("def(x) x"));
        assert!(!fast_parses("def def(x) x"));
        assert!(fast_parses("def foo(x) x"));
        assert!(!fast_parses("def foo(def) def"));
    }
}
