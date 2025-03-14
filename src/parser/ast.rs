// src/ast.rs

/// Centralized type information for Lua values.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Unknown,
    String,
    Number,
    Boolean,
    Table,
    Function,
    // Additional types (e.g. Union, Optional) can be added later.
}

/// Represents an export item in a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportItem {
    pub name: String,
    pub type_info: TypeInfo,
}

/// A simple expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Literal(String), // For now, literals are represented as strings.
    FunctionCall {
        callee: String,
        args: Vec<Expression>,
    },
    // More expression types (e.g. binary operations) can be added here.
}

/// AST nodes for Lua code.
#[derive(Debug, Clone, PartialEq)]
pub enum CodeASTNode {
    /// A module declaration (e.g. `local M = { ... }`).
    ModuleDeclaration {
        name: String,
        exports: Vec<ExportItem>,
        /// Optional documentation comment attached to the module.
        doc: Option<String>,
        /// Annotations attached to the module.
        annotations: Vec<AnnotationASTNode>,
    },
    /// A function definition.
    FunctionDef {
        name: String,
        params: Vec<(String, TypeInfo)>,
        return_types: Vec<TypeInfo>,
        /// Optional documentation comment.
        doc: Option<String>,
        /// Annotations (e.g. @param, @return) attached to the function.
        annotations: Vec<AnnotationASTNode>,
        body: Vec<CodeASTNode>,
    },
    /// A variable declaration.
    VariableDeclaration {
        name: String,
        value: Option<Box<CodeASTNode>>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A return statement.
    ReturnStatement(Vec<Expression>),
    /// A standalone comment.
    Comment(String),
    /// A table constructor.
    TableConstructor(Vec<(String, Expression)>),
    /// An assignment statement.
    Assignment {
        lhs: Vec<String>, // multiple identifiers
        rhs: Vec<Expression>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// An if statement.
    IfStatement {
        condition: Expression,
        then_block: Vec<CodeASTNode>,
        else_block: Option<Vec<CodeASTNode>>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A while loop.
    WhileLoop {
        condition: Expression,
        body: Vec<CodeASTNode>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A numeric for loop.
    ForNumeric {
        var: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: Vec<CodeASTNode>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A do block.
    DoBlock {
        body: Vec<CodeASTNode>,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A repeat-until loop.
    RepeatUntil {
        body: Vec<CodeASTNode>,
        condition: Expression,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
    /// A function call statement.
    FunctionCallStmt {
        call: Expression,
        doc: Option<String>,
        annotations: Vec<AnnotationASTNode>,
    },
}

/// AST nodes for annotations.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationASTNode {
    Alias {
        name: String,
        /// Each variant: value and an optional description.
        variants: Vec<(String, Option<String>)>,
    },
    As {
        target: String,
    },
    Async,
    Cast {
        variable: String,
        /// Each cast: type and a flag indicating addition (true) or removal (false).
        casts: Vec<(String, bool)>,
    },
    Class {
        name: String,
        parents: Vec<String>,
        exact: bool,
        fields: Vec<(String, TypeInfo)>,
    },
    Deprecated,
    Diagnostic {
        action: String,
        diagnostic: Option<String>,
    },
    Enum {
        name: String,
        key: bool,
        members: Vec<(String, Option<String>)>,
    },
    Field {
        scope: Option<String>,
        name: String,
        type_field: String,
        description: Option<String>,
    },
    Generic {
        keyword: String,
        content: String,
    },
    Meta {
        name: Option<String>,
    },
    Module {
        module_name: String,
    },
    Nondiscard,
    Operator {
        operator: String,
        signature: Option<String>,
    },
    Overload {
        signature: String,
    },
    Package,
    Param {
        name: String,
        type_field: String,
        description: Option<String>,
    },
    Private,
    Protected,
    Return {
        type_field: String,
        name: Option<String>,
        description: Option<String>,
    },
    See {
        reference: String,
    },
    Source {
        path: String,
    },
    Type {
        type_field: String,
    },
    Vararg {
        type_field: Option<String>,
    },
    Version {
        version: String,
        comparison: Option<String>,
    },
}
