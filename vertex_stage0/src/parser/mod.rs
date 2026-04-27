use std::mem;

use crate::ast::NodeId;
use crate::error::{CompileError, ErrorAccumulator, ErrorCode, ErrorKind};
use crate::lexer::token::{Token, TokenKind};
use crate::span::Span;

pub mod expr;
pub mod stmt;

static EOF_KIND: TokenKind = TokenKind::Eof;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub errors: ErrorAccumulator,
    next_node_id: u32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: ErrorAccumulator::new(),
            next_node_id: 0,
        }
    }

    fn new_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id = self.next_node_id.wrapping_add(1);
        NodeId(id)
    }

    pub fn peek(&self) -> &TokenKind {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos].kind
        } else {
            &EOF_KIND
        }
    }

    pub fn peek_at(&self, offset: usize) -> &TokenKind {
        let idx = self.pos.saturating_add(offset);
        if idx < self.tokens.len() {
            &self.tokens[idx].kind
        } else {
            &EOF_KIND
        }
    }

    pub fn bump(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() {
            self.pos = (self.pos + 1).min(self.tokens.len());
        }
        tok
    }

    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        if mem::discriminant(self.peek()) == mem::discriminant(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: &TokenKind) -> Result<Token, CompileError> {
        if mem::discriminant(self.peek()) == mem::discriminant(kind) {
            Ok(self.bump())
        } else {
            let found = self.peek();
            let span = self.current_span();
            let message = format!("expected {}, found {}", describe(kind), describe(found));
            Err(CompileError::new(
                ErrorCode::E0100,
                ErrorKind::Syntax,
                span,
                message,
            ))
        }
    }

    pub fn expect_one_of(&mut self, kinds: &[TokenKind]) -> Result<Token, CompileError> {
        let peeked = mem::discriminant(self.peek());
        if kinds.iter().any(|k| mem::discriminant(k) == peeked) {
            Ok(self.bump())
        } else {
            let found = self.peek();
            let span = self.current_span();
            let message = format!(
                "expected {}, found {}",
                format_candidate_list(kinds),
                describe(found),
            );
            Err(CompileError::new(
                ErrorCode::E0100,
                ErrorKind::Syntax,
                span,
                message,
            ))
        }
    }

    pub fn expected_token_error(&mut self, expected: &TokenKind) {
        let found = self.peek();
        let span = self.current_span();
        let message = format!("expected {}, found {}", describe(expected), describe(found));
        let err = CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, span, message);
        self.errors.push(err);
        self.recover_to_sync();
    }

    pub fn expected_one_of_error(&mut self, kinds: &[TokenKind]) {
        let found = self.peek();
        let span = self.current_span();
        let message = format!(
            "expected {}, found {}",
            format_candidate_list(kinds),
            describe(found),
        );
        let err = CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, span, message);
        self.errors.push(err);
        self.recover_to_sync();
    }

    pub fn recover_to_sync(&mut self) {
        while !is_sync_point(self.peek()) {
            self.bump();
        }
        if matches!(self.peek(), TokenKind::Semi) {
            self.bump();
        }
    }

    fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            Span::new(crate::span::FileId(0), 0, 0)
        }
    }
}

fn is_sync_point(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Semi
            | TokenKind::RBrace
            | TokenKind::Eof
            | TokenKind::Fn
            | TokenKind::Struct
            | TokenKind::Enum
            | TokenKind::Trait
            | TokenKind::Impl
            | TokenKind::Mod
            | TokenKind::Use
            | TokenKind::Const
            | TokenKind::Type
            | TokenKind::Pub
            | TokenKind::Unsafe
            | TokenKind::Extern
    )
}

fn format_candidate_list(kinds: &[TokenKind]) -> String {
    debug_assert!(!kinds.is_empty(), "format_candidate_list: empty candidate slice");
    match kinds {
        [] => "token".to_string(),
        [a] => describe(a).to_string(),
        [a, b] => format!("{} or {}", describe(a), describe(b)),
        [head @ .., last] => {
            let mut out = String::new();
            for k in head {
                out.push_str(describe(k));
                out.push_str(", ");
            }
            out.push_str("or ");
            out.push_str(describe(last));
            out
        }
    }
}

