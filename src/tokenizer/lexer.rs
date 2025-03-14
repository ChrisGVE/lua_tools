// src/lexer.rs

use crate::tokenizer::token::Span;

pub struct Lexer {
    pub input: Vec<char>,
    pub pos: usize,
    pub line: usize,
    pub column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    pub fn peek_n(&self, n: usize) -> Option<char> {
        self.input.get(self.pos + n).cloned()
    }

    pub fn current_char(&self) -> char {
        self.input[self.pos]
    }

    pub fn current_char_opt(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    pub fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    pub fn consume_whitespace(&mut self) {
        while self.pos < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    pub fn collect_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> String {
        let mut result = String::new();
        while self.pos < self.input.len() && predicate(self.current_char()) {
            result.push(self.current_char());
            self.advance();
        }
        result
    }

    pub fn collect_until(&mut self, delimiter: char) -> String {
        let mut result = String::new();
        while self.pos < self.input.len() && self.current_char() != delimiter {
            result.push(self.current_char());
            self.advance();
        }
        result
    }

    pub fn collect_until_str(&mut self, delimiter: &str) -> String {
        let mut result = String::new();
        while self.pos < self.input.len() {
            let remaining: String = self.input[self.pos..].iter().collect();
            if remaining.starts_with(delimiter) {
                break;
            }
            result.push(self.current_char());
            self.advance();
        }
        result
    }
}
