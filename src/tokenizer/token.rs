// src/token.rs

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize, // exclusive end offset
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// Structured subtokens for annotation content.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationSubToken {
    Prefix(String),
    Identifier(Vec<String>),
    Operator(String),
    Colon,
    Comma,
    LessThan,
    GreaterThan,
    OpenParen,
    CloseParen,
    StringLiteral(String),
    NumberLiteral(String),
    Text(String),
}

impl AnnotationSubToken {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match self {
            AnnotationSubToken::Prefix(s) => format!("{}Prefix({})", indent_str, s),
            AnnotationSubToken::Identifier(parts) => {
                format!("{}Identifier({})", indent_str, parts.join("."))
            }
            AnnotationSubToken::Operator(s) => format!("{}Operator({})", indent_str, s),
            AnnotationSubToken::Colon => format!("{}Colon(:)", indent_str),
            AnnotationSubToken::Comma => format!("{}Comma(,)", indent_str),
            AnnotationSubToken::LessThan => format!("{}LessThan(<)", indent_str),
            AnnotationSubToken::GreaterThan => format!("{}GreaterThan(>)", indent_str),
            AnnotationSubToken::OpenParen => format!("{}OpenParen(()", indent_str),
            AnnotationSubToken::CloseParen => format!("{}CloseParen())", indent_str),
            AnnotationSubToken::StringLiteral(s) => {
                format!("{}StringLiteral({})", indent_str, s)
            }
            AnnotationSubToken::NumberLiteral(s) => {
                format!("{}NumberLiteral({})", indent_str, s)
            }
            AnnotationSubToken::Text(s) => format!("{}Text({})", indent_str, s),
        }
    }
}

/// Unified token types.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(Vec<String>, Span),
    DroppedIdentifier(Span),
    Keyword(String, Span),
    Operator(String, Span),
    Assignment(Span),
    /// Annotation tokens carry a vector of annotation subtokens.
    Annotation(Vec<AnnotationSubToken>, Span),
    BlockCommentOpen(Span),
    BlockComment(String, Span),
    BlockCommentClose(Span),
    Comment(String, Span),
    StringLiteral(String, Span),
    NumberLiteral(String, Span),
    VarArg(Span),
    ParenOpen(Span),
    ParenClose(Span),
    BraceOpen(Span),
    BraceClose(Span),
    BracketOpen(Span),
    BracketClose(Span),
}

impl Token {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match self {
            Token::Identifier(parts, _) => {
                format!("{}Identifier({})", indent_str, parts.join("."))
            }
            Token::DroppedIdentifier(_) => format!("{}DroppedIdentifier", indent_str),
            Token::Keyword(s, _) => format!("{}Keyword({})", indent_str, s),
            Token::Operator(s, _) => format!("{}Operator({})", indent_str, s),
            Token::Assignment(_) => format!("{}Assignment(=)", indent_str),
            Token::Annotation(subtokens, _) => {
                let mut s = format!("{}Annotation:\n", indent_str);
                for sub in subtokens {
                    s.push_str(&sub.pretty_print(indent + 1));
                    s.push('\n');
                }
                s
            }
            Token::BlockCommentOpen(_) => format!("{}BlockCommentOpen", indent_str),
            Token::BlockComment(text, _) => {
                format!("{}BlockComment({})", indent_str, text)
            }
            Token::BlockCommentClose(_) => format!("{}BlockCommentClose", indent_str),
            Token::Comment(text, _) => format!("{}Comment({})", indent_str, text),
            Token::StringLiteral(text, _) => format!("{}StringLiteral({})", indent_str, text),
            Token::NumberLiteral(text, _) => format!("{}NumberLiteral({})", indent_str, text),
            Token::VarArg(_) => format!("{}VarArg(...)", indent_str),
            Token::ParenOpen(_) => format!("{}ParenOpen", indent_str),
            Token::ParenClose(_) => format!("{}ParenClose", indent_str),
            Token::BraceOpen(_) => format!("{}BraceOpen", indent_str),
            Token::BraceClose(_) => format!("{}BraceClose", indent_str),
            Token::BracketOpen(_) => format!("{}BracketOpen", indent_str),
            Token::BracketClose(_) => format!("{}BracketClose", indent_str),
        }
    }
}

/// Helper function to pretty-print a slice of tokens.
pub fn pretty_print_tokens(tokens: &[Token]) -> String {
    let mut output = String::new();
    for token in tokens {
        output.push_str(&token.pretty_print(0));
        output.push('\n');
    }
    output
}
