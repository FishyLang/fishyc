use std::collections::HashMap;
use std::sync::Arc;

use crate::token::{ Literal, Token, TokenType };

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

pub struct Scanner {
    source: Vec<char>,
    pub tokens: Vec<Token>,
    pub errors: Vec<ParseError>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    interpolation_depth: usize,
    keywords: HashMap<&'static str, TokenType>,
    file_path: Arc<String>,
}

impl Scanner {
    pub fn new(source: &str, file_path: &str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("struct", TokenType::Struct);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fn", TokenType::Fn);
        keywords.insert("if", TokenType::If);
        keywords.insert("null", TokenType::Null);
        keywords.insert("return", TokenType::Return);
        keywords.insert("self", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("let", TokenType::Let);
        keywords.insert("while", TokenType::While);
        keywords.insert("extends", TokenType::Extends);
        keywords.insert("in", TokenType::In);
        keywords.insert("trait", TokenType::Trait);
        keywords.insert("with", TokenType::With);
        keywords.insert("throw", TokenType::Throw);
        keywords.insert("enum", TokenType::Enum);
        keywords.insert("is", TokenType::Is);
        keywords.insert("abstract", TokenType::Abstract);
        keywords.insert("typeof", TokenType::Typeof);
        keywords.insert("lazy", TokenType::Lazy);
        keywords.insert("catch", TokenType::Catch);
        keywords.insert("finally", TokenType::Finally);
        keywords.insert("using", TokenType::Using);
        keywords.insert("from", TokenType::From);
        keywords.insert("union", TokenType::Union);
        keywords.insert("match", TokenType::Match);
        keywords.insert("impl", TokenType::Impl);
        keywords.insert("async", TokenType::Async);
        keywords.insert("await", TokenType::Await);
        keywords.insert("try", TokenType::Try);
        keywords.insert("new", TokenType::New);
        keywords.insert("alias", TokenType::Alias);
        keywords.insert("extern", TokenType::Extern);
        keywords.insert("mut", TokenType::Mut);
        keywords.insert("const", TokenType::Const);
        keywords.insert("as", TokenType::As);
        keywords.insert("pub", TokenType::Pub);

        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 0,
            interpolation_depth: 0,
            keywords,
            file_path: Arc::new(file_path.to_string()),
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(
            Token::new(
                TokenType::Eof,
                String::new(),
                Literal::None,
                self.line,
                self.column + 1,
                Arc::clone(&self.file_path)
            )
        );
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.column += 1;
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' } else { self.source[self.current + 1] }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, Literal::None);
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Literal) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(
            Token::new(
                token_type,
                lexeme,
                literal,
                self.line,
                self.column,
                Arc::clone(&self.file_path)
            )
        );
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || c.is_ascii_digit()
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = self.keywords.get(text.as_str()).cloned().unwrap_or(TokenType::Identifier);

        self.add_token(token_type);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let is_float = self.peek() == '.' && self.peek_next().is_ascii_digit();

        if is_float {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value_str: String = self.source[self.start..self.current].iter().collect();

        if is_float {
            let value: f64 = value_str.parse().unwrap_or(0.0);
            self.add_token_literal(TokenType::Number, Literal::Number(value));
        } else {
            let value: i64 = value_str.parse().unwrap_or(0);
            self.add_token_literal(TokenType::Number, Literal::Integer(value));
        }
    }

    fn string(&mut self) {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '$' && self.peek_next() == '{' {
                let trim_start = if self.source[self.start] == '"' {
                    self.start + 1
                } else {
                    self.start
                };
                let value: String = self.source[trim_start..self.current].iter().collect();

                self.add_token_literal(TokenType::StringPart, Literal::String(value));

                self.advance(); // consumes '$'
                self.advance(); // consumes '{'
                self.interpolation_depth = 1;
                self.add_token(TokenType::InterpolationStart);
                return;
            }

            // ignore escaped characters
            if self.peek() == '\\' {
                self.advance(); // consume '\'
                if !self.is_at_end() {
                    self.advance(); // consume escaped char
                }
                continue;
            }

            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.errors.push(ParseError {
                line: self.line,
                column: self.column,
                message: "Unterminated string.".to_string(),
            });
            return;
        }

        self.advance();

        let mut begin_index = if self.source[self.start] == '"' {
            self.start + 1
        } else {
            self.start
        };
        begin_index = std::cmp::min(begin_index, self.current.saturating_sub(1));

        let raw_value: String = self.source[begin_index..self.current - 1].iter().collect();

        let value = raw_value
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");

        self.add_token_literal(TokenType::String, Literal::String(value));
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),

