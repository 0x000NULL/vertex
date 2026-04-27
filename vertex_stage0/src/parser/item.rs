use crate::ast::expr::{Expr, GenericArg, IntLit, Path, PathSegment};
use crate::ast::generics::{Generics, TraitBound, TypeParam, WhereClause, WherePred};
use crate::ast::item::{
    ConstDef, EnumDef, EnumVariant, Field, FnDef, Item, ModDef, ModKind, Param, StaticDef,
    StructDef, StructKind, TraitDef, TraitItem, TraitItemConst, TraitItemFn, TraitItemType,
    TypeAliasDef, UseDef, UseTree, VariantKind,
};
use crate::ast::ty::Type;
use crate::error::{CompileError, ErrorCode, ErrorKind};
use crate::lexer::token::{IntSuffix, TokenKind};
use crate::parser::Parser;
use crate::span::Span;

impl Parser {
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

    fn parse_fn_signature_tail(
        &mut self,
    ) -> Result<(Vec<Param>, Option<Type>, Option<WhereClause>), CompileError> {
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

        Ok((params, ret_ty, where_clause))
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

        let (params, ret_ty, where_clause) = self.parse_fn_signature_tail()?;

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

        let mut fields: Vec<Field> = Vec::new();
        let kind: StructKind;
        let end_span: Span;
        match self.peek() {
            TokenKind::LBrace => {
                self.bump();
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
                end_span = rbrace_tok.span;
                kind = StructKind::Record;
            }
            TokenKind::LParen => {
                self.bump();
                let mut index: usize = 0;
                while !matches!(self.peek(), TokenKind::RParen) {
                    let pub_span = if matches!(self.peek(), TokenKind::Pub) {
                        let t = self.bump();
                        Some(t.span)
                    } else {
                        None
                    };
                    let is_pub = pub_span.is_some();
                    let fty = self.parse_type()?;
                    let ty_span = type_span(&fty);
                    let fspan = match pub_span {
                        Some(ps) => ps.merge(&ty_span),
                        None => ty_span,
                    };
                    let fid = self.new_node_id();
                    fields.push(Field {
                        id: fid,
                        span: fspan,
                        name: index.to_string(),
                        ty: fty,
                        is_pub,
                    });
                    index += 1;
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                let semi_tok = self.expect(&TokenKind::Semi)?;
                end_span = semi_tok.span;
                kind = StructKind::Tuple;
            }
            TokenKind::Semi => {
                let semi_tok = self.bump();
                end_span = semi_tok.span;
                kind = StructKind::Unit;
            }
            _ => {
                let _ = self.expect_one_of(&[
                    TokenKind::LBrace,
                    TokenKind::LParen,
                    TokenKind::Semi,
                ])?;
                unreachable!();
            }
        }

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
            kind,
        }))
    }

    pub fn parse_enum(&mut self) -> Result<Item, CompileError> {
        let enum_kw = self.expect(&TokenKind::Enum)?;
        let start_span = enum_kw.span;

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
        let mut variants: Vec<EnumVariant> = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace) {
            let vname_tok = self.expect(&TokenKind::Ident(String::new()))?;
            let vname_span = vname_tok.span;
            let vname = match vname_tok.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            };

            let mut variant_end_span = vname_span;
            let kind = match self.peek() {
                TokenKind::LParen => {
                    self.bump();
                    let mut tys: Vec<Type> = Vec::new();
                    while !matches!(self.peek(), TokenKind::RParen) {
                        let ty = self.parse_type()?;
                        tys.push(ty);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    let rparen_tok = self.expect(&TokenKind::RParen)?;
                    variant_end_span = rparen_tok.span;
                    VariantKind::Tuple(tys)
                }
                TokenKind::LBrace => {
                    self.bump();
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
                    variant_end_span = rbrace_tok.span;
                    VariantKind::Struct(fields)
                }
                _ => VariantKind::Unit,
            };

            let discriminant = if self.eat(&TokenKind::Eq) {
                let lit_tok =
                    self.expect(&TokenKind::IntLiteral(0, IntSuffix::Unsuffixed))?;
                let lit_span = lit_tok.span;
                let (value, suffix) = match lit_tok.kind {
                    TokenKind::IntLiteral(v, s) => (v, s),
                    _ => unreachable!(),
                };
                let lit_id = self.new_node_id();
                variant_end_span = lit_span;
                Some(Expr::IntLit(IntLit {
                    id: lit_id,
                    span: lit_span,
                    value,
                    suffix,
                }))
            } else {
                None
            };

            let variant_span = vname_span.merge(&variant_end_span);
            let variant_id = self.new_node_id();
            variants.push(EnumVariant {
                id: variant_id,
                span: variant_span,
                name: vname,
                kind,
                discriminant,
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
        Ok(Item::Enum(EnumDef {
            id,
            span,
            name,
            generics,
            variants,
        }))
    }

    pub fn parse_trait(&mut self) -> Result<Item, CompileError> {
        let trait_kw = self.expect(&TokenKind::Trait)?;
        let start_span = trait_kw.span;

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

        // Stopgap: supertraits go through `parse_bounds`, which still accepts
        // bare-ident bounds only. Replaced by `parse-path-types-with-generic-args`.
        let supertraits = if self.eat(&TokenKind::Colon) {
            self.parse_bounds()?
        } else {
            Vec::new()
        };

        let where_clause = if matches!(self.peek(), TokenKind::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        self.expect(&TokenKind::LBrace)?;
        let mut items: Vec<TraitItem> = Vec::new();
        loop {
            match self.peek() {
                TokenKind::RBrace => break,
                TokenKind::Fn => {
                    items.push(self.parse_trait_method()?);
                }
                TokenKind::Type => {
                    let type_kw = self.bump();
                    let type_span = type_kw.span;
                    let ident_tok = self.expect(&TokenKind::Ident(String::new()))?;
                    let ident = match ident_tok.kind {
                        TokenKind::Ident(s) => s,
                        _ => unreachable!(),
                    };
                    let semi_tok = self.expect(&TokenKind::Semi)?;
                    let span = type_span.merge(&semi_tok.span);
                    let id = self.new_node_id();
                    items.push(TraitItem::Type(TraitItemType {
                        id,
                        span,
                        name: ident,
                    }));
                }
                TokenKind::Const => {
                    let const_kw = self.bump();
                    let const_span = const_kw.span;
                    let ident_tok = self.expect(&TokenKind::Ident(String::new()))?;
                    let ident = match ident_tok.kind {
                        TokenKind::Ident(s) => s,
                        _ => unreachable!(),
                    };
                    self.expect(&TokenKind::Colon)?;
                    let ty = self.parse_type()?;
                    let semi_tok = self.expect(&TokenKind::Semi)?;
                    let span = const_span.merge(&semi_tok.span);
                    let id = self.new_node_id();
                    items.push(TraitItem::Const(TraitItemConst {
                        id,
                        span,
                        name: ident,
                        ty,
                    }));
                }
                _ => {
                    self.expected_one_of_error(&[
                        TokenKind::Fn,
                        TokenKind::Type,
                        TokenKind::Const,
                        TokenKind::RBrace,
                    ]);
                    break;
                }
            }
        }
        let rbrace_tok = self.expect(&TokenKind::RBrace)?;
        let end_span = rbrace_tok.span;

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

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Trait(TraitDef {
            id,
            span,
            name,
            generics,
            supertraits,
            items,
        }))
    }

    fn parse_trait_method(&mut self) -> Result<TraitItem, CompileError> {
        let fn_kw = self.expect(&TokenKind::Fn)?;
        let start_span = fn_kw.span;

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

        let (params, ret_ty, where_clause) = self.parse_fn_signature_tail()?;

        let (default, end_span) = match self.peek() {
            TokenKind::Semi => {
                let semi_tok = self.bump();
                (None, semi_tok.span)
            }
            TokenKind::LBrace => {
                let block = match self.parse_block()? {
                    Expr::Block(b) => b,
                    other => unreachable!("parse_block returned non-block: {:?}", other),
                };
                let block_span = block.span;
                (Some(block), block_span)
            }
            _ => {
                let _ = self.expect_one_of(&[TokenKind::Semi, TokenKind::LBrace])?;
                unreachable!();
            }
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

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(TraitItem::Fn(TraitItemFn {
            id,
            span,
            name,
            generics,
            params,
            ret_ty,
            default,
        }))
    }

    pub fn parse_mod(&mut self) -> Result<Item, CompileError> {
        let mod_kw = self.expect(&TokenKind::Mod)?;
        let start_span = mod_kw.span;

        let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        let (kind, end_span) = match self.peek() {
            TokenKind::Semi => {
                let semi_tok = self.bump();
                (ModKind::External, semi_tok.span)
            }
            TokenKind::LBrace => {
                self.bump();
                let mut items: Vec<Item> = Vec::new();
                while !matches!(self.peek(), TokenKind::RBrace) {
                    items.push(self.parse_mod_inline_item()?);
                }
                let rbrace_tok = self.expect(&TokenKind::RBrace)?;
                (ModKind::Inline(items), rbrace_tok.span)
            }
            _ => {
                let _ = self.expect_one_of(&[TokenKind::Semi, TokenKind::LBrace])?;
                unreachable!();
            }
        };

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Mod(ModDef {
            id,
            span,
            name,
            kind,
        }))
    }

    // Local `pub` handling here will be subsumed by the dedicated visibility item.
    pub fn parse_use(&mut self) -> Result<Item, CompileError> {
        let mut is_pub = false;
        let mut start_span: Option<Span> = None;
        if matches!(self.peek(), TokenKind::Pub) {
            let pub_tok = self.bump();
            is_pub = true;
            start_span = Some(pub_tok.span);
        }
        let use_kw = self.expect(&TokenKind::Use)?;
        let start_span = start_span.unwrap_or(use_kw.span);

        let tree = self.parse_use_tree()?;

        let semi_tok = self.expect(&TokenKind::Semi)?;
        let end_span = semi_tok.span;

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Use(UseDef {
            id,
            span,
            is_pub,
            tree,
        }))
    }

    fn parse_use_tree(&mut self) -> Result<UseTree, CompileError> {
        if matches!(self.peek(), TokenKind::LBrace) {
            let items = self.parse_use_tree_group()?;
            return Ok(UseTree::Nested {
                segments: Vec::new(),
                items,
            });
        }

        let mut segments: Vec<String> = Vec::new();
        let head_tok = self.expect(&TokenKind::Ident(String::new()))?;
        match head_tok.kind {
            TokenKind::Ident(s) => segments.push(s),
            _ => unreachable!(),
        }
        while self.eat(&TokenKind::ColonColon) {
            match self.peek() {
                TokenKind::Star => {
                    self.bump();
                    return Ok(UseTree::Glob { segments });
                }
                TokenKind::LBrace => {
                    let items = self.parse_use_tree_group()?;
                    return Ok(UseTree::Nested { segments, items });
                }
                _ => {
                    let seg_tok = self.expect(&TokenKind::Ident(String::new()))?;
                    match seg_tok.kind {
                        TokenKind::Ident(s) => segments.push(s),
                        _ => unreachable!(),
                    }
                }
            }
        }

        let mut alias: Option<String> = None;
        if let TokenKind::Ident(s) = self.peek().clone() {
            if s == "as" {
                self.bump();
                let alias_tok = self.expect(&TokenKind::Ident(String::new()))?;
                match alias_tok.kind {
                    TokenKind::Ident(name) => alias = Some(name),
                    _ => unreachable!(),
                }
            }
        }

        Ok(UseTree::Simple { segments, alias })
    }

    fn parse_use_tree_group(&mut self) -> Result<Vec<UseTree>, CompileError> {
        self.expect(&TokenKind::LBrace)?;
        let mut items: Vec<UseTree> = Vec::new();
        if !matches!(self.peek(), TokenKind::RBrace) {
            loop {
                let item = self.parse_use_tree()?;
                items.push(item);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if matches!(self.peek(), TokenKind::RBrace) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(items)
    }

    fn parse_mod_inline_item(&mut self) -> Result<Item, CompileError> {
        match self.peek() {
            TokenKind::Fn => self.parse_fn(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Mod => self.parse_mod(),
            _ => {
                let _ = self.expect_one_of(&[
                    TokenKind::Fn,
                    TokenKind::Struct,
                    TokenKind::Enum,
                    TokenKind::Trait,
                    TokenKind::Mod,
                    TokenKind::RBrace,
                ])?;
                unreachable!();
            }
        }
    }

    pub fn parse_const(&mut self) -> Result<Item, CompileError> {
        let const_kw = self.expect(&TokenKind::Const)?;
        let start_span = const_kw.span;

        let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        let semi_tok = self.expect(&TokenKind::Semi)?;
        let end_span = semi_tok.span;

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Const(ConstDef {
            id,
            span,
            name,
            ty,
            value,
        }))
    }

    pub fn parse_static(&mut self) -> Result<Item, CompileError> {
        let static_kw = self.expect(&TokenKind::Static)?;
        let start_span = static_kw.span;

        let is_mut = self.eat(&TokenKind::Mut);

        let name_tok = self.expect(&TokenKind::Ident(String::new()))?;
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => unreachable!(),
        };

        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        let semi_tok = self.expect(&TokenKind::Semi)?;
        let end_span = semi_tok.span;

        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Item::Static(StaticDef {
            id,
            span,
            name,
            ty,
            value,
            is_mut,
        }))
    }

    pub fn parse_type_alias(&mut self) -> Result<Item, CompileError> {
        let type_kw = self.expect(&TokenKind::Type)?;
        let start_span = type_kw.span;

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

        self.expect(&TokenKind::Eq)?;
        let ty = self.parse_type()?;
        let semi_tok = self.expect(&TokenKind::Semi)?;
        let end_span = semi_tok.span;

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
        Ok(Item::TypeAlias(TypeAliasDef {
            id,
            span,
            name,
            generics,
            ty,
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
    fn struct_tuple_unit() {
        // struct Name<T>(T, T);
        let mut p = Parser::new(vec![
            tok(TokenKind::Struct),
            ident_tok("Name"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Gt),
            tok(TokenKind::LParen),
            ident_tok("T"),
            tok(TokenKind::Comma),
            ident_tok("T"),
            tok(TokenKind::RParen),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let s = as_struct(p.parse_struct().expect("parse_struct"));
        assert_eq!(s.name, "Name");
        let generics = s.generics.as_ref().expect("generics");
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");
        assert!(matches!(s.kind, StructKind::Tuple));
        assert_eq!(s.fields.len(), 2);
        assert!(!s.fields[0].is_pub);
        assert_eq!(type_ident(&s.fields[0].ty), "T");
        assert_eq!(s.fields[0].name, "0");
        assert!(!s.fields[1].is_pub);
        assert_eq!(type_ident(&s.fields[1].ty), "T");
        assert_eq!(s.fields[1].name, "1");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // struct Unit;
        let mut p = Parser::new(vec![
            tok(TokenKind::Struct),
            ident_tok("Unit"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let s = as_struct(p.parse_struct().expect("parse_struct"));
        assert_eq!(s.name, "Unit");
        assert!(s.generics.is_none());
        assert!(matches!(s.kind, StructKind::Unit));
        assert!(s.fields.is_empty());
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

    fn as_enum(item: Item) -> EnumDef {
        match item {
            Item::Enum(e) => e,
            other => panic!("expected Item::Enum, got {:?}", other),
        }
    }

    #[test]
    fn enum_all_variant_kinds() {
        // enum E { A, B(i32, i32), C { x: i32, y: i32 } }
        let mut p = Parser::new(vec![
            tok(TokenKind::Enum),
            ident_tok("E"),
            tok(TokenKind::LBrace),
            ident_tok("A"),
            tok(TokenKind::Comma),
            ident_tok("B"),
            tok(TokenKind::LParen),
            ident_tok("i32"),
            tok(TokenKind::Comma),
            ident_tok("i32"),
            tok(TokenKind::RParen),
            tok(TokenKind::Comma),
            ident_tok("C"),
            tok(TokenKind::LBrace),
            ident_tok("x"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Comma),
            ident_tok("y"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::RBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let e = as_enum(p.parse_enum().expect("parse_enum"));
        assert_eq!(e.name, "E");
        assert!(e.generics.is_none());
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "A");
        assert!(matches!(e.variants[0].kind, VariantKind::Unit));
        assert!(e.variants[0].discriminant.is_none());
        assert_eq!(e.variants[1].name, "B");
        match &e.variants[1].kind {
            VariantKind::Tuple(tys) => {
                assert_eq!(tys.len(), 2);
                assert_eq!(type_ident(&tys[0]), "i32");
                assert_eq!(type_ident(&tys[1]), "i32");
            }
            other => panic!("expected VariantKind::Tuple, got {:?}", other),
        }
        assert!(e.variants[1].discriminant.is_none());
        assert_eq!(e.variants[2].name, "C");
        match &e.variants[2].kind {
            VariantKind::Struct(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert_eq!(type_ident(&fields[0].ty), "i32");
                assert_eq!(fields[1].name, "y");
                assert_eq!(type_ident(&fields[1].ty), "i32");
            }
            other => panic!("expected VariantKind::Struct, got {:?}", other),
        }
        assert!(e.variants[2].discriminant.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // enum E { Foo = 5, Bar = 7, }
        let mut p = Parser::new(vec![
            tok(TokenKind::Enum),
            ident_tok("E"),
            tok(TokenKind::LBrace),
            ident_tok("Foo"),
            tok(TokenKind::Eq),
            int_tok(5),
            tok(TokenKind::Comma),
            ident_tok("Bar"),
            tok(TokenKind::Eq),
            int_tok(7),
            tok(TokenKind::Comma),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let e = as_enum(p.parse_enum().expect("parse_enum"));
        assert_eq!(e.name, "E");
        assert!(e.generics.is_none());
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].name, "Foo");
        assert!(matches!(e.variants[0].kind, VariantKind::Unit));
        match e.variants[0].discriminant.as_ref().expect("discriminant Foo") {
            Expr::IntLit(lit) => assert_eq!(lit.value, 5),
            other => panic!("expected IntLit discriminant, got {:?}", other),
        }
        assert_eq!(e.variants[1].name, "Bar");
        assert!(matches!(e.variants[1].kind, VariantKind::Unit));
        match e.variants[1].discriminant.as_ref().expect("discriminant Bar") {
            Expr::IntLit(lit) => assert_eq!(lit.value, 7),
            other => panic!("expected IntLit discriminant, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // enum Result<T, E> { Ok(T), Err(E) }
        let mut p = Parser::new(vec![
            tok(TokenKind::Enum),
            ident_tok("Result"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Comma),
            ident_tok("E"),
            tok(TokenKind::Gt),
            tok(TokenKind::LBrace),
            ident_tok("Ok"),
            tok(TokenKind::LParen),
            ident_tok("T"),
            tok(TokenKind::RParen),
            tok(TokenKind::Comma),
            ident_tok("Err"),
            tok(TokenKind::LParen),
            ident_tok("E"),
            tok(TokenKind::RParen),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let e = as_enum(p.parse_enum().expect("parse_enum"));
        assert_eq!(e.name, "Result");
        let generics = e.generics.as_ref().expect("generics");
        assert_eq!(generics.params.len(), 2);
        assert_eq!(generics.params[0].name, "T");
        assert!(generics.params[0].bounds.is_empty());
        assert_eq!(generics.params[1].name, "E");
        assert!(generics.params[1].bounds.is_empty());
        assert!(generics.where_clause.is_none());
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].name, "Ok");
        match &e.variants[0].kind {
            VariantKind::Tuple(tys) => {
                assert_eq!(tys.len(), 1);
                assert_eq!(type_ident(&tys[0]), "T");
            }
            other => panic!("expected VariantKind::Tuple, got {:?}", other),
        }
        assert!(e.variants[0].discriminant.is_none());
        assert_eq!(e.variants[1].name, "Err");
        match &e.variants[1].kind {
            VariantKind::Tuple(tys) => {
                assert_eq!(tys.len(), 1);
                assert_eq!(type_ident(&tys[0]), "E");
            }
            other => panic!("expected VariantKind::Tuple, got {:?}", other),
        }
        assert!(e.variants[1].discriminant.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_trait(item: Item) -> TraitDef {
        match item {
            Item::Trait(t) => t,
            other => panic!("expected Item::Trait, got {:?}", other),
        }
    }

    #[test]
    fn trait_with_assoc() {
        // trait Name<T>: Super + Super2 {
        //     fn req(&self);
        //     fn def(&self) { }
        //     type Item;
        //     const MAX: usize;
        // }
        let mut p = Parser::new(vec![
            tok(TokenKind::Trait),
            ident_tok("Name"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Gt),
            tok(TokenKind::Colon),
            ident_tok("Super"),
            tok(TokenKind::Plus),
            ident_tok("Super2"),
            tok(TokenKind::LBrace),
            tok(TokenKind::Fn),
            ident_tok("req"),
            tok(TokenKind::LParen),
            tok(TokenKind::Amp),
            tok(TokenKind::SelfLower),
            tok(TokenKind::RParen),
            tok(TokenKind::Semi),
            tok(TokenKind::Fn),
            ident_tok("def"),
            tok(TokenKind::LParen),
            tok(TokenKind::Amp),
            tok(TokenKind::SelfLower),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Type),
            ident_tok("Item"),
            tok(TokenKind::Semi),
            tok(TokenKind::Const),
            ident_tok("MAX"),
            tok(TokenKind::Colon),
            ident_tok("usize"),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let t = as_trait(p.parse_trait().expect("parse_trait"));
        assert_eq!(t.name, "Name");
        let generics = t.generics.as_ref().expect("generics");
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");
        assert!(generics.params[0].bounds.is_empty());
        assert!(generics.where_clause.is_none());
        assert_eq!(t.supertraits.len(), 2);
        assert_eq!(t.supertraits[0].path.segments.len(), 1);
        assert_eq!(t.supertraits[0].path.segments[0].ident, "Super");
        assert_eq!(t.supertraits[1].path.segments.len(), 1);
        assert_eq!(t.supertraits[1].path.segments[0].ident, "Super2");
        assert_eq!(t.items.len(), 4);
        match &t.items[0] {
            TraitItem::Fn(f) => {
                assert_eq!(f.name, "req");
                assert!(f.default.is_none());
            }
            other => panic!("expected TraitItem::Fn, got {:?}", other),
        }
        match &t.items[1] {
            TraitItem::Fn(f) => {
                assert_eq!(f.name, "def");
                assert!(f.default.is_some());
            }
            other => panic!("expected TraitItem::Fn, got {:?}", other),
        }
        match &t.items[2] {
            TraitItem::Type(ty) => {
                assert_eq!(ty.name, "Item");
            }
            other => panic!("expected TraitItem::Type, got {:?}", other),
        }
        match &t.items[3] {
            TraitItem::Const(c) => {
                assert_eq!(c.name, "MAX");
                assert_eq!(type_ident(&c.ty), "usize");
            }
            other => panic!("expected TraitItem::Const, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_mod(item: Item) -> ModDef {
        match item {
            Item::Mod(m) => m,
            other => panic!("expected Item::Mod, got {:?}", other),
        }
    }

    #[test]
    fn mod_external_vs_inline() {
        // mod foo;
        let mut p = Parser::new(vec![
            tok(TokenKind::Mod),
            ident_tok("foo"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let m = as_mod(p.parse_mod().expect("parse_mod"));
        assert_eq!(m.name, "foo");
        assert!(matches!(m.kind, ModKind::External));
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // mod bar { fn x() {} }
        let mut p = Parser::new(vec![
            tok(TokenKind::Mod),
            ident_tok("bar"),
            tok(TokenKind::LBrace),
            tok(TokenKind::Fn),
            ident_tok("x"),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let m = as_mod(p.parse_mod().expect("parse_mod"));
        assert_eq!(m.name, "bar");
        match m.kind {
            ModKind::Inline(items) => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    Item::Fn(f) => assert_eq!(f.name, "x"),
                    other => panic!("expected Item::Fn, got {:?}", other),
                }
            }
            other => panic!("expected ModKind::Inline, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // mod outer { mod inner; }
        let mut p = Parser::new(vec![
            tok(TokenKind::Mod),
            ident_tok("outer"),
            tok(TokenKind::LBrace),
            tok(TokenKind::Mod),
            ident_tok("inner"),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let m = as_mod(p.parse_mod().expect("parse_mod"));
        assert_eq!(m.name, "outer");
        match m.kind {
            ModKind::Inline(items) => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    Item::Mod(inner) => {
                        assert_eq!(inner.name, "inner");
                        assert!(matches!(inner.kind, ModKind::External));
                    }
                    other => panic!("expected Item::Mod, got {:?}", other),
                }
            }
            other => panic!("expected ModKind::Inline, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_use(item: Item) -> UseDef {
        match item {
            Item::Use(u) => u,
            other => panic!("expected Item::Use, got {:?}", other),
        }
    }

    #[test]
    fn use_simple_and_alias() {
        // use foo::bar;
        let mut p = Parser::new(vec![
            tok(TokenKind::Use),
            ident_tok("foo"),
            tok(TokenKind::ColonColon),
            ident_tok("bar"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let u = as_use(p.parse_use().expect("parse_use"));
        assert!(!u.is_pub);
        match &u.tree {
            UseTree::Simple { segments, alias } => {
                assert_eq!(*segments, vec!["foo".to_string(), "bar".to_string()]);
                assert!(alias.is_none());
            }
            other => panic!("expected UseTree::Simple, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // use foo::bar as baz;
        let mut p = Parser::new(vec![
            tok(TokenKind::Use),
            ident_tok("foo"),
            tok(TokenKind::ColonColon),
            ident_tok("bar"),
            ident_tok("as"),
            ident_tok("baz"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let u = as_use(p.parse_use().expect("parse_use"));
        assert!(!u.is_pub);
        match &u.tree {
            UseTree::Simple { segments, alias } => {
                assert_eq!(*segments, vec!["foo".to_string(), "bar".to_string()]);
                assert_eq!(*alias, Some("baz".to_string()));
            }
            other => panic!("expected UseTree::Simple, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    #[test]
    fn use_nested_glob_pub() {
        // use { a, b::c, d::{e, f} };
        let mut p = Parser::new(vec![
            tok(TokenKind::Use),
            tok(TokenKind::LBrace),
            ident_tok("a"),
            tok(TokenKind::Comma),
            ident_tok("b"),
            tok(TokenKind::ColonColon),
            ident_tok("c"),
            tok(TokenKind::Comma),
            ident_tok("d"),
            tok(TokenKind::ColonColon),
            tok(TokenKind::LBrace),
            ident_tok("e"),
            tok(TokenKind::Comma),
            ident_tok("f"),
            tok(TokenKind::RBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let u = as_use(p.parse_use().expect("parse_use"));
        assert!(!u.is_pub);
        match &u.tree {
            UseTree::Nested { segments, items } => {
                assert!(segments.is_empty());
                assert_eq!(items.len(), 3);
                match &items[0] {
                    UseTree::Simple { segments, alias } => {
                        assert_eq!(*segments, vec!["a".to_string()]);
                        assert!(alias.is_none());
                    }
                    other => panic!("expected items[0] UseTree::Simple, got {:?}", other),
                }
                match &items[1] {
                    UseTree::Simple { segments, alias } => {
                        assert_eq!(*segments, vec!["b".to_string(), "c".to_string()]);
                        assert!(alias.is_none());
                    }
                    other => panic!("expected items[1] UseTree::Simple, got {:?}", other),
                }
                match &items[2] {
                    UseTree::Nested { segments, items } => {
                        assert_eq!(*segments, vec!["d".to_string()]);
                        assert_eq!(items.len(), 2);
                        match &items[0] {
                            UseTree::Simple { segments, alias } => {
                                assert_eq!(*segments, vec!["e".to_string()]);
                                assert!(alias.is_none());
                            }
                            other => {
                                panic!("expected nested items[0] UseTree::Simple, got {:?}", other)
                            }
                        }
                        match &items[1] {
                            UseTree::Simple { segments, alias } => {
                                assert_eq!(*segments, vec!["f".to_string()]);
                                assert!(alias.is_none());
                            }
                            other => {
                                panic!("expected nested items[1] UseTree::Simple, got {:?}", other)
                            }
                        }
                    }
                    other => panic!("expected items[2] UseTree::Nested, got {:?}", other),
                }
            }
            other => panic!("expected UseTree::Nested, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // use foo::*;
        let mut p = Parser::new(vec![
            tok(TokenKind::Use),
            ident_tok("foo"),
            tok(TokenKind::ColonColon),
            tok(TokenKind::Star),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let u = as_use(p.parse_use().expect("parse_use"));
        assert!(!u.is_pub);
        match &u.tree {
            UseTree::Glob { segments } => {
                assert_eq!(*segments, vec!["foo".to_string()]);
            }
            other => panic!("expected UseTree::Glob, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // pub use bar;
        let mut p = Parser::new(vec![
            tok(TokenKind::Pub),
            tok(TokenKind::Use),
            ident_tok("bar"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let u = as_use(p.parse_use().expect("parse_use"));
        assert!(u.is_pub);
        match &u.tree {
            UseTree::Simple { segments, alias } => {
                assert_eq!(*segments, vec!["bar".to_string()]);
                assert!(alias.is_none());
            }
            other => panic!("expected UseTree::Simple, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_const(item: Item) -> ConstDef {
        match item {
            Item::Const(c) => c,
            other => panic!("expected Item::Const, got {:?}", other),
        }
    }

    #[test]
    fn const_item() {
        // const N: i32 = 1i32;
        let mut p = Parser::new(vec![
            tok(TokenKind::Const),
            ident_tok("N"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Eq),
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let c = as_const(p.parse_const().expect("parse_const"));
        assert_eq!(c.name, "N");
        assert_eq!(type_ident(&c.ty), "i32");
        match &c.value {
            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
            other => panic!("expected Expr::IntLit, got {:?}", other),
        }
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_static(item: Item) -> StaticDef {
        match item {
            Item::Static(s) => s,
            other => panic!("expected Item::Static, got {:?}", other),
        }
    }

    #[test]
    fn static_item() {
        // static N: i32 = 1i32;
        let mut p = Parser::new(vec![
            tok(TokenKind::Static),
            ident_tok("N"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Eq),
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let s = as_static(p.parse_static().expect("parse_static"));
        assert_eq!(s.name, "N");
        assert_eq!(type_ident(&s.ty), "i32");
        match &s.value {
            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
            other => panic!("expected Expr::IntLit, got {:?}", other),
        }
        assert!(!s.is_mut);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // static mut N: i32 = 1i32;
        let mut p = Parser::new(vec![
            tok(TokenKind::Static),
            tok(TokenKind::Mut),
            ident_tok("N"),
            tok(TokenKind::Colon),
            ident_tok("i32"),
            tok(TokenKind::Eq),
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let s = as_static(p.parse_static().expect("parse_static"));
        assert_eq!(s.name, "N");
        assert_eq!(type_ident(&s.ty), "i32");
        match &s.value {
            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
            other => panic!("expected Expr::IntLit, got {:?}", other),
        }
        assert!(s.is_mut);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }

    fn as_type_alias(item: Item) -> TypeAliasDef {
        match item {
            Item::TypeAlias(t) => t,
            other => panic!("expected Item::TypeAlias, got {:?}", other),
        }
    }

    #[test]
    fn type_alias() {
        // type Alias = i32;
        let mut p = Parser::new(vec![
            tok(TokenKind::Type),
            ident_tok("Alias"),
            tok(TokenKind::Eq),
            ident_tok("i32"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let t = as_type_alias(p.parse_type_alias().expect("parse_type_alias"));
        assert_eq!(t.name, "Alias");
        assert!(t.generics.is_none());
        assert_eq!(type_ident(&t.ty), "i32");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // type Alias<T> = T;
        let mut p = Parser::new(vec![
            tok(TokenKind::Type),
            ident_tok("Alias"),
            tok(TokenKind::Lt),
            ident_tok("T"),
            tok(TokenKind::Gt),
            tok(TokenKind::Eq),
            ident_tok("T"),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        let t = as_type_alias(p.parse_type_alias().expect("parse_type_alias"));
        assert_eq!(t.name, "Alias");
        let g = t.generics.expect("expected generics");
        assert_eq!(g.params.len(), 1);
        assert_eq!(g.params[0].name, "T");
        assert_eq!(type_ident(&t.ty), "T");
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }
}
