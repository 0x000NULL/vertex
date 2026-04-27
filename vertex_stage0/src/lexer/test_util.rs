// Drives the scanner over `$input` and asserts the resulting `TokenKind` sequence
// (with spans dropped, and trailing `Eof` excluded) equals `$expected`.
#[macro_export]
macro_rules! lex_eq {
    ($input:expr, $expected:expr) => {{
        let input: &str = $input;
        let mut scanner = $crate::lexer::scan::Scanner::new(input, $crate::span::FileId(0));
        let mut kinds: ::std::vec::Vec<$crate::lexer::token::TokenKind> = ::std::vec::Vec::new();
        loop {
            let tok = scanner.next_token();
            if matches!(tok.kind, $crate::lexer::token::TokenKind::Eof) {
                break;
            }
            kinds.push(tok.kind);
        }
        assert_eq!(
            kinds, $expected,
            "lex_eq! token sequence mismatch for input {:?}",
            input
        );
    }};
}

#[cfg(test)]
mod tests {
    use crate::lexer::token::TokenKind;

    #[test]
    fn macro_works() {
        lex_eq!("let x", vec![TokenKind::Let, TokenKind::Ident("x".into())]);
    }
}
