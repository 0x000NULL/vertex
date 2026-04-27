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
    /// Invalid character in source.
    pub const E0001: ErrorCode = ErrorCode(1);
    /// Unterminated string literal.
    pub const E0002: ErrorCode = ErrorCode(2);
    /// Invalid numeric literal.
    pub const E0003: ErrorCode = ErrorCode(3);

    /// Unexpected token.
    pub const E0100: ErrorCode = ErrorCode(100);
    /// Unclosed delimiter.
    pub const E0101: ErrorCode = ErrorCode(101);
    /// Missing semicolon.
    pub const E0102: ErrorCode = ErrorCode(102);

    /// Trait bound not satisfied (rustc-style code; outside the type band).
    pub const E0277: ErrorCode = ErrorCode(277);
    /// Type mismatch (rustc-style code; outside the type band).
    pub const E0308: ErrorCode = ErrorCode(308);
    /// Binary operation not supported for these types.
    pub const E0369: ErrorCode = ErrorCode(369);
    /// Use of moved value.
    pub const E0382: ErrorCode = ErrorCode(382);
    /// Unresolved name.
    pub const E0425: ErrorCode = ErrorCode(425);
    /// Failed to resolve import.
    pub const E0433: ErrorCode = ErrorCode(433);
    /// Cannot borrow as mutable more than once at a time.
    pub const E0499: ErrorCode = ErrorCode(499);
    /// Cannot borrow as mutable because it is also borrowed as immutable.
    pub const E0502: ErrorCode = ErrorCode(502);
    /// Cannot use a value because it was mutably borrowed.
    pub const E0503: ErrorCode = ErrorCode(503);
    /// Cannot move out of a value because it is borrowed.
    pub const E0505: ErrorCode = ErrorCode(505);
    /// Method not found.
    pub const E0599: ErrorCode = ErrorCode(599);
    /// Cannot index into a value of type `str`.
    pub const E0608: ErrorCode = ErrorCode(608);

    /// Internal compiler error / placeholder.
    pub const E1000: ErrorCode = ErrorCode(1000);
    /// Const evaluation failed.
    pub const E1001: ErrorCode = ErrorCode(1001);
    /// Unsafe operation in const context.
    pub const E1002: ErrorCode = ErrorCode(1002);
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
