use crate::tokenizer::Token;
use crate::type_inference::{ScopeContext, TypeInfo};
use std::collections::HashMap;

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
    pub type_info: TypeInfo, // Added type info
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    current_scope: ScopeContext,
}

impl Token {
    fn as_comment(&self) -> Option<String> {
        match self {
            Token::Comment(c) => Some(c.clone()),
            _ => None,
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            current_scope: ScopeContext::new(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token) // Fixed missing closing parenthesis
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Vec<ASTNode> {
        let mut ast = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Keyword(kw) if kw == "local" => {
                    self.consume();
                    if let Some(Token::Identifier(name)) = self.consume() {
                        if self.peek() == Some(&Token::Operator("=".to_string())) {
                            self.consume();
                            ast.push(self.parse_module_declaration(&name));
                        }
                    }
                }
                Token::RequireCall => {
                    ast.push(self.parse_require_statement());
                }
                Token::Comment(comment) => {
                    ast.push(ASTNode::CommentBlock(comment.clone()));
                    self.consume();
                }
                _ => {
                    self.consume();
                }
            }
        }
        ast
    }

    fn parse_module_declaration(&mut self, name: &str) -> ASTNode {
        self.consume(); // Consume '='

        // Handle table constructor detection
        if self.consume() != Some(Token::TableConstructor) {
            return ASTNode::ModuleDeclaration {
                name: name.to_string(),
                exports: Vec::new(),
            };
        }

        let mut exports = Vec::new();
        loop {
            match self.peek() {
                Some(Token::ExportSymbol(field)) => {
                    exports.push(ExportItem {
                        name: field.clone(),
                        type_info: TypeInfo::Unknown,
                    });
                    self.consume();
                }
                Some(Token::TableEnd) => {
                    self.consume();
                    break;
                }
                Some(_) => {
                    self.consume(); // Simply discard other tokens
                }
                None => break,
            }
        }

        ASTNode::ModuleDeclaration {
            name: name.to_string(),
            exports,
        }
    }

    fn parse_require_statement(&mut self) -> ASTNode {
        let module = if let Some(Token::StringLiteral(s)) = self.consume() {
            s
        } else {
            String::new()
        };
        ASTNode::RequireStatement {
            module,
            alias: None,
        }
    }

    fn parse_function_definition(&mut self) -> ASTNode {
        self.consume(); // Consume 'function'

        let name = match self.consume() {
            Some(Token::Identifier(name)) => name,
            _ => String::new(),
        };

        // Parse parameters
        let mut params = Vec::new();
        if self.consume() == Some(Token::Punctuation('(')) {
            while self.peek() != Some(&Token::Punctuation(')')) {
                if let Some(Token::Identifier(param_name)) = self.consume() {
                    params.push((param_name, TypeInfo::Unknown));
                }
                if self.peek() == Some(&Token::Punctuation(',')) {
                    self.consume();
                }
            }
            self.consume(); // Consume ')'
        }

        // Parse body and infer return type
        let body = self.parse_block();
        let return_info = self.infer_return_type(&body);

        let body = self.parse_block();

        ASTNode::FunctionDef {
            name,
            params,
            return_types: return_info,
            scope: self.current_scope.clone(),
            docs: Vec::new(),
            body,
        }
    }

    // Add to Parser impl
    fn infer_return_type(&self, body: &[ASTNode]) -> Vec<TypeInfo> {
        body.iter()
            .filter_map(|node| match node {
                ASTNode::ReturnStatement(_) => Some(TypeInfo::Unknown),
                _ => None,
            })
            .collect()
    }

    fn parse_block(&mut self) -> Vec<ASTNode> {
        let mut body = Vec::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek() {
                Some(Token::Keyword(kw)) if kw == "end" => {
                    depth -= 1;
                    self.consume();
                }
                Some(Token::Keyword(kw)) if kw == "function" => {
                    depth += 1;
                    body.push(self.parse_function_definition());
                }
                _ => {
                    if let Some(node) = self.parse_node() {
                        body.push(node);
                    } else {
                        self.consume();
                    }
                }
            }
        }
        body
    }

    // Add to Parser impl
    fn parse_node(&mut self) -> Option<ASTNode> {
        match self.peek()? {
            Token::Keyword(kw) if kw == "function" => Some(self.parse_function_definition()),
            Token::Comment(_) => {
                let comment = self.consume()?.as_comment()?;
                Some(ASTNode::CommentBlock(comment))
            }
            _ => None,
        }
    }
}
