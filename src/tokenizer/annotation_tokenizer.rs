// src/annotation_tokenizer.rs

use crate::tokenizer::lexer::Lexer;
use crate::tokenizer::token::AnnotationSubToken;

/// Utility function to check if a character is considered punctuation in annotation tokenization.
fn is_annotation_punctuation(ch: char) -> bool {
    matches!(ch, ':' | ',' | '<' | '>' | '(' | ')' | '|' | '#')
}

/// Reads an identifier from the lexer, supporting dotted names.
/// Returns a vector of identifier parts.
fn read_identifier_vector_from_lexer(lexer: &mut Lexer) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    while lexer.pos < lexer.input.len() {
        let c = lexer.current_char();
        if c.is_alphanumeric() || c == '_' {
            current.push(c);
            lexer.advance();
        } else if c == '.' {
            if !current.is_empty() {
                parts.push(current);
                current = String::new();
            }
            lexer.advance(); // consume the dot
        } else {
            break;
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Tokenizes the annotation text into a vector of structured annotation subtokens.
/// This implementation leverages the existing lexer for proper position tracking.
fn tokenize_annotation(text: &str) -> Vec<AnnotationSubToken> {
    let mut tokens = Vec::new();

    // Initialize a new lexer instance with the annotation text.
    let mut lexer = Lexer::new(text);

    // Check and preserve the annotation prefix.
    if text.trim_start().starts_with("---@") {
        tokens.push(AnnotationSubToken::Prefix("---@".to_string()));
        lexer.advance_by(4); // Advance past the prefix.
    } else if text.trim_start().starts_with("---|") {
        tokens.push(AnnotationSubToken::Prefix("---|".to_string()));
        lexer.advance_by(4);
    }

    while lexer.pos < lexer.input.len() {
        let ch = lexer.current_char();
        if ch.is_whitespace() {
            lexer.advance();
            continue;
        }
        if is_annotation_punctuation(ch) {
            let token = match ch {
                ':' => AnnotationSubToken::Colon,
                ',' => AnnotationSubToken::Comma,
                '<' => AnnotationSubToken::LessThan,
                '>' => AnnotationSubToken::GreaterThan,
                '(' => AnnotationSubToken::OpenParen,
                ')' => AnnotationSubToken::CloseParen,
                '|' => AnnotationSubToken::Operator("|".to_string()),
                '#' => AnnotationSubToken::Operator("#".to_string()),
                other => AnnotationSubToken::Operator(other.to_string()),
            };
            tokens.push(token);
            lexer.advance();
            continue;
        }
        // If the character starts an identifier (alphabetic or underscore), read the full (possibly dotted) identifier.
        if ch.is_alphabetic() || ch == '_' {
            let parts = read_identifier_vector_from_lexer(&mut lexer);
            tokens.push(AnnotationSubToken::Identifier(parts));
            continue;
        }
        // For any other characters, accumulate them as generic text.
        let mut text_token = String::new();
        while lexer.pos < lexer.input.len() {
            let c = lexer.current_char();
            if c.is_whitespace() || is_annotation_punctuation(c) {
                break;
            }
            text_token.push(c);
            lexer.advance();
        }
        tokens.push(AnnotationSubToken::Text(text_token));
    }
    tokens
}

/// Parses annotation text into a vector of structured annotation subtokens.
pub fn parse_annotation_subtokens(text: &str) -> Vec<AnnotationSubToken> {
    tokenize_annotation(text)
}
