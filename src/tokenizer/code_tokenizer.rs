// src/tokenizer/code_tokenizer.rs

use crate::tokenizer::annotation_tokenizer::parse_annotation_subtokens;
use crate::tokenizer::lexer::Lexer;
use crate::tokenizer::token::{AnnotationSubToken, Span, Token};

pub struct CodeTokenizer {
    pub lexer: Lexer,
}

impl CodeTokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.lexer.pos < self.lexer.input.len() {
            let ch = self.lexer.current_char();
            if ch.is_whitespace() {
                self.lexer.consume_whitespace();
                continue;
            }
            // Comments and annotations.
            else if ch == '-' && self.lexer.peek() == Some('-') {
                let start_pos = self.lexer.pos;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                // Consume the first two dashes.
                self.lexer.advance();
                self.lexer.advance();
                // Check for block comment open marker: exactly "--[["
                if self.lexer.current_char_opt() == Some('[') && self.lexer.peek_n(1) == Some('[') {
                    let open_span = Span::new(start_pos, start_pos + 4, start_line, start_col);
                    self.lexer.advance(); // consume first '['
                    self.lexer.advance(); // consume second '['
                    let content = self.lexer.collect_until_str("--]]");
                    let content_span = Span::new(
                        self.lexer.pos,
                        self.lexer.pos + content.len(),
                        start_line,
                        start_col,
                    );
                    tokens.push(Token::BlockComment(content, content_span));
                    self.lexer.advance_by(4); // consume "--]]"
                    continue;
                }
                // Check for annotation (if a third dash is present).
                if let Some(third_char) = self.lexer.current_char_opt() {
                    if third_char == '-' {
                        if let Some(prefix_char) = self.lexer.peek_n(1) {
                            if prefix_char == '@' || prefix_char == '|' {
                                self.lexer.advance(); // consume third dash
                                self.lexer.advance(); // consume the prefix character
                                let annotation_body = self.lexer.collect_until('\n');
                                let span =
                                    Span::new(start_pos, self.lexer.pos, start_line, start_col);
                                // Tokenize the annotation text into annotation subtokens.
                                let mut text = String::new();
                                text.push(prefix_char);
                                text.push_str(&annotation_body);
                                let subtokens = parse_annotation_subtokens(&format!("---{}", text));
                                tokens.push(Token::Annotation(subtokens, span));
                                continue;
                            }
                        }
                    }
                }
                // Otherwise, it's a normal comment.
                let comment = self.lexer.collect_until('\n');
                let span = Span::new(start_pos, self.lexer.pos, start_line, start_col);
                tokens.push(Token::Comment(comment, span));
                continue;
            }
            // ... (other tokenization logic for identifiers, numbers, strings, etc.) ...
            // Identifiers or keywords.
            else if ch.is_alphabetic() || ch == '_' {
                let start_pos = self.lexer.pos;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let ident = self
                    .lexer
                    .collect_while(|c| c.is_alphanumeric() || c == '_');
                let span = Span::new(start_pos, self.lexer.pos, start_line, start_col);
                // For simplicity, assume that if ident is a reserved keyword we return a Keyword,
                // else an Identifier (here you can add more logic for dotted identifiers).
                if is_keyword(&ident) {
                    tokens.push(Token::Keyword(ident, span));
                } else {
                    tokens.push(Token::Identifier(vec![ident], span));
                }
            }
            // ... (handle numbers, strings, operators, punctuation, etc.) ...
            else if ch.is_digit(10) {
                let start_pos = self.lexer.pos;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let number = self.lexer.collect_while(|c| c.is_digit(10));
                let span = Span::new(start_pos, self.lexer.pos, start_line, start_col);
                tokens.push(Token::NumberLiteral(number, span));
            } else if ch == '"' || ch == '\'' {
                let start_pos = self.lexer.pos;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let quote = ch;
                self.lexer.advance(); // consume opening quote
                let string_val = self.lexer.collect_until(quote);
                self.lexer.advance(); // consume closing quote
                let span = Span::new(start_pos, self.lexer.pos, start_line, start_col);
                tokens.push(Token::StringLiteral(string_val, span));
            }
            // Operators and punctuation are handled similarly...
            else {
                let start_pos = self.lexer.pos;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let span = Span::new(start_pos, start_pos + 1, start_line, start_col);
                tokens.push(Token::Operator(ch.to_string(), span));
                self.lexer.advance();
            }
        }
        tokens
    }
}

fn is_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "and"
            | "break"
            | "do"
            | "else"
            | "elseif"
            | "end"
            | "false"
            | "for"
            | "function"
            | "if"
            | "in"
            | "local"
            | "nil"
            | "not"
            | "or"
            | "repeat"
            | "return"
            | "then"
            | "true"
            | "until"
            | "while"
            | "require"
    )
}
