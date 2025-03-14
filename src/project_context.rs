// src/project_context.rs

use crate::parser::ast::{ExportItem, TypeInfo};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// The module path used in require() statements.
    pub required_path: String,
    /// Local alias if specified (e.g. `local mymod = require("some.module")`).
    pub local_alias: Option<String>,
    /// Resolved absolute path to the dependency (if available).
    pub resolved_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ModuleInfo {
    pub exports: HashMap<String, ExportItem>,
    pub dependencies: Vec<DependencyInfo>,
    pub source_path: PathBuf,
}

pub struct TypeRegistry {
    pub standard_types: HashMap<&'static str, TypeInfo>,
    pub custom_types: HashMap<String, TypeInfo>,
}

pub struct ProjectContext {
    pub modules: HashMap<String, ModuleInfo>,
    pub type_registry: TypeRegistry,
}

impl ProjectContext {
    pub fn new() -> Self {
        let mut registry = TypeRegistry {
            standard_types: HashMap::new(),
            custom_types: HashMap::new(),
        };

        // Initialize standard Lua types using our centralized TypeInfo from ast.
        registry.standard_types.insert("string", TypeInfo::String);
        registry.standard_types.insert("number", TypeInfo::Number);
        registry.standard_types.insert("boolean", TypeInfo::Boolean);
        registry.standard_types.insert("table", TypeInfo::Table);
        registry
            .standard_types
            .insert("function", TypeInfo::Function);

        Self {
            modules: HashMap::new(),
            type_registry: registry,
        }
    }

    pub fn add_module(&mut self, name: String, info: ModuleInfo) {
        self.modules.insert(name, info);
    }

    pub fn resolve_type(&self, name: &str) -> Option<TypeInfo> {
        self.type_registry
            .custom_types
            .get(name)
            .or_else(|| self.type_registry.standard_types.get(name))
            .cloned()
    }

    pub fn add_export(&mut self, module_name: &str, export: ExportItem) {
        self.modules
            .entry(module_name.to_string())
            .or_insert_with(|| ModuleInfo {
                exports: HashMap::new(),
                dependencies: Vec::new(),
                source_path: PathBuf::new(),
            })
            .exports
            .insert(export.name.clone(), export);
    }
}
