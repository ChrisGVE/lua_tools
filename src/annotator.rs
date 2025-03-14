// src/annotator.rs

use crate::parser::ast::{CodeASTNode, ExportItem, TypeInfo};

pub struct Annotator {
    current_module: String,
    pub preserve_existing: bool,
}

impl Annotator {
    pub fn new() -> Self {
        Self {
            current_module: String::new(),
            preserve_existing: true,
        }
    }

    pub fn generate_docs(&mut self, ast: &[CodeASTNode]) -> String {
        let mut output = String::new();

        for node in ast {
            match node {
                CodeASTNode::ModuleDeclaration { name, exports, .. } => {
                    self.current_module = name.clone();
                    output.push_str(&self.format_module_header(name, exports));
                }
                CodeASTNode::FunctionDef {
                    name,
                    params,
                    return_types,
                    doc,
                    annotations,
                    body,
                } => {
                    let full_name = if self.current_module.is_empty() || name.contains('.') {
                        name.clone()
                    } else {
                        format!("{}.{}", self.current_module, name)
                    };
                    let docs_vec = doc
                        .as_ref()
                        .map(|s| vec![s.clone()])
                        .unwrap_or_else(Vec::new);
                    output.push_str(&self.format_function(
                        &full_name,
                        params,
                        return_types,
                        &docs_vec,
                    ));
                }
                CodeASTNode::Comment(text) => {
                    if text.contains('\n') {
                        output.push_str(&self.format_block_comment(text));
                    } else {
                        output.push_str(&self.format_line_comment(text));
                    }
                }
                _ => {}
            }
            output.push('\n');
        }

        output
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
        &self,
        name: &str,
        params: &[(String, TypeInfo)],
        returns: &[TypeInfo],
        existing_docs: &[String],
    ) -> String {
        let mut output = String::new();

        if existing_docs.is_empty() {
            output.push_str("-- TODO: Describe the function\n");
        } else {
            for doc in existing_docs {
                output.push_str(&format!("{}\n", doc));
            }
        }

        output.push_str(&format!("---@function {}\n", name));

        for (param, type_info) in params {
            let type_str = self.type_to_string(type_info);
            let placeholder = if type_str == "any" {
                " @TODO: Specify type and describe"
            } else {
                ""
            };
            output.push_str(&format!(
                "---@param {} {}{}\n",
                param, type_str, placeholder
            ));
        }

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

    fn format_line_comment(&self, text: &str) -> String {
        if text.starts_with('-') {
            format!("--{}", text)
        } else {
            format!("-- {}", text)
        }
    }

    fn format_block_comment(&self, text: &str) -> String {
        format!("--[[\n{}\n--]]", text)
    }

    fn type_to_string(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String => "string".to_string(),
            TypeInfo::Number => "number".to_string(),
            TypeInfo::Boolean => "boolean".to_string(),
            TypeInfo::Table => "table".to_string(),
            TypeInfo::Function => "function".to_string(),
            TypeInfo::Unknown => "any".to_string(),
            _ => "any".to_string(),
        }
    }
}
