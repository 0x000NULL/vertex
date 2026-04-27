use std::collections::HashSet;

use crate::span::{FileId, Span};

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Debug)]
pub struct ErrorAccumulator {
    errors: Vec<CompileError>,
    seen: HashSet<(ErrorCode, FileId, u32)>,
    dropped: u32,
}

impl ErrorAccumulator {
    pub const MAX_ERRORS: usize = 100;

    pub fn new() -> Self {
        ErrorAccumulator {
            errors: Vec::new(),
            seen: HashSet::new(),
            dropped: 0,
        }
    }

    pub fn push(&mut self, e: CompileError) {
        let key = (e.code, e.span.file_id, e.span.start);
        if self.seen.contains(&key) {
            return;
        }
        if self.errors.len() >= Self::MAX_ERRORS {
            self.dropped += 1;
            return;
        }
        self.seen.insert(key);
        self.errors.push(e);
    }

    pub fn into_result<T>(self, ok: T) -> Result<T, Vec<CompileError>> {
        if self.errors.is_empty() {
            Ok(ok)
        } else {
            Err(self.errors)
        }
    }

    pub fn dropped(&self) -> u32 {
        self.dropped
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ErrorAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, Span};

    fn make_err(code: ErrorCode, file_id: FileId, start: u32) -> CompileError {
        CompileError::new(
            code,
            ErrorKind::Other,
            Span::new(file_id, start, start),
            "test",
        )
    }

    #[test]
    fn accumulator_caps_at_100() {
        let file_id = FileId(0);
        let mut acc = ErrorAccumulator::new();
        for i in 0..150u32 {
            acc.push(make_err(ErrorCode::E0001, file_id, i));
        }
        assert_eq!(acc.len(), 100);
        assert_eq!(acc.dropped(), 50);

        let result = acc.into_result(());
        match result {
            Err(v) => assert_eq!(v.len(), 100),
            Ok(()) => panic!("expected Err"),
        }
    }

    #[test]
    fn accumulator_dedupes() {
        let file_id = FileId(0);
        let mut acc = ErrorAccumulator::new();

        for _ in 0..5 {
            acc.push(make_err(ErrorCode::E0001, file_id, 10));
        }
        acc.push(make_err(ErrorCode::E0002, file_id, 10));
        acc.push(make_err(ErrorCode::E0001, file_id, 20));

        let result = acc.into_result(());
        match result {
            Err(v) => assert_eq!(v.len(), 3),
            Ok(()) => panic!("expected Err"),
        }
    }
}
