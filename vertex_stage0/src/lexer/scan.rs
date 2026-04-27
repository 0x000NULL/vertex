use crate::lexer::token::DocStyle;
use crate::lexer::token::FloatSuffix;
use crate::lexer::token::IntSuffix;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
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

fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

pub enum ScanStringOutcome {
    Ok(String, Span),
    Unterminated(Span),
    Failed,
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

    pub fn skip_comments(&mut self) -> bool {
        let start = self.pos;

        if self.peek() == Some(b'/') && self.peek_at(1) == Some(b'/') {
            match self.peek_at(2) {
                Some(b'/') | Some(b'!') => return false,
                _ => {}
            }
            self.pos += 2;
            self.eat_while(|b| b != b'\n');
            return true;
        }

        if self.peek() == Some(b'/') && self.peek_at(1) == Some(b'*') {
            self.pos += 2;
            let mut depth: usize = 1;
            loop {
                match (self.peek(), self.peek_at(1)) {
                    (Some(b'/'), Some(b'*')) => {
                        self.pos += 2;
                        depth += 1;
                    }
                    (Some(b'*'), Some(b'/')) => {
                        self.pos += 2;
                        depth -= 1;
                        if depth == 0 {
                            return true;
                        }
                    }
                    (Some(_), _) => {
                        self.pos += 1;
                    }
                    (None, _) => {
                        self.pos = start;
                        return false;
                    }
                }
            }
        }

        false
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

    pub fn scan_string(&mut self) -> ScanStringOutcome {
        if self.peek() != Some(b'"') {
            return ScanStringOutcome::Failed;
        }
        let start = self.pos;
        self.pos += 1;

        let mut buf = String::new();
        loop {
            match self.peek() {
                None => {
                    self.pos = self.bytes.len();
                    let span = Span::new(self.file_id, start as u32, self.pos as u32);
                    return ScanStringOutcome::Unterminated(span);
                }
                Some(b'"') => {
                    self.pos += 1;
                    let span = Span::new(self.file_id, start as u32, self.pos as u32);
                    return ScanStringOutcome::Ok(buf, span);
                }
                Some(b'\\') => match self.scan_escape_char() {
                    Some(c) => buf.push(c),
                    None => {
                        self.pos = start;
                        return ScanStringOutcome::Failed;
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
                            return ScanStringOutcome::Failed;
                        }
                    }
                }
            }
        }
    }

