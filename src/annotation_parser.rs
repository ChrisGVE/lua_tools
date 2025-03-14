// src/annotation_parser.rs

/// Enum representing a parsed annotation.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    /// @alias <name>\n---| <value> [# description]\n---| ...
    Alias {
        name: String,
        variants: Vec<(String, Option<String>)>,
    },
    /// @param <name> <type> [description...]
    Param {
        name: String,
        type_field: String,
        description: Option<String>,
    },
    /// @return <type> [<name>] [description...]
    Return {
        type_field: String,
        name: Option<String>,
        description: Option<String>,
    },
    /// Other annotation types, parsed generically.
    GenericAnnotation { keyword: String, content: String },
}

/// Parses a multi-line annotation block into a structured `AnnotationKind`.
///
/// The input `text` should be the full annotation block (all lines starting with `---`).
/// For example:
/// ```lua
/// ---@alias DeviceSide
/// ---| '"left"' # The left side of the device
/// ---| '"right"' # The right side of the device
/// ```
///
/// For unrecognized annotation keywords, it returns a GenericAnnotation.

pub fn parse_annotation(text: &str) -> AnnotationKind {
    // Split the annotation block into lines and remove the leading '---' (or extra dashes).
    let lines: Vec<&str> = text
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("---") {
                // Remove the three dashes and any following whitespace.
                trimmed.trim_start_matches("---").trim()
            } else {
                trimmed
            }
        })
        .collect();

    if lines.is_empty() {
        return AnnotationKind::GenericAnnotation {
            keyword: "".to_string(),
            content: "".to_string(),
        };
    }

    // Use the original keyword without forcing lowercase.
    let mut parts = lines[0].split_whitespace();
    let original_keyword = parts.next().unwrap_or("").to_string();
    let keyword = original_keyword.to_lowercase();

    match keyword.as_str() {
        "alias" => {
            // Expect: alias <name>
            let name = parts.next().unwrap_or("").to_string(); // preserve original case
            let mut variants = Vec::new();
            // Subsequent lines starting with '|' are variant definitions.
            for line in lines.iter().skip(1) {
                let trimmed = line.trim();
                if trimmed.starts_with("|") {
                    // Remove the leading pipe and any surrounding whitespace.
                    let variant_line = trimmed.trim_start_matches("|").trim();
                    // If a '#' is present, split into variant and description.
                    if let Some(idx) = variant_line.find('#') {
                        let variant = variant_line[..idx].trim().to_string();
                        let desc = variant_line[idx + 1..].trim().to_string();
                        variants.push((variant, if desc.is_empty() { None } else { Some(desc) }));
                    } else {
                        variants.push((variant_line.to_string(), None));
                    }
                }
            }
            AnnotationKind::Alias { name, variants }
        }
        "param" => {
            // Expect: param <name> <type> [description...]
            let name = parts.next().unwrap_or("").to_string();
            let type_field = parts.next().unwrap_or("").to_string();
            let description = if parts.clone().count() > 0 {
                Some(parts.collect::<Vec<&str>>().join(" "))
            } else {
                None
            };
            AnnotationKind::Param {
                name,
                type_field,
                description,
            }
        }
        "return" => {
            // Expect: return <type> [<name>] [description...]
            let type_field = parts.next().unwrap_or("").to_string();
            let maybe_name = parts.next().map(|s| s.to_string());
            let description = if parts.clone().count() > 0 {
                Some(parts.collect::<Vec<&str>>().join(" "))
            } else {
                None
            };
            AnnotationKind::Return {
                type_field,
                name: maybe_name,
                description,
            }
        }
        _ => {
            // Default: treat as generic annotation.
            AnnotationKind::GenericAnnotation {
                keyword,
                content: parts.collect::<Vec<&str>>().join(" "),
            }
        }
    }
}
