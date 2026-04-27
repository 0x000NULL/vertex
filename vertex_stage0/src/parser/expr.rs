use crate::ast::expr::{
    Binary, BinaryOp, BoolLit, CharLit, Expr, FloatLit, IntLit, StrLit, TupleLit, Unary, UnaryOp,
};
use crate::error::{CompileError, ErrorCode, ErrorKind};
use crate::lexer::token::TokenKind;
use crate::parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpClass {
    Comparison,
    Assignment,
    Other,
}

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

    pub fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_bp: u8) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_unary()?;

        while let Some((left_bp, right_bp, op, class)) = infix_binding_power(self.peek()) {
            if left_bp < min_bp {
                break;
            }

            // Non-associative comparisons: reject `a < b < c` style chains.
            // NOTE: `parse_paren_or_tuple` currently unwraps `(expr)` to the inner
            // `Expr`, so a parenthesized comparison still surfaces as `Expr::Binary`
            // and would also trip this check. TODO: revisit if real code hits this
            // (e.g. add a `Paren` wrapper or track a "made-at-this-level" flag).
            if class == OpClass::Comparison {
                if let Expr::Binary(b) = &lhs {
                    if is_comparison_op(b.op) {
                        let op_span = if self.pos < self.tokens.len() {
                            self.tokens[self.pos].span
                        } else if let Some(last) = self.tokens.last() {
                            last.span
                        } else {
                            crate::span::Span::new(crate::span::FileId(0), 0, 0)
                        };
                        return Err(CompileError::new(
                            ErrorCode::E0100,
                            ErrorKind::Syntax,
                            op_span,
                            "chained comparison operators require parentheses",
                        ));
                    }
                }
            }

            self.bump();
            let rhs = self.parse_binary(right_bp)?;
            let span = lhs.span().merge(&rhs.span());
            let id = self.new_node_id();
            lhs = Expr::Binary(Binary {
                id,
                span,
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            });
        }

        Ok(lhs)
    }

    pub fn parse_unary(&mut self) -> Result<Expr, CompileError> {
        let op_span = if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            crate::span::Span::new(crate::span::FileId(0), 0, 0)
        };
        let op = match self.peek() {
            TokenKind::Minus => {
                self.bump();
                UnaryOp::Neg
            }
            TokenKind::Not => {
                self.bump();
                UnaryOp::Not
            }
            TokenKind::Star => {
                self.bump();
                UnaryOp::Deref
            }
            TokenKind::Amp => {
                self.bump();
                if matches!(self.peek(), TokenKind::Mut) {
                    self.bump();
                    UnaryOp::RefMut
                } else {
                    UnaryOp::Ref
                }
            }
            _ => return self.parse_primary_for_paren(),
        };
        let operand = self.parse_unary()?;
        let span = op_span.merge(&operand.span());
        let id = self.new_node_id();
        Ok(Expr::Unary(Unary {
            id,
            span,
            op,
            operand: Box::new(operand),
        }))
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

fn is_comparison_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge
    )
}

fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8, BinaryOp, OpClass)> {
    let result = match kind {
        TokenKind::Eq => (2, 1, BinaryOp::Assign, OpClass::Assignment),
        TokenKind::PlusEq => (2, 1, BinaryOp::AddAssign, OpClass::Assignment),
        TokenKind::MinusEq => (2, 1, BinaryOp::SubAssign, OpClass::Assignment),
        TokenKind::StarEq => (2, 1, BinaryOp::MulAssign, OpClass::Assignment),
        TokenKind::SlashEq => (2, 1, BinaryOp::DivAssign, OpClass::Assignment),
        TokenKind::PercentEq => (2, 1, BinaryOp::RemAssign, OpClass::Assignment),
        TokenKind::Or => (3, 4, BinaryOp::Or, OpClass::Other),
        TokenKind::And => (5, 6, BinaryOp::And, OpClass::Other),
        TokenKind::EqEq => (7, 8, BinaryOp::Eq, OpClass::Comparison),
        TokenKind::BangEq => (7, 8, BinaryOp::Ne, OpClass::Comparison),
        TokenKind::Lt => (7, 8, BinaryOp::Lt, OpClass::Comparison),
        TokenKind::Gt => (7, 8, BinaryOp::Gt, OpClass::Comparison),
        TokenKind::Le => (7, 8, BinaryOp::Le, OpClass::Comparison),
        TokenKind::Ge => (7, 8, BinaryOp::Ge, OpClass::Comparison),
        TokenKind::Pipe => (9, 10, BinaryOp::BitOr, OpClass::Other),
        TokenKind::Caret => (11, 12, BinaryOp::BitXor, OpClass::Other),
        TokenKind::Amp => (13, 14, BinaryOp::BitAnd, OpClass::Other),
        TokenKind::Shl => (15, 16, BinaryOp::Shl, OpClass::Other),
        TokenKind::Shr => (15, 16, BinaryOp::Shr, OpClass::Other),
        TokenKind::Plus => (17, 18, BinaryOp::Add, OpClass::Other),
        TokenKind::Minus => (17, 18, BinaryOp::Sub, OpClass::Other),
        TokenKind::Star => (19, 20, BinaryOp::Mul, OpClass::Other),
        TokenKind::Slash => (19, 20, BinaryOp::Div, OpClass::Other),
        TokenKind::Percent => (19, 20, BinaryOp::Rem, OpClass::Other),
        _ => return None,
    };
    Some(result)
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

    #[test]
    fn unary_prefix() {
        // -7i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Minus),
            tok(TokenKind::IntLiteral(7, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(u)) => {
                assert_eq!(u.op, UnaryOp::Neg);
                match *u.operand {
                    Expr::IntLit(lit) => {
                        assert_eq!(lit.value, 7);
                        assert_eq!(lit.suffix, IntSuffix::I32);
                    }
                    other => panic!("expected IntLit operand, got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Neg, IntLit)) for `-7i32`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // not true
        let mut p = Parser::new(vec![
            tok(TokenKind::Not),
            tok(TokenKind::True),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(u)) => {
                assert_eq!(u.op, UnaryOp::Not);
                match *u.operand {
                    Expr::BoolLit(lit) => assert!(lit.value),
                    other => panic!("expected BoolLit operand, got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Not, BoolLit)) for `not true`, got {:?}",
                other
            ),
        }
        assert!(p.errors.is_empty());

        // *1i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Star),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(u)) => {
                assert_eq!(u.op, UnaryOp::Deref);
                match *u.operand {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit operand, got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Deref, IntLit)) for `*1i32`, got {:?}",
                other
            ),
        }
        assert!(p.errors.is_empty());

        // &1i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(u)) => {
                assert_eq!(u.op, UnaryOp::Ref);
                match *u.operand {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit operand, got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Ref, IntLit)) for `&1i32`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // &mut 1i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::Mut),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(u)) => {
                assert_eq!(u.op, UnaryOp::RefMut);
                match *u.operand {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit operand, got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(RefMut, IntLit)) for `&mut 1i32`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // - - 7i32 (chained, depth 2)
        let mut p = Parser::new(vec![
            tok(TokenKind::Minus),
            tok(TokenKind::Minus),
            tok(TokenKind::IntLiteral(7, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(outer)) => {
                assert_eq!(outer.op, UnaryOp::Neg);
                match *outer.operand {
                    Expr::Unary(inner) => {
                        assert_eq!(inner.op, UnaryOp::Neg);
                        match *inner.operand {
                            Expr::IntLit(lit) => assert_eq!(lit.value, 7),
                            other => panic!("expected IntLit at depth 2, got {:?}", other),
                        }
                    }
                    other => panic!("expected inner Unary(Neg), got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Neg, Unary(Neg, _))) for `- - 7i32`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // & * 1i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::Star),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Unary(outer)) => {
                assert_eq!(outer.op, UnaryOp::Ref);
                match *outer.operand {
                    Expr::Unary(inner) => {
                        assert_eq!(inner.op, UnaryOp::Deref);
                        match *inner.operand {
                            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                            other => panic!("expected IntLit, got {:?}", other),
                        }
                    }
                    other => panic!("expected inner Unary(Deref), got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Unary(Ref, Unary(Deref, _))) for `& * 1i32`, got {:?}",
                other
            ),
        }
        assert!(p.errors.is_empty());

        // pass-through: literal head returns Expr::IntLit directly
        let mut p = Parser::new(vec![
            tok(TokenKind::IntLiteral(42, IntSuffix::I32)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::IntLit(lit)) => {
                assert_eq!(lit.value, 42);
                assert_eq!(lit.suffix, IntSuffix::I32);
            }
            other => panic!("expected Ok(IntLit) pass-through, got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());

        // wrong head: [Plus, Eof] → Err, pos == 0
        let mut p = Parser::new(vec![tok(TokenKind::Plus), tok(TokenKind::Eof)]);
        assert!(p.parse_unary().is_err());
        assert_eq!(p.pos, 0);
    }

    fn int_tok(v: u64) -> Token {
        tok(TokenKind::IntLiteral(v, IntSuffix::I32))
    }

    fn binary_of(e: &Expr) -> &crate::ast::expr::Binary {
        match e {
            Expr::Binary(b) => b,
            other => panic!("expected Binary, got {:?}", other),
        }
    }

    fn int_value(e: &Expr) -> u64 {
        match e {
            Expr::IntLit(lit) => lit.value,
            other => panic!("expected IntLit, got {:?}", other),
        }
    }

    #[test]
    fn operator_precedence() {
        use crate::ast::expr::BinaryOp;

        // 1 + 2 * 3 → Add(1, Mul(2, 3))
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Plus),
            int_tok(2),
            tok(TokenKind::Star),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Add);
        assert_eq!(int_value(&outer.lhs), 1);
        let inner = binary_of(&outer.rhs);
        assert_eq!(inner.op, BinaryOp::Mul);
        assert_eq!(int_value(&inner.lhs), 2);
        assert_eq!(int_value(&inner.rhs), 3);

        // 1 * 2 + 3 → Add(Mul(1, 2), 3)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Star),
            int_tok(2),
            tok(TokenKind::Plus),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Add);
        assert_eq!(int_value(&outer.rhs), 3);
        let inner = binary_of(&outer.lhs);
        assert_eq!(inner.op, BinaryOp::Mul);
        assert_eq!(int_value(&inner.lhs), 1);
        assert_eq!(int_value(&inner.rhs), 2);

        // 1 - 2 - 3 → Sub(Sub(1, 2), 3)  (left-assoc)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Minus),
            int_tok(2),
            tok(TokenKind::Minus),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Sub);
        assert_eq!(int_value(&outer.rhs), 3);
        let inner = binary_of(&outer.lhs);
        assert_eq!(inner.op, BinaryOp::Sub);
        assert_eq!(int_value(&inner.lhs), 1);
        assert_eq!(int_value(&inner.rhs), 2);

        // 1 = 2 = 3 → Assign(1, Assign(2, 3))  (right-assoc; placeholders for paths)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Eq),
            int_tok(2),
            tok(TokenKind::Eq),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Assign);
        assert_eq!(int_value(&outer.lhs), 1);
        let inner = binary_of(&outer.rhs);
        assert_eq!(inner.op, BinaryOp::Assign);
        assert_eq!(int_value(&inner.lhs), 2);
        assert_eq!(int_value(&inner.rhs), 3);

        // 1 | 2 & 3 → BitOr(1, BitAnd(2, 3))
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Pipe),
            int_tok(2),
            tok(TokenKind::Amp),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::BitOr);
        assert_eq!(int_value(&outer.lhs), 1);
        let inner = binary_of(&outer.rhs);
        assert_eq!(inner.op, BinaryOp::BitAnd);
        assert_eq!(int_value(&inner.lhs), 2);
        assert_eq!(int_value(&inner.rhs), 3);

        // 1 == 2 and 3 == 4 → And(Eq(1, 2), Eq(3, 4))
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::EqEq),
            int_tok(2),
            tok(TokenKind::And),
            int_tok(3),
            tok(TokenKind::EqEq),
            int_tok(4),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::And);
        let left = binary_of(&outer.lhs);
        assert_eq!(left.op, BinaryOp::Eq);
        assert_eq!(int_value(&left.lhs), 1);
        assert_eq!(int_value(&left.rhs), 2);
        let right = binary_of(&outer.rhs);
        assert_eq!(right.op, BinaryOp::Eq);
        assert_eq!(int_value(&right.lhs), 3);
        assert_eq!(int_value(&right.rhs), 4);

        // 1 and 2 or 3 → Or(And(1, 2), 3)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::And),
            int_tok(2),
            tok(TokenKind::Or),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Or);
        assert_eq!(int_value(&outer.rhs), 3);
        let inner = binary_of(&outer.lhs);
        assert_eq!(inner.op, BinaryOp::And);
        assert_eq!(int_value(&inner.lhs), 1);
        assert_eq!(int_value(&inner.rhs), 2);

        // 1 << 2 + 3 → Shl(1, Add(2, 3))  (`+` binds tighter than `<<`)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Shl),
            int_tok(2),
            tok(TokenKind::Plus),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        let expr = p.parse_expr().expect("ok");
        let outer = binary_of(&expr);
        assert_eq!(outer.op, BinaryOp::Shl);
        assert_eq!(int_value(&outer.lhs), 1);
        let inner = binary_of(&outer.rhs);
        assert_eq!(inner.op, BinaryOp::Add);
        assert_eq!(int_value(&inner.lhs), 2);
        assert_eq!(int_value(&inner.rhs), 3);
    }

    #[test]
    fn comparison_non_associative_rejected() {
        // `1 < 2 < 3` → Err with E0100
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Lt),
            int_tok(2),
            tok(TokenKind::Lt),
            int_tok(3),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Err(e) => assert_eq!(e.code, ErrorCode::E0100),
            Ok(other) => panic!(
                "expected Err with E0100 for `1 < 2 < 3`, got Ok({:?})",
                other
            ),
        }
    }
}
