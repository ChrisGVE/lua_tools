use regex::Regex;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(f64),
    StringLiteral(String),
    Operator(String),
    Punctuation(char),
    Comment(String),
    Whitespace,
    Boolean(bool),
    Nil,
    TableStart,
    TableEnd,
    ModuleKeyword,
    ExportSymbol(String),
    TableConstructor,
    RequireCall,
    EndOfFile,
    Error(String),
}

pub struct Tokenizer<'a> {
    input: Chars<'a>,
    current: Option<char>,
    lookahead: Option<char>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Tokenizer {
            input: input.chars(),
            current: None,
            lookahead: None,
        };
        tokenizer.advance();
        tokenizer
    }

    fn advance(&mut self) {
        self.current = self.lookahead.take().or_else(|| self.input.next());
        self.lookahead = self.input.next();
    }

    fn peek_next(&self) -> Option<char> {
        self.lookahead
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, condition: F) -> String {
        let mut result = String::new();
        while let Some(c) = self.current {
            if !condition(c) {
                break;
            }
            result.push(c);
            self.advance();
        }
        result
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Whitespace {
                continue;
            }
            if token == Token::EndOfFile {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                    return Token::Whitespace;
                }

                '-' => {
                    self.advance();
                    if self.current == Some('-') {
                        self.advance();
                        let comment = self.consume_while(|c| c != '\n');
                        return Token::Comment(comment);
                    }
                    return Token::Operator("-".to_string());
                }

                'l' if self.peek_next() == Some('o') => {
                    let keyword = self.consume_while(|c| c.is_alphanumeric());
                    if keyword == "local" {
                        self.advance();
                        return Token::Keyword(keyword);
                    }
                }

                'r' if self.peek_next() == Some('e') => {
                    let keyword = self.consume_while(|c| c.is_alphanumeric());
                    if keyword == "require" {
                        return Token::RequireCall;
                    }
                }

                '=' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    return Token::Operator("==".to_string());
                }

                '{' => {
                    self.advance();
                    if self.current == Some('}') {
                        self.advance();
                        return Token::TableConstructor;
                    }
                    return Token::TableStart;
                }

                '}' => {
                    self.advance();
                    return Token::TableEnd;
                }

                '\'' | '"' => {
                    let delimiter = c;
                    let mut s = String::new();
                    self.advance();

                    while let Some(c) = self.current {
                        if c == '\\' {
                            self.advance();
                            if let Some(escaped) = self.current {
                                s.push(escaped);
                                self.advance();
                            }
                        } else if c == delimiter {
                            self.advance();
                            break;
                        } else {
                            s.push(c);
                            self.advance();
                        }
                    }
                    return Token::StringLiteral(s);
                }

                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.consume_while(|c| c.is_alphanumeric() || c == '_');
                    match ident.as_str() {
                        "true" => return Token::Boolean(true),
                        "false" => return Token::Boolean(false),
                        "nil" => return Token::Nil,
                        _ => {
                            if self.current == Some('.') {
                                self.advance();
                                let field = self.consume_while(|c| c.is_alphanumeric() || c == '_');
                                return Token::ExportSymbol(field);
                            }
                            return Token::Identifier(ident);
                        }
                    }
                }

                '0'..='9' => {
                    let num_str = self.consume_while(|c| c.is_ascii_digit() || c == '.');
                    return Token::Number(num_str.parse().unwrap_or(0.0));
                }

                _ => {
                    if c.is_alphabetic() {
                        let ident = self.consume_while(|c| c.is_alphanumeric());
                        return Token::Identifier(ident);
                    }
                    let punct = c;
                    self.advance();
                    return Token::Punctuation(punct);
                }
            }
        }
        Token::EndOfFile
    }
}
