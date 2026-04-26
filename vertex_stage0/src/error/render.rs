use std::fmt::Write;

use crate::error::CompileError;
use crate::span::SourceMap;

// TODO: termcolor when stdout is a tty and !NO_COLOR
pub fn render(err: &CompileError, src: &SourceMap) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "error[E{:04}]: {}", err.code.0, err.message);

    let file = src.file(err.span.file);
    let (line, col) = src.line_col(err.span.file, err.span.start);
    let _ = writeln!(out, "  --> {}:{}:{}", file.name.display(), line, col);

    let line_idx = (line - 1) as usize;
    let line_start = file.line_starts[line_idx] as usize;
    let line_end = file
        .line_starts
        .get(line_idx + 1)
        .map(|&s| s as usize - 1)
        .unwrap_or(file.content.len());
    let line_text = &file.content[line_start..line_end];

    let span_end_on_line = (err.span.end as usize).min(line_end);
    let end_col = if span_end_on_line > line_start {
        file.content[line_start..span_end_on_line].chars().count() as u32 + 1
    } else {
        col
    };
    let caret_count = end_col.saturating_sub(col).max(1) as usize;

    let _ = writeln!(out, "   |");
    let _ = writeln!(out, "{:>4} | {}", line, line_text);
    let pad = " ".repeat((col - 1) as usize);
    let carets = "^".repeat(caret_count);
    let _ = writeln!(out, "     | {}{}", pad, carets);
    let _ = writeln!(out, "   |");

    for note in &err.notes {
        let _ = writeln!(out, "   = note: {}", note);
    }

    for suggestion in &err.suggestions {
        let _ = writeln!(out, "   = help: {}", suggestion.message);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorCode, ErrorKind, Suggestion};
    use crate::span::{Span, SourceMap};

    #[test]
    fn renders_e0308_format() {
        std::env::set_var("NO_COLOR", "1");

        let mut map = SourceMap::new();
        let file = map.add_file("src/main.vx", "    x + \"hello\"");

        let span = Span::new(file, 4, 15);

        let err = CompileError::new(ErrorCode::E0308, ErrorKind::Type, span, "type mismatch")
            .with_note("cannot add integer and string")
            .with_suggestion(Suggestion {
                message: "convert the string to a number with: x + \"hello\".parse()?".into(),
                replacement: None,
                span,
            });

        let rendered = render(&err, &map);

        assert!(rendered.contains("error[E0308]: type mismatch"), "header missing: {rendered}");
        assert!(rendered.contains("--> src/main.vx:1:"), "location missing: {rendered}");
        assert!(rendered.contains("    x + \"hello\""), "source line missing: {rendered}");
        assert!(rendered.contains("^"), "caret missing: {rendered}");
        assert!(
            rendered.contains("= note: cannot add integer and string"),
            "note missing: {rendered}"
        );
        assert!(
            rendered.contains("= help: convert the string to a number"),
            "help missing: {rendered}"
        );
    }
}
