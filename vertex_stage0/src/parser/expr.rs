use crate::ast::expr::{
    ArrayLit, ArrayRepeat, Binary, BinaryOp, Block, BoolLit, Call, Cast, CastTy, CharLit, Closure,
    ClosureParam, Expr, FieldAccess, FloatLit, If, Index, IntLit, MethodCall, Range, StrLit, Try,
    TupleFieldAccess, TupleLit, Unary, UnaryOp,
};
use crate::ast::stmt::Stmt;
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

    fn parse_array_literal(&mut self) -> Result<Expr, CompileError> {
        if !matches!(self.peek(), TokenKind::LBracket) {
            return Err(self.unexpected_token_error("`[`"));
        }
        let lbracket_tok = self.bump();
        let lbracket_span = lbracket_tok.span;

        if matches!(self.peek(), TokenKind::RBracket) {
            let rbracket_tok = self.bump();
            let id = self.new_node_id();
            return Ok(Expr::ArrayLit(ArrayLit {
                id,
                span: lbracket_span.merge(&rbracket_tok.span),
                elems: vec![],
            }));
        }

        let first = self.parse_expr()?;

        match self.peek() {
            TokenKind::Semi => {
                self.bump();
                let count = self.parse_expr()?;
                let rbracket_tok = self.expect(&TokenKind::RBracket)?;
                let id = self.new_node_id();
                Ok(Expr::ArrayRepeat(ArrayRepeat {
                    id,
                    span: lbracket_span.merge(&rbracket_tok.span),
                    value: Box::new(first),
                    count: Box::new(count),
                }))
            }
            TokenKind::Comma => {
                self.bump();
                let mut elems = vec![first];
                while !matches!(self.peek(), TokenKind::RBracket | TokenKind::Eof) {
                    let elem = self.parse_expr()?;
                    elems.push(elem);
                    if matches!(self.peek(), TokenKind::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                let rbracket_tok = self.expect(&TokenKind::RBracket)?;
                let id = self.new_node_id();
                Ok(Expr::ArrayLit(ArrayLit {
                    id,
                    span: lbracket_span.merge(&rbracket_tok.span),
                    elems,
                }))
            }
            TokenKind::RBracket => {
                let rbracket_tok = self.bump();
                let id = self.new_node_id();
                Ok(Expr::ArrayLit(ArrayLit {
                    id,
                    span: lbracket_span.merge(&rbracket_tok.span),
                    elems: vec![first],
                }))
            }
            _ => Err(self.unexpected_token_error("`,`, `;`, or `]`")),
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        if matches!(self.peek(), TokenKind::Pipe) {
            return self.parse_closure();
        }
        if matches!(self.peek(), TokenKind::Ident(s) if s == "move")
            && matches!(self.peek_at(1), TokenKind::Pipe)
        {
            return self.parse_closure();
        }
        self.parse_range()
    }

    fn parse_closure(&mut self) -> Result<Expr, CompileError> {
        let start_span = if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            crate::span::Span::new(crate::span::FileId(0), 0, 0)
        };

        let move_kw = if matches!(self.peek(), TokenKind::Ident(s) if s == "move") {
            self.bump();
            true
        } else {
            false
        };

        self.expect(&TokenKind::Pipe)?;

        let mut params: Vec<ClosureParam> = Vec::new();
        while !matches!(self.peek(), TokenKind::Pipe | TokenKind::Eof) {
            let param = self.parse_closure_param()?;
            params.push(param);
            if matches!(self.peek(), TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }

        self.expect(&TokenKind::Pipe)?;

        // TODO: store the parsed return type on `Closure` once a `return_ty`
        // field exists; for now the `-> Ty` stub is consumed and discarded.
        if matches!(self.peek(), TokenKind::Arrow) {
            self.bump();
            // TODO: replace when type parser lands
            match self.peek() {
                TokenKind::Ident(_) | TokenKind::SelfUpper => {
                    self.bump();
                }
                _ => return Err(self.unexpected_token_error("type after `->`")),
            }
        }

        let body = if matches!(self.peek(), TokenKind::LBrace) {
            self.parse_block()?
        } else {
            self.parse_expr()?
        };

        let span = start_span.merge(&body.span());
        let id = self.new_node_id();
        Ok(Expr::Closure(Closure {
            id,
            span,
            params,
            body: Box::new(body),
            move_kw,
        }))
    }

    fn parse_closure_param(&mut self) -> Result<ClosureParam, CompileError> {
        match self.peek() {
            TokenKind::Ident(_) => {
                self.bump();
            }
            _ => return Err(self.unexpected_token_error("identifier")),
        }
        if matches!(self.peek(), TokenKind::Colon) {
            self.bump();
            // TODO: replace when type parser lands
            match self.peek() {
                TokenKind::Ident(_) | TokenKind::SelfUpper => {
                    self.bump();
                }
                _ => return Err(self.unexpected_token_error("type after `:`")),
            }
        }
        Ok(ClosureParam::Placeholder)
    }

    fn parse_if(&mut self) -> Result<Expr, CompileError> {
        let if_tok = self.expect(&TokenKind::If)?;
        let start_span = if_tok.span;
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let else_branch = if matches!(self.peek(), TokenKind::Else) {
            self.bump();
            match self.peek() {
                TokenKind::If => Some(Box::new(self.parse_if()?)),
                TokenKind::LBrace => Some(Box::new(self.parse_block()?)),
                _ => return Err(self.unexpected_token_error("`{` or `if`")),
            }
        } else {
            None
        };
        let end_span = match &else_branch {
            Some(b) => b.span(),
            None => then.span(),
        };
        let span = start_span.merge(&end_span);
        let id = self.new_node_id();
        Ok(Expr::If(If {
            id,
            span,
            cond: Box::new(cond),
            then: Box::new(then),
            else_branch,
        }))
    }

    pub fn parse_block(&mut self) -> Result<Expr, CompileError> {
        let lbrace_tok = self.expect(&TokenKind::LBrace)?;
        let lbrace_span = lbrace_tok.span;

        let mut stmts: Vec<Stmt> = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let expr = self.parse_expr()?;
            match self.peek() {
                TokenKind::Semi => {
                    self.bump();
                    stmts.push(Stmt::Expr {
                        expr,
                        has_semi: true,
                    });
                }
                TokenKind::RBrace => {
                    tail = Some(Box::new(expr));
                    break;
                }
                _ => {
                    stmts.push(Stmt::Expr {
                        expr,
                        has_semi: false,
                    });
                }
            }
        }

        let rbrace_tok = self.expect(&TokenKind::RBrace)?;
        let span = lbrace_span.merge(&rbrace_tok.span);
        let id = self.new_node_id();
        Ok(Expr::Block(Block {
            id,
            span,
            stmts,
            tail,
        }))
    }

    fn parse_range(&mut self) -> Result<Expr, CompileError> {
        // Prefix range: `..` or `..=` with optional RHS.
        if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
            let inclusive = matches!(self.peek(), TokenKind::DotDotEq);
            let op_tok = self.bump();
            let op_span = op_tok.span;
            if range_rhs_starts_here(self.peek()) {
                let end = self.parse_binary(0)?;
                let span = op_span.merge(&end.span());
                let id = self.new_node_id();
                return Ok(Expr::Range(Range {
                    id,
                    span,
                    start: None,
                    end: Some(Box::new(end)),
                    inclusive,
                }));
            } else {
                let id = self.new_node_id();
                return Ok(Expr::Range(Range {
                    id,
                    span: op_span,
                    start: None,
                    end: None,
                    inclusive,
                }));
            }
        }

        // Infix range: LHS [`..` | `..=` [RHS]]?
        let lhs = self.parse_binary(0)?;
        if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
            let inclusive = matches!(self.peek(), TokenKind::DotDotEq);
            let op_tok = self.bump();
            let op_span = op_tok.span;
            if range_rhs_starts_here(self.peek()) {
                let end = self.parse_binary(0)?;
                let span = lhs.span().merge(&end.span());
                let id = self.new_node_id();
                Ok(Expr::Range(Range {
                    id,
                    span,
                    start: Some(Box::new(lhs)),
                    end: Some(Box::new(end)),
                    inclusive,
                }))
            } else {
                let span = lhs.span().merge(&op_span);
                let id = self.new_node_id();
                Ok(Expr::Range(Range {
                    id,
                    span,
                    start: Some(Box::new(lhs)),
                    end: None,
                    inclusive,
                }))
            }
        } else {
            Ok(lhs)
        }
    }

    fn parse_binary(&mut self, min_bp: u8) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_cast()?;

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

    fn parse_cast(&mut self) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_unary()?;
        while matches!(self.peek(), TokenKind::Ident(s) if s == "as") {
            self.bump();
            // TODO: replace when type parser lands
            let ty_tok = match self.peek() {
                TokenKind::Ident(_) | TokenKind::SelfUpper => self.bump(),
                _ => return Err(self.unexpected_token_error("type after `as`")),
            };
            let span = lhs.span().merge(&ty_tok.span);
            let id = self.new_node_id();
            lhs = Expr::Cast(Cast {
                id,
                span,
                expr: Box::new(lhs),
                ty: Box::new(CastTy::Placeholder),
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
            _ => return self.parse_postfix(),
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

    fn parse_postfix(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary_for_paren()?;
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.bump();
                    let mut args: Vec<Expr> = Vec::new();
                    while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                        let arg = self.parse_expr()?;
                        args.push(arg);
                        if matches!(self.peek(), TokenKind::Comma) {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let rparen_tok = self.expect(&TokenKind::RParen)?;
                    let span = expr.span().merge(&rparen_tok.span);
                    let id = self.new_node_id();
                    expr = Expr::Call(Call {
                        id,
                        span,
                        callee: Box::new(expr),
                        args,
                    });
                }
                TokenKind::LBracket => {
                    self.bump();
                    let idx = self.parse_expr()?;
                    let rbracket_tok = self.expect(&TokenKind::RBracket)?;
                    let span = expr.span().merge(&rbracket_tok.span);
                    let id = self.new_node_id();
                    expr = Expr::Index(Index {
                        id,
                        span,
                        receiver: Box::new(expr),
                        idx: Box::new(idx),
                    });
                }
                TokenKind::Question => {
                    let q_tok = self.bump();
                    let span = expr.span().merge(&q_tok.span);
                    let id = self.new_node_id();
                    expr = Expr::Try(Try {
                        id,
                        span,
                        expr: Box::new(expr),
                    });
                }
                TokenKind::Dot => {
                    self.bump();
                    match self.peek() {
                        TokenKind::Ident(_) => {
                            let ident_span = self.tokens[self.pos].span;
                            let ident_tok = self.bump();
                            let name = match ident_tok.kind {
                                TokenKind::Ident(s) => s,
                                _ => unreachable!(),
                            };
                            if matches!(self.peek(), TokenKind::LParen) {
                                self.bump();
                                let mut args: Vec<Expr> = Vec::new();
                                while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                                    let arg = self.parse_expr()?;
                                    args.push(arg);
                                    if matches!(self.peek(), TokenKind::Comma) {
                                        self.bump();
                                    } else {
                                        break;
                                    }
                                }
                                let rparen_tok = self.expect(&TokenKind::RParen)?;
                                let span = expr.span().merge(&rparen_tok.span);
                                let id = self.new_node_id();
                                expr = Expr::MethodCall(MethodCall {
                                    id,
                                    span,
                                    receiver: Box::new(expr),
                                    method: name,
                                    args,
                                    generic_args: vec![],
                                });
                            } else {
                                let span = expr.span().merge(&ident_span);
                                let id = self.new_node_id();
                                expr = Expr::Field(FieldAccess {
                                    id,
                                    span,
                                    receiver: Box::new(expr),
                                    name,
                                });
                            }
                        }
                        TokenKind::IntLiteral(_, _) => {
                            let intlit_span = self.tokens[self.pos].span;
                            let intlit_tok = self.bump();
                            let v = match intlit_tok.kind {
                                TokenKind::IntLiteral(v, _) => v,
                                _ => unreachable!(),
                            };
                            if v > u32::MAX as u64 {
                                return Err(CompileError::new(
                                    ErrorCode::E0100,
                                    ErrorKind::Syntax,
                                    intlit_span,
                                    "tuple field index exceeds u32::MAX",
                                ));
                            }
                            let span = expr.span().merge(&intlit_span);
                            let id = self.new_node_id();
                            expr = Expr::TupleField(TupleFieldAccess {
                                id,
                                span,
                                receiver: Box::new(expr),
                                idx: v as u32,
                            });
                        }
                        _ => {
                            return Err(
                                self.unexpected_token_error("identifier or integer literal")
                            );
                        }
                    }
                }
                _ => return Ok(expr),
            }
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
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::LBrace => self.parse_block(),
            TokenKind::If => self.parse_if(),
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

// TODO: extend this set when `Ident`/`Match`/`Loop`/`Block`/`[`/path heads
// become valid expression starters; otherwise `a.. <new-form>` will be misparsed
// as `a..` followed by stray tokens.
fn range_rhs_starts_here(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::IntLiteral(_, _)
            | TokenKind::FloatLiteral(_, _)
            | TokenKind::CharLiteral(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::RawStringLiteral(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::LParen
            | TokenKind::Minus
            | TokenKind::Not
            | TokenKind::Star
            | TokenKind::Amp
            | TokenKind::If
    )
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
    fn call_method_field() {
        // 42()
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Call(c)) => {
                assert!(c.args.is_empty());
                match *c.callee {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit callee, got {:?}", other),
                }
            }
            other => panic!("expected Ok(Call) for `42()`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // 42(1, 2,)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::LParen),
            int_tok(1),
            tok(TokenKind::Comma),
            int_tok(2),
            tok(TokenKind::Comma),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Call(c)) => {
                assert_eq!(c.args.len(), 2);
                assert_eq!(int_value(&c.args[0]), 1);
                assert_eq!(int_value(&c.args[1]), 2);
            }
            other => panic!("expected Ok(Call) for `42(1, 2,)`, got {:?}", other),
        }
        assert_eq!(p.pos, 7);
        assert!(p.errors.is_empty());

        // 42 . foo
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::Ident("foo".to_string())),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Field(f)) => {
                assert_eq!(f.name, "foo");
                match *f.receiver {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit receiver, got {:?}", other),
                }
            }
            other => panic!("expected Ok(Field) for `42.foo`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // 42 . foo (7)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::Ident("foo".to_string())),
            tok(TokenKind::LParen),
            int_tok(7),
            tok(TokenKind::RParen),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::MethodCall(m)) => {
                assert_eq!(m.method, "foo");
                assert!(m.generic_args.is_empty());
                assert_eq!(m.args.len(), 1);
                assert_eq!(int_value(&m.args[0]), 7);
                match *m.receiver {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit receiver, got {:?}", other),
                }
            }
            other => panic!("expected Ok(MethodCall) for `42.foo(7)`, got {:?}", other),
        }
        assert_eq!(p.pos, 6);
        assert!(p.errors.is_empty());

        // 42 . 0  (tuple field)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::IntLiteral(0, IntSuffix::Unsuffixed)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::TupleField(t)) => {
                assert_eq!(t.idx, 0);
                match *t.receiver {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit receiver, got {:?}", other),
                }
            }
            other => panic!("expected Ok(TupleField) for `42.0`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // 42 . <u64::MAX> → Err(E0100)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::IntLiteral(u64::MAX, IntSuffix::Unsuffixed)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Err(e) => assert_eq!(e.code, ErrorCode::E0100),
            Ok(other) => panic!("expected Err(E0100) for tuple-field overflow, got Ok({:?})", other),
        }

        // 42 . + → Err
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::Plus),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Err(e) => assert_eq!(e.code, ErrorCode::E0100),
            Ok(other) => panic!("expected Err(E0100) for `42.+`, got Ok({:?})", other),
        }

        // 42 . foo () . bar . 0  → TupleField(Field(MethodCall(IntLit, "foo", []), "bar"), 0)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Dot),
            tok(TokenKind::Ident("foo".to_string())),
            tok(TokenKind::LParen),
            tok(TokenKind::RParen),
            tok(TokenKind::Dot),
            tok(TokenKind::Ident("bar".to_string())),
            tok(TokenKind::Dot),
            tok(TokenKind::IntLiteral(0, IntSuffix::Unsuffixed)),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::TupleField(t)) => {
                assert_eq!(t.idx, 0);
                match *t.receiver {
                    Expr::Field(f) => {
                        assert_eq!(f.name, "bar");
                        match *f.receiver {
                            Expr::MethodCall(m) => {
                                assert_eq!(m.method, "foo");
                                assert!(m.args.is_empty());
                                match *m.receiver {
                                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                                    other => {
                                        panic!("expected IntLit at chain root, got {:?}", other)
                                    }
                                }
                            }
                            other => panic!("expected MethodCall in chain, got {:?}", other),
                        }
                    }
                    other => panic!("expected Field in chain, got {:?}", other),
                }
            }
            other => panic!("expected Ok(TupleField) for chain, got {:?}", other),
        }
        assert_eq!(p.pos, 9);
        assert!(p.errors.is_empty());
    }

    #[test]
    fn index_cast_try() {
        // 42[1i32]
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Index(i)) => {
                match *i.receiver {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit receiver, got {:?}", other),
                }
                match *i.idx {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit idx, got {:?}", other),
                }
            }
            other => panic!("expected Ok(Index) for `42[1i32]`, got {:?}", other),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // 42?
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Question),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Try(t)) => match *t.expr {
                Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                other => panic!("expected IntLit, got {:?}", other),
            },
            other => panic!("expected Ok(Try) for `42?`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // 42 as i32 (via parse_expr)
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::Ident("as".to_string())),
            tok(TokenKind::Ident("i32".to_string())),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Cast(c)) => {
                match *c.expr {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                    other => panic!("expected IntLit, got {:?}", other),
                }
                assert!(matches!(*c.ty, CastTy::Placeholder));
            }
            other => panic!("expected Ok(Cast) for `42 as i32`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // - 7i32 as i32 → Cast(Unary(Neg, _), Placeholder)
        let mut p = Parser::new(vec![
            tok(TokenKind::Minus),
            int_tok(7),
            tok(TokenKind::Ident("as".to_string())),
            tok(TokenKind::Ident("i32".to_string())),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Cast(c)) => {
                assert!(matches!(*c.ty, CastTy::Placeholder));
                match *c.expr {
                    Expr::Unary(u) => {
                        assert_eq!(u.op, UnaryOp::Neg);
                        match *u.operand {
                            Expr::IntLit(lit) => assert_eq!(lit.value, 7),
                            other => panic!("expected IntLit operand, got {:?}", other),
                        }
                    }
                    other => panic!("expected Unary(Neg, _), got {:?}", other),
                }
            }
            other => panic!(
                "expected Ok(Cast(Unary(Neg, _), _)) for `- 7i32 as i32`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // 42[1i32]? → Try(Index(IntLit, IntLit))
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::RBracket),
            tok(TokenKind::Question),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Ok(Expr::Try(t)) => match *t.expr {
                Expr::Index(i) => {
                    match *i.receiver {
                        Expr::IntLit(lit) => assert_eq!(lit.value, 42),
                        other => panic!("expected IntLit receiver, got {:?}", other),
                    }
                    match *i.idx {
                        Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                        other => panic!("expected IntLit idx, got {:?}", other),
                    }
                }
                other => panic!("expected Index, got {:?}", other),
            },
            other => panic!("expected Ok(Try) for `42[1i32]?`, got {:?}", other),
        }
        assert_eq!(p.pos, 5);
        assert!(p.errors.is_empty());

        // 42[1 (missing `]`) → Err E0100
        let mut p = Parser::new(vec![
            int_tok(42),
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_unary() {
            Err(e) => assert_eq!(e.code, ErrorCode::E0100),
            Ok(other) => panic!("expected Err(E0100) for `42[1`, got Ok({:?})", other),
        }
    }

    #[test]
    fn range_forms() {
        // 1i32 .. 2i32  → Range(Some(1), Some(2), inclusive=false)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::DotDot),
            int_tok(2),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Range(r)) => {
                assert!(!r.inclusive);
                let s = r.start.expect("start");
                assert_eq!(int_value(&s), 1);
                let e = r.end.expect("end");
                assert_eq!(int_value(&e), 2);
            }
            other => panic!("expected Range for `1..2`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // 1i32 ..= 2i32  → Range(Some(1), Some(2), inclusive=true)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::DotDotEq),
            int_tok(2),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Range(r)) => {
                assert!(r.inclusive);
                let s = r.start.expect("start");
                assert_eq!(int_value(&s), 1);
                let e = r.end.expect("end");
                assert_eq!(int_value(&e), 2);
            }
            other => panic!("expected Range for `1..=2`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // 1i32 ..  → Range(Some(1), None, inclusive=false)
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::DotDot),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Range(r)) => {
                assert!(!r.inclusive);
                let s = r.start.expect("start");
                assert_eq!(int_value(&s), 1);
                assert!(r.end.is_none());
            }
            other => panic!("expected Range for `1..`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // .. 2i32  → Range(None, Some(2), inclusive=false)
        let mut p = Parser::new(vec![
            tok(TokenKind::DotDot),
            int_tok(2),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Range(r)) => {
                assert!(!r.inclusive);
                assert!(r.start.is_none());
                let e = r.end.expect("end");
                assert_eq!(int_value(&e), 2);
            }
            other => panic!("expected Range for `..2`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // ..  → Range(None, None, inclusive=false)
        let mut p = Parser::new(vec![tok(TokenKind::DotDot), tok(TokenKind::Eof)]);
        match p.parse_expr() {
            Ok(Expr::Range(r)) => {
                assert!(!r.inclusive);
                assert!(r.start.is_none());
                assert!(r.end.is_none());
            }
            other => panic!("expected Range for `..`, got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());
    }

    #[test]
    fn closure_forms() {
        // `|| 1i32` → Closure { params: [], move_kw: false, body: IntLit(1) }
        let mut p = Parser::new(vec![
            tok(TokenKind::Pipe),
            tok(TokenKind::Pipe),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Closure(c)) => {
                assert!(c.params.is_empty());
                assert!(!c.move_kw);
                match *c.body {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit body, got {:?}", other),
                }
            }
            other => panic!("expected Closure for `|| 1i32`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // `|x| 1i32` → 1 param, move_kw=false
        let mut p = Parser::new(vec![
            tok(TokenKind::Pipe),
            tok(TokenKind::Ident("x".to_string())),
            tok(TokenKind::Pipe),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Closure(c)) => {
                assert_eq!(c.params.len(), 1);
                assert!(matches!(c.params[0], ClosureParam::Placeholder));
                assert!(!c.move_kw);
                match *c.body {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit body, got {:?}", other),
                }
            }
            other => panic!("expected Closure for `|x| 1i32`, got {:?}", other),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // `move || 1i32` → move_kw=true, no params
        let mut p = Parser::new(vec![
            tok(TokenKind::Ident("move".to_string())),
            tok(TokenKind::Pipe),
            tok(TokenKind::Pipe),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Closure(c)) => {
                assert!(c.move_kw);
                assert!(c.params.is_empty());
                match *c.body {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit body, got {:?}", other),
                }
            }
            other => panic!("expected Closure for `move || 1i32`, got {:?}", other),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // `|x: i32| 1i32` → 1 param with type-stub consumed
        let mut p = Parser::new(vec![
            tok(TokenKind::Pipe),
            tok(TokenKind::Ident("x".to_string())),
            tok(TokenKind::Colon),
            tok(TokenKind::Ident("i32".to_string())),
            tok(TokenKind::Pipe),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Closure(c)) => {
                assert_eq!(c.params.len(), 1);
                assert!(matches!(c.params[0], ClosureParam::Placeholder));
                assert!(!c.move_kw);
                match *c.body {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                    other => panic!("expected IntLit body, got {:?}", other),
                }
            }
            other => panic!("expected Closure for `|x: i32| 1i32`, got {:?}", other),
        }
        assert_eq!(p.pos, 6);
        assert!(p.errors.is_empty());

        // `|x: i32| -> i32 { 1i32 }` → block body, return-type stub consumed
        let mut p = Parser::new(vec![
            tok(TokenKind::Pipe),
            tok(TokenKind::Ident("x".to_string())),
            tok(TokenKind::Colon),
            tok(TokenKind::Ident("i32".to_string())),
            tok(TokenKind::Pipe),
            tok(TokenKind::Arrow),
            tok(TokenKind::Ident("i32".to_string())),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Closure(c)) => {
                assert_eq!(c.params.len(), 1);
                assert!(!c.move_kw);
                match *c.body {
                    Expr::Block(b) => {
                        assert!(b.stmts.is_empty());
                        let tail = b.tail.expect("tail");
                        match *tail {
                            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
                            other => panic!("expected IntLit tail, got {:?}", other),
                        }
                    }
                    other => panic!("expected Block body, got {:?}", other),
                }
            }
            other => panic!("expected Closure with Block body, got {:?}", other),
        }
        assert_eq!(p.pos, 10);
        assert!(p.errors.is_empty());

        // negative: `| 1i32` (missing closing `|`) → Err(E0100)
        let mut p = Parser::new(vec![
            tok(TokenKind::Pipe),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Err(e) => assert_eq!(e.code, ErrorCode::E0100),
            Ok(other) => panic!("expected Err(E0100) for `| 1i32`, got Ok({:?})", other),
        }
    }

    #[test]
    fn array_literal_and_repeat() {
        // []
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::ArrayLit(a)) => assert_eq!(a.elems.len(), 0),
            other => panic!("expected ArrayLit for `[]`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // [1i32]
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::ArrayLit(a)) => {
                assert_eq!(a.elems.len(), 1);
                assert_eq!(int_value(&a.elems[0]), 1);
            }
            other => panic!("expected ArrayLit for `[1i32]`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // [1i32, 2i32, 3i32]
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::Comma),
            int_tok(2),
            tok(TokenKind::Comma),
            int_tok(3),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::ArrayLit(a)) => {
                assert_eq!(a.elems.len(), 3);
                assert_eq!(int_value(&a.elems[0]), 1);
                assert_eq!(int_value(&a.elems[1]), 2);
                assert_eq!(int_value(&a.elems[2]), 3);
            }
            other => panic!(
                "expected ArrayLit for `[1i32, 2i32, 3i32]`, got {:?}",
                other
            ),
        }
        assert_eq!(p.pos, 7);
        assert!(p.errors.is_empty());

        // [1i32, 2i32,]  (trailing comma)
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::Comma),
            int_tok(2),
            tok(TokenKind::Comma),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::ArrayLit(a)) => {
                assert_eq!(a.elems.len(), 2);
                assert_eq!(int_value(&a.elems[0]), 1);
                assert_eq!(int_value(&a.elems[1]), 2);
            }
            other => panic!("expected ArrayLit for `[1i32, 2i32,]`, got {:?}", other),
        }
        assert_eq!(p.pos, 6);
        assert!(p.errors.is_empty());

        // [0i32; 4i32]
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            int_tok(0),
            tok(TokenKind::Semi),
            int_tok(4),
            tok(TokenKind::RBracket),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::ArrayRepeat(r)) => {
                match *r.value {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 0),
                    other => panic!("expected IntLit value, got {:?}", other),
                }
                match *r.count {
                    Expr::IntLit(lit) => assert_eq!(lit.value, 4),
                    other => panic!("expected IntLit count, got {:?}", other),
                }
            }
            other => panic!("expected ArrayRepeat for `[0i32; 4i32]`, got {:?}", other),
        }
        assert_eq!(p.pos, 5);
        assert!(p.errors.is_empty());

        // [1i32 (missing `]`) → Err
        let mut p = Parser::new(vec![
            tok(TokenKind::LBracket),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        assert!(p.parse_expr().is_err());
    }

    #[test]
    fn block_trailing_expr() {
        use crate::ast::stmt::Stmt;

        // {} → empty block, no tail
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Block(b)) => {
                assert!(b.stmts.is_empty());
                assert!(b.tail.is_none());
            }
            other => panic!("expected Block for `{{}}`, got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // { 1i32 } → tail = Some(IntLit), no stmts
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Block(b)) => {
                assert!(b.stmts.is_empty());
                let tail = b.tail.expect("tail");
                assert_eq!(int_value(&tail), 1);
            }
            other => panic!("expected Block for `{{ 1i32 }}`, got {:?}", other),
        }
        assert_eq!(p.pos, 3);
        assert!(p.errors.is_empty());

        // { 1i32; } → one Stmt::Expr { has_semi: true }, no tail
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Block(b)) => {
                assert_eq!(b.stmts.len(), 1);
                match &b.stmts[0] {
                    Stmt::Expr { expr, has_semi } => {
                        assert!(*has_semi);
                        assert_eq!(int_value(expr), 1);
                    }
                    other => panic!("expected Stmt::Expr, got {:?}", other),
                }
                assert!(b.tail.is_none());
            }
            other => panic!("expected Block for `{{ 1i32; }}`, got {:?}", other),
        }
        assert_eq!(p.pos, 4);
        assert!(p.errors.is_empty());

        // { 1i32; 2i32 } → one stmt with semi, tail = Some(2)
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            int_tok(2),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Block(b)) => {
                assert_eq!(b.stmts.len(), 1);
                match &b.stmts[0] {
                    Stmt::Expr { expr, has_semi } => {
                        assert!(*has_semi);
                        assert_eq!(int_value(expr), 1);
                    }
                    other => panic!("expected Stmt::Expr, got {:?}", other),
                }
                let tail = b.tail.expect("tail");
                assert_eq!(int_value(&tail), 2);
            }
            other => panic!("expected Block for `{{ 1i32; 2i32 }}`, got {:?}", other),
        }
        assert_eq!(p.pos, 5);
        assert!(p.errors.is_empty());

        // { 1i32; 2i32; } → two stmts with semi, no tail
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            int_tok(2),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::Block(b)) => {
                assert_eq!(b.stmts.len(), 2);
                match &b.stmts[0] {
                    Stmt::Expr { expr, has_semi } => {
                        assert!(*has_semi);
                        assert_eq!(int_value(expr), 1);
                    }
                    other => panic!("expected Stmt::Expr, got {:?}", other),
                }
                match &b.stmts[1] {
                    Stmt::Expr { expr, has_semi } => {
                        assert!(*has_semi);
                        assert_eq!(int_value(expr), 2);
                    }
                    other => panic!("expected Stmt::Expr, got {:?}", other),
                }
                assert!(b.tail.is_none());
            }
            other => panic!("expected Block for `{{ 1i32; 2i32; }}`, got {:?}", other),
        }
        assert_eq!(p.pos, 6);
        assert!(p.errors.is_empty());

        // { 1i32 (missing `}`) → Err
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        assert!(p.parse_expr().is_err());
    }

    #[test]
    fn if_else_chain() {
        // if true { 1i32 } → Expr::If with no else
        let mut p = Parser::new(vec![
            tok(TokenKind::If),
            tok(TokenKind::True),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::If(e)) => {
                match &*e.cond {
                    Expr::BoolLit(b) => assert!(b.value),
                    other => panic!("expected BoolLit cond, got {:?}", other),
                }
                match &*e.then {
                    Expr::Block(b) => {
                        let tail = b.tail.as_ref().expect("tail");
                        assert_eq!(int_value(tail), 1);
                    }
                    other => panic!("expected Block then, got {:?}", other),
                }
                assert!(e.else_branch.is_none());
            }
            other => panic!("expected If for `if true {{ 1 }}`, got {:?}", other),
        }
        assert!(p.errors.is_empty());

        // if true { 1i32 } else { 2i32 } → Expr::If with else_branch = Some(Block)
        let mut p = Parser::new(vec![
            tok(TokenKind::If),
            tok(TokenKind::True),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Else),
            tok(TokenKind::LBrace),
            int_tok(2),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::If(e)) => {
                let else_e = e.else_branch.as_ref().expect("else_branch");
                match &**else_e {
                    Expr::Block(b) => {
                        let tail = b.tail.as_ref().expect("tail");
                        assert_eq!(int_value(tail), 2);
                    }
                    other => panic!("expected Block else_branch, got {:?}", other),
                }
            }
            other => panic!(
                "expected If for `if true {{ 1 }} else {{ 2 }}`, got {:?}",
                other
            ),
        }
        assert!(p.errors.is_empty());

        // if true { 1i32 } else if false { 2i32 } else { 3i32 }
        let mut p = Parser::new(vec![
            tok(TokenKind::If),
            tok(TokenKind::True),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Else),
            tok(TokenKind::If),
            tok(TokenKind::False),
            tok(TokenKind::LBrace),
            int_tok(2),
            tok(TokenKind::RBrace),
            tok(TokenKind::Else),
            tok(TokenKind::LBrace),
            int_tok(3),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr() {
            Ok(Expr::If(outer)) => {
                let outer_else = outer.else_branch.as_ref().expect("outer else");
                match &**outer_else {
                    Expr::If(inner) => {
                        match &*inner.cond {
                            Expr::BoolLit(b) => assert!(!b.value),
                            other => panic!("expected BoolLit inner cond, got {:?}", other),
                        }
                        match &*inner.then {
                            Expr::Block(b) => {
                                let tail = b.tail.as_ref().expect("tail");
                                assert_eq!(int_value(tail), 2);
                            }
                            other => panic!("expected Block inner then, got {:?}", other),
                        }
                        let inner_else = inner.else_branch.as_ref().expect("inner else");
                        match &**inner_else {
                            Expr::Block(b) => {
                                let tail = b.tail.as_ref().expect("tail");
                                assert_eq!(int_value(tail), 3);
                            }
                            other => panic!("expected Block inner else, got {:?}", other),
                        }
                    }
                    other => panic!("expected If as outer else_branch, got {:?}", other),
                }
            }
            other => panic!("expected outer If for else-if chain, got {:?}", other),
        }
        assert!(p.errors.is_empty());

        // Error: if true 1i32 (non-block then)
        let mut p = Parser::new(vec![
            tok(TokenKind::If),
            tok(TokenKind::True),
            int_tok(1),
            tok(TokenKind::Eof),
        ]);
        assert!(p.parse_expr().is_err());

        // Error: if true { 1i32 } else 2i32 (non-block else)
        let mut p = Parser::new(vec![
            tok(TokenKind::If),
            tok(TokenKind::True),
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Else),
            int_tok(2),
            tok(TokenKind::Eof),
        ]);
        assert!(p.parse_expr().is_err());
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