            '.' => {
                let token = if self.match_char('.') {
                    if self.match_char('.') { TokenType::Spread } else { TokenType::DotDot }
                } else {
                    TokenType::Dot
                };

                self.add_token(token);
            }

            '-' => {
                let token = if self.match_char('-') {
                    TokenType::MinusMinus
                } else if self.match_char('=') {
                    TokenType::MinusEqual
                } else if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                };

                self.add_token(token);
            }
            '+' => {
                let token = if self.match_char('+') {
                    TokenType::PlusPlus
                } else if self.match_char('=') {
                    TokenType::PlusEqual
                } else {
                    TokenType::Plus
                };

                self.add_token(token);
            }

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        if self.peek() == '/' && self.peek_next() == '*' {
                            self.advance();
                            self.advance();
                            depth += 1;
                        } else if self.peek() == '*' && self.peek_next() == '/' {
                            self.advance();
                            self.advance();
                            depth -= 1;
                        } else {
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 0;
                            }
                            self.advance();
                        }
                    }
                    if depth > 0 {
                        self.errors.push(ParseError {
                            line: self.line,
                            column: self.column,
                            message: "Comment block not closed.".to_string(),
                        });
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            ' ' | '\r' | '\t' => {}

            '\n' => {
                self.line += 1;
                self.column = 0;
            }

            '{' => {
                if self.interpolation_depth > 0 {
                    self.interpolation_depth += 1;
                }

                self.add_token(TokenType::LeftBrace);
            }

            '}' => {
                self.add_token(TokenType::RightBrace);

                if self.interpolation_depth > 0 {
                    self.interpolation_depth -= 1;
                    if self.interpolation_depth == 0 {
                        self.start = self.current;
                        self.string();
                    }
                }
            }

            '"' => self.string(),

            '%' => self.add_token(TokenType::Percent),

            '^' => self.add_token(TokenType::Caret),

            '|' => {
                let token = if self.match_char('|') {
                    TokenType::LogicalOr
                } else {
                    TokenType::BitwiseOr
                };

                self.add_token(token);
            }

            '&' => {
                let token = if self.match_char('&') {
                    TokenType::LogicalAnd
                } else {
                    TokenType::BitwiseAnd
                };

                self.add_token(token);
            }

            '?' => {
                let token = if self.match_char('?') {
                    TokenType::QuestionQuestion
                } else {
                    TokenType::Question
                };

                self.add_token(token);
            }

            '*' => {
                let token = if self.match_char('*') {
                    TokenType::Exponentiation
                } else {
                    TokenType::Star
                };

                self.add_token(token);
            }

            '!' => {
                let token = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(token);
            }

            '=' => {
                let token = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(token);
            }

            '<' => {
                let token = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(token);
            }

            '>' => {
                let token = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token);
            }

            ':' => self.add_token(TokenType::Colon),

            ';' => self.add_token(TokenType::Semicolon),

            '$' =>
                self.errors.push(ParseError {
                    line: self.line,
                    column: self.column,
                    message: "Unexpected character '$'. If you want to do string interpolation, use '${}'.".to_string(),
                }),

            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.errors.push(ParseError {
                        line: self.line,
                        column: self.column,
                        message: format!("Unexpected character: '{}'", c),
                    });
                }
            }
        }
    }
}
