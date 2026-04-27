use crate::span::Span;

pub mod render;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub primary: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub const E0308: ErrorCode = ErrorCode(308);
    pub const E0502: ErrorCode = ErrorCode(502);
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    NameResolution,
    Type,
    BorrowCheck,
    Other,
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub code: ErrorCode,
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
    pub labels: Vec<Label>,
    pub suggestions: Vec<Suggestion>,
    pub notes: Vec<String>,
}

impl CompileError {
    pub fn new(code: ErrorCode, kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        CompileError {
            code,
            kind,
            span,
            message: message.into(),
            labels: Vec::new(),
            suggestions: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
            primary: false,
        });
        self
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}
