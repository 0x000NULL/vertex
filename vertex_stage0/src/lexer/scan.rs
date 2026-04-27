use crate::lexer::token::FloatSuffix;
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

    pub fn scan_float(&mut self) -> Option<(f64, FloatSuffix, Span)> {
        let start = self.pos;

        match self.peek() {
            Some(b) if b.is_ascii_digit() => {}
            _ => return None,
        }

        let mut buf = String::new();

        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                buf.push(b as char);
                self.pos += 1;
            } else if b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let dot_followed_by_digit = self.peek_at(0) == Some(b'.')
            && self.peek_at(1).map_or(false, |b| b.is_ascii_digit());
        if !dot_followed_by_digit {
            self.pos = start;
            return None;
        }

        buf.push('.');
        self.pos += 1;

        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                buf.push(b as char);
                self.pos += 1;
            } else if b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if let Some(b) = self.peek() {
            if b == b'e' || b == b'E' {
                buf.push('e');
                self.pos += 1;

                if let Some(sign) = self.peek() {
                    if sign == b'+' || sign == b'-' {
                        buf.push(sign as char);
                        self.pos += 1;
                    }
                }

                let mut saw_exp_digit = false;
                while let Some(b) = self.peek() {
                    if b.is_ascii_digit() {
                        saw_exp_digit = true;
                        buf.push(b as char);
                        self.pos += 1;
                    } else if b == b'_' {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }

                if !saw_exp_digit {
                    self.pos = start;
                    return None;
                }
            }
        }

        let suffix = self.scan_float_suffix();

        let value = match <f64 as core::str::FromStr>::from_str(&buf) {
            Ok(v) => v,
            Err(_) => {
                self.pos = start;
                return None;
            }
        };

        let span = Span::new(self.file_id, start as u32, self.pos as u32);
        Some((value, suffix, span))
    }

    pub fn scan_char(&mut self) -> Option<(char, Span)> {
        if self.peek() != Some(b'\'') {
            return None;
        }
        let start = self.pos;
        self.pos += 1;

        let ch = match self.peek() {
            None | Some(b'\'') | Some(b'\n') => {
                self.pos = start;
                return None;
            }
            Some(b'\\') => match self.scan_escape_char() {
                Some(c) => c,
                None => {
                    self.pos = start;
                    return None;
                }
            },
            Some(_) => {
                let rest = &self.src[self.pos..];
                match rest.chars().next() {
                    Some(c) => {
                        self.pos += c.len_utf8();
                        c
                    }
                    None => {
                        self.pos = start;
                        return None;
                    }
                }
            }
        };

        if self.peek() != Some(b'\'') {
            self.pos = start;
            return None;
        }
        self.pos += 1;

        Some((ch, Span::new(self.file_id, start as u32, self.pos as u32)))
    }

    pub fn scan_string(&mut self) -> Option<(String, Span)> {
        if self.peek() != Some(b'"') {
            return None;
        }
        let start = self.pos;
        self.pos += 1;

        let mut buf = String::new();
        loop {
            match self.peek() {
                None => {
                    self.pos = start;
                    return None;
                }
                Some(b'"') => {
                    self.pos += 1;
                    let span = Span::new(self.file_id, start as u32, self.pos as u32);
                    return Some((buf, span));
                }
                Some(b'\\') => match self.scan_escape_char() {
                    Some(c) => buf.push(c),
                    None => {
                        self.pos = start;
                        return None;
                    }
                },
                Some(_) => {
                    let rest = &self.src[self.pos..];
                    match rest.chars().next() {
                        Some(c) => {
                            self.pos += c.len_utf8();
                            buf.push(c);
                        }
                        None => {
                            self.pos = start;
                            return None;
                        }
                    }
                }
            }
        }
    }

    fn scan_escape_char(&mut self) -> Option<char> {
        self.pos += 1;
        let kind = self.bump()?;
        match kind {
            b'n' => Some('\n'),
            b't' => Some('\t'),
            b'r' => Some('\r'),
            b'\\' => Some('\\'),
            b'\'' => Some('\''),
            b'"' => Some('"'),
            b'0' => Some('\0'),
            b'x' => {
                let h1 = self.bump()?;
                let h2 = self.bump()?;
                if !h1.is_ascii_hexdigit() || !h2.is_ascii_hexdigit() {
                    return None;
                }
                let v = (hex_digit_value(h1) as u32) * 16 + hex_digit_value(h2) as u32;
                if v > 0x7F {
                    return None;
                }
                char::from_u32(v)
            }
            b'u' => {
                if self.bump()? != b'{' {
                    return None;
                }
                let mut v: u32 = 0;
                let mut count = 0u32;
                loop {
                    let b = self.peek()?;
                    if b == b'}' {
                        break;
                    }
                    if !b.is_ascii_hexdigit() {
                        return None;
                    }
                    if count >= 6 {
                        return None;
                    }
                    v = v * 16 + hex_digit_value(b) as u32;
                    count += 1;
                    self.pos += 1;
                }
                if count == 0 {
                    return None;
                }
                self.pos += 1;
                char::from_u32(v)
            }
            _ => None,
        }
    }

    fn scan_float_suffix(&mut self) -> FloatSuffix {
        if self.peek() != Some(b'f') {
            return FloatSuffix::Unsuffixed;
        }
        let rest = &self.bytes[self.pos..];
        if rest.starts_with(b"f32") {
            self.pos += 3;
            FloatSuffix::F32
        } else if rest.starts_with(b"f64") {
            self.pos += 3;
            FloatSuffix::F64
        } else {
            FloatSuffix::Unsuffixed
        }
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
    fn float_literal_forms() {
        let happy: &[(&str, f64, FloatSuffix)] = &[
            ("1.0", 1.0, FloatSuffix::Unsuffixed),
            ("1.0e10", 1.0e10, FloatSuffix::Unsuffixed),
            ("1.0E-3", 1.0e-3, FloatSuffix::Unsuffixed),
            ("3.14f32", 3.14, FloatSuffix::F32),
            ("2.5f64", 2.5, FloatSuffix::F64),
            ("1_000.000_5", 1000.0005, FloatSuffix::Unsuffixed),
            ("1.0e+2", 100.0, FloatSuffix::Unsuffixed),
        ];

        for (input, expected_value, expected_suffix) in happy {
            let mut s = Scanner::new(input, FileId(5));
            let (value, suffix, span) = s.scan_float().expect(input);
            assert!(
                (value - *expected_value).abs() < 1e-12,
                "value for {:?}: got {}, expected {}",
                input,
                value,
                expected_value
            );
            assert_eq!(suffix, *expected_suffix, "suffix for {:?}", input);
            assert_eq!(span.file_id, FileId(5), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        for input in &[".5", "1", "1.0e"] {
            let mut s = Scanner::new(input, FileId(0));
            assert!(s.scan_float().is_none(), "expected None for {:?}", input);
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn char_literal_escapes() {
        let happy: &[(&str, char)] = &[
            ("'a'", 'a'),
            ("' '", ' '),
            ("'!'", '!'),
            ("'é'", 'é'),
            ("'\\n'", '\n'),
            ("'\\t'", '\t'),
            ("'\\r'", '\r'),
            ("'\\\\'", '\\'),
            ("'\\''", '\''),
            ("'\\\"'", '"'),
            ("'\\0'", '\0'),
            ("'\\x7F'", '\x7F'),
            ("'\\u{1F600}'", '\u{1F600}'),
        ];

        for (input, expected) in happy {
            let mut s = Scanner::new(input, FileId(9));
            let (ch, span) = s.scan_char().expect(input);
            assert_eq!(ch, *expected, "char for {:?}", input);
            assert_eq!(span.file_id, FileId(9), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let rejections: &[&str] = &[
            "''",
            "'ab'",
            "'a",
            "'",
            "'\n",
            "'\\q'",
            "'\\xZZ'",
            "'\\u{}'",
            "'\\u{D800}'",
            "'\\u{110000}'",
        ];

        for input in rejections {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                s.scan_char().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn string_literal_escapes() {
        let happy: &[(&str, &str)] = &[
            ("\"\"", ""),
            ("\"hello\"", "hello"),
            ("\"\\n\"", "\n"),
            ("\"\\t\"", "\t"),
            ("\"\\r\"", "\r"),
            ("\"\\\\\"", "\\"),
            ("\"\\\"\"", "\""),
            ("\"\\'\"", "'"),
            ("\"\\0\"", "\0"),
            ("\"\\x7F\"", "\x7F"),
            ("\"\\u{0041}\"", "A"),
            ("\"\\u{1F600}\"", "\u{1F600}"),
            ("\"é\"", "é"),
            ("\"a\nb\"", "a\nb"),
        ];

        for (input, expected) in happy {
            let mut s = Scanner::new(input, FileId(11));
            let (value, span) = s.scan_string().expect(input);
            assert_eq!(value, *expected, "value for {:?}", input);
            assert_eq!(span.file_id, FileId(11), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let rejections: &[&str] = &[
            "\"abc",
            "\"\\",
            "\"\\q\"",
            "\"\\xZZ\"",
            "\"\\xFF\"",
            "\"\\u{}\"",
            "\"\\u{D800}\"",
            "\"\\u{110000}\"",
        ];

        for input in rejections {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                s.scan_string().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }

        let mut not_string = Scanner::new("abc", FileId(0));
        assert!(not_string.scan_string().is_none());
        assert_eq!(not_string.pos, 0);
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
