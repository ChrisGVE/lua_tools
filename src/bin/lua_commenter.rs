use lua_tools::{annotator, parser, tokenizer};

fn main() {
    let input = "
    -- Sample module
    local M = {}

    -- Example input
    function M.get_user(id, options)
        if not id then return nil end
        return user_data, error_message
    end

    return M";

    // Tokenize
    let mut tokenizer = tokenizer::Tokenizer::new(input);
    let tokens = tokenizer.tokenize();

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse(); // Now using the parse method

    // Annotate
    let mut annotator = annotator::Annotator::new();
    let annotations = annotator.generate_docs(&ast);

    println!("{}", annotations);
}
