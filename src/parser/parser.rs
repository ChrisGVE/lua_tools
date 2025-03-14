// src/parser.rs

use crate::tokenizer::token::{Span, Token};
use crate::type_inference::{ScopeContext, TypeInfo};

#[derive(Debug, Clone)]
pub struct ExportItem {
    pub name: String,
    pub type_info: TypeInfo,
}

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    Literal(TypeInfo),
    FunctionCall(String, Vec<Expression>),
    TableFieldAccess(Box<Expression>, String),
}

#[derive(Debug)]
pub enum ASTNode {
    ModuleDeclaration {
        name: String,
        exports: Vec<ExportItem>,
    },
    FunctionDef {
        name: String,
        params: Vec<(String, TypeInfo)>,
        return_types: Vec<TypeInfo>,
        scope: ScopeContext,
        docs: Vec<String>,
        body: Vec<ASTNode>,
    },
    RequireStatement {
        module: String,
        alias: Option<String>,
    },
    TypeAlias {
        name: String,
        type_def: TypeInfo,
    },
    CommentBlock(String),
    TableConstructor(Vec<TableField>),
    ReturnStatement(Vec<Expression>),
}

#[derive(Debug)]
pub struct TableField {
    pub key: Option<Expression>,
    pub value: Expression,
    pub type_info: TypeInfo,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub current_scope: ScopeContext,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            current_scope: ScopeContext::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<ASTNode> {
        let mut ast = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(node) = self.parse_node() {
                ast.push(node);
            } else {
                self.advance();
            }
        }
        ast
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn expect_keyword(&mut self, kw: &str) -> bool {
        if let Some(Token::Keyword(ref s, _)) = self.peek() {
            if s == kw {
                self.advance();
                return true;
            }
        }
        false
    }

