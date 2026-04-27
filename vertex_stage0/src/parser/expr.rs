use crate::ast::expr::{BoolLit, CharLit, Expr, FloatLit, IntLit, StrLit, TupleLit};
use crate::error::{CompileError, ErrorCode, ErrorKind};
use crate::lexer::token::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub fn parse_int_lit(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::IntLiteral(_, _)) {
            return Err(self.unexpected_token_error("integer literal"));
        }
        let tok = self.bump();
        let span = tok.span;
        let (value, suffix) = match tok.kind {
            TokenKind::IntLiteral(v, s) => (v, s),
            _ => unreachable!(),
        };
        let id = self.new_node_id();
        Ok(Expr::IntLit(IntLit {
            id,
            span,
            value,
            suffix,
        }))
    }

    pub fn parse_float_lit(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::FloatLiteral(_, _)) {
            return Err(self.unexpected_token_error("float literal"));
        }
        let tok = self.bump();
        let span = tok.span;
        let (value, suffix) = match tok.kind {
            TokenKind::FloatLiteral(v, s) => (v, s),
            _ => unreachable!(),
        };
        let id = self.new_node_id();
        Ok(Expr::FloatLit(FloatLit {
            id,
            span,
            value,
            suffix,
        }))
    }

    pub fn parse_char_lit(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::CharLiteral(_)) {
            return Err(self.unexpected_token_error("char literal"));
        }
        let tok = self.bump();
        let span = tok.span;
        let value = match tok.kind {
            TokenKind::CharLiteral(c) => c,
            _ => unreachable!(),
        };
        let id = self.new_node_id();
        Ok(Expr::CharLit(CharLit { id, span, value }))
    }

    pub fn parse_str_lit(&mut self) -> Result<Expr, CompileError> {
        if !matches!(
            self.peek(),
            TokenKind::StringLiteral(_) | TokenKind::RawStringLiteral(_)
        ) {
            return Err(self.unexpected_token_error("string literal"));
        }
        let tok = self.bump();
        let span = tok.span;
        let value = match tok.kind {
            TokenKind::StringLiteral(s) | TokenKind::RawStringLiteral(s) => s,
            _ => unreachable!(),
        };
        let id = self.new_node_id();
        Ok(Expr::StrLit(StrLit { id, span, value }))
    }

    pub fn parse_bool_lit(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::True | TokenKind::False) {
            return Err(self.unexpected_token_error("`true` or `false`"));
        }
        let tok = self.bump();
        let span = tok.span;
        let value = matches!(tok.kind, TokenKind::True);
        let id = self.new_node_id();
        Ok(Expr::BoolLit(BoolLit { id, span, value }))
    }

    pub fn parse_paren_or_tuple(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::LParen) {
            return Err(self.unexpected_token_error("`(`"));
        }
        let lparen_tok = self.bump();
        let lparen_span = lparen_tok.span;

        if matches!(self.peek(), TokenKind::RParen) {
            let rparen_tok = self.bump();
            let rparen_span = rparen_tok.span;
            let id = self.new_node_id();
            return Ok(Expr::TupleLit(TupleLit {
                id,
                span: lparen_span.merge(&rparen_span),
                elems: vec![],
            }));
        }

        let first = self.parse_primary_for_paren()?;

        match self.peek() {
            TokenKind::RParen => {
                self.bump();
                Ok(first)
            }
            TokenKind::Comma => {
                self.bump();
                let mut elems = vec![first];
                while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                    let elem = self.parse_primary_for_paren()?;
                    elems.push(elem);
                    if matches!(self.peek(), TokenKind::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                let rparen_tok = self.expect(&TokenKind::RParen)?;
                let rparen_span = rparen_tok.span;
                let id = self.new_node_id();
                Ok(Expr::TupleLit(TupleLit {
                    id,
                    span: lparen_span.merge(&rparen_span),
                    elems,
                }))
            }
            _ => Err(self.unexpected_token_error("`,` or `)`")),
        }
    }

    // Temporary stub: only handles literal heads. Will be replaced by
    // `parse_primary` in item 49 once the Pratt driver lands.
    fn parse_primary_for_paren(&mut self) -> Result<Expr, CompileError> {
        match self.peek() {
            TokenKind::IntLiteral(_, _) => self.parse_int_lit(),
            TokenKind::FloatLiteral(_, _) => self.parse_float_lit(),
            TokenKind::CharLiteral(_) => self.parse_char_lit(),
            TokenKind::StringLiteral(_) | TokenKind::RawStringLiteral(_) => self.parse_str_lit(),
            TokenKind::True | TokenKind::False => self.parse_bool_lit(),
            _ => Err(self.unexpected_token_error("expression")),
        }
    }

    fn unexpected_token_error(&self, expected: &str) -> CompileError {
        let span = if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            crate::span::Span::new(crate::span::FileId(0), 0, 0)
        };
        let message = format!(
            "expected {}, found {}",
            expected,
            describe_kind(self.peek())
        );
        CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, span, message)
    }
}

