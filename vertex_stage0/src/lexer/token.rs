use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntSuffix {
    I8,
    I16,
    I32,
    I64,
    ISize,
    U8,
    U16,
    U32,
    U64,
    USize,
    Unsuffixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatSuffix {
    F32,
    F64,
    Unsuffixed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Break,
    Const,
    Continue,
    Else,
    Enum,
    Extern,
    False,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Mut,
    Not,
    Or,
    Pub,
    Return,
    SelfLower,
    SelfUpper,
    Struct,
    Trait,
    True,
    Type,
    Unsafe,
    Use,
    Where,
    While,
    And,
    IntLiteral(u64, IntSuffix),
    FloatLiteral(f64, FloatSuffix),
    CharLiteral(char),
    StringLiteral(String),
    RawStringLiteral(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    BangEq,
    Lt,
    Gt,
    Le,
    Ge,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Dot,
    ColonColon,
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Question,
    DotDot,
    DotDotEq,
    Arrow,
    FatArrow,
    Semi,
    Comma,
    Colon,
    Underscore,
    Eof,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}
