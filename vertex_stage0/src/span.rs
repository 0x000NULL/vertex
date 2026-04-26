use std::path::{Path, PathBuf};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileId(pub u32);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Span {
        Span { file, start, end }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub name: PathBuf,
    pub content: String,
    pub line_starts: Vec<u32>,
}

fn compute_line_starts(content: &str) -> Vec<u32> {
    let mut starts = Vec::with_capacity(1 + content.matches('\n').count());
    starts.push(0);
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' {
            starts.push((i + 1) as u32);
        }
    }
    starts
}

#[derive(Default, Debug)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap { files: Vec::new() }
    }

    pub fn add_file(
        &mut self,
        name: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> FileId {
        let id = FileId(self.files.len() as u32);
        let content = content.into();
        let line_starts = compute_line_starts(&content);
        self.files.push(SourceFile {
            id,
            name: name.into(),
            content,
            line_starts,
        });
        id
    }

    pub fn file(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }

    pub fn snippet(&self, span: Span) -> &str {
        &self.file(span.file).content[span.start as usize..span.end as usize]
    }

    pub fn line_col(&self, file: FileId, byte_offset: u32) -> (u32, u32) {
        let f = self.file(file);
        let line_idx = f.line_starts.partition_point(|&s| s <= byte_offset) - 1;
        let line_start = f.line_starts[line_idx] as usize;
        let col = f.content[line_start..byte_offset as usize].chars().count() as u32 + 1;
        (line_idx as u32 + 1, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_map_round_trip_ascii_and_utf8() {
        let mut map = SourceMap::new();

        let ascii_id = map.add_file("a.vx", "abc\ndef\nghi");
        assert_eq!(ascii_id, FileId(0));
        assert_eq!(map.snippet(Span::new(ascii_id, 0, 3)), "abc");
        assert_eq!(map.snippet(Span::new(ascii_id, 4, 7)), "def");
        assert_eq!(map.line_col(ascii_id, 0), (1, 1));
        assert_eq!(map.line_col(ascii_id, 4), (2, 1));
        assert_eq!(map.line_col(ascii_id, 5), (2, 2));
        assert_eq!(map.line_col(ascii_id, 8), (3, 1));

        let utf8_id = map.add_file("u.vx", "αβ\nγδε");
        assert_eq!(utf8_id, FileId(1));
        assert_eq!(map.snippet(Span::new(utf8_id, 0, 4)), "αβ");
        assert_eq!(map.line_col(utf8_id, 2), (1, 2));
        assert_eq!(map.line_col(utf8_id, 5), (2, 1));
        assert_eq!(map.line_col(utf8_id, 9), (2, 3));
    }

    #[test]
    fn line_col_handles_multibyte() {
        let mut map = SourceMap::new();
        let id = map.add_file("m.vx", "a—b\n😀c");

        assert_eq!(map.snippet(Span::new(id, 1, 4)), "—");
        assert_eq!(map.snippet(Span::new(id, 6, 10)), "😀");

        assert_eq!(map.line_col(id, 0), (1, 1));
        assert_eq!(map.line_col(id, 1), (1, 2));
        assert_eq!(map.line_col(id, 4), (1, 3));
        assert_eq!(map.line_col(id, 6), (2, 1));
        assert_eq!(map.line_col(id, 10), (2, 2));
    }
}
