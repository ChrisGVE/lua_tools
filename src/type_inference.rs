// src/type_inference.rs

use crate::parser::ast::{CodeASTNode, ExportItem, Expression, TypeInfo};
use crate::project_context::ProjectContext;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub type_info: TypeInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub params: Vec<(String, TypeInfo)>,
    pub returns: Vec<TypeInfo>,
}

#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub variables: HashMap<String, TypeInfo>,
    pub parent: Option<Box<ScopeContext>>,
    pub function_returns: Vec<TypeInfo>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
            function_returns: Vec::new(),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<TypeInfo> {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }
}

pub struct TypeAnalyzer {
    pub current_scope: ScopeContext,
    pub project_context: ProjectContext,
}

impl TypeAnalyzer {
    pub fn new(project: ProjectContext) -> Self {
        Self {
            current_scope: ScopeContext::new(),
            project_context: project,
        }
    }

    pub fn analyze(&mut self, ast: &[CodeASTNode]) {
        for node in ast {
            match node {
                CodeASTNode::FunctionDef {
                    params,
                    body,
                    return_types: _,
                    ..
                } => {
                    let inferred_returns = self.infer_return_types(body);
                    // In a full integration, we might update the function node's return_types here.
                    self.analyze_function(params, body);
                }
                CodeASTNode::ModuleDeclaration { name, exports, .. } => {
                    self.analyze_module(name, exports);
                }
                _ => {}
            }
        }
    }

    fn analyze_function(&mut self, params: &[(String, TypeInfo)], body: &[CodeASTNode]) {
        let mut fn_scope = ScopeContext::new();
        fn_scope.parent = Some(Box::new(self.current_scope.clone()));
        for (name, type_info) in params {
            fn_scope.variables.insert(name.clone(), type_info.clone());
        }
        let previous_scope = std::mem::replace(&mut self.current_scope, fn_scope);
        self.analyze(body);
        self.current_scope = previous_scope;
    }

    fn analyze_module(&mut self, module_name: &str, exports: &[ExportItem]) {
        for export in exports {
            self.project_context.add_export(module_name, export.clone());
        }
    }

    pub fn infer_return_types(&self, body: &[CodeASTNode]) -> Vec<TypeInfo> {
        let mut collected_types = Vec::new();
        for node in body {
            match node {
                CodeASTNode::ReturnStatement(exprs) => {
                    let mut ret_types = Vec::new();
                    for expr in exprs {
                        let t = self.infer_expression_type(expr);
                        ret_types.push(t);
                    }
                    let union_type = if ret_types.is_empty() {
                        TypeInfo::Unknown // or consider a Nil variant if desired
                    } else if ret_types.len() == 1 {
                        ret_types[0].clone()
                    } else {
                        // For now, multiple types result in Unknown.
                        TypeInfo::Unknown
                    };
                    collected_types.push(union_type);
                }
                CodeASTNode::FunctionDef {
                    body: inner_body, ..
                } => {
                    let inner_returns = self.infer_return_types(inner_body);
                    collected_types.extend(inner_returns);
                }
                _ => {}
            }
        }
        collected_types.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        collected_types.dedup();
        collected_types
    }

    pub fn infer_expression_type(&self, expr: &Expression) -> TypeInfo {
        match expr {
            Expression::Identifier(id) => {
                self.current_scope.lookup(id).unwrap_or(TypeInfo::Unknown)
            }
            Expression::Literal(lit) => {
                // For simplicity, treat all literals as strings.
                TypeInfo::String
            }
            Expression::FunctionCall { callee: _, args } => {
                // Infer function call type based on its arguments.
                let _arg_types = args
                    .iter()
                    .map(|a| self.infer_expression_type(a))
                    .collect::<Vec<_>>();
                TypeInfo::Function
            }
        }
    }
}
