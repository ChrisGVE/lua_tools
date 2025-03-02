// File: src/bin/lua_header.rs
// Relative Path: lua_tools/src/bin/lua_header.rs

use clap::{Arg, ArgAction, Command};
use std::path::Path;
use std::fs;
use regex::Regex;

/// Common function to set up CLI parsing for both tools
fn build_cli() -> Command {
    Command::new("lua_header")
        .about("Generates header files for Lua modules.")
        .arg(
            Arg::new("input")
                .help("Lua source file(s) or pattern")
                .required(true)
                .num_args(1..)
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recursively process files in directories")
                .action(ArgAction::SetTrue)
        )
}

fn process_file(path: &Path) {
    println!("Processing file: {:?}", path);
    
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {:?}: {}", path, err);
            return;
        }
    };
    
    let header_content = extract_lua_header(&content);
    let header_path = path.with_extension("header.lua");
    
    if let Err(err) = fs::write(&header_path, header_content) {
        eprintln!("Error writing to {:?}: {}", header_path, err);
    } else {
        println!("Header file saved: {:?}", header_path);
    }
}

fn extract_lua_header(content: &str) -> String {
    let function_regex = Regex::new(r"(?m)^\s*function\s+(\w+(\.\w+)*)\(([^)]*)\)").unwrap();
    let mut header_content = String::new();
    
    header_content.push_str("-- Lua Module Header\n\n");
    
    for line in content.lines() {
        if let Some(caps) = function_regex.captures(line) {
            let func_name = &caps[1];
            let params = &caps[3];
            
            header_content.push_str(&format!(
                "--- Function: {}\n-- @param {}\n-- @return TODO\nfunction {}({}) end\n\n", 
                func_name, params.replace(",", "\n-- @param"), func_name, params
            ));
        }
    }
    
    header_content
}

fn main() {
    let matches = build_cli().get_matches();

    let input_files: Vec<&str> = matches.get_many::<String>("input")
        .unwrap()
        .map(String::as_str)
        .collect();

    let recursive = matches.get_flag("recursive");

    for input in &input_files {
        let path = Path::new(input);
        if path.is_file() {
            process_file(path);
        } else if path.is_dir() && recursive {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        process_file(&file_path);
                    }
                }
            }
        }
    }
}
