// src/parser/code_parser.rs

use crate::parser::ast::{CodeASTNode, ExportItem, Expression, TypeInfo};
use crate::parser::parser_helpers;
use crate::tokenizer::token::{Span, Token};

pub struct CodeParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl CodeParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<CodeASTNode> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            } else {
                self.advance();
            }
        }
        nodes
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    /// Skip any annotation tokens, returning when the next token is a code token.
    fn skip_annotation_tokens(&mut self) {
        while let Some(token) = self.peek() {
            if parser_helpers::extract_annotation_token(token).is_some() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// If the next token is a Comment, consume it and return its text.
    fn parse_doc(&mut self) -> Option<String> {
        if let Some(Token::Comment(text, _)) = self.peek().cloned() {
            self.advance();
            Some(text.clone())
        } else {
            None
        }
    }

    /// Main dispatch: first skip annotation tokens, then decide how to parse the next code node.
    fn parse_node(&mut self) -> Option<CodeASTNode> {
        self.skip_annotation_tokens();
        let doc = self.parse_doc();
        let token = self.peek()?.clone();
        match token {
            Token::Keyword(ref s, _) if s == "function" => self.parse_function_def(doc),
            Token::Keyword(ref s, _) if s == "local" => self.parse_variable_declaration(doc),
            Token::Keyword(ref s, _) if s == "return" => self.parse_return_statement(doc),
            Token::Keyword(ref s, _) if s == "if" => self.parse_if_statement(doc),
            Token::Keyword(ref s, _) if s == "while" => self.parse_while_loop(doc),
            Token::Keyword(ref s, _) if s == "for" => self.parse_for_numeric(doc),
            Token::Keyword(ref s, _) if s == "do" => self.parse_do_block(doc),
            Token::Keyword(ref s, _) if s == "repeat" => self.parse_repeat_until(doc),
            Token::Identifier(_, _) => {
                if self.peek_assignment() {
                    self.parse_assignment(doc)
                } else if self.peek_function_call() {
                    self.parse_function_call_stmt(doc)
                } else {
                    None
                }
            }
            Token::BraceOpen(_) => self.parse_table_constructor(),
            _ => None,
        }
    }

    fn peek_assignment(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map_or(false, |token| matches!(token, Token::Assignment(_)))
    }

    fn peek_function_call(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map_or(false, |token| matches!(token, Token::ParenOpen(_)))
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

    // --- Parsing Functions for Code AST Nodes ---

    fn parse_function_def(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "function"
        let name = self.parse_qualified_name()?;
        if !self.match_token_variant("ParenOpen") {
            return None;
        }
        self.advance(); // consume '('
        let params = self.parse_parameters();
        if !self.match_token_variant("ParenClose") {
            return None;
        }
        self.advance(); // consume ')'
        let body = self.parse_block();
        Some(CodeASTNode::FunctionDef {
            name,
            params,
            return_types: vec![],
            doc,
            annotations: vec![],
            body,
        })
    }

    fn parse_qualified_name(&mut self) -> Option<String> {
        let mut name = String::new();
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(parts, _) => {
                    name.push_str(&parts.join("."));
                    self.advance();
                }
                Token::Keyword(s, _) => {
                    name.push_str(&s);
                    self.advance();
                }
                _ => return None,
            }
        } else {
            return None;
        }
        while let Some(token) = self.peek().cloned() {
            if let Token::Operator(ref op, _) = token {
                if op == "." {
                    self.advance(); // consume dot
                    if let Some(next_token) = self.peek().cloned() {
                        match next_token {
                            Token::Identifier(parts, _) => {
                                name.push('.');
                                name.push_str(&parts.join("."));
                                self.advance();
                            }
                            Token::Keyword(s, _) => {
                                name.push('.');
                                name.push_str(&s);
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

    fn parse_parameters(&mut self) -> Vec<(String, TypeInfo)> {
        let mut params = Vec::new();
        while let Some(token) = self.peek().cloned() {
            if let Token::ParenClose(_) = token {
                break;
            }
            if let Token::Identifier(parts, _) = token {
                let param_name = parts.join(".");
                self.advance();
                params.push((param_name, TypeInfo::Unknown));
            } else {
                self.advance();
            }
            if let Some(Token::Operator(ref op, _)) = self.peek().cloned() {
                if op == "," {
                    self.advance();
                }
            }
        }
        params
    }

    fn parse_block(&mut self) -> Vec<CodeASTNode> {
        let mut nodes = Vec::new();
        while let Some(token) = self.peek().cloned() {
            if let Token::Keyword(ref s, _) = token {
                if s == "end" {
                    self.advance(); // consume "end"
                    break;
                }
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            } else {
                self.advance();
            }
        }
        nodes
    }

    fn parse_variable_declaration(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "local"
        let name = if let Some(Token::Identifier(parts, _)) = self.peek().cloned() {
            let n = parts.join(".");
            self.advance();
            n
        } else {
            return None;
        };
        if let Some(Token::Assignment(_)) = self.peek().cloned() {
            self.advance(); // consume '='
                            // If initializer is a table constructor, treat as a module declaration.
            if let Some(Token::BraceOpen(_)) = self.peek().cloned() {
                let exports = self.parse_table_exports();
                Some(CodeASTNode::ModuleDeclaration {
                    name,
                    exports,
                    doc,
                    annotations: vec![],
                })
            } else {
                let expr = self.parse_expression();
                Some(CodeASTNode::VariableDeclaration {
                    name,
                    value: expr.map(|e| Box::new(CodeASTNode::ReturnStatement(vec![e]))),
                    doc,
                    annotations: vec![],
                })
            }
        } else {
            Some(CodeASTNode::VariableDeclaration {
                name,
                value: None,
                doc,
                annotations: vec![],
            })
        }
    }

    fn parse_assignment(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        // Assume a single identifier on the LHS.
        let lhs = if let Some(Token::Identifier(parts, _)) = self.peek().cloned() {
            let id = parts.join(".");
            self.advance();
            vec![id]
        } else {
            return None;
        };
        if let Some(Token::Assignment(_)) = self.peek().cloned() {
            self.advance(); // consume '='
        } else {
            return None;
        }
        let rhs_expr = self.parse_expression()?;
        Some(CodeASTNode::Assignment {
            lhs,
            rhs: vec![rhs_expr],
            doc,
            annotations: vec![],
        })
    }

    fn parse_return_statement(&mut self, _doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "return"
        let mut exprs = Vec::new();
        while let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(parts, _) => {
                    exprs.push(Expression::Identifier(parts.join(".")));
                    self.advance();
                }
                Token::NumberLiteral(s, _) => {
                    exprs.push(Expression::Literal(s.clone()));
                    self.advance();
                }
                Token::StringLiteral(s, _) => {
                    exprs.push(Expression::Literal(s.clone()));
                    self.advance();
                }
                Token::Operator(ref op, _) if op == "," => {
                    self.advance();
                }
                _ => break,
            }
        }
        Some(CodeASTNode::ReturnStatement(exprs))
    }

    fn parse_table_constructor(&mut self) -> Option<CodeASTNode> {
        if !self.match_token_variant("BraceOpen") {
            return None;
        }
        self.advance(); // consume '{'
        let mut fields = Vec::new();
        while let Some(token) = self.peek().cloned() {
            if let Token::BraceClose(_) = token {
                self.advance();
                break;
            }
            let key = if let Some(Token::Identifier(parts, _)) = self.peek().cloned() {
                let k = parts.join(".");
                self.advance();
                k
            } else if let Some(Token::StringLiteral(s, _)) = self.peek().cloned() {
                let k = s.clone();
                self.advance();
                k
            } else {
                self.advance();
                continue;
            };
            if let Some(Token::Operator(op, _)) = self.peek().cloned() {
                if op == "=" {
                    self.advance(); // consume '='
                }
            }
            let value = self.parse_expression()?;
            fields.push((key, value));
            if let Some(Token::Operator(ref op, _)) = self.peek().cloned() {
                if op == "," {
                    self.advance();
                }
            }
        }
        Some(CodeASTNode::TableConstructor(fields))
    }

    fn parse_table_exports(&mut self) -> Vec<ExportItem> {
        let mut exports = Vec::new();
        if !self.match_token_variant("BraceOpen") {
            return exports;
        }
        self.advance(); // consume '{'
        while let Some(token) = self.peek().cloned() {
            if let Token::BraceClose(_) = token {
                self.advance();
                break;
            }
            let name = if let Some(Token::Identifier(parts, _)) = self.peek().cloned() {
                let n = parts.join(".");
                self.advance();
                n
            } else {
                self.advance();
                continue;
            };
            exports.push(ExportItem {
                name,
                type_info: TypeInfo::Unknown,
            });
            if let Some(Token::Operator(ref op, _)) = self.peek().cloned() {
                if op == "," {
                    self.advance();
                }
            }
        }
        exports
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(parts, _) => {
                    let expr = Expression::Identifier(parts.join("."));
                    self.advance();
                    Some(expr)
                }
                Token::NumberLiteral(s, _) => {
                    let expr = Expression::Literal(s.clone());
                    self.advance();
                    Some(expr)
                }
                Token::StringLiteral(s, _) => {
                    let expr = Expression::Literal(s.clone());
                    self.advance();
                    Some(expr)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse_if_statement(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "if"
        let condition = self.parse_expression()?;
        let then_block = self.parse_block();
        let mut else_block = None;
        if let Some(token) = self.peek().cloned() {
            if let Token::Keyword(ref s, _) = token {
                if s == "else" {
                    self.advance();
                    else_block = Some(self.parse_block());
                }
            }
        }
        Some(CodeASTNode::IfStatement {
            condition,
            then_block,
            else_block,
            doc,
            annotations: vec![],
        })
    }

    fn parse_while_loop(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "while"
        let condition = self.parse_expression()?;
        let body = self.parse_block();
        Some(CodeASTNode::WhileLoop {
            condition,
            body,
            doc,
            annotations: vec![],
        })
    }

    fn parse_for_numeric(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "for"
        let var = if let Some(Token::Identifier(parts, _)) = self.peek().cloned() {
            let v = parts.join(".");
            self.advance();
            v
        } else {
            return None;
        };
        if let Some(Token::Assignment(_)) = self.peek().cloned() {
            self.advance();
        } else {
            return None;
        }
        let start = self.parse_expression()?;
        if let Some(Token::Operator(ref op, _)) = self.peek().cloned() {
            if op == "," {
                self.advance();
            }
        }
        let end = self.parse_expression()?;
        let mut step = None;
        if let Some(Token::Operator(ref op, _)) = self.peek().cloned() {
            if op == "," {
                self.advance();
                step = self.parse_expression();
            }
        }
        let body = self.parse_block();
        Some(CodeASTNode::ForNumeric {
            var,
            start,
            end,
            step,
            body,
            doc,
            annotations: vec![],
        })
    }

    fn parse_do_block(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "do"
        let body = self.parse_block();
        Some(CodeASTNode::DoBlock {
            body,
            doc,
            annotations: vec![],
        })
    }

    fn parse_repeat_until(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        self.advance(); // consume "repeat"
        let body = self.parse_block();
        if let Some(token) = self.peek().cloned() {
            if let Token::Keyword(ref s, _) = token {
                if s == "until" {
                    self.advance(); // consume "until"
                }
            }
        }
        let condition = self.parse_expression()?;
        Some(CodeASTNode::RepeatUntil {
            body,
            condition,
            doc,
            annotations: vec![],
        })
    }

    fn parse_function_call_stmt(&mut self, doc: Option<String>) -> Option<CodeASTNode> {
        let call = self.parse_expression()?;
        Some(CodeASTNode::FunctionCallStmt {
            call,
            doc,
            annotations: vec![],
        })
    }
}
