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
        // Remove the while loop and handle single token per call
        let Some(c) = self.current else {
            return Token::EndOfFile;
        };

        let token = match c {
            ' ' | '\t' | '\n' | '\r' => {
                self.advance();
                Token::Whitespace
            }

            '-' => {
                self.advance();
                if self.current == Some('-') {
                    self.advance();
                    let comment = self.consume_while(|c| c != '\n');
                    Token::Comment(comment)
                } else {
                    Token::Operator("-".to_string())
                }
            }

            'l' if self.peek_next() == Some('o') => {
                let keyword = self.consume_while(|c| c.is_alphanumeric());
                if keyword == "local" {
                    self.advance();
                    Token::Keyword(keyword)
                } else {
                    Token::Identifier(keyword)
                }
            }

            'r' if self.peek_next() == Some('e') => {
                let keyword = self.consume_while(|c| c.is_alphanumeric());
                if keyword == "require" {
                    Token::RequireCall
                } else {
                    Token::Identifier(keyword)
                }
            }

            '=' if self.peek_next() == Some('=') => {
                self.advance();
                self.advance();
                Token::Operator("==".to_string())
            }

            '{' => {
                self.advance();
                if self.current == Some('}') {
                    self.advance();
                    Token::TableConstructor
                } else {
                    Token::TableStart
                }
            }

            '}' => {
                self.advance();
                Token::TableEnd
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
                Token::StringLiteral(s)
            }

            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.consume_while(|c| c.is_alphanumeric() || c == '_' || c == '.');

                if ident.contains('.') {
                    let parts: Vec<&str> = ident.split('.').collect();
                    if parts.len() > 1 {
                        Token::ExportSymbol(parts[1].to_string())
                    } else {
                        Token::Identifier(ident)
                    }
                } else {
                    match ident.as_str() {
                        "function" => Token::Keyword(ident),
                        "true" => Token::Boolean(true),
                        "false" => Token::Boolean(false),
                        "nil" => Token::Nil,
                        _ => Token::Identifier(ident),
                    }
                }
            }

            '0'..='9' => {
                let num_str = self.consume_while(|c| c.is_ascii_digit() || c == '.');
                Token::Number(num_str.parse().unwrap_or(0.0))
            }

            _ => {
                if c.is_alphabetic() {
                    let ident = self.consume_while(|c| c.is_alphanumeric());
                    Token::Identifier(ident)
                } else {
                    let punct = c;
                    self.advance();
                    Token::Punctuation(punct)
                }
            }
        };

        token
    }
}
