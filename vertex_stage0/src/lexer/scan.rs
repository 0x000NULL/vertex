use crate::span::FileId;

pub struct Scanner<'a> {
    pub src: &'a str,
    pub bytes: &'a [u8],
    pub pos: usize,
    pub file_id: FileId,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str, file_id: FileId) -> Self {
        Scanner {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            file_id,
        }
    }

    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    pub fn bump(&mut self) -> Option<u8> {
        let b = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    pub fn eat_while<F: Fn(u8) -> bool>(&mut self, pred: F) {
        while let Some(b) = self.peek() {
            if !pred(b) {
                break;
            }
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_cursor_at_zero() {
        let s = Scanner::new("abc", FileId(0));
        assert_eq!(s.src, "abc");
        assert_eq!(s.bytes, b"abc");
        assert_eq!(s.pos, 0);
        assert_eq!(s.file_id, FileId(0));
    }

    #[test]
    fn peek_and_peek_at_return_none_past_eof() {
        let s = Scanner::new("xy", FileId(1));
        assert_eq!(s.peek(), Some(b'x'));
        assert_eq!(s.peek_at(0), Some(b'x'));
        assert_eq!(s.peek_at(1), Some(b'y'));
        assert_eq!(s.peek_at(2), None);
        assert_eq!(s.peek_at(99), None);

        let empty = Scanner::new("", FileId(0));
        assert_eq!(empty.peek(), None);
        assert_eq!(empty.peek_at(0), None);
    }

    #[test]
    fn bump_advances_pos_and_returns_none_at_eof() {
        let mut s = Scanner::new("ab", FileId(0));
        assert_eq!(s.bump(), Some(b'a'));
        assert_eq!(s.pos, 1);
        assert_eq!(s.bump(), Some(b'b'));
        assert_eq!(s.pos, 2);
        assert_eq!(s.bump(), None);
        assert_eq!(s.pos, 2);
    }

    #[test]
    fn eat_while_consumes_run() {
        let mut s = Scanner::new("   \t\nrest", FileId(0));
        s.eat_while(|b| b == b' ' || b == b'\t' || b == b'\n');
        assert_eq!(s.pos, 5);
        assert_eq!(s.peek(), Some(b'r'));

        let mut none = Scanner::new("abc", FileId(0));
        none.eat_while(|b| b == b' ');
        assert_eq!(none.pos, 0);

        let mut all = Scanner::new("   ", FileId(0));
        all.eat_while(|b| b == b' ');
        assert_eq!(all.pos, 3);
        assert_eq!(all.peek(), None);
    }
}
