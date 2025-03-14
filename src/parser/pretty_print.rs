// src/pretty_print.rs

use crate::parser::ast_annotations_printer;
use crate::parser::ast_code_printer;

pub fn pretty_print_code_ast(ast: &[crate::parser::ast::CodeASTNode]) -> String {
    ast_code_printer::pretty_print_code_ast(ast, 0)
}

pub fn pretty_print_annotation_ast(ast: &[crate::parser::ast::AnnotationASTNode]) -> String {
    ast_annotations_printer::pretty_print_annotation_ast(ast, 0)
}

pub fn pretty_print_merged(
    code_ast: &[crate::parser::ast::CodeASTNode],
    annotation_ast: &[crate::parser::ast::AnnotationASTNode]
) -> String {
    let mut output = String::new();
    output.push_str("=== Merged AST ===\n\n");
    output.push_str("---- Code AST ----\n");
    output.push_str(&ast_code_printer::pretty_print_code_ast(code_ast, 0));
    output.push_str("\n---- Annotation AST ----\n");
    output.push_str(&ast_annotations_printer::pretty_print_annotation_ast(annotation_ast, 0));
    output
}
