// src/parser/parser_helpers.rs

use crate::tokenizer::token::Token;

/// Given a token, returns Some(token) if it is an annotation token,
/// otherwise returns None.
pub fn extract_annotation_token(token: &Token) -> Option<&Token> {
    match token {
        Token::Annotation(_, _) => Some(token),
        _ => None,
    }
}

/// Given a token, returns Some(token) if it is a code token (i.e. not an annotation),
/// otherwise returns None.
pub fn extract_code_token(token: &Token) -> Option<&Token> {
    match token {
        Token::Annotation(_, _) => None,
        other => Some(other),
    }
}
