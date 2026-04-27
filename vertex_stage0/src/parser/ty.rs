use crate::ast::expr::{Path, PathSegment};
use crate::ast::ty::Type;
use crate::error::{CompileError, ErrorCode, ErrorKind};
use crate::lexer::token::TokenKind;
use crate::parser::Parser;
use crate::span::{FileId, Span};

impl Parser {
    pub fn parse_type(&mut self) -> Result<Type, CompileError> {
        if matches!(self.peek(), TokenKind::Amp) {
            return self.parse_ref_type();
        }
        if matches!(self.peek(), TokenKind::Star) {
            return self.parse_ptr_type();
        }
        if matches!(self.peek(), TokenKind::LBracket) {
            return self.parse_bracketed_type();
        }
        if matches!(self.peek(), TokenKind::LParen) {
            return self.parse_tuple_or_grouped_type();
        }
        // Stopgap path-type body: replaced by `parse-path-types-with-generic-args`.
        let ident_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let span = ident_tok.span;
        let ident = match ident_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };
        let id = self.new_node_id();
        Ok(Type::Path(Path {
            id,
            span,
            segments: vec![PathSegment {
                ident,
                generic_args: Vec::new(),
            }],
        }))
    }

    fn parse_ptr_type(&mut self) -> Result<Type, CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::Star));
        let star_tok = self.bump();

        let mutable = match self.peek() {
            TokenKind::Mut => {
                self.bump();
                true
            }
            TokenKind::Const => {
                self.bump();
                false
            }
            _ => {
                let err = CompileError::new(
                    ErrorCode::E0100,
                    ErrorKind::Syntax,
                    star_tok.span,
                    "expected `const` or `mut` after `*`",
                );
                self.errors.push(err);
                false
            }
        };

        let inner = self.parse_type()?;
        Ok(Type::Ptr {
            mutable,
            ty: Box::new(inner),
        })
    }

    fn parse_bracketed_type(&mut self) -> Result<Type, CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::LBracket));
        self.bump();
        let elem = self.parse_type()?;
        if matches!(self.peek(), TokenKind::Semi) {
            self.bump();
            let len = self.parse_expr()?;
            self.expect(&TokenKind::RBracket)?;
            return Ok(Type::Array {
                elem: Box::new(elem),
                len: Box::new(len),
            });
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(Type::Slice {
            elem: Box::new(elem),
        })
    }

    fn parse_tuple_or_grouped_type(&mut self) -> Result<Type, CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::LParen));
        self.bump();

        if matches!(self.peek(), TokenKind::RParen) {
            self.bump();
            return Ok(Type::Tuple(Vec::new()));
        }

        let first = self.parse_type()?;

        if matches!(self.peek(), TokenKind::RParen) {
            self.bump();
            return Ok(first);
        }

        self.expect(&TokenKind::Comma)?;

        let mut elems = vec![first];
        loop {
            if matches!(self.peek(), TokenKind::RParen) {
                break;
            }
            elems.push(self.parse_type()?);
            if matches!(self.peek(), TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        self.expect(&TokenKind::RParen)?;
        Ok(Type::Tuple(elems))
    }

    fn parse_ref_type(&mut self) -> Result<Type, CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::Amp));
        let amp_tok = self.bump();
        let start_span = amp_tok.span;

        // Defensive lifetime swallow: when the lexer learns to emit lifetime
        // tokens (today it does not), an `Ident` whose name begins with `'`
        // is the most likely shape. Discard it; `Type::Ref` carries no
        // lifetime field in Stage 0.
        if let TokenKind::Ident(name) = self.peek() {
            if name.starts_with('\'') {
                self.bump();
            }
        }

        let mutable = self.eat(&TokenKind::Mut);
        let inner = self.parse_type()?;
        let inner_span = type_span(&inner);
        let span = start_span.merge(&inner_span);
        let id = self.new_node_id();
        Ok(Type::Ref {
            mutable,
            ty: Box::new(inner),
            span,
            id,
        })
    }
}

