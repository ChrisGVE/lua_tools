// src/ast_annotations_printer.rs

use crate::parser::ast::*;

pub fn pretty_print_annotation_ast(ast: &[AnnotationASTNode], indent: usize) -> String {
    let mut output = String::new();
    let indent_str = "  ".repeat(indent);
    output.push_str(&format!("{}--- Annotation AST ---\n", indent_str));
    for ann in ast {
        output.push_str(&pretty_print_annotation_node(ann, indent + 1));
    }
    output
}

pub fn pretty_print_annotation_node(ann: &AnnotationASTNode, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match ann {
        AnnotationASTNode::Alias { name, variants } => {
            let mut s = format!("{}Alias: {}\n", indent_str, name);
            for (variant, desc) in variants {
                s.push_str(&format!("{}  Variant: {}", indent_str, variant));
                if let Some(desc) = desc {
                    s.push_str(&format!(" // {}", desc));
                }
                s.push('\n');
            }
            s
        }
        AnnotationASTNode::As { target } => format!("{}As: {}\n", indent_str, target),
        AnnotationASTNode::Async => format!("{}Async\n", indent_str),
        AnnotationASTNode::Cast { variable, casts } => {
            let casts_str = casts
                .iter()
                .map(|(t, add)| format!("{}{}", t, if *add { "+" } else { "-" }))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}Cast: {} => [{}]\n", indent_str, variable, casts_str)
        }
        AnnotationASTNode::Class {
            name,
            parents,
            exact,
            fields,
        } => {
            let mut s = format!(
                "{}Class: {}{} with parents: {:?}\n",
                indent_str,
                name,
                if *exact { " (exact)" } else { "" },
                parents
            );
            for (field, typ) in fields {
                s.push_str(&format!("{}  Field: {} : {:?}\n", indent_str, field, typ));
            }
            s
        }
        AnnotationASTNode::Deprecated => format!("{}Deprecated\n", indent_str),
        AnnotationASTNode::Diagnostic { action, diagnostic } => {
            let diag = diagnostic.clone().unwrap_or_default();
            format!("{}Diagnostic: {} - {}\n", indent_str, action, diag)
        }
        AnnotationASTNode::Enum { name, key, members } => {
            let mut s = format!(
                "{}Enum: {}{}\n",
                indent_str,
                name,
                if *key { " (key)" } else { "" }
            );
            for (member, desc) in members {
                s.push_str(&format!("{}  Member: {}", indent_str, member));
                if let Some(desc) = desc {
                    s.push_str(&format!(" // {}", desc));
                }
                s.push('\n');
            }
            s
        }
        AnnotationASTNode::Field {
            scope,
            name,
            type_field,
            description,
        } => {
            let scope_str = scope.as_deref().unwrap_or("default");
            let mut s = format!(
                "{}Field: {}.{} : {}",
                indent_str, scope_str, name, type_field
            );
            if let Some(desc) = description {
                s.push_str(&format!(" // {}", desc));
            }
            s.push('\n');
            s
        }
        AnnotationASTNode::Generic { keyword, content } => {
            format!("{}Generic: {} - {}\n", indent_str, keyword, content)
        }
        AnnotationASTNode::Meta { name } => format!("{}Meta: {:?}\n", indent_str, name),
        AnnotationASTNode::Module { module_name } => {
            format!("{}Module: {}\n", indent_str, module_name)
        }
        AnnotationASTNode::Nondiscard => format!("{}Nondiscard\n", indent_str),
        AnnotationASTNode::Operator {
            operator,
            signature,
        } => {
            let sig = signature.clone().unwrap_or_default();
            format!("{}Operator: {} - {}\n", indent_str, operator, sig)
        }
        AnnotationASTNode::Overload { signature } => {
            format!("{}Overload: {}\n", indent_str, signature)
        }
        AnnotationASTNode::Package => format!("{}Package\n", indent_str),
        AnnotationASTNode::Param {
            name,
            type_field,
            description,
        } => {
            let mut s = format!("{}Param: {} : {}", indent_str, name, type_field);
            if let Some(desc) = description {
                s.push_str(&format!(" // {}", desc));
            }
            s.push('\n');
            s
        }
        AnnotationASTNode::Private => format!("{}Private\n", indent_str),
        AnnotationASTNode::Protected => format!("{}Protected\n", indent_str),
        AnnotationASTNode::Return {
            type_field,
            name,
            description,
        } => {
            let mut s = format!("{}Return: {}", indent_str, type_field);
            if let Some(n) = name {
                s.push_str(&format!(" (named {})", n));
            }
            if let Some(desc) = description {
                s.push_str(&format!(" // {}", desc));
            }
            s.push('\n');
            s
        }
        AnnotationASTNode::See { reference } => format!("{}See: {}\n", indent_str, reference),
        AnnotationASTNode::Source { path } => format!("{}Source: {}\n", indent_str, path),
        AnnotationASTNode::Type { type_field } => format!("{}Type: {}\n", indent_str, type_field),
        AnnotationASTNode::Vararg { type_field } => {
            let tf = type_field.clone().unwrap_or_default();
            format!("{}Vararg: {}\n", indent_str, tf)
        }
        AnnotationASTNode::Version {
            version,
            comparison,
        } => {
            let comp = comparison.clone().unwrap_or_default();
            format!("{}Version: {} {}\n", indent_str, comp, version)
        }
    }
}

