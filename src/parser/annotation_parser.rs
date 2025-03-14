// src/parser/annotation_parser.rs

use crate::parser::ast::AnnotationASTNode;
use crate::tokenizer::token::{AnnotationSubToken, Token};

pub struct AnnotationParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl AnnotationParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Iterates over the unified token stream and processes tokens of variant Annotation,
    /// returning a vector of parsed AnnotationASTNodes.
    pub fn parse(&mut self) -> Vec<AnnotationASTNode> {
        let mut annotations = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(token) = self.peek() {
                match token {
                    Token::Annotation(subtokens, _) => {
                        if let Some(ann) = self.parse_annotation_token(subtokens.clone()) {
                            annotations.push(ann);
                        }
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
        }
        annotations
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    /// Parses an annotation token (given its AnnotationSubToken vector) into an AnnotationASTNode.
    fn parse_annotation_token(
        &self,
        subtokens: Vec<AnnotationSubToken>,
    ) -> Option<AnnotationASTNode> {
        // Create a mutable local copy of the subtokens for parsing.
        let mut tokens = subtokens;
        // If the first token is a prefix, remove it.
        if let Some(token) = tokens.get(0) {
            if let AnnotationSubToken::Prefix(_) = token {
                tokens.remove(0);
            }
        }
        // Expect the first token to be an Identifier representing the keyword.
        let keyword = match tokens.get(0) {
            Some(AnnotationSubToken::Identifier(parts)) => parts.join("."),
            _ => return self.parse_generic(&tokens),
        };

        match keyword.as_str() {
            "alias" => self.parse_alias(&tokens),
            "as" => self.parse_as(&tokens),
            "async" => self.parse_async(&tokens),
            "cast" => self.parse_cast(&tokens),
            "class" => self.parse_class(&tokens),
            "deprecated" => self.parse_deprecated(&tokens),
            "diagnostic" => self.parse_diagnostic(&tokens),
            "enum" => self.parse_enum(&tokens),
            "field" => self.parse_field(&tokens),
            "generic" => self.parse_generic(&tokens),
            "meta" => self.parse_meta(&tokens),
            "module" => self.parse_module(&tokens),
            "nodiscard" => self.parse_nondiscard(&tokens),
            "operator" => self.parse_operator(&tokens),
            "overload" => self.parse_overload(&tokens),
            "package" => self.parse_package(&tokens),
            "param" => self.parse_param(&tokens),
            "private" => self.parse_private(&tokens),
            "protected" => self.parse_protected(&tokens),
            "return" => self.parse_return(&tokens),
            "see" => self.parse_see(&tokens),
            "source" => self.parse_source(&tokens),
            "type" => self.parse_type(&tokens),
            "vararg" => self.parse_vararg(&tokens),
            "version" => self.parse_version(&tokens),
            _ => self.parse_generic(&tokens),
        }
    }

    // --- Annotation Parsing Functions ---
    // Each function expects the full token vector (after optional prefix removal)
    // and uses a local position index.

    fn parse_alias(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip keyword "alias"
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut variants = Vec::new();
        while pos < tokens.len() {
            match tokens.get(pos) {
                Some(AnnotationSubToken::Operator(op)) if op == "|" => {
                    pos += 1; // consume '|'
                    let variant =
                        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                            pos += 1;
                            parts.join(".")
                        } else if let Some(AnnotationSubToken::Text(text)) = tokens.get(pos) {
                            pos += 1;
                            text.clone()
                        } else {
                            "".to_string()
                        };
                    let mut desc = None;
                    if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                        if op == "#" {
                            pos += 1;
                            if let Some(AnnotationSubToken::Text(text)) = tokens.get(pos) {
                                desc = Some(text.clone());
                                pos += 1;
                            }
                        }
                    }
                    variants.push((variant, desc));
                }
                _ => break,
            }
        }
        Some(AnnotationASTNode::Alias { name, variants })
    }

    fn parse_as(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "as"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            Some(AnnotationASTNode::As {
                target: parts.join("."),
            })
        } else {
            None
        }
    }

    fn parse_async(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Async)
    }

    fn parse_cast(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "cast"
        let variable = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut casts = Vec::new();
        while pos < tokens.len() {
            if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                if op == "+" || op == "-" {
                    let add = op == "+";
                    pos += 1;
                    if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                        casts.push((parts.join("."), add));
                        pos += 1;
                    }
                } else if op == "," {
                    pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Some(AnnotationASTNode::Cast { variable, casts })
    }

    // --- Generic Annotation Parser ---
    fn parse_generic(&mut self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        // Consume the keyword if not already consumed.
        let keyword = if let Some(AnnotationSubToken::Identifier(parts)) = self.advance() {
            parts.join(".")
        } else {
            return None;
        };
        let mut content = String::new();
        while let Some(tok) = self.peek() {
            if let AnnotationSubToken::Text(text) = tok {
                content.push_str(text);
                content.push(' ');
                self.advance();
            } else {
                break;
            }
        }
        Some(AnnotationASTNode::Generic {
            keyword,
            content: content.trim().to_string(),
        })
    }

    fn parse_class(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "class"
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut parents = Vec::new();
        if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
            if op == ":" {
                pos += 1;
                while let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                    parents.push(parts.join("."));
                    pos += 1;
                    if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                        if op == "," {
                            pos += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        let mut exact = false;
        if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
            if op == "(" {
                pos += 1;
                if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                    if parts.join(".").to_lowercase() == "exact" {
                        exact = true;
                    }
                    pos += 1;
                }
                if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                    if op == ")" {
                        pos += 1;
                    }
                }
            }
        }
        let mut fields = Vec::new();
        while pos < tokens.len() {
            if let AnnotationSubToken::Operator(_) = tokens.get(pos).unwrap() {
                break;
            }
            if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                let field_name = parts.join(".");
                pos += 1;
                let mut type_field = "any".to_string();
                if let Some(AnnotationSubToken::Colon) = tokens.get(pos) {
                    pos += 1;
                    if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                        type_field = parts.join(".");
                        pos += 1;
                    }
                }
                // For simplicity, we store the type field as a string.
                fields.push((field_name, type_field));
            } else {
                break;
            }
        }
        Some(AnnotationASTNode::Class {
            name,
            parents,
            exact,
            fields,
        })
    }

    fn parse_deprecated(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Deprecated)
    }

    fn parse_diagnostic(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "diagnostic"
        let action = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            "".to_string()
        };
        let diagnostic = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(parts.join("."))
        } else {
            None
        };
        Some(AnnotationASTNode::Diagnostic { action, diagnostic })
    }

    fn parse_enum(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "enum"
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut key = false;
        if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
            if op.to_lowercase() == "(key)" {
                key = true;
                pos += 1;
            }
        }
        let mut members = Vec::new();
        while pos < tokens.len() {
            if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                if op == "|" {
                    pos += 1;
                    let member =
                        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                            pos += 1;
                            parts.join(".")
                        } else {
                            "".to_string()
                        };
                    let mut desc = None;
                    if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                        if op == "#" {
                            pos += 1;
                            if let Some(AnnotationSubToken::Text(text)) = tokens.get(pos) {
                                desc = Some(text.clone());
                                pos += 1;
                            }
                        }
                    }
                    members.push((member, desc));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Some(AnnotationASTNode::Enum { name, key, members })
    }

    fn parse_field(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "field"
        let scope = if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
            if op == "[" {
                pos += 1;
                let s = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                    pos += 1;
                    Some(parts.join("."))
                } else {
                    None
                };
                if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
                    if op == "]" {
                        pos += 1;
                    }
                }
                s
            } else {
                None
            }
        } else {
            None
        };
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let type_field = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            "any".to_string()
        };
        let mut description = String::new();
        while pos < tokens.len() {
            if let AnnotationSubToken::Text(text) = &tokens[pos] {
                description.push_str(text);
                description.push(' ');
                pos += 1;
            } else {
                break;
            }
        }
        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        };
        Some(AnnotationASTNode::Field {
            scope,
            name,
            type_field,
            description,
        })
    }

    fn parse_meta(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "meta"
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(parts.join("."))
        } else {
            None
        };
        Some(AnnotationASTNode::Meta { name })
    }

    fn parse_module(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "module"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(AnnotationASTNode::Module {
                module_name: parts.join("."),
            })
        } else {
            None
        }
    }

    fn parse_nondiscard(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Nondiscard)
    }

    fn parse_operator(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "operator"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            let operator = parts.join(".");
            let signature = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
                pos += 1;
                Some(parts.join("."))
            } else {
                None
            };
            Some(AnnotationASTNode::Operator {
                operator,
                signature,
            })
        } else {
            None
        }
    }

    fn parse_overload(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "overload"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(AnnotationASTNode::Overload {
                signature: parts.join("."),
            })
        } else {
            None
        }
    }

    fn parse_package(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Package)
    }

    fn parse_param(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "param"
        let name = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let type_field = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut description = String::new();
        while pos < tokens.len() {
            if let AnnotationSubToken::Text(text) = &tokens[pos] {
                description.push_str(text);
                description.push(' ');
                pos += 1;
            } else {
                break;
            }
        }
        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        };
        Some(AnnotationASTNode::Param {
            name,
            type_field,
            description,
        })
    }

    fn parse_private(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Private)
    }

    fn parse_protected(&self, _tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        Some(AnnotationASTNode::Protected)
    }

    fn parse_return(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "return"
        let type_field = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            parts.join(".")
        } else {
            return None;
        };
        let mut name = None;
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            name = Some(parts.join("."));
            pos += 1;
        }
        let mut description = String::new();
        while pos < tokens.len() {
            if let AnnotationSubToken::Text(text) = &tokens[pos] {
                description.push_str(text);
                description.push(' ');
                pos += 1;
            } else {
                break;
            }
        }
        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        };
        Some(AnnotationASTNode::Return {
            type_field,
            name,
            description,
        })
    }

    fn parse_see(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "see"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(AnnotationASTNode::See {
                reference: parts.join("."),
            })
        } else {
            None
        }
    }

    fn parse_source(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "source"
        if let Some(AnnotationSubToken::Text(text)) = tokens.get(pos) {
            pos += 1;
            Some(AnnotationASTNode::Source { path: text.clone() })
        } else {
            None
        }
    }

    fn parse_type(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "type"
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(AnnotationASTNode::Type {
                type_field: parts.join("."),
            })
        } else {
            None
        }
    }

    fn parse_vararg(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "vararg"
        let type_field = if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            Some(parts.join("."))
        } else {
            None
        };
        Some(AnnotationASTNode::Vararg { type_field })
    }

    fn parse_version(&self, tokens: &[AnnotationSubToken]) -> Option<AnnotationASTNode> {
        let mut pos = 1; // skip "version"
        let comparison = if let Some(AnnotationSubToken::Operator(op)) = tokens.get(pos) {
            pos += 1;
            Some(op.clone())
        } else {
            None
        };
        if let Some(AnnotationSubToken::Identifier(parts)) = tokens.get(pos) {
            pos += 1;
            let version = parts.join(".");
            Some(AnnotationASTNode::Version {
                version,
                comparison,
            })
        } else {
            None
        }
    }
}