    pub fn scan_raw_string(&mut self) -> Option<(String, Span)> {
        if self.peek() != Some(b'r') {
            return None;
        }
        let start = self.pos;
        self.pos += 1;

        let mut hash_count: usize = 0;
        while self.peek() == Some(b'#') {
            hash_count += 1;
            self.pos += 1;
        }

        if self.peek() != Some(b'"') {
            self.pos = start;
            return None;
        }
        self.pos += 1;
        let content_start = self.pos;

        loop {
            match self.peek() {
                None => {
                    self.pos = start;
                    return None;
                }
                Some(b'"') => {
                    let mut all_match = true;
                    for i in 0..hash_count {
                        if self.peek_at(1 + i) != Some(b'#') {
                            all_match = false;
                            break;
                        }
                    }
                    if all_match {
                        let content_end = self.pos;
                        self.pos += 1 + hash_count;
                        let content = String::from(&self.src[content_start..content_end]);
                        let span = Span::new(self.file_id, start as u32, self.pos as u32);
                        return Some((content, span));
                    } else {
                        self.pos += 1;
                    }
                }
                Some(_) => {
                    let rest = &self.src[self.pos..];
                    match rest.chars().next() {
                        Some(c) => {
                            self.pos += c.len_utf8();
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

    pub fn scan_operator(&mut self) -> Option<(TokenKind, Span)> {
        let start = self.pos as u32;
        let first = self.peek()?;

        let (kind, len) = match first {
            b'.' => {
                if self.peek_at(1) == Some(b'.') && self.peek_at(2) == Some(b'=') {
                    (TokenKind::DotDotEq, 3)
                } else if self.peek_at(1) == Some(b'.') {
                    (TokenKind::DotDot, 2)
                } else {
                    (TokenKind::Dot, 1)
                }
            }
            b'<' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::Le, 2)
                } else if self.peek_at(1) == Some(b'<') {
                    (TokenKind::Shl, 2)
                } else {
                    (TokenKind::Lt, 1)
                }
            }
            b'>' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::Ge, 2)
                } else if self.peek_at(1) == Some(b'>') {
                    (TokenKind::Shr, 2)
                } else {
                    (TokenKind::Gt, 1)
                }
            }
            b'=' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::EqEq, 2)
                } else if self.peek_at(1) == Some(b'>') {
                    (TokenKind::FatArrow, 2)
                } else {
                    (TokenKind::Eq, 1)
                }
            }
            b'-' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::MinusEq, 2)
                } else if self.peek_at(1) == Some(b'>') {
                    (TokenKind::Arrow, 2)
                } else {
                    (TokenKind::Minus, 1)
                }
            }
            b'+' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::PlusEq, 2)
                } else {
                    (TokenKind::Plus, 1)
                }
            }
            b'*' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::StarEq, 2)
                } else {
                    (TokenKind::Star, 1)
                }
            }
            b'/' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::SlashEq, 2)
                } else {
                    (TokenKind::Slash, 1)
                }
            }
            b'%' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::PercentEq, 2)
                } else {
                    (TokenKind::Percent, 1)
                }
            }
            b'!' => {
                if self.peek_at(1) == Some(b'=') {
                    (TokenKind::BangEq, 2)
                } else {
                    return None;
                }
            }
            b':' => {
                if self.peek_at(1) == Some(b':') {
                    (TokenKind::ColonColon, 2)
                } else {
                    (TokenKind::Colon, 1)
                }
            }
            b'&' => (TokenKind::Amp, 1),
            b'|' => (TokenKind::Pipe, 1),
            b'^' => (TokenKind::Caret, 1),
            b'~' => (TokenKind::Tilde, 1),
            b'(' => (TokenKind::LParen, 1),
            b')' => (TokenKind::RParen, 1),
            b'[' => (TokenKind::LBracket, 1),
            b']' => (TokenKind::RBracket, 1),
            b'{' => (TokenKind::LBrace, 1),
            b'}' => (TokenKind::RBrace, 1),
            b'?' => (TokenKind::Question, 1),
            b';' => (TokenKind::Semi, 1),
            b',' => (TokenKind::Comma, 1),
            _ => return None,
        };

        self.pos += len;
        Some((kind, Span::new(self.file_id, start, self.pos as u32)))
    }

    pub fn scan_ident_or_keyword(&mut self) -> Option<(TokenKind, Span)> {
        let start = self.pos;
        let first = self.peek()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        self.pos += 1;
        self.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');

        let lex = &self.src[start..self.pos];
        let kind = match lex {
            "and" => TokenKind::And,
            "break" => TokenKind::Break,
            "const" => TokenKind::Const,
            "continue" => TokenKind::Continue,
            "else" => TokenKind::Else,
            "enum" => TokenKind::Enum,
            "extern" => TokenKind::Extern,
            "false" => TokenKind::False,
            "fn" => TokenKind::Fn,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "impl" => TokenKind::Impl,
            "in" => TokenKind::In,
            "let" => TokenKind::Let,
            "loop" => TokenKind::Loop,
            "match" => TokenKind::Match,
            "mod" => TokenKind::Mod,
            "mut" => TokenKind::Mut,
            "not" => TokenKind::Not,
            "or" => TokenKind::Or,
            "pub" => TokenKind::Pub,
            "return" => TokenKind::Return,
            "self" => TokenKind::SelfLower,
            "Self" => TokenKind::SelfUpper,
            "struct" => TokenKind::Struct,
            "trait" => TokenKind::Trait,
            "true" => TokenKind::True,
            "type" => TokenKind::Type,
            "unsafe" => TokenKind::Unsafe,
            "use" => TokenKind::Use,
            "where" => TokenKind::Where,
            "while" => TokenKind::While,
            _ => TokenKind::Ident(lex.to_string()),
        };
        Some((kind, Span::new(self.file_id, start as u32, self.pos as u32)))
    }

    pub fn scan_doc_comment(&mut self) -> Option<(String, DocStyle, Span)> {
        if self.peek() != Some(b'/') || self.peek_at(1) != Some(b'/') {
            return None;
        }
        let style = match self.peek_at(2) {
            Some(b'/') => DocStyle::Outer,
            Some(b'!') => DocStyle::Inner,
            _ => return None,
        };

        let start = self.pos;
        self.pos += 3;

        let content_start = self.pos;
        self.eat_while(|b| b != b'\n');
        let content_end = self.pos;

        let content = String::from(&self.src[content_start..content_end]);
        let span = Span::new(self.file_id, start as u32, self.pos as u32);
        Some((content, style, span))
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

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let before = self.pos;
            self.eat_while(is_whitespace);
            let _ = self.skip_comments();
            if self.pos == before {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.pos as u32;
        let first = match self.peek() {
            Some(b) => b,
            None => return Token::new(TokenKind::Eof, Span::new(self.file_id, start, start)),
        };

        if first == b'/' {
            if let Some((body, style, span)) = self.scan_doc_comment() {
                return Token::new(TokenKind::DocComment(body, style), span);
            }
        }

        if first == b'r' && matches!(self.peek_at(1), Some(b'#') | Some(b'"')) {
            if let Some((s, span)) = self.scan_raw_string() {
                return Token::new(TokenKind::RawStringLiteral(s), span);
            }
        }

        if first == b'"' {
            match self.scan_string() {
                ScanStringOutcome::Ok(s, span) => {
                    return Token::new(TokenKind::StringLiteral(s), span);
                }
                ScanStringOutcome::Unterminated(span) => {
                    return Token::new(
                        TokenKind::Error("unterminated string literal".to_string()),
                        span,
                    );
                }
                ScanStringOutcome::Failed => {
                    self.pos += 1;
                    let span = Span::new(self.file_id, start, self.pos as u32);
                    return Token::new(TokenKind::Error("\"".to_string()), span);
                }
            }
        }

        if first == b'\'' {
            match self.scan_char() {
                Some((c, span)) => return Token::new(TokenKind::CharLiteral(c), span),
                None => {
                    self.pos += 1;
                    let span = Span::new(self.file_id, start, self.pos as u32);
                    return Token::new(TokenKind::Error("'".to_string()), span);
                }
            }
        }

        if first.is_ascii_digit() {
            if first == b'0' {
                match self.peek_at(1) {
                    Some(b'x') | Some(b'X') => {
                        if let Some((v, suf, span)) = self.scan_int_hex() {
                            return Token::new(TokenKind::IntLiteral(v, suf), span);
                        }
                        self.pos += 2;
                        self.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
                        let span = Span::new(self.file_id, start, self.pos as u32);
                        let lex = &self.src[start as usize..self.pos];
                        return Token::new(
                            TokenKind::Error(format!("invalid numeric literal: {}", lex)),
                            span,
                        );
                    }
                    Some(b'b') | Some(b'B') => {
                        if let Some((v, suf, span)) = self.scan_int_bin() {
                            return Token::new(TokenKind::IntLiteral(v, suf), span);
                        }
                        self.pos += 2;
                        self.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
                        let span = Span::new(self.file_id, start, self.pos as u32);
                        let lex = &self.src[start as usize..self.pos];
                        return Token::new(
                            TokenKind::Error(format!("invalid numeric literal: {}", lex)),
                            span,
                        );
                    }
                    _ => {}
                }
            }
            if let Some((v, suf, span)) = self.scan_float() {
                return Token::new(TokenKind::FloatLiteral(v, suf), span);
            }
            let (v, suf, span) = self.scan_int_decimal();
            return Token::new(TokenKind::IntLiteral(v, suf), span);
        }

        if first.is_ascii_alphabetic() {
            if let Some((kind, span)) = self.scan_ident_or_keyword() {
                return Token::new(kind, span);
            }
        }

        if first == b'_' {
            let cont = self
                .peek_at(1)
                .map_or(false, |b| b.is_ascii_alphanumeric() || b == b'_');
            if cont {
                let lex_start = self.pos;
                self.pos += 1;
                self.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
                let lex = self.src[lex_start..self.pos].to_string();
                let span = Span::new(self.file_id, start, self.pos as u32);
                return Token::new(TokenKind::Ident(lex), span);
            } else {
                self.pos += 1;
                let span = Span::new(self.file_id, start, self.pos as u32);
                return Token::new(TokenKind::Underscore, span);
            }
        }

        if let Some((kind, span)) = self.scan_operator() {
            return Token::new(kind, span);
        }

        let rest = &self.src[self.pos..];
        let ch = rest.chars().next().expect("peek was Some, so a char must exist");
        self.pos += ch.len_utf8();
        let span = Span::new(self.file_id, start, self.pos as u32);
        Token::new(TokenKind::Error(format!("invalid character: {}", ch)), span)
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
            let (value, span) = match s.scan_string() {
                ScanStringOutcome::Ok(v, sp) => (v, sp),
                ScanStringOutcome::Unterminated(_) => panic!("expected Ok, got Unterminated for {:?}", input),
                ScanStringOutcome::Failed => panic!("expected Ok, got Failed for {:?}", input),
            };
            assert_eq!(value, *expected, "value for {:?}", input);
            assert_eq!(span.file_id, FileId(11), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let unterminated: &[&str] = &["\"abc"];
        for input in unterminated {
            let mut s = Scanner::new(input, FileId(0));
            match s.scan_string() {
                ScanStringOutcome::Unterminated(span) => {
                    assert_eq!(span.file_id, FileId(0), "file_id for {:?}", input);
                    assert_eq!(span.start, 0, "span.start for {:?}", input);
                    assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
                }
                _ => panic!("expected Unterminated for {:?}", input),
            }
            assert_eq!(
                s.pos,
                input.len(),
                "expected pos=len after unterminated {:?}",
                input
            );
        }

        let failed: &[&str] = &[
            "\"\\",
            "\"\\q\"",
            "\"\\xZZ\"",
            "\"\\xFF\"",
            "\"\\u{}\"",
            "\"\\u{D800}\"",
            "\"\\u{110000}\"",
        ];

        for input in failed {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                matches!(s.scan_string(), ScanStringOutcome::Failed),
                "expected Failed for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }

        let mut not_string = Scanner::new("abc", FileId(0));
        assert!(matches!(not_string.scan_string(), ScanStringOutcome::Failed));
        assert_eq!(not_string.pos, 0);
    }

    #[test]
    fn raw_string_arbitrary_hashes() {
        let happy: &[(&str, &str)] = &[
            ("r\"\"", ""),
            ("r\"hello\"", "hello"),
            ("r#\"a\"b\"#", "a\"b"),
            ("r##\"x\"#y\"##", "x\"#y"),
            ("r###\"contains \"## inside\"###", "contains \"## inside"),
            ("r\"\\n\"", "\\n"),
            ("r\"line1\\nline2\"", "line1\\nline2"),
            ("r\"a\nb\"", "a\nb"),
        ];

        for (input, expected) in happy {
            let mut s = Scanner::new(input, FileId(13));
            let (value, span) = s.scan_raw_string().expect(input);
            assert_eq!(value, *expected, "value for {:?}", input);
            assert_eq!(span.file_id, FileId(13), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let rejections: &[&str] = &[
            "r",
            "r#",
            "r\"abc",
            "r#\"abc\"",
            "r##\"abc\"#",
            "abc",
        ];

        for input in rejections {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                s.scan_raw_string().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn nested_block_comments() {
        let mut s = Scanner::new("// hello\n", FileId(0));
        assert!(s.skip_comments());
        assert_eq!(s.pos, "// hello".len());
        assert_eq!(s.peek(), Some(b'\n'));

        let mut s = Scanner::new("/* hi */rest", FileId(0));
        assert!(s.skip_comments());
        assert_eq!(&s.src[s.pos..], "rest");

        let mut s = Scanner::new("/* a /* b */ c */tail", FileId(0));
        assert!(s.skip_comments());
        assert_eq!(&s.src[s.pos..], "tail");

        let mut s = Scanner::new("/*/*/*x*/*/*/Z", FileId(0));
        assert!(s.skip_comments());
        assert_eq!(&s.src[s.pos..], "Z");

        let mut s = Scanner::new("/**/X", FileId(0));
        assert!(s.skip_comments());
        assert_eq!(&s.src[s.pos..], "X");

        let mut s = Scanner::new("/* never ends", FileId(0));
        assert!(!s.skip_comments());
        assert_eq!(s.pos, 0);

        let mut s = Scanner::new("/* a /* b */ c", FileId(0));
        assert!(!s.skip_comments());
        assert_eq!(s.pos, 0);

        let mut s = Scanner::new("/// doc\n", FileId(0));
        assert!(!s.skip_comments());
        assert_eq!(s.pos, 0);

        let mut s = Scanner::new("//! doc\n", FileId(0));
        assert!(!s.skip_comments());
        assert_eq!(s.pos, 0);

        let mut s = Scanner::new("abc", FileId(0));
        assert!(!s.skip_comments());
        assert_eq!(s.pos, 0);
    }

    #[test]
    fn doc_comments_preserved() {
        let outer_happy: &[(&str, &str, usize)] = &[
            ("/// hello\n", " hello", "/// hello".len()),
            ("///hello", "hello", "///hello".len()),
            ("///\n", "", "///".len()),
            ("///", "", "///".len()),
            ("/// trailing", " trailing", "/// trailing".len()),
        ];
        for (input, expected_body, expected_pos) in outer_happy {
            let mut s = Scanner::new(input, FileId(21));
            let (body, style, span) = s.scan_doc_comment().expect(input);
            assert_eq!(body, *expected_body, "body for {:?}", input);
            assert_eq!(style, DocStyle::Outer, "style for {:?}", input);
            assert_eq!(span.file_id, FileId(21), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(
                span.end as usize, *expected_pos,
                "span.end for {:?}",
                input
            );
            assert_eq!(s.pos, *expected_pos, "pos for {:?}", input);
        }

        let inner_happy: &[(&str, &str, usize)] = &[
            ("//! crate doc\n", " crate doc", "//! crate doc".len()),
            ("//!body", "body", "//!body".len()),
            ("//!", "", "//!".len()),
        ];
        for (input, expected_body, expected_pos) in inner_happy {
            let mut s = Scanner::new(input, FileId(22));
            let (body, style, span) = s.scan_doc_comment().expect(input);
            assert_eq!(body, *expected_body, "body for {:?}", input);
            assert_eq!(style, DocStyle::Inner, "style for {:?}", input);
            assert_eq!(span.file_id, FileId(22), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(
                span.end as usize, *expected_pos,
                "span.end for {:?}",
                input
            );
            assert_eq!(s.pos, *expected_pos, "pos for {:?}", input);
        }

        let rejections: &[&str] = &[
            "// regular\n",
            "//\n",
            "/* block */",
            "abc",
            "/",
            "",
        ];
        for input in rejections {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                s.scan_doc_comment().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn operator_maximal_munch() {
        let cases: &[(&str, TokenKind, usize)] = &[
            ("..=", TokenKind::DotDotEq, 3),
            ("..", TokenKind::DotDot, 2),
            (".", TokenKind::Dot, 1),
            ("<=", TokenKind::Le, 2),
            ("<<", TokenKind::Shl, 2),
            ("<", TokenKind::Lt, 1),
            (">=", TokenKind::Ge, 2),
            (">>", TokenKind::Shr, 2),
            (">", TokenKind::Gt, 1),
            ("==", TokenKind::EqEq, 2),
            ("=>", TokenKind::FatArrow, 2),
            ("=", TokenKind::Eq, 1),
            ("-=", TokenKind::MinusEq, 2),
            ("->", TokenKind::Arrow, 2),
            ("-", TokenKind::Minus, 1),
            ("::", TokenKind::ColonColon, 2),
            (":", TokenKind::Colon, 1),
            ("!=", TokenKind::BangEq, 2),
            ("+=", TokenKind::PlusEq, 2),
            ("+", TokenKind::Plus, 1),
            ("*=", TokenKind::StarEq, 2),
            ("*", TokenKind::Star, 1),
            ("/=", TokenKind::SlashEq, 2),
            ("/", TokenKind::Slash, 1),
            ("%=", TokenKind::PercentEq, 2),
            ("%", TokenKind::Percent, 1),
            ("&", TokenKind::Amp, 1),
            ("|", TokenKind::Pipe, 1),
            ("^", TokenKind::Caret, 1),
            ("~", TokenKind::Tilde, 1),
            ("?", TokenKind::Question, 1),
            (";", TokenKind::Semi, 1),
            (",", TokenKind::Comma, 1),
            ("(", TokenKind::LParen, 1),
            (")", TokenKind::RParen, 1),
            ("[", TokenKind::LBracket, 1),
            ("]", TokenKind::RBracket, 1),
            ("{", TokenKind::LBrace, 1),
            ("}", TokenKind::RBrace, 1),
            ("..=x", TokenKind::DotDotEq, 3),
            ("..x", TokenKind::DotDot, 2),
            (".x", TokenKind::Dot, 1),
        ];

        for (input, expected_kind, expected_len) in cases {
            let mut s = Scanner::new(input, FileId(31));
            let (kind, span) = s.scan_operator().expect(input);
            assert_eq!(&kind, expected_kind, "kind for {:?}", input);
            assert_eq!(span.file_id, FileId(31), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(
                span.end as usize,
                *expected_len,
                "span.end for {:?}",
                input
            );
            assert_eq!(s.pos, *expected_len, "pos for {:?}", input);
        }

        let rejections: &[&str] = &["a", "_", "#", "@", "$", "", "!"];
        for input in rejections {
            let mut s = Scanner::new(input, FileId(31));
            assert!(
                s.scan_operator().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn keywords_take_priority_over_idents() {
        let keyword_cases: &[(&str, TokenKind)] = &[
            ("and", TokenKind::And),
            ("break", TokenKind::Break),
            ("const", TokenKind::Const),
            ("continue", TokenKind::Continue),
            ("else", TokenKind::Else),
            ("enum", TokenKind::Enum),
            ("extern", TokenKind::Extern),
            ("false", TokenKind::False),
            ("fn", TokenKind::Fn),
            ("for", TokenKind::For),
            ("if", TokenKind::If),
            ("impl", TokenKind::Impl),
            ("in", TokenKind::In),
            ("let", TokenKind::Let),
            ("loop", TokenKind::Loop),
            ("match", TokenKind::Match),
            ("mod", TokenKind::Mod),
            ("mut", TokenKind::Mut),
            ("not", TokenKind::Not),
            ("or", TokenKind::Or),
            ("pub", TokenKind::Pub),
            ("return", TokenKind::Return),
            ("self", TokenKind::SelfLower),
            ("Self", TokenKind::SelfUpper),
            ("struct", TokenKind::Struct),
            ("trait", TokenKind::Trait),
            ("true", TokenKind::True),
            ("type", TokenKind::Type),
            ("unsafe", TokenKind::Unsafe),
            ("use", TokenKind::Use),
            ("where", TokenKind::Where),
            ("while", TokenKind::While),
        ];

        for (input, expected_kind) in keyword_cases {
            let mut s = Scanner::new(input, FileId(41));
            let (kind, span) = s.scan_ident_or_keyword().expect(input);
            assert_eq!(&kind, expected_kind, "kind for {:?}", input);
            assert_eq!(span.file_id, FileId(41), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let ident_cases: &[(&str, &str)] = &[
            ("foo", "foo"),
            ("Foo", "Foo"),
            ("foo_bar", "foo_bar"),
            ("x1", "x1"),
            ("FOO", "FOO"),
            ("fnord", "fnord"),
            ("self_", "self_"),
            ("Self2", "Self2"),
            ("returnn", "returnn"),
        ];

        for (input, expected_name) in ident_cases {
            let mut s = Scanner::new(input, FileId(42));
            let (kind, span) = s.scan_ident_or_keyword().expect(input);
            assert_eq!(
                kind,
                TokenKind::Ident((*expected_name).to_string()),
                "kind for {:?}",
                input
            );
            assert_eq!(span.file_id, FileId(42), "file_id for {:?}", input);
            assert_eq!(span.start, 0, "span.start for {:?}", input);
            assert_eq!(span.end as usize, input.len(), "span.end for {:?}", input);
            assert_eq!(s.pos, input.len(), "pos for {:?}", input);
        }

        let mut boundary = Scanner::new("fn x", FileId(43));
        let (kind, span) = boundary.scan_ident_or_keyword().expect("fn x");
        assert_eq!(kind, TokenKind::Fn);
        assert_eq!(span.file_id, FileId(43));
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 2);
        assert_eq!(boundary.pos, 2);

        let rejections: &[&str] = &["_", "_foo", "1abc", "", " foo", "123", "!"];
        for input in rejections {
            let mut s = Scanner::new(input, FileId(0));
            assert!(
                s.scan_ident_or_keyword().is_none(),
                "expected None for {:?}",
                input
            );
            assert_eq!(s.pos, 0, "expected pos=0 after rejecting {:?}", input);
        }
    }

    #[test]
    fn tokenizes_full_program() {
        let src = "/// doc\nfn let if foo _ _bar 123 0xFF 0b101 1.5 'a' \"hi\" r\"raw\"\n// line comment\n/* block */\n+ - * / % == != <= >= << >> .. ..= -> => :: ; , ( ) { } [ ] ? & | ^ ~ . : = ";
        let mut s = Scanner::new(src, FileId(99));

        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let t = s.next_token();
            let is_eof = matches!(&t.kind, TokenKind::Eof);
            tokens.push(t);
            if is_eof {
                break;
            }
        }

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();

        let expected: Vec<TokenKind> = vec![
            TokenKind::DocComment(" doc".to_string(), DocStyle::Outer),
            TokenKind::Fn,
            TokenKind::Let,
            TokenKind::If,
            TokenKind::Ident("foo".to_string()),
            TokenKind::Underscore,
            TokenKind::Ident("_bar".to_string()),
            TokenKind::IntLiteral(123, IntSuffix::Unsuffixed),
            TokenKind::IntLiteral(255, IntSuffix::Unsuffixed),
            TokenKind::IntLiteral(5, IntSuffix::Unsuffixed),
            TokenKind::FloatLiteral(1.5, FloatSuffix::Unsuffixed),
            TokenKind::CharLiteral('a'),
            TokenKind::StringLiteral("hi".to_string()),
            TokenKind::RawStringLiteral("raw".to_string()),
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::Le,
            TokenKind::Ge,
            TokenKind::Shl,
            TokenKind::Shr,
            TokenKind::DotDot,
            TokenKind::DotDotEq,
            TokenKind::Arrow,
            TokenKind::FatArrow,
            TokenKind::ColonColon,
            TokenKind::Semi,
            TokenKind::Comma,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::Question,
            TokenKind::Amp,
            TokenKind::Pipe,
            TokenKind::Caret,
            TokenKind::Tilde,
            TokenKind::Dot,
            TokenKind::Colon,
            TokenKind::Eq,
            TokenKind::Eof,
        ];

        assert_eq!(kinds, expected);

        let mut prev_end: u32 = 0;
        for t in &tokens {
            assert_eq!(t.span.file_id, FileId(99), "file_id for {:?}", t.kind);
            match &t.kind {
                TokenKind::Eof => {
                    assert_eq!(
                        t.span.start, t.span.end,
                        "Eof span must be empty, got {:?}",
                        t.span
                    );
                    assert_eq!(t.span.start as usize, src.len(), "Eof at end of input");
                }
                _ => {
                    assert!(
                        !t.span.is_empty(),
                        "non-Eof token {:?} has empty span {:?}",
                        t.kind,
                        t.span
                    );
                }
            }
            assert!(
                t.span.start >= prev_end,
                "spans not non-decreasing: prev_end={}, token={:?}",
                prev_end,
                t
            );
            let gap = &src[prev_end as usize..t.span.start as usize];
            let mut gap_scanner = Scanner::new(gap, FileId(99));
            gap_scanner.skip_whitespace_and_comments();
            assert_eq!(
                gap_scanner.pos,
                gap.len(),
                "gap before {:?} contains non-whitespace/comment content: {:?}",
                t.kind,
                gap
            );
            prev_end = t.span.end;
        }
    }

    #[test]
    fn all_tokens_have_nonzero_span() {
        let src = "/// outer doc\n//! inner doc\nfn let if else while for loop return struct enum trait impl mod use pub const static mut self Self as in match break continue type where foo _ _bar 0 123 0xFF 0b1010 1.5 'a' \"hi\" r\"raw\" r#\"raw#hash\"# // line\n/* block /* nested */ */\n+ - * / % += -= *= /= %= == != < > <= >= << >> .. ..= -> => :: ; , ( ) { } [ ] ? & | ^ ~ . : = $";
        let file_id = FileId(7);
        let mut s = Scanner::new(src, file_id);

        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let t = s.next_token();
            let is_eof = matches!(&t.kind, TokenKind::Eof);
            tokens.push(t);
            if is_eof {
                break;
            }
        }

        let non_eof_count = tokens
            .iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .count();
        assert!(
            non_eof_count > 0,
            "expected at least one non-Eof token, got {} total",
            tokens.len()
        );

        for t in &tokens {
            assert_eq!(
                t.span.file_id, file_id,
                "file_id mismatch for token {:?}",
                t.kind
            );
            match &t.kind {
                TokenKind::Eof => {
                    assert_eq!(
                        t.span.start, t.span.end,
                        "Eof span must be empty, got {:?}",
                        t.span
                    );
                    assert_eq!(
                        t.span.start as usize,
                        src.len(),
                        "Eof must sit at end of input, got {:?}",
                        t.span
                    );
                }
                _ => {
                    assert!(
                        t.span.start < t.span.end,
                        "non-Eof token {:?} has empty span {:?}",
                        t.kind,
                        t.span
                    );
                }
            }
        }
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

    #[test]
    fn invalid_char_recovers() {
        let src = "a $ b @ c € d";
        let file_id = FileId(11);
        let mut s = Scanner::new(src, file_id);

        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let t = s.next_token();
            let is_eof = matches!(&t.kind, TokenKind::Eof);
            tokens.push(t);
            if is_eof {
                break;
            }
        }

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::Ident("a".to_string()),
            TokenKind::Error("invalid character: $".to_string()),
            TokenKind::Ident("b".to_string()),
            TokenKind::Error("invalid character: @".to_string()),
            TokenKind::Ident("c".to_string()),
            TokenKind::Error("invalid character: \u{20ac}".to_string()),
            TokenKind::Ident("d".to_string()),
            TokenKind::Eof,
        ];
        assert_eq!(kinds, expected);

        for t in &tokens {
            assert_eq!(t.span.file_id, file_id);
        }

        let errors: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Error(_)))
            .collect();
        assert_eq!(errors.len(), 3);

        let dollar_span = errors[0].span;
        assert_eq!(dollar_span.end - dollar_span.start, '$'.len_utf8() as u32);
        assert_eq!(dollar_span.end - dollar_span.start, 1);

        let at_span = errors[1].span;
        assert_eq!(at_span.end - at_span.start, '@'.len_utf8() as u32);
        assert_eq!(at_span.end - at_span.start, 1);

        let euro_span = errors[2].span;
        assert_eq!(euro_span.end - euro_span.start, '\u{20ac}'.len_utf8() as u32);
        assert!(euro_span.end - euro_span.start > 1);

        let last = tokens.last().expect("tokens has Eof");
        assert!(matches!(last.kind, TokenKind::Eof));
        assert_eq!(last.span.start, last.span.end);
        assert_eq!(last.span.start as usize, src.len());
    }

    #[test]
    fn invalid_numeric_recovers() {
        let src = "0x foo 0xg bar 0x_ baz 0b qux 0b2 quux 0b_ end";
        let file_id = FileId(61);
        let mut s = Scanner::new(src, file_id);

        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let t = s.next_token();
            let is_eof = matches!(&t.kind, TokenKind::Eof);
            tokens.push(t);
            if is_eof {
                break;
            }
        }

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::Error("invalid numeric literal: 0x".to_string()),
            TokenKind::Ident("foo".to_string()),
            TokenKind::Error("invalid numeric literal: 0xg".to_string()),
            TokenKind::Ident("bar".to_string()),
            TokenKind::Error("invalid numeric literal: 0x_".to_string()),
            TokenKind::Ident("baz".to_string()),
            TokenKind::Error("invalid numeric literal: 0b".to_string()),
            TokenKind::Ident("qux".to_string()),
            TokenKind::Error("invalid numeric literal: 0b2".to_string()),
            TokenKind::Ident("quux".to_string()),
            TokenKind::Error("invalid numeric literal: 0b_".to_string()),
            TokenKind::Ident("end".to_string()),
            TokenKind::Eof,
        ];
        assert_eq!(kinds, expected);

        for t in &tokens {
            assert_eq!(t.span.file_id, file_id);
        }

        let errors: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Error(_)))
            .collect();
        assert_eq!(errors.len(), 6);

        let cases: &[(&str, &str)] = &[
            ("invalid numeric literal: 0x", "0x"),
            ("invalid numeric literal: 0xg", "0xg"),
            ("invalid numeric literal: 0x_", "0x_"),
            ("invalid numeric literal: 0b", "0b"),
            ("invalid numeric literal: 0b2", "0b2"),
            ("invalid numeric literal: 0b_", "0b_"),
        ];
        for (i, (msg, lex)) in cases.iter().enumerate() {
            let tok = errors[i];
            match &tok.kind {
                TokenKind::Error(m) => assert_eq!(m, msg, "error message for {:?}", lex),
                _ => unreachable!(),
            }
            let start = tok.span.start as usize;
            let end = tok.span.end as usize;
            let expected_start = src
                .find(*lex)
                .expect("test source should contain each malformed run");
            assert_eq!(start, expected_start, "span.start for {:?}", lex);
            assert_eq!(end, expected_start + lex.len(), "span.end for {:?}", lex);
            assert_eq!(&src[start..end], *lex, "span lexeme for {:?}", lex);
        }

        let mut prev_end: u32 = 0;
        for t in &tokens {
            assert!(
                t.span.start >= prev_end,
                "span not monotonic: prev_end={}, token={:?}",
                prev_end,
                t
            );
            prev_end = t.span.end;
        }

        let last = tokens.last().expect("tokens has Eof");
        assert!(matches!(last.kind, TokenKind::Eof));
        assert_eq!(last.span.start, last.span.end);
        assert_eq!(last.span.start as usize, src.len());
    }

    #[test]
    fn unterminated_string_recovers() {
        fn drive(src: &str, file_id: FileId) -> Vec<Token> {
            let mut s = Scanner::new(src, file_id);
            let mut tokens: Vec<Token> = Vec::new();
            loop {
                let t = s.next_token();
                let is_eof = matches!(&t.kind, TokenKind::Eof);
                tokens.push(t);
                if is_eof {
                    break;
                }
            }
            tokens
        }

        // Case 1: bare unterminated string ending mid-content.
        let src = "\"abc";
        let file_id = FileId(51);
        let tokens = drive(src, file_id);
        assert_eq!(tokens.len(), 2, "tokens for {:?}: {:?}", src, tokens);
        assert_eq!(
            tokens[0].kind,
            TokenKind::Error("unterminated string literal".to_string())
        );
        assert_eq!(tokens[0].span.file_id, file_id);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end as usize, src.len());
        assert!(matches!(tokens[1].kind, TokenKind::Eof));
        assert_eq!(tokens[1].span.file_id, file_id);
        assert_eq!(tokens[1].span.start, tokens[1].span.end);
        assert_eq!(tokens[1].span.start as usize, src.len());

        // Case 2: unterminated string with embedded newline (newlines are allowed inside strings).
        let src = "\"abc\n";
        let file_id = FileId(52);
        let tokens = drive(src, file_id);
        assert_eq!(tokens.len(), 2, "tokens for {:?}: {:?}", src, tokens);
        assert_eq!(
            tokens[0].kind,
            TokenKind::Error("unterminated string literal".to_string())
        );
        assert_eq!(tokens[0].span.file_id, file_id);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end as usize, src.len());
        assert!(matches!(tokens[1].kind, TokenKind::Eof));
        assert_eq!(tokens[1].span.file_id, file_id);
        assert_eq!(tokens[1].span.start, tokens[1].span.end);
        assert_eq!(tokens[1].span.start as usize, src.len());

        // Case 3: error span starts at the open-quote, not byte 0.
        let src = "prefix \"abc";
        let file_id = FileId(53);
        let tokens = drive(src, file_id);
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("prefix".to_string()),
                TokenKind::Error("unterminated string literal".to_string()),
                TokenKind::Eof,
            ]
        );
        let open_quote = src.find('"').expect("input contains a quote");
        assert_eq!(tokens[1].span.file_id, file_id);
        assert_eq!(tokens[1].span.start as usize, open_quote);
        assert_eq!(tokens[1].span.end as usize, src.len());
        assert!(matches!(tokens[2].kind, TokenKind::Eof));
        assert_eq!(tokens[2].span.file_id, file_id);
        assert_eq!(tokens[2].span.start, tokens[2].span.end);
        assert_eq!(tokens[2].span.start as usize, src.len());

        // Cross-cutting invariant: every error token's span ends at src.len() and final Eof has empty span at src.len().
        for src in &["\"abc", "\"abc\n", "prefix \"abc"] {
            let file_id = FileId(54);
            let tokens = drive(src, file_id);
            for t in &tokens {
                assert_eq!(t.span.file_id, file_id);
                if let TokenKind::Error(_) = &t.kind {
                    assert_eq!(
                        t.span.end as usize,
                        src.len(),
                        "error span end for {:?}",
                        src
                    );
                }
            }
            let last = tokens.last().expect("at least Eof");
            assert!(matches!(last.kind, TokenKind::Eof));
            assert_eq!(last.span.start, last.span.end);
            assert_eq!(last.span.start as usize, src.len());
        }
    }

    fn next_rand(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }

    #[test]
    fn fuzz_random_bytes_no_panic() {
        let seed: u64 = 0x9E3779B97F4A7C15;
        let mut state = seed;

        for iter in 0..1000usize {
            let len = (next_rand(&mut state) as usize) % 257;
            let mut bytes: Vec<u8> = Vec::with_capacity(len);
            for _ in 0..len {
                bytes.push(next_rand(&mut state) as u8);
            }

            let s = String::from_utf8_lossy(&bytes).into_owned();
            let mut scanner = Scanner::new(&s, FileId(0));

            let cap = 4 * s.len() + 16;
            let mut steps = 0usize;
            loop {
                let prev_pos = scanner.pos;
                let tok = scanner.next_token();
                steps += 1;

                if matches!(tok.kind, TokenKind::Eof) {
                    break;
                }

                assert!(
                    scanner.pos > prev_pos,
                    "scanner failed to advance (seed={:#x}, iter={}, pos={}, input={:?})",
                    seed, iter, scanner.pos, bytes
                );

                assert!(
                    steps <= cap,
                    "scanner exceeded iteration cap {} (seed={:#x}, iter={}, input={:?})",
                    cap, seed, iter, bytes
                );
            }
        }
    }

    mod spec_section_2 {
        use super::*;
        use crate::lex_eq;

        // Keywords (one per `TokenKind` keyword variant present in the spec's
        // §2 keyword list — `defer` is intentionally omitted, see plan).

        #[test]
        fn kw_break() {
            lex_eq!("break", vec![TokenKind::Break]);
        }

        #[test]
        fn kw_const() {
            lex_eq!("const", vec![TokenKind::Const]);
        }

        #[test]
        fn kw_continue() {
            lex_eq!("continue", vec![TokenKind::Continue]);
        }

        #[test]
        fn kw_else() {
            lex_eq!("else", vec![TokenKind::Else]);
        }

        #[test]
        fn kw_enum() {
            lex_eq!("enum", vec![TokenKind::Enum]);
        }

        #[test]
        fn kw_extern() {
            lex_eq!("extern", vec![TokenKind::Extern]);
        }

        #[test]
        fn kw_false() {
            lex_eq!("false", vec![TokenKind::False]);
        }

        #[test]
        fn kw_fn() {
            lex_eq!("fn", vec![TokenKind::Fn]);
        }

        #[test]
        fn kw_for() {
            lex_eq!("for", vec![TokenKind::For]);
        }

        #[test]
        fn kw_if() {
            lex_eq!("if", vec![TokenKind::If]);
        }

        #[test]
        fn kw_impl() {
            lex_eq!("impl", vec![TokenKind::Impl]);
        }

        #[test]
        fn kw_in() {
            lex_eq!("in", vec![TokenKind::In]);
        }

        #[test]
        fn kw_let() {
            lex_eq!("let", vec![TokenKind::Let]);
        }

        #[test]
        fn kw_loop() {
            lex_eq!("loop", vec![TokenKind::Loop]);
        }

        #[test]
        fn kw_match() {
            lex_eq!("match", vec![TokenKind::Match]);
        }

        #[test]
        fn kw_mod() {
            lex_eq!("mod", vec![TokenKind::Mod]);
        }

        #[test]
        fn kw_mut() {
            lex_eq!("mut", vec![TokenKind::Mut]);
        }

        #[test]
        fn kw_pub() {
            lex_eq!("pub", vec![TokenKind::Pub]);
        }

        #[test]
        fn kw_return() {
            lex_eq!("return", vec![TokenKind::Return]);
        }

        #[test]
        fn kw_self_lower() {
            lex_eq!("self", vec![TokenKind::SelfLower]);
        }

        #[test]
        fn kw_self_upper() {
            lex_eq!("Self", vec![TokenKind::SelfUpper]);
        }

        #[test]
        fn kw_struct() {
            lex_eq!("struct", vec![TokenKind::Struct]);
        }

        #[test]
        fn kw_trait() {
            lex_eq!("trait", vec![TokenKind::Trait]);
        }

        #[test]
        fn kw_true() {
            lex_eq!("true", vec![TokenKind::True]);
        }

        #[test]
        fn kw_type() {
            lex_eq!("type", vec![TokenKind::Type]);
        }

        #[test]
        fn kw_unsafe() {
            lex_eq!("unsafe", vec![TokenKind::Unsafe]);
        }

        #[test]
        fn kw_use() {
            lex_eq!("use", vec![TokenKind::Use]);
        }

        #[test]
        fn kw_where() {
            lex_eq!("where", vec![TokenKind::Where]);
        }

        #[test]
        fn kw_while() {
            lex_eq!("while", vec![TokenKind::While]);
        }

        // Logical word operators (spec §2: "Logical (words only, no symbols)").

        #[test]
        fn op_word_and() {
            lex_eq!("and", vec![TokenKind::And]);
        }

        #[test]
        fn op_word_or() {
            lex_eq!("or", vec![TokenKind::Or]);
        }

        #[test]
        fn op_word_not() {
            lex_eq!("not", vec![TokenKind::Not]);
        }

        // Operator packs — one test per spec §2 operator subsection.

        #[test]
        fn op_arith_pack() {
            lex_eq!(
                "+ - * / %",
                vec![
                    TokenKind::Plus,
                    TokenKind::Minus,
                    TokenKind::Star,
                    TokenKind::Slash,
                    TokenKind::Percent,
                ]
            );
        }

        #[test]
        fn op_cmp_pack() {
            lex_eq!(
                "== != < > <= >=",
                vec![
                    TokenKind::EqEq,
                    TokenKind::BangEq,
                    TokenKind::Lt,
                    TokenKind::Gt,
                    TokenKind::Le,
                    TokenKind::Ge,
                ]
            );
        }

        #[test]
        fn op_bitwise_pack() {
            lex_eq!(
                "& | ^ ~ << >>",
                vec![
                    TokenKind::Amp,
                    TokenKind::Pipe,
                    TokenKind::Caret,
                    TokenKind::Tilde,
                    TokenKind::Shl,
                    TokenKind::Shr,
                ]
            );
        }

        #[test]
        fn op_assign_pack() {
            lex_eq!(
                "= += -= *= /= %=",
                vec![
                    TokenKind::Eq,
                    TokenKind::PlusEq,
                    TokenKind::MinusEq,
                    TokenKind::StarEq,
                    TokenKind::SlashEq,
                    TokenKind::PercentEq,
                ]
            );
        }

        #[test]
        fn op_access_pack() {
            lex_eq!(
                ". :: [] ()",
                vec![
                    TokenKind::Dot,
                    TokenKind::ColonColon,
                    TokenKind::LBracket,
                    TokenKind::RBracket,
                    TokenKind::LParen,
                    TokenKind::RParen,
                ]
            );
        }

        #[test]
        fn op_control_flow_pack() {
            lex_eq!(
                "? .. ..= ->",
                vec![
                    TokenKind::Question,
                    TokenKind::DotDot,
                    TokenKind::DotDotEq,
                    TokenKind::Arrow,
                ]
            );
        }

        #[test]
        fn op_special_pack() {
            lex_eq!(
                "; , : _",
                vec![
                    TokenKind::Semi,
                    TokenKind::Comma,
                    TokenKind::Colon,
                    TokenKind::Underscore,
                ]
            );
        }

        #[test]
        fn op_fat_arrow() {
            lex_eq!("=>", vec![TokenKind::FatArrow]);
        }

        #[test]
        fn op_amp_mut() {
            lex_eq!("&mut", vec![TokenKind::Amp, TokenKind::Mut]);
        }

        // Literals — one per `TokenKind` literal variant referenced in §2.

        #[test]
        fn lit_int_decimal() {
            lex_eq!(
                "42",
                vec![TokenKind::IntLiteral(42, IntSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_int_underscored() {
            lex_eq!(
                "1_000_000",
                vec![TokenKind::IntLiteral(1_000_000, IntSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_int_hex() {
            lex_eq!(
                "0xff",
                vec![TokenKind::IntLiteral(0xff, IntSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_int_binary() {
            lex_eq!(
                "0b1010",
                vec![TokenKind::IntLiteral(0b1010, IntSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_float_simple() {
            lex_eq!(
                "3.14",
                vec![TokenKind::FloatLiteral(3.14, FloatSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_float_exp() {
            lex_eq!(
                "1.0e-10",
                vec![TokenKind::FloatLiteral(1.0e-10, FloatSuffix::Unsuffixed)]
            );
        }

        #[test]
        fn lit_char() {
            lex_eq!("'a'", vec![TokenKind::CharLiteral('a')]);
        }

        #[test]
        fn lit_string() {
            lex_eq!(
                "\"hello\"",
                vec![TokenKind::StringLiteral("hello".into())]
            );
        }

        #[test]
        fn lit_raw_string() {
            lex_eq!(
                "r\"raw string\"",
                vec![TokenKind::RawStringLiteral("raw string".into())]
            );
        }

        #[test]
        fn lit_bool_true() {
            lex_eq!("true", vec![TokenKind::True]);
        }

        #[test]
        fn lit_bool_false() {
            lex_eq!("false", vec![TokenKind::False]);
        }

        // Built-in syntax forms drawn from §2's "Built-in Syntax" subsection.
        // The spec keeps the literal `!` for `vec![…]` even while declaring
        // these are not macros; the current scanner has no `Bang` variant, so
        // the `!` snapshots as an `Error` token (see plan Risks).

        #[test]
        fn builtin_vec_macro() {
            lex_eq!(
                "vec![1, 2, 3]",
                vec![
                    TokenKind::Ident("vec".into()),
                    TokenKind::Error("invalid character: !".into()),
                    TokenKind::LBracket,
                    TokenKind::IntLiteral(1, IntSuffix::Unsuffixed),
                    TokenKind::Comma,
                    TokenKind::IntLiteral(2, IntSuffix::Unsuffixed),
                    TokenKind::Comma,
                    TokenKind::IntLiteral(3, IntSuffix::Unsuffixed),
                    TokenKind::RBracket,
                ]
            );
        }

        #[test]
        fn builtin_println() {
            lex_eq!(
                "println(\"text\")",
                vec![
                    TokenKind::Ident("println".into()),
                    TokenKind::LParen,
                    TokenKind::StringLiteral("text".into()),
                    TokenKind::RParen,
                ]
            );
        }

        #[test]
        fn builtin_format() {
            lex_eq!(
                "format(\"Hello {}\", name)",
                vec![
                    TokenKind::Ident("format".into()),
                    TokenKind::LParen,
                    TokenKind::StringLiteral("Hello {}".into()),
                    TokenKind::Comma,
                    TokenKind::Ident("name".into()),
                    TokenKind::RParen,
                ]
            );
        }

        #[test]
        fn builtin_array_repeat() {
            lex_eq!(
                "[0; 256]",
                vec![
                    TokenKind::LBracket,
                    TokenKind::IntLiteral(0, IntSuffix::Unsuffixed),
                    TokenKind::Semi,
                    TokenKind::IntLiteral(256, IntSuffix::Unsuffixed),
                    TokenKind::RBracket,
                ]
            );
        }

        #[test]
        fn builtin_assert_msg() {
            lex_eq!(
                "assert(x == y, \"x must equal y\")",
                vec![
                    TokenKind::Ident("assert".into()),
                    TokenKind::LParen,
                    TokenKind::Ident("x".into()),
                    TokenKind::EqEq,
                    TokenKind::Ident("y".into()),
                    TokenKind::Comma,
                    TokenKind::StringLiteral("x must equal y".into()),
                    TokenKind::RParen,
                ]
            );
        }

        #[test]
        fn builtin_derive_attr() {
            lex_eq!(
                "#[derive(Clone)]",
                vec![
                    TokenKind::Error("invalid character: #".into()),
                    TokenKind::LBracket,
                    TokenKind::Ident("derive".into()),
                    TokenKind::LParen,
                    TokenKind::Ident("Clone".into()),
                    TokenKind::RParen,
                    TokenKind::RBracket,
                ]
            );
        }
    }
}
