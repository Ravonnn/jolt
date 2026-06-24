use jolt_source::{FileId, Span};

use crate::error::LexErrorKind;
use crate::token::{ident_or_keyword, Token, TokenKind};

/// Hand-rolled lexer with explicit max-munch rules (`docs/spec/jolt-grammar.md` §1).
pub struct Lexer<'a> {
    file: FileId,
    source: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(file: FileId, source: &'a str) -> Self {
        Self {
            file,
            source,
            pos: 0,
        }
    }

    pub fn file(&self) -> FileId {
        self.file
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span {
            file: self.file,
            start: start as u32,
            end: end as u32,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(n)
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    fn consume_if(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                self.bump();
            }
            if self.starts_with("//") && !self.preceded_by_token_char() {
                self.pos += 2;
                while let Some(c) = self.peek() {
                    self.bump();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// `//` is integer division when immediately after a token character (e.g. `a//b`).
    fn preceded_by_token_char(&self) -> bool {
        if self.pos == 0 {
            return false;
        }
        let prev = self.source.as_bytes()[self.pos - 1];
        prev.is_ascii_alphanumeric() || matches!(prev, b')' | b']' | b'"')
    }

    fn token(&self, start: usize, kind: TokenKind) -> Token {
        Token::new(kind, self.span(start, self.pos))
    }

    fn error_token(&self, start: usize, kind: LexErrorKind) -> Token {
        Token::new(TokenKind::Error(kind), self.span(start, self.pos))
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        let start = self.pos;

        let Some(ch) = self.peek() else {
            let end = self.source.len();
            return Token::new(TokenKind::Eof, self.span(end, end));
        };

        if ch == '"' {
            return self.lex_string(start);
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.lex_ident(start);
        }

        if ch.is_ascii_digit() {
            return self.lex_number(start);
        }

        if ch == '@' {
            self.bump();
            return self.token(start, TokenKind::At);
        }

        if ch == '$' {
            self.bump();
            if self.peek() == Some('$') {
                self.bump();
                return self.token(start, TokenKind::DollarDollar);
            }
            return self.token(start, TokenKind::Dollar);
        }

        if ch == '-' && self.peek_nth(1).is_some_and(|c| c.is_ascii_digit()) {
            return self.lex_number(start);
        }

        if let Some(kind) = self.lex_operator() {
            return self.token(start, kind);
        }

        self.bump();
        self.error_token(start, LexErrorKind::UnexpectedChar)
    }

    fn lex_operator(&mut self) -> Option<TokenKind> {
        macro_rules! try_op {
            ($s:literal, $kind:expr) => {
                if self.consume_if($s) {
                    return Some($kind);
                }
            };
        }

        try_op!(">>>=", TokenKind::GtGtGtEq);
        try_op!("<<<=", TokenKind::LtLtLtEq);
        try_op!(">>|=", TokenKind::GtGtPipeEq);
        try_op!("<<|=", TokenKind::LtLtPipeEq);
        try_op!(">>=", TokenKind::GtGtEq);
        try_op!("<<=", TokenKind::LtLtEq);
        try_op!("//=", TokenKind::SlashSlashEq);
        try_op!("~%|=", TokenKind::TildePercentPipeEq);
        try_op!("~|=", TokenKind::TildePipeEq);
        try_op!("~&=", TokenKind::TildeAmpEq);
        try_op!("%|=", TokenKind::PercentPipeEq);
        try_op!("+=", TokenKind::PlusEq);
        try_op!("-=", TokenKind::MinusEq);
        try_op!("*=", TokenKind::StarEq);
        try_op!("/=", TokenKind::SlashEq);
        try_op!("%=", TokenKind::PercentEq);
        try_op!("^=", TokenKind::CaretEq);
        try_op!("&=", TokenKind::AmpEq);
        try_op!("|=", TokenKind::PipeEq);
        try_op!("~%|", TokenKind::TildePercentPipe);
        try_op!("~|", TokenKind::TildePipe);
        try_op!("~&", TokenKind::TildeAmp);
        try_op!("%|", TokenKind::PercentPipe);
        try_op!(">>|", TokenKind::GtGtPipe);
        try_op!("<<|", TokenKind::LtLtPipe);
        try_op!(">>>", TokenKind::GtGtGt);
        try_op!("<<<", TokenKind::LtLtLt);
        try_op!(">>", TokenKind::GtGt);
        try_op!("<<", TokenKind::LtLt);
        try_op!("==", TokenKind::EqEq);
        try_op!("!=", TokenKind::NotEq);
        try_op!("<=", TokenKind::Le);
        try_op!(">=", TokenKind::Ge);
        try_op!("//", TokenKind::SlashSlash);
        try_op!("...", TokenKind::DotDotDot);
        try_op!("..", TokenKind::DotDot);
        try_op!("->", TokenKind::Arrow);
        try_op!(";;", TokenKind::SemiSemi);
        try_op!("&&", TokenKind::AndAnd);
        try_op!("||", TokenKind::OrOr);
        try_op!("=", TokenKind::Eq);
        try_op!(";", TokenKind::Semi);
        try_op!("<", TokenKind::Lt);
        try_op!(">", TokenKind::Gt);
        try_op!("+", TokenKind::Plus);
        try_op!("-", TokenKind::Minus);
        try_op!("*", TokenKind::Star);
        try_op!("/", TokenKind::Slash);
        try_op!("%", TokenKind::Percent);
        try_op!("^", TokenKind::Caret);
        try_op!("&", TokenKind::Amp);
        try_op!("|", TokenKind::Pipe);
        try_op!("~", TokenKind::Tilde);
        try_op!("!", TokenKind::Not);
        try_op!("(", TokenKind::LParen);
        try_op!(")", TokenKind::RParen);
        try_op!(",", TokenKind::Comma);
        try_op!(":", TokenKind::Colon);
        try_op!("[", TokenKind::LBracket);
        try_op!("]", TokenKind::RBracket);

        None
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.bump();
        }
        let text = &self.source[start..self.pos];
        self.token(start, ident_or_keyword(text))
    }

    fn lex_number(&mut self, start: usize) -> Token {
        let negative = self.peek() == Some('-');
        if negative {
            self.bump();
        }

        let base_start = self.pos;
        let base: u32 = if self.starts_with("0x") || self.starts_with("0X") {
            self.pos += 2;
            16
        } else if self.starts_with("0o") || self.starts_with("0O") {
            self.pos += 2;
            8
        } else if self.starts_with("0b") || self.starts_with("0B") {
            self.pos += 2;
            2
        } else {
            10
        };

        let digit_start = self.pos;
        while let Some(c) = self.peek() {
            if c == '_' {
                self.bump();
                continue;
            }
            let valid = match base {
                16 => c.is_ascii_hexdigit(),
                8 => matches!(c, '0'..='7'),
                2 => matches!(c, '0' | '1'),
                _ => c.is_ascii_digit(),
            };
            if valid {
                self.bump();
            } else {
                break;
            }
        }

        if self.pos == digit_start && base_start == digit_start {
            return self.error_token(start, LexErrorKind::InvalidDigit);
        }

        let digits: String = self.source[digit_start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();

        if digits.is_empty() {
            return self.error_token(start, LexErrorKind::InvalidDigit);
        }

        let parsed = match base {
            16 => i64::from_str_radix(&digits, 16),
            8 => i64::from_str_radix(&digits, 8),
            2 => i64::from_str_radix(&digits, 2),
            _ => digits.parse::<i64>(),
        };

        match parsed {
            Ok(n) => {
                let value = if negative { -n } else { n };
                self.token(start, TokenKind::IntLit(value))
            }
            Err(_) => self.error_token(start, LexErrorKind::IntOverflow),
        }
    }

    fn lex_string(&mut self, start: usize) -> Token {
        self.bump(); // opening "
        let mut value = String::new();
        loop {
            let Some(ch) = self.peek() else {
                return self.error_token(start, LexErrorKind::UnterminatedString);
            };
            if ch == '"' {
                self.bump();
                return self.token(start, TokenKind::StringLit(value));
            }
            if ch == '\n' {
                return self.error_token(start, LexErrorKind::UnterminatedString);
            }
            if ch == '\\' {
                self.bump();
                let esc = self.peek();
                self.bump();
                match esc {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('0') => value.push('\0'),
                    Some('{') => value.push('{'),
                    Some('}') => value.push('}'),
                    _ => return self.error_token(start, LexErrorKind::InvalidEscape),
                }
                continue;
            }
            value.push(ch);
            self.bump();
        }
    }
}

/// Lex all tokens in `source`, including a final [`TokenKind::Eof`].
pub fn lex(source: &str, file: FileId) -> Vec<Token> {
    let mut lexer = Lexer::new(file, source);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = matches!(tok.kind, TokenKind::Eof);
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    tokens
}
