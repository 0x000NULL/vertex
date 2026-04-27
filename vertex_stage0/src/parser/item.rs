use crate::ast::expr::{Expr, Path, PathSegment};
use crate::ast::item::{FnDef, Item, Param};
use crate::ast::ty::Type;
use crate::error::{CompileError, ErrorCode, ErrorKind};
use crate::lexer::token::TokenKind;
use crate::parser::Parser;
use crate::span::Span;

impl Parser {
    // Stopgap: replaced by `parse-path-types-with-generic-args`.
    fn parse_type(&mut self) -> Result<Type, CompileError> {
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

    pub fn parse_fn(&mut self) -> Result<Item, CompileError> {
        let mut is_const = false;
        let mut is_unsafe = false;
        let mut extern_abi: Option<String> = None;
        let mut first_modifier_span: Option<Span> = None;

        loop {
            match self.peek() {
                TokenKind::Const => {
                    let t = self.bump();
                    if first_modifier_span.is_none() {
                        first_modifier_span = Some(t.span);
                    }
                    if is_const {
                        let err = CompileError::new(
                            ErrorCode::E0100,
                            ErrorKind::Syntax,
                            t.span,
                            "duplicate `const` modifier on function",
                        );
                        self.errors.push(err);
                    } else {
                        is_const = true;
                    }
                }
                TokenKind::Unsafe => {
                    let t = self.bump();
                    if first_modifier_span.is_none() {
                        first_modifier_span = Some(t.span);
                    }
                    if is_unsafe {
                        let err = CompileError::new(
                            ErrorCode::E0100,
                            ErrorKind::Syntax,
                            t.span,
                            "duplicate `unsafe` modifier on function",
                        );
                        self.errors.push(err);
                    } else {
                        is_unsafe = true;
                    }
                }
                TokenKind::Extern => {
                    let t = self.bump();
                    if first_modifier_span.is_none() {
                        first_modifier_span = Some(t.span);
                    }
                    let abi = if matches!(self.peek(), TokenKind::StringLiteral(_)) {
                        let abi_tok = self.bump();
                        match abi_tok.kind {
                            TokenKind::StringLiteral(s) => s,
                            _ => unreachable!(),
                        }
                    } else {
                        String::new()
                    };
                    if extern_abi.is_some() {
                        let err = CompileError::new(
                            ErrorCode::E0100,
                            ErrorKind::Syntax,
                            t.span,
                            "duplicate `extern` modifier on function",
                        );
                        self.errors.push(err);
                    } else {
                        extern_abi = Some(abi);
                    }
                }
                _ => break,
            }
        }

        let fn_kw = self.expect(&TokenKind::Fn)?;
        let start_span = first_modifier_span.unwrap_or(fn_kw.span);

        let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        self.expect(&TokenKind::LParen)?;
        let mut params: Vec<Param> = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            let pname_tok = self.expect(&TokenKind::Ident(String::new()))?;
            let pname_span = pname_tok.span;
            let pname = match pname_tok.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            };
            self.expect(&TokenKind::Colon)?;
            let pty = self.parse_type()?;
            let pspan = pname_span.merge(&type_span(&pty));
            let pid = self.new_node_id();
            params.push(Param {
                id: pid,
                span: pspan,
                name: pname,
                ty: pty,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen)?;

        let ret_ty = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = match self.parse_block()? {
            Expr::Block(b) => b,
            other => unreachable!("parse_block returned non-block: {:?}", other),
        };

        let span = start_span.merge(&body.span);
        let id = self.new_node_id();
        Ok(Item::Fn(FnDef {
            id,
            span,
            name,
            params,
            ret_ty,
            body,
            is_const,
            is_unsafe,
            extern_abi,
        }))
    }
}

fn type_span(ty: &Type) -> crate::span::Span {
    match ty {
        Type::Path(p) => p.span,
        Type::Ref { span, .. } => *span,
        // Other variants are not produced by the local stopgap `parse_type`;
        // when richer types land, this helper goes away with them.
        _ => unreachable!("type_span: unexpected variant from stopgap parse_type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expr;
    use crate::lexer::token::{IntSuffix, Token};
    use crate::span::{FileId, Span};

    fn tok(kind: TokenKind) -> Token {
        Token::new(kind, Span::new(FileId(0), 0, 0))
    }

    fn ident_tok(s: &str) -> Token {
        tok(TokenKind::Ident(s.to_string()))
    }

    fn int_tok(v: u64) -> Token {
        tok(TokenKind::IntLiteral(v, IntSuffix::I32))
    }

    fn as_fn(item: Item) -> FnDef {
        match item {
            Item::Fn(f) => f,
            other => panic!("expected Item::Fn, got {:?}", other),
        }
    }

    fn type_ident(ty: &Type) -> &str {
        match ty {
            Type::Path(p) => {
                assert_eq!(p.segments.len(), 1, "expected single-segment path type");
                let seg = &p.segments[0];
                assert!(seg.generic_args.is_empty(), "expected no generic args");
                seg.ident.as_str()
            }
            other => panic!("expected Type::Path, got {:?}", other),
        }
    }

    #[test]
    fn plain_fn() {
        // fn f() {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("f"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "f");
        assert!(f.params.is_empty());
        assert!(f.ret_ty.is_none());
        assert!(f.body.stmts.is_empty());
        assert!(f.body.tail.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn k() -> i32 { 1i32 }
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("k"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::Arrow),
            ident_tok("i32"),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "k");
        assert!(f.params.is_empty());
        assert_eq!(type_ident(f.ret_ty.as_ref().expect("ret_ty")), "i32");
        assert!(f.body.stmts.is_empty());
        let tail = f.body.tail.as_ref().expect("tail");
        match tail.as_ref() {
            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
            other => panic!("expected IntLit tail, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn add(a: i32, b: i32,) -> i32 { 0i32 }
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("add"),
            tok(TokenKind::LParen),
            ident_tok("a"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Comma),
            ident_tok("b"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Comma),
            tok(TokenKind::RParen),
            tok(TokenKind::Arrow),
            ident_tok("i32"),
            tok(TokenKind::LBrace),
            int_tok(0),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert_eq!(type_ident(&f.params[0].ty), "i32");
        assert_eq!(f.params[1].name, "b");
        assert_eq!(type_ident(&f.params[1].ty), "i32");
        assert_eq!(type_ident(f.ret_ty.as_ref().expect("ret_ty")), "i32");
        assert!(f.body.stmts.is_empty());
        let tail = f.body.tail.as_ref().expect("tail");
        match tail.as_ref() {
            Expr::IntLit(lit) => assert_eq!(lit.value, 0),
            other => panic!("expected IntLit tail, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn string_tok(s: &str) -> Token {
        tok(TokenKind::StringLiteral(s.to_string()))
    }

    #[test]
    fn fn_modifiers() {
        // const fn f() {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Const),
            tok(TokenKind::Fn),
            ident_tok("f"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "f");
        assert!(f.is_const);
        assert!(!f.is_unsafe);
        assert!(f.extern_abi.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // unsafe fn g() {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Unsafe),
            tok(TokenKind::Fn),
            ident_tok("g"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "g");
        assert!(!f.is_const);
        assert!(f.is_unsafe);
        assert!(f.extern_abi.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // extern "C" fn h() {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Extern),
            string_tok("C"),
            tok(TokenKind::Fn),
            ident_tok("h"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "h");
        assert!(!f.is_const);
        assert!(!f.is_unsafe);
        assert_eq!(f.extern_abi.as_deref(), Some("C"));
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // extern fn i() {} (bare extern, no ABI literal)
        let mut p = Parser::new(vec![
            tok(TokenKind::Extern),
            tok(TokenKind::Fn),
            ident_tok("i"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "i");
        assert_eq!(f.extern_abi.as_deref(), Some(""));
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // const unsafe extern "C" fn j() {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Const),
            tok(TokenKind::Unsafe),
            tok(TokenKind::Extern),
            string_tok("C"),
            tok(TokenKind::Fn),
            ident_tok("j"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "j");
        assert!(f.is_const);
        assert!(f.is_unsafe);
        assert_eq!(f.extern_abi.as_deref(), Some("C"));
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // unsafe const fn k() {} (any-order acceptance)
        let mut p = Parser::new(vec![
            tok(TokenKind::Unsafe),
            tok(TokenKind::Const),
            tok(TokenKind::Fn),
            ident_tok("k"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "k");
        assert!(f.is_const);
        assert!(f.is_unsafe);
        assert!(f.extern_abi.is_none());
        assert!(p.errors.is_empty());

        // duplicate `const` produces an E0100 error but still parses
        let mut p = Parser::new(vec![
            tok(TokenKind::Const),
            tok(TokenKind::Const),
            tok(TokenKind::Fn),
            ident_tok("dup"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "dup");
        assert!(f.is_const);
        assert_eq!(p.errors.len(), 1);
        let errs = std::mem::take(&mut p.errors)
            .into_result(())
            .expect_err("expected accumulated error");
        assert_eq!(errs[0].code, crate::error::ErrorCode::E0100);
        assert_eq!(errs[0].kind, crate::error::ErrorKind::Syntax);
    }
}
