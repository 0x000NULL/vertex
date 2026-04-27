use std::mem;

use crate::error::{CompileError, ErrorAccumulator, ErrorCode, ErrorKind};
use crate::lexer::token::{Token, TokenKind};
use crate::span::Span;

static EOF_KIND: TokenKind = TokenKind::Eof;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub errors: ErrorAccumulator,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: ErrorAccumulator::new(),
        }
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
            let message = format!(
                "expected {}, found {}",
                describe(kind),
                describe(found)
            );
            Err(CompileError::new(
                ErrorCode::E0100,
                ErrorKind::Syntax,
                span,
                message,
            ))
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
}
