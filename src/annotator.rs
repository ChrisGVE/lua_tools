// src/annotator.rs
use crate::parser::{ASTNode, ExportItem};
use crate::type_inference::TypeInfo;
use std::collections::HashSet;

pub struct Annotator {
    current_module: String,
    pub preserve_existing: bool,
    processed_comments: HashSet<String>,
}

impl Annotator {
    pub fn new() -> Self {
        Self {
            current_module: String::new(),
            preserve_existing: true,
            processed_comments: HashSet::new(),
        }
    }

    /// Main entry point for generating documentation
    pub fn generate_docs(&mut self, ast: &[ASTNode]) -> String {
        let mut output = String::new();

        for node in ast {
            output.push_str(&self.process_node(node));
            output.push('\n');
        }

        output
    }

    fn process_node(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::ModuleDeclaration { name, exports } => {
                self.current_module = name.clone();
                self.format_module_header(name, exports)
            }
            ASTNode::FunctionDef {
                name,
                params,
                return_types,
                docs,
                ..
            } => {
                let full_name = format!("{}.{}", self.current_module, name);
                self.format_function(&full_name, params, return_types, docs)
            }
            ASTNode::RequireStatement { module, alias } => {
                self.format_require(module, alias.as_deref())
            }
            ASTNode::CommentBlock(text) => self.format_existing_comment(text),
            _ => String::new(),
        }
    }

    fn format_module_header(&self, name: &str, exports: &[ExportItem]) -> String {
        let mut output = format!("---@module {}\n", name);
        if !exports.is_empty() {
            output.push_str("---Exports:\n");
            for export in exports {
                output.push_str(&format!(
                    "---@field {} {}\n",
                    export.name,
                    self.type_to_string(&export.type_info)
                ));
            }
        }
        output
    }

    fn format_function(
        &mut self,
        full_name: &str,
        params: &[(String, TypeInfo)],
        returns: &[TypeInfo],
        docs: &[String],
    ) -> String {
        let mut output = String::new();

        // Strip module prefix for annotation
        let name = full_name.split('.').last().unwrap_or(full_name);

        // Preserve existing docs if requested
        if self.preserve_existing {
            for doc in docs {
                output.push_str(&format!("--{}\n", doc));
            }
        }

        // Always add function annotation
        output.push_str(&format!("---@function {}\n", name));

        // Parameter annotations
        for (param_name, type_info) in params {
            let type_str = self.type_to_string(type_info);
            output.push_str(
                &(format!(
                    "---@param {} {}{}\n",
                    param_name,
                    type_str,
                    self.type_comment_suffix(type_info)
                )),
            );
        }

        // Return annotation (even if unknown)
        if !returns.is_empty() {
            let return_types = returns
                .iter()
                .map(|t| self.type_to_string(t))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("---@return {}\n", return_types));
        }

        output
    }

    fn format_require(&self, module: &str, alias: Option<&str>) -> String {
        if let Some(alias) = alias {
            format!("---@dependency {} : {}\n", alias, module)
        } else {
            format!("---@dependency {}\n", module)
        }
    }

    fn format_existing_comment(&mut self, text: &str) -> String {
        if self.preserve_existing {
            text.lines()
                .map(|line| format!("--{}", line))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        }
    }

    fn type_to_string(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String => "string".to_string(),
            TypeInfo::Number => "number".to_string(),
            TypeInfo::Boolean => "boolean".to_string(),
            TypeInfo::Union(types) => types
                .iter()
                .map(|t| self.type_to_string(t))
                .collect::<Vec<_>>()
                .join("|"),
            TypeInfo::Table(fields) => format!(
                "table<{}>",
                fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, self.type_to_string(&f.type_info)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TypeInfo::Function(sig) => format!(
                "fun({}) -> {}",
                sig.params
                    .iter()
                    .map(|(name, t)| format!("{}: {}", name, self.type_to_string(t)))
                    .collect::<Vec<_>>()
                    .join(", "),
                sig.returns
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TypeInfo::Optional(inner) => format!("{}?", self.type_to_string(inner)),
            TypeInfo::Unknown => "any".to_string(),
        }
    }

    fn type_comment_suffix(&self, type_info: &TypeInfo) -> &str {
        match type_info {
            TypeInfo::Unknown => " @TODO: Specify type",
            TypeInfo::Optional(inner) => self.type_comment_suffix(inner),
            _ => "",
        }
    }
}

// src/annotator.rs (add this at the end)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ASTNode;
    use crate::type_inference::{ScopeContext, TypeInfo};

    #[test]
    fn basic_annotation_generation() {
        let ast = vec![ASTNode::FunctionDef {
            name: "calculate".to_string(),
            params: vec![
                ("value".to_string(), TypeInfo::Number),
                ("options".to_string(), TypeInfo::Unknown),
            ],
            return_types: vec![
                TypeInfo::Number,
                TypeInfo::Optional(Box::new(TypeInfo::String)),
            ],
            scope: ScopeContext::new(),
            docs: vec![],
            body: vec![],
        }];

        let mut annotator = Annotator::new();
        let result = annotator.generate_docs(&ast);

        assert!(result.contains("---@function calculate"));
        assert!(result.contains("---@param value number"));
        assert!(result.contains("---@param options any @TODO: Specify type"));
        assert!(result.contains("---@return number, string?"));
    }
}
