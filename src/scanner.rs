use crate::print_error;
use crate::token::TokenType::*;
use crate::token::{Literal, Token, TokenType};
use std::collections::HashMap;

pub struct Scanner<'a> {
    source: String,
    tokens: &'a mut Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: String, tokens: &'a mut Vec<Token>) -> Self {
        Scanner {
            source,
            tokens,
            start: 0,
            current: 0,
            line: 1,
            keywords: Self::initialize_keywords(),
        }
    }

    fn initialize_keywords() -> HashMap<String, TokenType> {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), AND);
        keywords.insert("class".to_string(), CLASS);
        keywords.insert("else".to_string(), ELSE);
        keywords.insert("false".to_string(), FALSE);
        keywords.insert("for".to_string(), FOR);
        keywords.insert("fun".to_string(), FUN);
        keywords.insert("if".to_string(), IF);
        keywords.insert("nil".to_string(), NIL);
        keywords.insert("or".to_string(), OR);
        keywords.insert("print".to_string(), PRINT);
        keywords.insert("return".to_string(), RETURN);
        keywords.insert("super".to_string(), SUPER);
        keywords.insert("this".to_string(), THIS);
        keywords.insert("true".to_string(), TRUE);
        keywords.insert("var".to_string(), VAR);
        keywords.insert("while".to_string(), WHILE);
        keywords
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            token_type: EOF,
            lexeme: "".to_string(),
            literal: Literal::None,
            line: self.line,
        });
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(LEFT_PAREN, Literal::None),
            ')' => self.add_token(RIGHT_PAREN, Literal::None),
            '{' => self.add_token(LEFT_BRACE, Literal::None),
            '}' => self.add_token(RIGHT_BRACE, Literal::None),
            ',' => self.add_token(COMMA, Literal::None),
            '.' => self.add_token(DOT, Literal::None),
            '-' => self.add_token(MINUS, Literal::None),
            '+' => self.add_token(PLUS, Literal::None),
            ';' => self.add_token(SEMICOLON, Literal::None),
            '*' => self.add_token(STAR, Literal::None),
            '!' => {
                let token = if self.match_char('=') {
                    BANG_EQUAL
                } else {
                    BANG
                };
                self.add_token(token, Literal::None)
            }
            '=' => {
                let token = if self.match_char('=') {
                    EQUAL_EQUAL
                } else {
                    EQUAL
                };
                self.add_token(token, Literal::None)
            }
            '<' => {
                let token = if self.match_char('=') {
                    LESS_EQUAL
                } else {
                    LESS
                };
                self.add_token(token, Literal::None)
            }
            '>' => {
                let token = if self.match_char('=') {
                    GREATER_EQUAL
                } else {
                    GREATER
                };
                self.add_token(token, Literal::None)
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(SLASH, Literal::None)
                }
            }
            '\r' | '\t' | ' ' => {}
            '\n' => self.line = self.line + 1,
            '"' => self.string(),
            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    print_error(
                        self.line,
                        String::from(c),
                        &format!("Unexpected character: {}", c),
                    );
                }
            }
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line = self.line + 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            print_error(self.line, "at end".to_string(), "Unterminated string");
            return;
        }

        self.advance(); // consume "

        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(STRING, Literal::String(String::from(value)));
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token = self.keywords.get(text).unwrap_or_else(|| &IDENTIFIER);

        self.add_token(token.clone(), Literal::None);
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c) || c == '_'
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        self.add_token(
            NUMBER,
            Literal::Number(
                self.source[self.start..self.current]
                    .parse::<f64>()
                    .unwrap(),
            ),
        );
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if char::from(self.source.as_bytes()[self.current]) != expected {
            return false;
        }
        self.current = self.current + 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        })
    }

    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current];
        self.current = self.current + 1;
        char::from(c)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        char::from(self.source.as_bytes()[self.current])
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        char::from(self.source.as_bytes()[self.current + 1])
    }
}
