use crate::tokenizer::Token;
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
    pub type_info: TypeInfo, // Added type info
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            current_scope: ScopeContext::new(),
        }
    }

    #[allow(dead_code)]
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    #[allow(dead_code)]
    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token) // Fixed missing closing parenthesis
        } else {
            None
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    fn parse_params(&mut self) -> Option<Vec<(String, TypeInfo)>> {
        // Consume opening '('
        if self.consume()? != Token::Punctuation('(') {
            return None;
        }

        let mut params = Vec::new();

        while self.peek() != Some(&Token::Punctuation(')')) {
            // Parse parameter name
            let name = match self.consume()? {
                Token::Identifier(name) => name,
                _ => return None,
            };

            // Default to unknown type (will be inferred later)
            params.push((name, TypeInfo::Unknown));

            // Handle commas between parameters
            if self.peek() == Some(&Token::Punctuation(',')) {
                self.consume()?;
            }
        }

        // Consume closing ')'
        self.consume()?;

        Some(params)
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    // src/parser.rs
    fn parse_function_definition(&mut self) -> ASTNode {
        self.consume(); // 'function' keyword

        // Get full function name (e.g. "M.get_user")
        let full_name = match self.consume() {
            Some(Token::ExportSymbol(name)) => name,
            Some(Token::Identifier(name)) => name,
            _ => String::new(),
        };

        // Split into module and function name
        let (module, name) = match full_name.split_once('.') {
            Some((m, n)) => (m.to_string(), n.to_string()),
            None => ("".to_string(), full_name),
        };

        // Parse parameters with proper error handling
        let params = self.parse_params().unwrap_or_else(|| {
            eprintln!("Warning: Failed to parse function parameters");
            Vec::new()
        });

        // Parse body and infer return type
        let body = self.parse_block();
        let return_types = self.infer_return_type(&body);

        ASTNode::FunctionDef {
            name,
            params,
            return_types,
            scope: self.current_scope.clone(),
            docs: Vec::new(),
            body,
        }
    }

    // Add to Parser impl
    #[allow(dead_code)]
    fn infer_return_type(&self, body: &[ASTNode]) -> Vec<TypeInfo> {
        body.iter()
            .filter_map(|node| match node {
                ASTNode::ReturnStatement(_) => Some(TypeInfo::Unknown),
                _ => None,
            })
            .collect()
    }

    #[allow(dead_code)]
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

    fn parse_local_assignment(&mut self) -> Option<ASTNode> {
        // Consume 'local' keyword (already verified by caller)
        self.consume()?;

        // Parse identifier (module name)
        let name = match self.consume()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        // Consume '=' operator
        if self.consume() != Some(Token::Operator("=".to_string())) {
            return None;
        }

        // Check for table constructor
        match self.peek()? {
            Token::TableConstructor => {
                self.consume(); // Consume '{'
                let exports = self.parse_table_exports();
                self.consume(); // Consume '}'

                Some(ASTNode::ModuleDeclaration { name, exports })
            }
            _ => None, // Handle other assignments later
        }
    }

    fn parse_table_exports(&mut self) -> Vec<ExportItem> {
        let mut exports = Vec::new();
        while self.peek() != Some(&Token::TableEnd) {
            if let Some(Token::ExportSymbol(field)) = self.consume() {
                exports.push(ExportItem {
                    name: field,
                    type_info: TypeInfo::Unknown,
                });
            }
            if self.peek() == Some(&Token::Punctuation(',')) {
                self.consume();
            }
        }
        exports
    }
    // Add to Parser impl
    #[allow(dead_code)]
    fn parse_node(&mut self) -> Option<ASTNode> {
        match self.peek()? {
            Token::Keyword(kw) if kw == "function" => Some(self.parse_function_definition()),
            Token::Keyword(kw) if kw == "local" => self.parse_local_assignment(),
            Token::Comment(_) => {
                let comment = self.consume()?.as_comment()?;
                Some(ASTNode::CommentBlock(comment))
            }
            _ => {
                self.consume();
                None
            }
        }
    }
}
