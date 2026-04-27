use crate::ast::expr::{Expr, GenericArg, Path, PathSegment};
use crate::ast::generics::{Generics, TraitBound, TypeParam, WhereClause, WherePred};
use crate::ast::item::{Field, FnDef, Item, Param, StructDef};
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

    /// Stopgap helper for self-parameter explicit types: accepts a
    /// single-segment path with one optional generic argument
    /// (e.g. `Box<Self>`, `Rc<Self>`). Replaced by the general path-type
    /// parser when `parse-path-types-with-generic-args` lands.
    fn parse_self_explicit_type(&mut self) -> Result<Type, CompileError> {
        let head_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let head_span = head_tok.span;
        let ident = match head_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };
        let mut generic_args: Vec<GenericArg> = Vec::new();
        let mut end_span = head_span;
        if matches!(self.peek(), TokenKind::Lt) {
            self.bump();
            let arg_tok =
                self.expect_one_of(&[TokenKind::Ident(String::new()), TokenKind::SelfUpper])?;
            generic_args.push(GenericArg::Placeholder);
            let _ = arg_tok;
            let gt_tok = self.expect(&TokenKind::Gt)?;
            end_span = gt_tok.span;
        }
        let span = head_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Type::Path(Path {
            id,
            span,
            segments: vec![PathSegment {
                ident,
                generic_args,
            }],
        }))
    }

    fn synth_self_path_type(&mut self, span: Span) -> Type {
        let id = self.new_node_id();
        Type::Path(Path {
            id,
            span,
            segments: vec![PathSegment {
                ident: "Self".to_string(),
                generic_args: Vec::new(),
            }],
        })
    }

    /// Recognize the optional self-parameter form at the start of a `fn`
    /// param list. Returns `None` (without consuming any tokens) when the
    /// next tokens do not begin a self-parameter.
    fn try_parse_self_param(&mut self) -> Option<Result<Param, CompileError>> {
        match self.peek() {
            TokenKind::SelfLower => Some(self.parse_self_param_value()),
            TokenKind::Amp => match self.peek_at(1) {
                TokenKind::SelfLower => Some(self.parse_self_param_ref(false)),
                TokenKind::Mut => {
                    if matches!(self.peek_at(2), TokenKind::SelfLower) {
                        Some(self.parse_self_param_ref(true))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_self_param_value(&mut self) -> Result<Param, CompileError> {
        let self_tok = self.bump();
        let self_span = self_tok.span;
        let (ty, end_span) = if matches!(self.peek(), TokenKind::Colon) {
            self.bump();
            let ty = self.parse_self_explicit_type()?;
            let ty_span = type_span(&ty);
            (ty, ty_span)
        } else {
            let ty = self.synth_self_path_type(self_span);
            (ty, self_span)
        };
        let span = self_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Param {
            id,
            span,
            name: "self".to_string(),
            ty,
            is_self: true,
        })
    }

    fn parse_self_param_ref(&mut self, mutable: bool) -> Result<Param, CompileError> {
        let amp_tok = self.bump();
        let start_span = amp_tok.span;
        if mutable {
            self.bump();
        }
        let self_tok = self.bump();
        let self_span = self_tok.span;
        let inner = self.synth_self_path_type(self_span);
        let ref_span = start_span.merge(&self_span);
        let ref_id = self.new_node_id();
        let ty = Type::Ref {
            mutable,
            ty: Box::new(inner),
            span: ref_span,
            id: ref_id,
        };
        let id = self.new_node_id();
        Ok(Param {
            id,
            span: ref_span,
            name: "self".to_string(),
            ty,
            is_self: true,
        })
    }

    // Stopgap: replaced by `parse-path-types-with-generic-args`. Accepts a
    // single bare identifier as the bound's path (single-segment, no generic
    // args).
    fn parse_trait_bound(&mut self) -> Result<TraitBound, CompileError> {
        let ident_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let span = ident_tok.span;
        let ident = match ident_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };
        let path_id = self.new_node_id();
        let path = Path {
            id: path_id,
            span,
            segments: vec![PathSegment {
                ident,
                generic_args: Vec::new(),
            }],
        };
        let id = self.new_node_id();
        Ok(TraitBound {
            id,
            span,
            path,
            generic_args: Vec::new(),
        })
    }

    fn parse_bounds(&mut self) -> Result<Vec<TraitBound>, CompileError> {
        let mut bounds = vec![self.parse_trait_bound()?];
        while self.eat(&TokenKind::Plus) {
            bounds.push(self.parse_trait_bound()?);
        }
        Ok(bounds)
    }

    // Caller guarantees the next token is `Lt`. Returns the parsed parameter
    // list and the span of the closing `>`. Note: nested generics like
    // `Vec<Vec<T>>` would lex `>>` as `Shr` and are not handled here — full
    // type parsing arrives with `parse-path-types-with-generic-args`.
    fn parse_generics_params(&mut self) -> Result<(Vec<TypeParam>, Span), CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::Lt));
        self.bump();
        let mut params: Vec<TypeParam> = Vec::new();
        let gt_span;
        loop {
            if matches!(self.peek(), TokenKind::Gt) {
                let gt_tok = self.bump();
                gt_span = gt_tok.span;
                break;
            }
            let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
            let name_span = name_tok.span;
            let name = match name_tok.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            };
            let mut bounds: Vec<TraitBound> = Vec::new();
            let mut end_span = name_span;
            if self.eat(&TokenKind::Colon) {
                let parsed = self.parse_bounds()?;
                if let Some(last) = parsed.last() {
                    end_span = last.span;
                }
                bounds = parsed;
            }
            let span = name_span.merge(&end_span);
            let id = self.new_node_id();
            params.push(TypeParam {
                id,
                span,
                name,
                bounds,
            });
            let term = self.expect_one_of(&[TokenKind::Comma, TokenKind::Gt])?;
            if matches!(term.kind, TokenKind::Gt) {
                gt_span = term.span;
                break;
            }
        }
        Ok((params, gt_span))
    }

    // Caller guarantees the next token is `Where`. Stops the predicate loop
    // when the next token is `LBrace` (body) or `Semi`.
    fn parse_where_clause(&mut self) -> Result<WhereClause, CompileError> {
        debug_assert!(matches!(self.peek(), TokenKind::Where));
        let where_tok = self.bump();
        let where_span = where_tok.span;
        let mut predicates: Vec<WherePred> = Vec::new();
        let mut end_span = where_span;
        loop {
            if matches!(self.peek(), TokenKind::LBrace | TokenKind::Semi) {
                break;
            }
            let ty = self.parse_type()?;
            let ty_span = type_span(&ty);
            self.expect(&TokenKind::Colon)?;
            let bounds = self.parse_bounds()?;
            let last_bound_span = bounds.last().map(|b| b.span).unwrap_or(ty_span);
            let pred_span = ty_span.merge(&last_bound_span);
            let pred_id = self.new_node_id();
            end_span = pred_span;
            predicates.push(WherePred {
                id: pred_id,
                span: pred_span,
                ty,
                bounds,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        let id = self.new_node_id();
        let span = where_span.merge(&end_span);
        Ok(WhereClause {
            id,
            span,
            predicates,
        })
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

        let mut generic_params: Vec<TypeParam> = Vec::new();
        let mut generics_list_span: Option<Span> = None;
        if matches!(self.peek(), TokenKind::Lt) {
            let lt_span = self.tokens[self.pos].span;
            let (params, gt_span) = self.parse_generics_params()?;
            generic_params = params;
            generics_list_span = Some(lt_span.merge(&gt_span));
        }

        self.expect(&TokenKind::LParen)?;
        let mut params: Vec<Param> = Vec::new();
        let mut had_self = false;
        if let Some(self_param) = self.try_parse_self_param() {
            params.push(self_param?);
            had_self = true;
        }
        let mut continue_params = true;
        if had_self {
            continue_params = self.eat(&TokenKind::Comma);
        }
        while continue_params && !matches!(self.peek(), TokenKind::RParen) {
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
                is_self: false,
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

        let where_clause = if matches!(self.peek(), TokenKind::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let body = match self.parse_block()? {
            Expr::Block(b) => b,
            other => unreachable!("parse_block returned non-block: {:?}", other),
        };

        let generics = if generics_list_span.is_some() || where_clause.is_some() {
            let span = match (generics_list_span, where_clause.as_ref()) {
                (Some(a), Some(w)) => a.merge(&w.span),
                (Some(a), None) => a,
                (None, Some(w)) => w.span,
                (None, None) => unreachable!(),
            };
            let id = self.new_node_id();
            Some(Generics {
                id,
                span,
                params: generic_params,
                where_clause,
            })
        } else {
            None
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
            generics,
        }))
    }

    pub fn parse_struct(&mut self) -> Result<Item, CompileError> {
        let struct_kw = self.expect(&TokenKind::Struct)?;
        let start_span = struct_kw.span;

        let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        let mut generic_params: Vec<TypeParam> = Vec::new();
        let mut generics_list_span: Option<Span> = None;
        if matches!(self.peek(), TokenKind::Lt) {
            let lt_span = self.tokens[self.pos].span;
            let (params, gt_span) = self.parse_generics_params()?;
            generic_params = params;
            generics_list_span = Some(lt_span.merge(&gt_span));
        }

        self.expect(&TokenKind::LBrace)?;
        let mut fields: Vec<Field> = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace) {
            let is_pub = self.eat(&TokenKind::Pub);
            let fname_tok = self.expect(&TokenKind::Ident(String::new()))?;
            let fname_span = fname_tok.span;
            let fname = match fname_tok.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            };
            self.expect(&TokenKind::Colon)?;
            let fty = self.parse_type()?;
            let fspan = fname_span.merge(&type_span(&fty));
            let fid = self.new_node_id();
            fields.push(Field {
                id: fid,
                span: fspan,
                name: fname,
                ty: fty,
                is_pub,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        let rbrace_tok = self.expect(&TokenKind::RBrace)?;
        let end_span = rbrace_tok.span;

        let generics = generics_list_span.map(|span| {
            let id = self.new_node_id();
            Generics {
                id,
                span,
                params: generic_params,
                where_clause: None,
            }
        });

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Struct(StructDef {
            id,
            span,
            name,
            generics,
            fields,
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

    fn assert_self_path(ty: &Type) {
        match ty {
            Type::Path(p) => {
                assert_eq!(p.segments.len(), 1, "expected single-segment Self path");
                assert_eq!(p.segments[0].ident, "Self");
                assert!(
                    p.segments[0].generic_args.is_empty(),
                    "expected no generic args on Self",
                );
            }
            other => panic!("expected Type::Path(Self), got {:?}", other),
        }
    }

    #[test]
    fn self_params() {
        // fn m(self) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::SelfLower),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        assert_self_path(&f.params[0].ty);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn m(&self) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::Amp),
            tok(TokenKind::SelfLower),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        match &f.params[0].ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(!*mutable);
                assert_self_path(ty);
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn m(&mut self) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::Amp),
            tok(TokenKind::Mut),
            tok(TokenKind::SelfLower),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        match &f.params[0].ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(*mutable);
                assert_self_path(ty);
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn m(self: Box<Self>) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::SelfLower),
            tok(TokenKind::Colon),
            ident_tok("Box"),
            tok(TokenKind::Lt),
            tok(TokenKind::SelfUpper),
            tok(TokenKind::Gt),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        match &f.params[0].ty {
            Type::Path(path) => {
                assert_eq!(path.segments.len(), 1);
                assert_eq!(path.segments[0].ident, "Box");
                assert_eq!(path.segments[0].generic_args.len(), 1);
            }
            other => panic!("expected Type::Path, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn m(self: Rc<Self>) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::SelfLower),
            tok(TokenKind::Colon),
            ident_tok("Rc"),
            tok(TokenKind::Lt),
            tok(TokenKind::SelfUpper),
            tok(TokenKind::Gt),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        match &f.params[0].ty {
            Type::Path(path) => {
                assert_eq!(path.segments.len(), 1);
                assert_eq!(path.segments[0].ident, "Rc");
                assert_eq!(path.segments[0].generic_args.len(), 1);
            }
            other => panic!("expected Type::Path, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // fn m(&self, x: i32) {}
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("m"),
            tok(TokenKind::LParen),
            tok(TokenKind::Amp),
            tok(TokenKind::SelfLower),
            tok(TokenKind::Comma),
            ident_tok("x"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "self");
        assert!(f.params[0].is_self);
        match &f.params[0].ty {
            Type::Ref { mutable, ty, .. } => {
                assert!(!*mutable);
                assert_self_path(ty);
            }
            other => panic!("expected Type::Ref, got {:?}", other),
        }
        assert_eq!(f.params[1].name, "x");
        assert!(!f.params[1].is_self);
        assert_eq!(type_ident(&f.params[1].ty), "i32");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_struct(item: Item) -> StructDef {
        match item {
            Item::Struct(s) => s,
            other => panic!("expected Item::Struct, got {:?}", other),
        }
    }

    #[test]
    fn struct_normal() {
        // struct Name<T> { field: Ty, pub field2: Ty }
        let mut p = Parser::new(vec![
            tok(TokenKind::Struct),
            ident_tok("Name"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Gt),
            tok(TokenKind::LBrace),
            ident_tok("field"),
            tok(TokenKind::Colon),
            ident_tok("Ty"),
            tok(TokenKind::Comma),
            tok(TokenKind::Pub),
            ident_tok("field2"),
            tok(TokenKind::Colon),
            ident_tok("Ty"),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let s = as_struct(p.parse_struct().expect("parse_struct"));
        assert_eq!(s.name, "Name");
        let generics = s.generics.as_ref().expect("generics");
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");
        assert!(generics.params[0].bounds.is_empty());
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "field");
        assert_eq!(type_ident(&s.fields[0].ty), "Ty");
        assert!(!s.fields[0].is_pub);
        assert_eq!(s.fields[1].name, "field2");
        assert_eq!(type_ident(&s.fields[1].ty), "Ty");
        assert!(s.fields[1].is_pub);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    #[test]
    fn fn_generics_and_where() {
        // fn foo<T, U>(x: T) -> U where T: Clone + Debug { }
        let mut p = Parser::new(vec![
            tok(TokenKind::Fn),
            ident_tok("foo"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Comma),
            ident_tok("U"),
            tok(TokenKind::Gt),
            tok(TokenKind::LParen),
            ident_tok("x"),
            tok(TokenKind::Colon),
            ident_tok("T"),
            tok(TokenKind::RParen),
            tok(TokenKind::Arrow),
            ident_tok("U"),
            tok(TokenKind::Where),
            ident_tok("T"),
            tok(TokenKind::Colon),
            ident_tok("Clone"),
            tok(TokenKind::Plus),
            ident_tok("Debug"),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let f = as_fn(p.parse_fn().expect("parse_fn"));
        assert_eq!(f.name, "foo");
        let generics = f.generics.as_ref().expect("generics");
        assert_eq!(generics.params.len(), 2);
        assert_eq!(generics.params[0].name, "T");
        assert!(generics.params[0].bounds.is_empty());
        assert_eq!(generics.params[1].name, "U");
        assert!(generics.params[1].bounds.is_empty());
        let wc = generics.where_clause.as_ref().expect("where_clause");
        assert_eq!(wc.predicates.len(), 1);
        let pred = &wc.predicates[0];
        assert_eq!(type_ident(&pred.ty), "T");
        assert_eq!(pred.bounds.len(), 2);
        assert_eq!(pred.bounds[0].path.segments.len(), 1);
        assert_eq!(pred.bounds[0].path.segments[0].ident, "Clone");
        assert_eq!(pred.bounds[1].path.segments.len(), 1);
        assert_eq!(pred.bounds[1].path.segments[0].ident, "Debug");
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "x");
        assert_eq!(type_ident(&f.params[0].ty), "T");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }
}
