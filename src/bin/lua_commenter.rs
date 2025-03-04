use lua_tools::{annotator, parser, tokenizer};

fn main() {
    let input = "-- Example input
function get_user(id, options)
    if not id then return nil end
    return user_data, error_message
end";

    // Tokenize
    let mut tokenizer = tokenizer::Tokenizer::new(input);
    let tokens = tokenizer.tokenize();

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse(); // Now using the parse method

    // Annotate
    let annotator = annotator::Annotator;
    let annotations = annotator.generate(&ast);

    println!("{}", annotations.join("\n"));
}
