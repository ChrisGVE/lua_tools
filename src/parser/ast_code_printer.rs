// src/ast_code_printer.rs

use crate::parser::ast::*;

pub fn pretty_print_code_ast(ast: &[CodeASTNode], indent: usize) -> String {
    let mut output = String::new();
    let indent_str = "  ".repeat(indent);
    output.push_str(&format!("{}--- Code AST ---\n", indent_str));
    for node in ast {
        output.push_str(&pretty_print_code_node(node, indent + 1));
    }
    output
}

fn pretty_print_code_node(node: &CodeASTNode, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match node {
        CodeASTNode::ModuleDeclaration {
            name,
            exports,
            doc,
            annotations,
        } => {
            let mut s = format!("{}ModuleDeclaration: {}\n", indent_str, name);
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            if !exports.is_empty() {
                s.push_str(&format!("{}  Exports:\n", indent_str));
                for export in exports {
                    s.push_str(&format!(
                        "{}    {} : {:?}\n",
                        indent_str, export.name, export.type_info
                    ));
                }
            }
            s
        }
        CodeASTNode::FunctionDef {
            name,
            params,
            return_types,
            doc,
            annotations,
            body,
        } => {
            let mut s = format!("{}FunctionDef: {}\n", indent_str, name);
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !params.is_empty() {
                s.push_str(&format!("{}  Parameters:\n", indent_str));
                for (param, typ) in params {
                    s.push_str(&format!("{}    {}: {:?}\n", indent_str, param, typ));
                }
            }
            if !return_types.is_empty() {
                s.push_str(&format!(
                    "{}  Return Types: {:?}\n",
                    indent_str, return_types
                ));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            if !body.is_empty() {
                s.push_str(&format!("{}  Body:\n", indent_str));
                for b in body {
                    s.push_str(&pretty_print_code_node(b, indent + 2));
                }
            }
            s
        }
        CodeASTNode::VariableDeclaration {
            name,
            value,
            doc,
            annotations,
        } => {
            let mut s = format!("{}VariableDeclaration: {}\n", indent_str, name);
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            if let Some(val) = value {
                s.push_str(&format!("{}  Value:\n", indent_str));
                s.push_str(&pretty_print_code_node(val, indent + 2));
            }
            s
        }
        CodeASTNode::ReturnStatement(exprs) => {
            let mut s = format!("{}ReturnStatement:\n", indent_str);
            for expr in exprs {
                s.push_str(&format!("{}  Expression: {:?}\n", indent_str, expr));
            }
            s
        }
        CodeASTNode::Comment(text) => {
            format!("{}Comment: {}\n", indent_str, text)
        }
        CodeASTNode::TableConstructor(fields) => {
            let mut s = format!("{}TableConstructor:\n", indent_str);
            for (key, expr) in fields {
                s.push_str(&format!("{}  {}: {:?}\n", indent_str, key, expr));
            }
            s
        }
        CodeASTNode::Assignment {
            lhs,
            rhs,
            doc,
            annotations,
        } => {
            let mut s = format!("{}Assignment:\n", indent_str);
            s.push_str(&format!("{}  LHS: {:?}\n", indent_str, lhs));
            s.push_str(&format!("{}  RHS: {:?}\n", indent_str, rhs));
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s
        }
        CodeASTNode::IfStatement {
            condition,
            then_block,
            else_block,
            doc,
            annotations,
        } => {
            let mut s = format!("{}IfStatement:\n", indent_str);
            s.push_str(&format!("{}  Condition: {:?}\n", indent_str, condition));
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s.push_str(&format!("{}  Then:\n", indent_str));
            for node in then_block {
                s.push_str(&pretty_print_code_node(node, indent + 2));
            }
            if let Some(else_block) = else_block {
                s.push_str(&format!("{}  Else:\n", indent_str));
                for node in else_block {
                    s.push_str(&pretty_print_code_node(node, indent + 2));
                }
            }
            s
        }
        CodeASTNode::WhileLoop {
            condition,
            body,
            doc,
            annotations,
        } => {
            let mut s = format!("{}WhileLoop:\n", indent_str);
            s.push_str(&format!("{}  Condition: {:?}\n", indent_str, condition));
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s.push_str(&format!("{}  Body:\n", indent_str));
            for node in body {
                s.push_str(&pretty_print_code_node(node, indent + 2));
            }
            s
        }
        CodeASTNode::ForNumeric {
            var,
            start,
            end,
            step,
            body,
            doc,
            annotations,
        } => {
            let mut s = format!("{}ForNumeric: {}\n", indent_str, var);
            s.push_str(&format!("{}  Start: {:?}\n", indent_str, start));
            s.push_str(&format!("{}  End: {:?}\n", indent_str, end));
            if let Some(step) = step {
                s.push_str(&format!("{}  Step: {:?}\n", indent_str, step));
            }
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s.push_str(&format!("{}  Body:\n", indent_str));
            for node in body {
                s.push_str(&pretty_print_code_node(node, indent + 2));
            }
            s
        }
        CodeASTNode::DoBlock {
            body,
            doc,
            annotations,
        } => {
            let mut s = format!("{}DoBlock:\n", indent_str);
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s.push_str(&format!("{}  Body:\n", indent_str));
            for node in body {
                s.push_str(&pretty_print_code_node(node, indent + 2));
            }
            s
        }
        CodeASTNode::RepeatUntil {
            body,
            condition,
            doc,
            annotations,
        } => {
            let mut s = format!("{}RepeatUntil:\n", indent_str);
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s.push_str(&format!("{}  Body:\n", indent_str));
            for node in body {
                s.push_str(&pretty_print_code_node(node, indent + 2));
            }
            s.push_str(&format!("{}  Condition: {:?}\n", indent_str, condition));
            s
        }
        CodeASTNode::FunctionCallStmt {
            call,
            doc,
            annotations,
        } => {
            let mut s = format!("{}FunctionCallStmt:\n", indent_str);
            s.push_str(&format!("{}  Call: {:?}\n", indent_str, call));
            if let Some(d) = doc {
                s.push_str(&format!("{}  Doc: {}\n", indent_str, d));
            }
            if !annotations.is_empty() {
                s.push_str(&format!("{}  Annotations:\n", indent_str));
                for ann in annotations {
                    s.push_str(
                        &crate::parser::ast_annotations_printer::pretty_print_annotation_node(
                            ann,
                            indent + 2,
                        ),
                    );
                }
            }
            s
        }
    }
}
