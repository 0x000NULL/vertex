use crate::lexer::token::IntSuffix;
use crate::span::FileId;
use crate::span::Span;

fn hex_digit_value(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => unreachable!(),
    }
}

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

    pub fn scan_int_decimal(&mut self) -> (u64, IntSuffix, Span) {
        let start = self.pos as u32;
        let mut value: u64 = 0;
        let mut overflow = false;

        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                if !overflow {
                    match value
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((b - b'0') as u64))
                    {
                        Some(v) => value = v,
                        None => {
                            overflow = true;
                        }
                    }
                }
                self.pos += 1;
            } else if b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if overflow {
            value = u64::MAX;
        }

        let suffix = self.scan_int_suffix();
        let end = self.pos as u32;
        (value, suffix, Span::new(self.file_id, start, end))
    }

    pub fn scan_int_hex(&mut self) -> Option<(u64, IntSuffix, Span)> {
        let start = self.pos as u32;
        self.pos += 2;

        let mut value: u64 = 0;
        let mut overflow = false;
        let mut saw_digit = false;

        while let Some(b) = self.peek() {
            if b.is_ascii_hexdigit() {
                saw_digit = true;
                if !overflow {
                    let d = hex_digit_value(b) as u64;
                    match value.checked_mul(16).and_then(|v| v.checked_add(d)) {
                        Some(v) => value = v,
                        None => {
                            overflow = true;
                        }
                    }
                }
                self.pos += 1;
            } else if b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if !saw_digit {
            self.pos = start as usize;
            return None;
        }

        if overflow {
            value = u64::MAX;
        }

        let suffix = self.scan_int_suffix();
        let end = self.pos as u32;
        Some((value, suffix, Span::new(self.file_id, start, end)))
    }

    pub fn scan_int_bin(&mut self) -> Option<(u64, IntSuffix, Span)> {
        let start = self.pos as u32;
        self.pos += 2;

        let mut value: u64 = 0;
        let mut overflow = false;
        let mut saw_digit = false;

        while let Some(b) = self.peek() {
            if b == b'0' || b == b'1' {
                saw_digit = true;
                if !overflow {
                    match value
                        .checked_mul(2)
                        .and_then(|v| v.checked_add((b - b'0') as u64))
                    {
                        Some(v) => value = v,
                        None => {
                            overflow = true;
                        }
                    }
                }
                self.pos += 1;
            } else if b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if !saw_digit {
            self.pos = start as usize;
            return None;
        }

        if overflow {
            value = u64::MAX;
        }

        let suffix = self.scan_int_suffix();
        let end = self.pos as u32;
        Some((value, suffix, Span::new(self.file_id, start, end)))
    }

    fn scan_int_suffix(&mut self) -> IntSuffix {
        const SUFFIXES: &[(&[u8], IntSuffix)] = &[
            (b"isize", IntSuffix::ISize),
            (b"i64", IntSuffix::I64),
            (b"i32", IntSuffix::I32),
            (b"i16", IntSuffix::I16),
            (b"i8", IntSuffix::I8),
            (b"usize", IntSuffix::USize),
            (b"u64", IntSuffix::U64),
            (b"u32", IntSuffix::U32),
            (b"u16", IntSuffix::U16),
            (b"u8", IntSuffix::U8),
        ];

        let lead = match self.peek() {
            Some(b) => b,
            None => return IntSuffix::Unsuffixed,
        };
        if lead != b'i' && lead != b'u' {
            return IntSuffix::Unsuffixed;
        }

        let rest = &self.bytes[self.pos..];
        for (s, suf) in SUFFIXES {
            if rest.starts_with(s) {
                self.pos += s.len();
                return *suf;
            }
        }
        IntSuffix::Unsuffixed
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
    fn decimal_int_with_underscores_and_suffix() {
        let cases: &[(&str, u64, IntSuffix)] = &[
            ("123", 123, IntSuffix::Unsuffixed),
            ("1_000_000", 1_000_000, IntSuffix::Unsuffixed),
            ("42u32", 42, IntSuffix::U32),
            ("0i64", 0, IntSuffix::I64),
            ("9_isize", 9, IntSuffix::ISize),
            ("1_2_3u8", 123, IntSuffix::U8),
        ];

        for (input, expected_value, expected_suffix) in cases {
            let mut s = Scanner::new(input, FileId(7));
            let (value, suffix, span) = s.scan_int_decimal();
            assert_eq!(value, *expected_value, "value for {:?}", input);
            assert_eq!(suffix, *expected_suffix, "suffix for {:?}", input);
            assert_eq!(span.file_id, FileId(7), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(
                span.end as usize,
                input.len(),
                "span.end for {:?}",
                input
            );
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }
    }

    #[test]
    fn hex_and_bin_literals() {
        let happy_hex: &[(&str, u64, IntSuffix)] = &[
            ("0x1F", 31, IntSuffix::Unsuffixed),
            ("0xff_ffu32", 65535, IntSuffix::U32),
            ("0xDEAD_BEEFi64", 0xDEAD_BEEF, IntSuffix::I64),
            ("0xAbCd", 0xABCD, IntSuffix::Unsuffixed),
        ];
        for (input, expected_value, expected_suffix) in happy_hex {
            let mut s = Scanner::new(input, FileId(3));
            let (value, suffix, span) = s.scan_int_hex().expect(input);
            assert_eq!(value, *expected_value, "value for {:?}", input);
            assert_eq!(suffix, *expected_suffix, "suffix for {:?}", input);
            assert_eq!(span.file_id, FileId(3), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let happy_bin: &[(&str, u64, IntSuffix)] = &[
            ("0b0", 0, IntSuffix::Unsuffixed),
            ("0b1010_1010", 170, IntSuffix::Unsuffixed),
            ("0b1111_1111u8", 255, IntSuffix::U8),
        ];
        for (input, expected_value, expected_suffix) in happy_bin {
            let mut s = Scanner::new(input, FileId(3));
            let (value, suffix, span) = s.scan_int_bin().expect(input);
            assert_eq!(value, *expected_value, "value for {:?}", input);
            assert_eq!(suffix, *expected_suffix, "suffix for {:?}", input);
            assert_eq!(span.file_id, FileId(3), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        for input in &["0x", "0xg", "0x_"] {
            let mut s = Scanner::new(input, FileId(0));
            assert!(s.scan_int_hex().is_none(), "expected None for {:?}", input);
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }

        for input in &["0b", "0b2", "0b_"] {
            let mut s = Scanner::new(input, FileId(0));
            assert!(s.scan_int_bin().is_none(), "expected None for {:?}", input);
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }

        let mut overflow = Scanner::new("0xFFFF_FFFF_FFFF_FFFF_F", FileId(0));
        let (value, suffix, _) = overflow.scan_int_hex().expect("overflow input");
        assert_eq!(value, u64::MAX);
        assert_eq!(suffix, IntSuffix::Unsuffixed);
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
