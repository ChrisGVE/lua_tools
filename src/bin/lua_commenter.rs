// File: src/bin/lua_commenter.rs
// Relative Path: lua_tools/src/bin/lua_commenter.rs

use clap::{Arg, ArgAction, Command};
use std::path::Path;
use std::fs;
use regex::Regex;

/// Common function to set up CLI parsing for both tools
fn build_cli() -> Command {
    Command::new("lua_commenter")
        .about("Annotates Lua code with comments and LSP annotations.")
        .arg(
            Arg::new("input")
                .help("Lua source file(s) or pattern")
                .required(true)
                .num_args(1..)
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output filename pattern or directory. If a pattern is used, `{}` represents the input filename without extension. Example: `output_{}.lua` for `input.lua` would generate `output_input.lua`.")
                .required(false)
                .num_args(1)
        )
        .arg(
            Arg::new("overwrite")
                .short('w')
                .long("overwrite")
                .help("Overwrite input file(s)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recursively process files in directories")
                .action(ArgAction::SetTrue)
        )
}

fn process_file(path: &Path, output: Option<&String>, overwrite: bool) {
    println!("Processing file: {:?}", path);
    
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {:?}: {}", path, err);
            return;
        }
    };
    
    let annotated_content = annotate_lua_code(&content);
    
    let output_path = if overwrite {
        path.to_path_buf()
    } else if let Some(output) = output {
        let mut output_path = Path::new(output).to_path_buf();
        if output_path.is_dir() {
            output_path.push(path.file_name().unwrap());
        }
        output_path.set_extension("annotated.lua");
        output_path
    } else {
        path.with_extension("annotated.lua")
    };
    
    if let Err(err) = fs::write(&output_path, annotated_content) {
        eprintln!("Error writing to {:?}: {}", output_path, err);
    } else {
        println!("Annotated file saved: {:?}", output_path);
    }
}

fn annotate_lua_code(content: &str) -> String {
    let function_regex = Regex::new(r"(?m)^(\s*)(function|local function)\s+(\w+(\.\w+)*)\(([^)]*)\)").unwrap();
    let class_regex = Regex::new(r"(?m)^\s*(\w+)\s*=\s*\{\}$").unwrap();
    let return_regex = Regex::new(r"(?m)^\s*return\s*(\{.*\})$").unwrap();
    let mut annotated_content = String::new();
    
    for line in content.lines() {
        if let Some(caps) = function_regex.captures(line) {
            let indent = &caps[1];
            let func_name = &caps[3];
            let params = &caps[5];
            
            annotated_content.push_str(&format!(
                "{}---@function {}\n", indent, func_name
            ));
            
            if !params.is_empty() {
                for param in params.split(',').map(|p| p.trim()) {
                    annotated_content.push_str(&format!("{}---@param {} any\n", indent, param));
                }
            }
            
            if return_regex.is_match(content) {
                annotated_content.push_str(&format!("{}---@return table\n", indent));
            }
        } else if let Some(caps) = class_regex.captures(line) {
            let indent = "    ";
            let class_name = &caps[1];
            
            annotated_content.push_str(&format!(
                "{}---@class {}\n{}{}", 
                indent, class_name, indent, line
            ));
        }
        
        annotated_content.push_str(line);
        annotated_content.push('\n');
    }
    
    annotated_content
}

fn main() {
    let matches = build_cli().get_matches();

    let input_files: Vec<&str> = matches.get_many::<String>("input")
        .unwrap()
        .map(String::as_str)
        .collect();

    let output = matches.get_one::<String>("output");
    let overwrite = matches.get_flag("overwrite");
    let recursive = matches.get_flag("recursive");

    for input in &input_files {
        let path = Path::new(input);
        if path.is_file() {
            process_file(path, output, overwrite);
        } else if path.is_dir() && recursive {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        process_file(&file_path, output, overwrite);
                    }
                }
            }
        }
    }
}
