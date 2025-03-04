// File: src/bin/lua_commenter.rs
// Relative Path: lua_tools/src/bin/lua_commenter.rs

use clap::{Arg, ArgAction, Command};
use std::fs;
use std::path::Path;
use regex::Regex;

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
                .help("Output filename pattern or directory.")
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
}

fn process_file(path: &Path, output: Option<&String>, overwrite: bool) {
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
    }
}

fn annotate_lua_code(content: &str) -> String {
    let function_regex = Regex::new(r"(?m)^(\s*)(function|local function)\s+(\w+(\.\w+)*)\(([^)]*)\)").unwrap();
    let return_regex = Regex::new(r"(?m)^\s*return\s+([^\n]+)").unwrap();
    let mut annotated_content = String::new();
    
    for line in content.lines() {
        if let Some(caps) = function_regex.captures(line) {
            let indent = &caps[1];
            let func_name = &caps[3];
            let params = &caps[5];
            
            annotated_content.push_str(&format!("{}-- TODO: Describe this function\n", indent));
            annotated_content.push_str(&format!("{}---@function {}\n", indent, func_name));
            
            if !params.is_empty() {
                for param in params.split(',').map(|p| p.trim()) {
                    annotated_content.push_str(&format!("{}---@param {} <TYPE> -- TODO: Specify type\n", indent, param));
                }
            }
            
            if let Some(ret_caps) = return_regex.captures(content) {
                let return_expr = ret_caps[1].trim();
                if return_expr.starts_with("{") && return_expr.ends_with("}") {
                    if return_expr.contains("=") {
                        annotated_content.push_str(&format!("{}---@return table<string, any> -- TODO: Specify key-value types\n", indent));
                    } else {
                        annotated_content.push_str(&format!("{}---@return table<any> -- TODO: Specify item type\n", indent));
                    }
                } else if return_expr == "true" || return_expr == "false" {
                    annotated_content.push_str(&format!("{}---@return boolean\n", indent));
                } else if return_expr.parse::<f64>().is_ok() {
                    annotated_content.push_str(&format!("{}---@return number\n", indent));
                } else if return_expr.starts_with("\"") && return_expr.ends_with("\"") {
                    annotated_content.push_str(&format!("{}---@return string\n", indent));
                } else if return_expr.contains(" or nil") {
                    annotated_content.push_str(&format!("{}---@return any?\n", indent));
                } else {
                    annotated_content.push_str(&format!("{}---@return any -- TODO: Specify return type\n", indent));
                }
            }
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
    
    for input in &input_files {
        let path = Path::new(input);
        if path.is_file() {
            process_file(path, output, overwrite);
        }
    }
}