fn type_span(ty: &Type) -> Span {
    match ty {
        Type::Path(p) => p.span,
        Type::Ref { span, .. } => *span,
        Type::Ptr { ty, .. } => type_span(ty),
        Type::Array { elem, .. } => type_span(elem),
        Type::Slice { elem } => type_span(elem),
        Type::Tuple(elems) => {
            if let Some(first) = elems.first() {
                type_span(first)
            } else {
                Span::new(FileId(0), 0, 0)
            }
        }
        _ => unreachable!("type_span: unexpected variant from stopgap parse_type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expr;
    use crate::lexer::token::{IntSuffix, Token};
    use crate::span::FileId;

    fn tok(kind: TokenKind) -> Token {
        Token::new(kind, Span::new(FileId(0), 0, 0))
    }

    fn ident_tok(s: &str) -> Token {
        tok(TokenKind::Ident(s.to_string()))
    }

    fn assert_path_ident(ty: &Type, expected: &str) {
        match ty {
            Type::Path(p) => {
                assert_eq!(p.segments.len(), 1, "expected single-segment path");
                assert!(
                    p.segments[0].generic_args.is_empty(),
                    "expected no generic args",
                );
                assert_eq!(p.segments[0].ident, expected);
            }
            other => panic!("expected Type::Path, got {:?}", other),
        }
    }

    #[test]
    fn ref_types() {
        // &i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type &i32");
        match ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(!mutable);
                assert_path_ident(&ty, "i32");
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // &mut i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::Mut),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type &mut i32");
        match ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(mutable);
                assert_path_ident(&ty, "i32");
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // &&i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::Amp),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type &&i32");
        match ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(!mutable);
                match *ty {
                    Type::Ref {
                        mutable: inner_mut,
                        ty: inner_ty,
                        ..
                    } => {
                        assert!(!inner_mut);
                        assert_path_ident(&inner_ty, "i32");
                    }
                    other => panic!("expected inner Type::Ref, got {:?}", other),
                }
            }
            other => panic!("expected outer Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    #[test]
    fn raw_ptr_types() {
        // *const i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Star),
            tok(TokenKind::Const),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type *const i32");
        match ty {
            Type::Ptr { mutable, ty } => {
                assert!(!mutable);
                assert_path_ident(&ty, "i32");
            }
            other => panic!("expected Type::Ptr, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // *mut i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Star),
            tok(TokenKind::Mut),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type *mut i32");
        match ty {
            Type::Ptr { mutable, ty } => {
                assert!(mutable);
                assert_path_ident(&ty, "i32");
            }
            other => panic!("expected Type::Ptr, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // *const *mut i32
        let mut p = Parser::new(vec![
            tok(TokenKind::Star),
            tok(TokenKind::Const),
            tok(TokenKind::Star),
            tok(TokenKind::Mut),
            ident_tok("i32"),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type *const *mut i32");
        match ty {
            Type::Ptr { mutable, ty } => {
                assert!(!mutable);
                match *ty {
                    Type::Ptr {
                        mutable: inner_mut,
                        ty: inner_ty,
                    } => {
                        assert!(inner_mut);
                        assert_path_ident(&inner_ty, "i32");
                    }
                    other => panic!("expected inner Type::Ptr, got {:?}", other),
                }
            }
            other => panic!("expected outer Type::Ptr, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    #[test]
    fn slice_and_array_types() {
        // &[i32]
        let mut p = Parser::new(vec![
            tok(TokenKind::Amp),
            tok(TokenKind::LBracket),
            ident_tok("i32"),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type &[i32]");
        match ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(!mutable);
                match *ty {
                    Type::Slice { elem } => {
                        assert_path_ident(&elem, "i32");
                    }
                    other => panic!("expected Type::Slice, got {:?}", other),
                }
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // [i32; 4]
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            ident_tok("i32"),
            tok(TokenKind::Semi),
            tok(TokenKind::IntLiteral(4, IntSuffix::Unsuffixed)),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type [i32; 4]");
        match ty {
            Type::Array { elem, len } => {
                assert_path_ident(&elem, "i32");
                match *len {
                    Expr::IntLit(lit) => {
                        assert_eq!(lit.value, 4);
                    }
                    other => panic!("expected Expr::IntLit, got {:?}", other),
                }
            }
            other => panic!("expected Type::Array, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    #[test]
    fn tuple_types() {
        // ()
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type ()");
        match ty {
            Type::Tuple(elems) => {
                assert!(elems.is_empty(), "expected empty tuple");
            }
            other => panic!("expected Type::Tuple, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // (i32, u8, bool)
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            ident_tok("i32"),
            tok(TokenKind::Comma),
            ident_tok("u8"),
            tok(TokenKind::Comma),
            ident_tok("bool"),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type (i32, u8, bool)");
        match ty {
            Type::Tuple(elems) => {
                assert_eq!(elems.len(), 3);
                assert_path_ident(&elems[0], "i32");
                assert_path_ident(&elems[1], "u8");
                assert_path_ident(&elems[2], "bool");
            }
            other => panic!("expected Type::Tuple, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // (i32) -- grouping, not a tuple
        let mut p = Parser::new(vec![
            tok(TokenKind::LParen),
            ident_tok("i32"),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        let ty = p.parse_type().expect("parse_type (i32)");
        assert_path_ident(&ty, "i32");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }
}
