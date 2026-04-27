use std::fmt::Write;

use crate::error::CompileError;
use crate::span::{SourceMap, Span};

// TODO: termcolor when stdout is a tty and !NO_COLOR
pub fn render(err: &CompileError, src: &SourceMap) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "error[E{:04}]: {}", err.code.0, err.message);

    let primary_idx = err.labels.iter().position(|l| l.primary);
    let (primary_span, primary_message): (Span, &str) = match primary_idx {
        Some(idx) => (err.labels[idx].span, err.labels[idx].message.as_str()),
        None => (err.span, ""),
    };

    let file = src.file(primary_span.file_id);
    let (line, col) = src.line_col(primary_span.file_id, primary_span.start);
    let _ = writeln!(out, "  --> {}:{}:{}", file.name.display(), line, col);

    let line_idx = (line - 1) as usize;
    let line_start = file.line_starts[line_idx] as usize;
    let line_end = file
        .line_starts
        .get(line_idx + 1)
        .map(|&s| s as usize - 1)
        .unwrap_or(file.content.len());
    let line_text = &file.content[line_start..line_end];

    let span_end_on_line = (primary_span.end as usize).min(line_end);
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
    if primary_message.is_empty() {
        let _ = writeln!(out, "     | {}{}", pad, carets);
    } else {
        let _ = writeln!(out, "     | {}{} {}", pad, carets, primary_message);
    }
    let _ = writeln!(out, "   |");

    for (i, label) in err.labels.iter().enumerate() {
        if Some(i) == primary_idx {
            continue;
        }
        let lf = src.file(label.span.file_id);
        let (lline, lcol) = src.line_col(label.span.file_id, label.span.start);
        let _ = writeln!(
            out,
            "  ::: {}:{}:{}: {}",
            lf.name.display(),
            lline,
            lcol,
            label.message
        );
    }

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
    use crate::error::{ErrorCode, ErrorKind, Label, Suggestion};
    use crate::span::{SourceMap, Span};

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

        assert!(
            rendered.contains("error[E0308]: type mismatch"),
            "header missing: {rendered}"
        );
        assert!(
            rendered.contains("--> src/main.vx:1:"),
            "location missing: {rendered}"
        );
        assert!(
            rendered.contains("    x + \"hello\""),
            "source line missing: {rendered}"
        );
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

    #[test]
    fn multi_label_layout() {
        std::env::set_var("NO_COLOR", "1");

        let mut map = SourceMap::new();
        let main = map.add_file("src/main.vx", "alpha = 1\nbeta = 2\ngamma = 3\n");
        let lib = map.add_file("src/lib.vx", "delta = 4\n");

        let primary_span = Span::new(main, 0, 5);
        let secondary_a = Span::new(main, 10, 14);
        let secondary_b = Span::new(lib, 0, 5);

        let err = CompileError::new(
            ErrorCode::E0502,
            ErrorKind::BorrowCheck,
            primary_span,
            "conflicting definitions",
        )
        .with_label(Label {
            span: primary_span,
            message: "first defined here".into(),
            primary: true,
        })
        .with_secondary_label(secondary_a, "later redefined here")
        .with_secondary_label(secondary_b, "and here");

        let rendered = render(&err, &map);

        assert!(
            rendered.contains("alpha = 1"),
            "primary snippet missing: {rendered}"
        );
        assert!(rendered.contains("^"), "caret missing: {rendered}");
        assert!(
            rendered.contains("first defined here"),
            "primary label message missing: {rendered}"
        );

        assert!(rendered.contains(":::"), "::: prefix missing: {rendered}");
        assert!(
            rendered.contains("src/main.vx:2:1: later redefined here"),
            "secondary a reference missing: {rendered}"
        );
        assert!(
            rendered.contains("src/lib.vx:1:1: and here"),
            "secondary b reference missing: {rendered}"
        );

        assert!(
            !rendered.contains("beta = 2"),
            "secondary snippet a leaked into render: {rendered}"
        );
        assert!(
            !rendered.contains("delta = 4"),
            "secondary snippet b leaked into render: {rendered}"
        );
    }
}
