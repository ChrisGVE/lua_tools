// src/type_inference.rs
use super::parser::{ASTNode, ExportItem, Expression};
use super::project_context::ProjectContext;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TypeInfo {
    String,
    Number,
    Boolean,
    Table(Vec<FieldInfo>),
    Function(FunctionSignature),
    Optional(Box<TypeInfo>),
    Union(Vec<TypeInfo>),
    Unknown,
    // ... keep other variants if needed ...
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_info: TypeInfo,
}

#[derive(Debug, Clone)]
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
    current_scope: ScopeContext,
    project_context: ProjectContext,
}

impl TypeAnalyzer {
    #[allow(dead_code)]
    pub fn new(project: ProjectContext) -> Self {
        Self {
            current_scope: ScopeContext::new(),
            project_context: project,
        }
    }

    pub fn analyze(&mut self, ast: &[ASTNode]) {
        for node in ast {
            match node {
                ASTNode::FunctionDef {
                    params,
                    body, // Now matches actual struct
                    ..
                } => {
                    self.analyze_function(params, body);
                }
                ASTNode::ModuleDeclaration { name, exports } => {
                    self.analyze_module(name, exports);
                }
                _ => {}
            }
        }
    }

    fn analyze_function(&mut self, params: &[(String, TypeInfo)], body: &[ASTNode]) {
        let mut fn_scope = ScopeContext::new();
        fn_scope.parent = Some(Box::new(self.current_scope.clone()));

        // Add parameters to scope
        for (name, type_info) in params {
            fn_scope.variables.insert(name.clone(), type_info.clone());
        }

        // Analyze body with new scope
        let previous_scope = std::mem::replace(&mut self.current_scope, fn_scope);
        self.analyze(body);
        self.current_scope = previous_scope;
    }

    fn analyze_module(&mut self, module_name: &str, exports: &[ExportItem]) {
        for export in exports {
            // Clone the actual ExportItem, not the reference
            self.project_context.add_export(module_name, export.clone());
        }
    }

    fn infer_expression_type(&self, expr: &Expression) -> TypeInfo {
        match expr {
            Expression::Identifier(id) => {
                self.current_scope.lookup(id).unwrap_or(TypeInfo::Unknown)
            }

            Expression::Literal(type_info) => type_info.clone(),

            Expression::FunctionCall(_, args) => {
                let arg_types = args.iter().map(|a| self.infer_expression_type(a)).collect();
                TypeInfo::Function(FunctionSignature {
                    params: vec![],
                    returns: arg_types,
                })
            }

            _ => TypeInfo::Unknown,
        }
    }
}