    /// When a declaration is about to be parsed, collect contiguous comment tokens
    /// (line, block, or annotation) immediately preceding it.
    fn collect_doc_block(&mut self) -> Vec<String> {
        let mut docs = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Comment(ref text, _)
                | Token::BlockComment(ref text, _)
                | Token::Annotation(ref text, _) => {
                    docs.push(text.clone());
                    self.advance();
                }
                _ => break,
            }
        }
        docs
    }

    fn parse_node(&mut self) -> Option<ASTNode> {
        // Collect doc block first.
        let docs = self.collect_doc_block();
        match self.peek()? {
            Token::Keyword(ref s, _) if s == "function" => self.parse_function_definition(docs),
            Token::Keyword(ref s, _) if s == "local" => self.parse_local_declaration(),
            Token::Keyword(ref s, _) if s == "return" => self.parse_return_statement(),
            // For a comment not attached to a declaration, return as a top-level comment block.
            Token::Comment(ref text, _)
            | Token::BlockComment(ref text, _)
            | Token::Annotation(ref text, _) => {
                let t = text.clone();
                self.advance();
                Some(ASTNode::CommentBlock(t))
            }
            _ => None,
        }
    }

    fn parse_function_definition(&mut self, docs: Vec<String>) -> Option<ASTNode> {
        self.expect_keyword("function"); // Consume "function" keyword.
        let name = self.parse_qualified_name()?;
        // Expect an opening parenthesis.
        if !self.match_token_variant("ParenOpen") {
            return None;
        }
        self.advance(); // Consume ParenOpen.
        let params = self.parse_parameters();
        // Expect a closing parenthesis.
        if !self.match_token_variant("ParenClose") {
            return None;
        }
        self.advance(); // Consume ParenClose.
        let body = self.parse_block()?;
        Some(ASTNode::FunctionDef {
            name,
            params,
            return_types: Vec::new(), // To be updated by type inference later.
            scope: self.current_scope.clone(),
            docs,
            body,
        })
    }

    /// Parses a qualified name (e.g. M.get_user) by consuming identifier tokens and dot operators.
    /// Since identifiers are now vectorized, we simply join their parts.
    fn parse_qualified_name(&mut self) -> Option<String> {
        let mut name = String::new();
        // Get the current token.
        if let Some(token) = self.peek() {
            match token {
                Token::Identifier(parts, _) => {
                    name.push_str(&parts.join("."));
                    self.advance();
                }
                Token::Keyword(s, _) => {
                    name.push_str(s);
                    self.advance();
                }
                _ => return None,
            }
        } else {
            return None;
        }
        // Consume dot operators and the following identifier tokens.
        while let Some(token) = self.peek() {
            if let Token::Operator(ref op, _) = token {
                if op == "." {
                    self.advance(); // Consume the dot.
                    if let Some(token) = self.peek() {
                        match token {
                            Token::Identifier(parts, _) => {
                                name.push('.');
                                name.push_str(&parts.join("."));
                                self.advance();
                            }
                            Token::Keyword(s, _) => {
                                name.push('.');
                                name.push_str(s);
                                self.advance();
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Some(name)
    }

    fn match_token_variant(&self, variant: &str) -> bool {
        if let Some(token) = self.peek() {
            match (variant, token) {
                ("ParenOpen", Token::ParenOpen(_)) => true,
                ("ParenClose", Token::ParenClose(_)) => true,
                ("BraceOpen", Token::BraceOpen(_)) => true,
                ("BraceClose", Token::BraceClose(_)) => true,
                ("BracketOpen", Token::BracketOpen(_)) => true,
                ("BracketClose", Token::BracketClose(_)) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn parse_parameters(&mut self) -> Vec<(String, TypeInfo)> {
        let mut params = Vec::new();
        while let Some(token) = self.peek() {
            // Stop if we encounter a closing parenthesis.
            if let Token::ParenClose(_) = token {
                break;
            }
            if let Token::Identifier(parts, _) = token {
                let param_name = parts.join(".");
                self.advance();
                // Parameter type is Unknown for now.
                params.push((param_name, TypeInfo::Unknown));
            } else {
                self.advance();
            }
            // If a comma operator appears, consume it.
            if let Some(Token::Operator(ref op, _)) = self.peek() {
                if op == "," {
                    self.advance();
                }
            }
        }
        params
    }

    fn parse_block(&mut self) -> Option<Vec<ASTNode>> {
        let mut body = Vec::new();
        while let Some(token) = self.peek() {
            // A block ends when the keyword "end" is encountered.
            if let Token::Keyword(ref s, _) = token {
                if s == "end" {
                    self.advance();
                    break;
                }
            }
            if let Some(node) = self.parse_node() {
                body.push(node);
            } else {
                self.advance();
            }
        }
        Some(body)
    }

    fn parse_local_declaration(&mut self) -> Option<ASTNode> {
        // Consume the "local" keyword.
        self.expect_keyword("local");

        // Get the variable name.
        let token = self.peek().cloned()?;
        let name = match token {
            Token::Identifier(parts, _) => {
                self.advance();
                parts.join(".")
            }
            _ => return None,
        };

        // Expect and consume an assignment operator.
        let token = self.peek().cloned()?;
        match token {
            Token::Assignment(_) => {
                self.advance();
            }
            _ => return None,
        }

        // Check if the assignment is a table constructor (module declaration).
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::BraceOpen(_) => {
                    self.advance(); // Consume the '{'
                    let exports = self.parse_table_exports();
                    if let Some(token) = self.peek().cloned() {
                        if let Token::BraceClose(_) = token {
                            self.advance(); // Consume the '}'
                        }
                    }
                    return Some(ASTNode::ModuleDeclaration { name, exports });
                }
                _ => {}
            }
        }
        None
    }

    fn parse_table_exports(&mut self) -> Vec<ExportItem> {
        let mut exports = Vec::new();
        while let Some(token) = self.peek() {
            if let Token::BraceClose(_) = token {
                break;
            }
            let field_name = if let Some(Token::Identifier(parts, _)) = self.peek() {
                let n = parts.join(".");
                self.advance();
                n
            } else if let Some(Token::StringLiteral(ref s, _)) = self.peek() {
                let n = s.clone();
                self.advance();
                n
            } else {
                self.advance();
                continue;
            };
            exports.push(ExportItem {
                name: field_name,
                type_info: TypeInfo::Unknown,
            });
            if let Some(Token::Operator(ref op, _)) = self.peek() {
                if op == "," {
                    self.advance();
                }
            }
        }
        exports
    }

    fn parse_return_statement(&mut self) -> Option<ASTNode> {
        self.expect_keyword("return");
        let mut expressions = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Identifier(parts, _) => {
                    expressions.push(Expression::Identifier(parts.join(".")));
                    self.advance();
                }
                Token::NumberLiteral(ref s, _) => {
                    expressions.push(Expression::Literal(TypeInfo::Unknown));
                    self.advance();
                }
                Token::StringLiteral(ref s, _) => {
                    expressions.push(Expression::Literal(TypeInfo::Unknown));
                    self.advance();
                }
                Token::Operator(ref op, _) if op == "," => {
                    self.advance();
                }
                _ => break,
            }
        }
        Some(ASTNode::ReturnStatement(expressions))
    }

    // A simple helper returning a placeholder span.
    fn current_span_placeholder(&self) -> Span {
        Span::new(0, 0, 0, 0)
    }
}
