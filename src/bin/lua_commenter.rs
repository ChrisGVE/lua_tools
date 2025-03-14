// src/bin/lua_commenter.rs

use clap::{Arg, ArgAction, Command};
use lua_tools::{annotator, parser, project_context, tokenizer, type_inference};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Walk upward from the given directory until a ".git" folder is found.
/// If none is found, return the current working directory.
fn find_project_root<P: AsRef<Path>>(start: P) -> PathBuf {
    let mut dir = fs::canonicalize(start).unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join(".git").is_dir() {
            return dir;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Compute the file path relative to the project root.
fn relative_path<P: AsRef<Path>>(file: P, root: P) -> String {
    let file = fs::canonicalize(file).unwrap_or_else(|_| PathBuf::from("unknown"));
    let root = fs::canonicalize(root).unwrap_or_else(|_| PathBuf::from("."));
    file.strip_prefix(root)
        .unwrap_or(&file)
        .to_string_lossy()
        .into_owned()
}

/// Process a single Lua file: tokenize, parse, infer types, and annotate.
fn process_file(path: &Path, output_pattern: &str, overwrite: bool) -> String {
    eprintln!("Processing file: {:?}", path);
    let content = fs::read_to_string(path).expect("Failed to read file");

    // Tokenize using our updated CodeTokenizer.
    let mut code_tokenizer = tokenizer::CodeTokenizer::new(&content);
    let tokens = code_tokenizer.tokenize();
    println!("{}", tokenizer::token::pretty_print_tokens(&tokens));

    // Parse tokens into an AST using the code parser.
    let mut code_parser = parser::code_parser::CodeParser::new(tokens);
    let code_ast = code_parser.parse();
    println!("{}", parser::pretty_print::pretty_print_code_ast(&code_ast));

    // Parse tokens into an AST using the annotations parser.
    // let mut annotation_parser = parser::annotation_parser::AnnotationParser::new(tokens);
    // let annotation_ast = annotation_parser.parse();

    // Run type inference on the AST.
    let proj_ctx = project_context::ProjectContext::new();
    let mut type_analyzer = type_inference::TypeAnalyzer::new(proj_ctx);
    type_analyzer.analyze(&code_ast);

    // Generate annotations from the AST.
    let mut ann = annotator::Annotator::new();
    let annotations = ann.generate_docs(&code_ast);

    // Prepend the relative file path as a header.
    let abs_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let parent = abs_path.parent().unwrap_or_else(|| Path::new("."));
    let project_root = find_project_root(parent);
    let rel_path = relative_path(&abs_path, &project_root);
    let header = format!("-- {}\n\n", rel_path);
    let final_output = format!("{}{}", header, annotations);

    // Write output based on CLI flags.
    if std::env::args().len() > 2 {
        if overwrite {
            fs::write(path, &final_output).expect("Failed to write output file");
            eprintln!("File overwritten: {:?}", path);
        } else {
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
            let new_filename = output_pattern.replace("{}", &filename);
            let output_path = path.with_file_name(new_filename);
            fs::write(&output_path, &final_output).expect("Failed to write output file");
            eprintln!("Output written to: {:?}", output_path);
        }
    }
    final_output
}

/// Process all Lua files in a directory (recursively if specified).
fn process_directory(dir: &Path, output_pattern: &str, overwrite: bool, recursive: bool) {
    let entries = fs::read_dir(dir).expect("Failed to read directory");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "lua") {
            process_file(&path, output_pattern, overwrite);
        } else if path.is_dir() && recursive {
            process_directory(&path, output_pattern, overwrite, recursive);
        }
    }
}

fn main() {
    let matches = Command::new("lua_commenter")
        .about("Annotates Lua files with Lua LSP annotations")
        .arg(
            Arg::new("input")
                .help("Input file(s) or directory")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output filename pattern, use {} as placeholder (default: annotated_{})")
                .value_name("pattern")
                .default_value("annotated_{}"),
        )
        .arg(
            Arg::new("overwrite")
                .short('w')
                .long("overwrite")
                .help("Modify files in place")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recursively process directories")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let inputs: Vec<String> = matches
        .get_many::<String>("input")
        .unwrap()
        .map(|s| s.to_string())
        .collect();
    let output_pattern = matches.get_one::<String>("output").unwrap();
    let overwrite = *matches.get_one::<bool>("overwrite").unwrap_or(&false);
    let recursive = *matches.get_one::<bool>("recursive").unwrap_or(&false);

    if inputs.len() == 1 {
        let path = Path::new(&inputs[0]);
        if path.is_file() {
            let annotated = process_file(path, output_pattern, overwrite);
            println!("{}", annotated);
        } else {
            eprintln!("Expected a file but found a directory.");
        }
    } else {
        for input in inputs {
            let path = Path::new(&input);
            if path.is_file() {
                process_file(path, output_pattern, overwrite);
            } else if path.is_dir() {
                process_directory(path, output_pattern, overwrite, recursive);
            }
        }
    }
}