fn describe(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Break => "`break`",
        TokenKind::Const => "`const`",
        TokenKind::Continue => "`continue`",
        TokenKind::Else => "`else`",
        TokenKind::Enum => "`enum`",
        TokenKind::Extern => "`extern`",
        TokenKind::False => "`false`",
        TokenKind::Fn => "`fn`",
        TokenKind::For => "`for`",
        TokenKind::If => "`if`",
        TokenKind::Impl => "`impl`",
        TokenKind::In => "`in`",
        TokenKind::Let => "`let`",
        TokenKind::Loop => "`loop`",
        TokenKind::Match => "`match`",
        TokenKind::Mod => "`mod`",
        TokenKind::Mut => "`mut`",
        TokenKind::Not => "`not`",
        TokenKind::Or => "`or`",
        TokenKind::Pub => "`pub`",
        TokenKind::Return => "`return`",
        TokenKind::SelfLower => "`self`",
        TokenKind::SelfUpper => "`Self`",
        TokenKind::Struct => "`struct`",
        TokenKind::Trait => "`trait`",
        TokenKind::True => "`true`",
        TokenKind::Type => "`type`",
        TokenKind::Unsafe => "`unsafe`",
        TokenKind::Use => "`use`",
        TokenKind::Where => "`where`",
        TokenKind::While => "`while`",
        TokenKind::And => "`and`",
        TokenKind::IntLiteral(_, _) => "integer literal",
        TokenKind::FloatLiteral(_, _) => "float literal",
        TokenKind::CharLiteral(_) => "char literal",
        TokenKind::StringLiteral(_) => "string literal",
        TokenKind::RawStringLiteral(_) => "raw string literal",
        TokenKind::DocComment(_, _) => "doc comment",
        TokenKind::Ident(_) => "identifier",
        TokenKind::Plus => "`+`",
        TokenKind::Minus => "`-`",
        TokenKind::Star => "`*`",
        TokenKind::Slash => "`/`",
        TokenKind::Percent => "`%`",
        TokenKind::EqEq => "`==`",
        TokenKind::BangEq => "`!=`",
        TokenKind::Lt => "`<`",
        TokenKind::Gt => "`>`",
        TokenKind::Le => "`<=`",
        TokenKind::Ge => "`>=`",
        TokenKind::Amp => "`&`",
        TokenKind::Pipe => "`|`",
        TokenKind::Caret => "`^`",
        TokenKind::Tilde => "`~`",
        TokenKind::Shl => "`<<`",
        TokenKind::Shr => "`>>`",
        TokenKind::Eq => "`=`",
        TokenKind::PlusEq => "`+=`",
        TokenKind::MinusEq => "`-=`",
        TokenKind::StarEq => "`*=`",
        TokenKind::SlashEq => "`/=`",
        TokenKind::PercentEq => "`%=`",
        TokenKind::Dot => "`.`",
        TokenKind::ColonColon => "`::`",
        TokenKind::LBracket => "`[`",
        TokenKind::RBracket => "`]`",
        TokenKind::LParen => "`(`",
        TokenKind::RParen => "`)`",
        TokenKind::LBrace => "`{`",
        TokenKind::RBrace => "`}`",
        TokenKind::Question => "`?`",
        TokenKind::DotDot => "`..`",
        TokenKind::DotDotEq => "`..=`",
        TokenKind::Arrow => "`->`",
        TokenKind::FatArrow => "`=>`",
        TokenKind::Semi => "`;`",
        TokenKind::Comma => "`,`",
        TokenKind::Colon => "`:`",
        TokenKind::Underscore => "`_`",
        TokenKind::Eof => "end of file",
        TokenKind::Error(_) => "lexer error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expr;
    use crate::lexer::token::IntSuffix;
    use crate::span::{FileId, Span};

    fn tok(kind: TokenKind) -> Token {
        Token::new(kind, Span::new(FileId(0), 0, 0))
    }

    #[test]
    fn peek_and_bump_basics() {
        let mut p = Parser::new(vec![
            tok(TokenKind::Plus),
            tok(TokenKind::Minus),
            tok(TokenKind::Eof),
        ]);

        assert_eq!(p.peek(), &TokenKind::Plus);
        assert_eq!(p.peek_at(1), &TokenKind::Minus);

        let t = p.bump();
        assert_eq!(t.kind, TokenKind::Plus);

        assert_eq!(p.peek(), &TokenKind::Minus);
        assert!(p.eat(&TokenKind::Minus));
        assert!(!p.eat(&TokenKind::Star));
        assert_eq!(p.peek(), &TokenKind::Eof);
        assert!(p.expect(&TokenKind::Eof).is_ok());
    }

    #[test]
    fn recovery_advances_past_garbage() {
        let mut p = Parser::new(vec![
            tok(TokenKind::Plus),
            tok(TokenKind::Star),
            tok(TokenKind::Star),
            tok(TokenKind::Semi),
            tok(TokenKind::Fn),
            tok(TokenKind::Eof),
        ]);
        p.recover_to_sync();
        assert_eq!(p.peek(), &TokenKind::Fn);

        let mut p = Parser::new(vec![
            tok(TokenKind::Plus),
            tok(TokenKind::Star),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        p.recover_to_sync();
        assert_eq!(p.peek(), &TokenKind::RBrace);

        let mut p = Parser::new(vec![
            tok(TokenKind::Plus),
            tok(TokenKind::Star),
            tok(TokenKind::Fn),
            tok(TokenKind::Eof),
        ]);
        p.recover_to_sync();
        assert_eq!(p.peek(), &TokenKind::Fn);
    }

    #[test]
    fn expected_message_lists_candidates() {
        let mut p = Parser::new(vec![tok(TokenKind::Semi), tok(TokenKind::Eof)]);
        let err = p
            .expect_one_of(&[TokenKind::Plus])
            .expect_err("mismatch should error");
        assert_eq!(err.code, ErrorCode::E0100);
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert!(
            err.message.contains("expected `+`, found `;`"),
            "single-kind message: {}",
            err.message,
        );

        let mut p = Parser::new(vec![tok(TokenKind::Semi), tok(TokenKind::Eof)]);
        let err = p
            .expect_one_of(&[TokenKind::Comma, TokenKind::RParen])
            .expect_err("mismatch should error");
        assert_eq!(err.code, ErrorCode::E0100);
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert!(
            err.message.contains("expected `,` or `)`, found `;`"),
            "two-kind message: {}",
            err.message,
        );

        let mut p = Parser::new(vec![tok(TokenKind::RBrace), tok(TokenKind::Eof)]);
        let err = p
            .expect_one_of(&[TokenKind::Comma, TokenKind::Semi, TokenKind::RBracket])
            .expect_err("mismatch should error");
        assert_eq!(err.code, ErrorCode::E0100);
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert!(
            err.message
                .contains("expected `,`, `;`, or `]`, found `}`"),
            "three-kind message: {}",
            err.message,
        );

        let mut p = Parser::new(vec![tok(TokenKind::Comma), tok(TokenKind::Eof)]);
        let start_pos = p.pos;
        let t = p
            .expect_one_of(&[TokenKind::Comma, TokenKind::Semi])
            .expect("matching kind should return Ok");
        assert_eq!(t.kind, TokenKind::Comma);
        assert_eq!(p.pos, start_pos + 1);
    }

    #[test]
    fn error_node_recovery() {
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            tok(TokenKind::Comma),
            tok(TokenKind::Semi),
            tok(TokenKind::IntLiteral(1, IntSuffix::I32)),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let block = match p.parse_block().expect("parse_block") {
            Expr::Block(b) => b,
            other => panic!("expected Block, got {:?}", other),
        };
        assert_eq!(block.stmts.len(), 1);
        match &block.stmts[0] {
            crate::ast::stmt::Stmt::Expr { expr, .. } => {
                assert!(matches!(expr, Expr::Error(_, _)));
            }
            other => panic!("expected Stmt::Expr, got {:?}", other),
        }
        let tail = block.tail.expect("tail");
        match &*tail {
            Expr::IntLit(lit) => assert_eq!(lit.value, 1),
            other => panic!("expected IntLit tail, got {:?}", other),
        }
        assert_eq!(p.errors.len(), 1);
        assert_eq!(p.peek(), &TokenKind::Eof);
        let errs = std::mem::take(&mut p.errors)
            .into_result(())
            .expect_err("expected accumulated error");
        assert_eq!(errs[0].code, ErrorCode::E0100);
        assert_eq!(errs[0].kind, ErrorKind::Syntax);
    }
}