fn describe_kind(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::IntLiteral(_, _) => "integer literal",
        TokenKind::FloatLiteral(_, _) => "float literal",
        TokenKind::CharLiteral(_) => "char literal",
        TokenKind::StringLiteral(_) => "string literal",
        TokenKind::RawStringLiteral(_) => "raw string literal",
        TokenKind::True => "`true`",
        TokenKind::False => "`false`",
        TokenKind::Ident(_) => "identifier",
        TokenKind::Eof => "end of file",
        _ => "token",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expr;
    use crate::lexer::token::{FloatSuffix, IntSuffix, Token};
    use crate::span::{FileId, Span};

    fn tok(kind: TokenKind) -> Token {
        Token::new(kind, Span::new(FileId(0), 0, 0))
    }

    #[test]
    fn literal_expressions() {
        // parse_int_lit
        let mut p = Parser::new(vec![
            tok(TokenKind::IntLiteral(42, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_int_lit() {
            Ok(Expr::IntLit(lit)) => {
                assert_eq!(lit.value, 42);
                assert_eq!(lit.suffix, IntSuffix::I32);
            }
            other => panic!("expected Ok(IntLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_float_lit
        let mut p = Parser::new(vec![
            tok(TokenKind::FloatLiteral(1.5, FloatSuffix::F64)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_float_lit() {
            Ok(Expr::FloatLit(lit)) => {
                assert_eq!(lit.value, 1.5);
                assert_eq!(lit.suffix, FloatSuffix::F64);
            }
            other => panic!("expected Ok(FloatLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_char_lit
        let mut p = Parser::new(vec![tok(TokenKind::CharLiteral('z')), tok(TokenKind::Eof)]);
        match p.parse_char_lit() {
            Ok(Expr::CharLit(lit)) => {
                assert_eq!(lit.value, 'z');
            }
            other => panic!("expected Ok(CharLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_str_lit (regular)
        let mut p = Parser::new(vec![
            tok(TokenKind::StringLiteral("hello".to_string())),
            tok(TokenKind::Eof),
        ]);
        match p.parse_str_lit() {
            Ok(Expr::StrLit(lit)) => {
                assert_eq!(lit.value, "hello");
            }
            other => panic!("expected Ok(StrLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_str_lit (raw)
        let mut p = Parser::new(vec![
            tok(TokenKind::RawStringLiteral("raw".to_string())),
            tok(TokenKind::Eof),
        ]);
        match p.parse_str_lit() {
            Ok(Expr::StrLit(lit)) => {
                assert_eq!(lit.value, "raw");
            }
            other => panic!("expected Ok(StrLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_bool_lit (true)
        let mut p = Parser::new(vec![tok(TokenKind::True), tok(TokenKind::Eof)]);
        match p.parse_bool_lit() {
            Ok(Expr::BoolLit(lit)) => {
                assert!(lit.value);
            }
            other => panic!("expected Ok(BoolLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // parse_bool_lit (false)
        let mut p = Parser::new(vec![tok(TokenKind::False), tok(TokenKind::Eof)]);
        match p.parse_bool_lit() {
            Ok(Expr::BoolLit(lit)) => {
                assert!(!lit.value);
            }
            other => panic!("expected Ok(BoolLit), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // wrong token: parse_int_lit on Plus returns Err and does not advance
        let mut p = Parser::new(vec![tok(TokenKind::Plus), tok(TokenKind::Eof)]);
        assert!(p.parse_int_lit().is_err());
        assert_eq!(p.pos, 0);
        assert!(p.errors.is_empty());
    }

    #[test]
    fn paren_tuple_unit() {
        // ()
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_paren_or_tuple() {
            Ok(Expr::TupleLit(t)) => assert_eq!(t.elems.len(), 0),
            other => panic!("expected Ok(TupleLit) for `()`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // ( 1i32 ) → unwrapped IntLit
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_paren_or_tuple() {
            Ok(Expr::IntLit(lit)) => {
                assert_eq!(lit.value, 1);
                assert_eq!(lit.suffix, IntSuffix::I32);
            }
            other => panic!("expected Ok(IntLit) for `(1i32)`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // ( 1i32 , ) → 1-tuple
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Comma),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_paren_or_tuple() {
            Ok(Expr::TupleLit(t)) => {
                assert_eq!(t.elems.len(), 1);
                match &t.elems[0] {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit element, got {:?}", other),
                }
            }
            other => panic!("expected Ok(TupleLit) for `(1i32,)`, got {:?}", other),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // ( 1i32 , 2i32 )
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Comma),
            tok(TokenKind::IntLiteral(2, IntSuffix::I32)),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_paren_or_tuple() {
            Ok(Expr::TupleLit(t)) => assert_eq!(t.elems.len(), 2),
            other => panic!("expected Ok(TupleLit) for `(1, 2)`, got {:?}", other),
        }
        assert_eq!(p.pos, 5);
        assert!(p.errors.is_empty());

        // ( 1i32 , 2i32 , ) — trailing comma tolerated
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Comma),
            tok(TokenKind::IntLiteral(2, IntSuffix::I32)),
            tok(TokenKind::Comma),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_paren_or_tuple() {
            Ok(Expr::TupleLit(t)) => assert_eq!(t.elems.len(), 2),
            other => panic!("expected Ok(TupleLit) for `(1, 2,)`, got {:?}", other),
        }
        assert_eq!(p.pos, 6);
        assert!(p.errors.is_empty());

        // wrong head: `+` followed by `Eof`
        let mut p = Parser::new(vec![tok(TokenKind::Plus), tok(TokenKind::Eof)]);
        assert!(p.parse_paren_or_tuple().is_err());
        assert_eq!(p.pos, 0);
        assert!(p.errors.is_empty());
    }
}
