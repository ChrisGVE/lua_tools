// src/project_context.rs

use crate::frameworks::FrameworkRegistry;
use crate::parser::ast::{ExportItem, TypeInfo};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// The module path used in require() statements.
    pub required_path: String,
    /// Local alias if specified (e.g. `local mymod = require("some.module")`).
    pub local_alias: Option<String>,
    /// Resolved absolute path to the dependency (if available).
    pub resolved_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Exported items from this module
    pub exports: HashMap<String, ExportItem>,
    /// Dependencies required by this module
    pub dependencies: Vec<DependencyInfo>,
    /// Absolute path to the source file
    pub source_path: PathBuf,
    /// Whether this is the main module (init.lua or main.lua)
    pub is_main: bool,
    /// Whether this module has been processed
    pub processed: bool,
}

#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub type_info: TypeInfo,
    pub description: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<TypeField>,
    pub methods: HashMap<String, FunctionSignature>,
    pub description: Option<String>,
    pub is_alias: bool,
    pub variants: Vec<String>,  // For alias/enum types
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub type_info: TypeInfo,
    pub description: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_types: Vec<TypeInfo>,
    pub description: Option<String>,
    pub is_method: bool,
}

/// Supported Lua versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaVersion {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
}

impl LuaVersion {
    /// Get version as string
    pub fn as_str(&self) -> &'static str {
        match self {
            LuaVersion::Lua51 => "5.1",
            LuaVersion::Lua52 => "5.2", 
            LuaVersion::Lua53 => "5.3",
            LuaVersion::Lua54 => "5.4",
        }
    }
    
    /// Parse version string
    pub fn from_str(version: &str) -> Option<Self> {
        match version {
            "5.1" => Some(LuaVersion::Lua51),
            "5.2" => Some(LuaVersion::Lua52),
            "5.3" => Some(LuaVersion::Lua53),
            "5.4" => Some(LuaVersion::Lua54),
            // Handle shorthand versions
            "51" => Some(LuaVersion::Lua51),
            "52" => Some(LuaVersion::Lua52),
            "53" => Some(LuaVersion::Lua53),
            "54" => Some(LuaVersion::Lua54),
            _ => None,
        }
    }
    
    /// Check if feature is available in this version
    pub fn has_feature(&self, feature: &str) -> bool {
        match (self, feature) {
            // Lua 5.1 specific features
            (LuaVersion::Lua51, "module") => true,
            (LuaVersion::Lua51, "setfenv") => true,
            (LuaVersion::Lua51, "getfenv") => true,
            (LuaVersion::Lua51, "unpack") => true,
            (LuaVersion::Lua51, "loadstring") => true,
            
            // Lua 5.2+ features
            (LuaVersion::Lua51, "goto") => false,
            (LuaVersion::Lua51, "bit32") => false,
            (_, "goto") => true,
            (_, "bit32") => true,
            
            // Lua 5.3+ features
            (LuaVersion::Lua51, "integer_division") => false,
            (LuaVersion::Lua52, "integer_division") => false,
            (_, "integer_division") => true,
            
            // Lua 5.3+ utf8 library
            (LuaVersion::Lua51, "utf8") => false,
            (LuaVersion::Lua52, "utf8") => false,
            (_, "utf8") => true,
            
            // Lua 5.4 specific features
            (LuaVersion::Lua54, "to_close") => true,
            (_, "to_close") => false,
            
            // Default to not supported
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct TypeRegistry {
    pub standard_types: HashMap<&'static str, TypeInfo>,
    pub custom_types: HashMap<String, CustomType>,
    pub function_signatures: HashMap<String, FunctionSignature>,
}

pub struct ProjectContext {
    /// All modules in the project, keyed by their module name
    pub modules: HashMap<String, ModuleInfo>,
    /// Type definitions for the project
    pub type_registry: TypeRegistry,
    /// Root directory of the project
    pub project_root: Option<PathBuf>,
    /// All Lua files in the project
    pub lua_files: Vec<PathBuf>,
    /// Dependency graph (module_name -> [dependent_module_names])
    pub dependency_graph: HashMap<String, HashSet<String>>,
    /// Whether the type definition file has been processed
    pub type_file_processed: bool,
    /// Target Lua version for type checking and features
    pub lua_version: LuaVersion,
    /// Framework registry for framework-specific type definitions
    pub framework_registry: Option<FrameworkRegistry>,
    /// Detected frameworks in the project
    pub detected_frameworks: Vec<(String, String)>, // (name, version)
}

impl ProjectContext {
    pub fn new() -> Self {
        // Default to latest Lua version
        Self::new_with_version(LuaVersion::Lua54)
    }
    
    pub fn new_with_version(lua_version: LuaVersion) -> Self {
        let mut registry = TypeRegistry {
            standard_types: HashMap::new(),
            custom_types: HashMap::new(),
            function_signatures: HashMap::new(),
        };

        // Initialize standard Lua types using our centralized TypeInfo from ast.
        registry.standard_types.insert("string", TypeInfo::String);
        registry.standard_types.insert("number", TypeInfo::Number);
        registry.standard_types.insert("boolean", TypeInfo::Boolean);
        registry.standard_types.insert("table", TypeInfo::Table);
        registry.standard_types.insert("function", TypeInfo::Function);
        registry.standard_types.insert("nil", TypeInfo::Unknown);
        registry.standard_types.insert("any", TypeInfo::Unknown);
        
        // Add Lua 5.3+ integer type
        if matches!(lua_version, LuaVersion::Lua53 | LuaVersion::Lua54) {
            registry.standard_types.insert("integer", TypeInfo::Number);
        }
        
        let mut ctx = Self {
            modules: HashMap::new(),
            type_registry: registry,
            project_root: None,
            lua_files: Vec::new(),
            dependency_graph: HashMap::new(),
            type_file_processed: false,
            lua_version,
            framework_registry: Some(FrameworkRegistry::new()),
            detected_frameworks: Vec::new(),
        };
        
        // Load standard library definitions
        ctx.load_standard_library();
        
        ctx
    }
    
    /// Set the Lua version for this project
    pub fn set_lua_version(&mut self, version: LuaVersion) {
        if self.lua_version != version {
            self.lua_version = version;
            
            // Reload standard library with new version
            self.modules.clear();
            self.load_standard_library();
        }
    }
    
    /// Load standard Lua library definitions
    pub fn load_standard_library(&mut self) {
        // Define libraries based on Lua version
        let mut std_libs = vec![
            "string", "table", "math", "io", "os", "debug", "coroutine"
        ];
        
        // Version-specific libraries
        match self.lua_version {
            LuaVersion::Lua51 => {
                // No bit32 or utf8 in 5.1
            },
            LuaVersion::Lua52 => {
                // Lua 5.2 has bit32 but no utf8
                std_libs.push("bit32");
            },
            LuaVersion::Lua53 | LuaVersion::Lua54 => {
                // Lua 5.3 and 5.4 have both bit32 and utf8
                std_libs.push("bit32");
                std_libs.push("utf8");
            }
        }
        
        for lib_name in std_libs {
            let module_info = ModuleInfo {
                exports: HashMap::new(),
                dependencies: Vec::new(),
                source_path: PathBuf::from(format!("stdlib/{}.lua", lib_name)),
                is_main: false,
                processed: true,
            };
            
            // Add standard module
            self.modules.insert(lib_name.to_string(), module_info);
            
            // Add standard library functions based on the library name
            match lib_name {
                "string" => self.load_string_library(),
                "table" => self.load_table_library(),
                "math" => self.load_math_library(),
                "bit32" => self.load_bit32_library(),
                "utf8" => self.load_utf8_library(),
                // Add more as needed
                _ => {}
            }
        }
        
        // Load global functions (version-specific)
        self.load_global_functions();
    }
    
    /// Load Lua global functions
    fn load_global_functions(&mut self) {
        // Core global functions (available in all versions)
        let mut global_functions = vec![
            ("assert", TypeInfo::Unknown),
            ("collectgarbage", TypeInfo::Unknown),
            ("dofile", TypeInfo::Unknown),
            ("error", TypeInfo::Unknown),
            ("getmetatable", TypeInfo::Table),
            ("ipairs", TypeInfo::Function),
            ("load", TypeInfo::Function),
            ("loadfile", TypeInfo::Function),
            ("next", TypeInfo::Unknown),
            ("pairs", TypeInfo::Function),
            ("pcall", TypeInfo::Boolean),
            ("print", TypeInfo::Unknown),
            ("rawequal", TypeInfo::Boolean),
            ("rawget", TypeInfo::Unknown),
            ("rawset", TypeInfo::Unknown),
            ("require", TypeInfo::Unknown),
            ("select", TypeInfo::Unknown),
            ("setmetatable", TypeInfo::Table),
            ("tonumber", TypeInfo::Number),
            ("tostring", TypeInfo::String),
            ("type", TypeInfo::String),
            ("xpcall", TypeInfo::Boolean),
        ];
        
        // Version-specific global functions
        match self.lua_version {
            LuaVersion::Lua51 => {
                // Lua 5.1 specific globals
                global_functions.push(("getfenv", TypeInfo::Table));
                global_functions.push(("loadstring", TypeInfo::Function));
                global_functions.push(("module", TypeInfo::Unknown));
                global_functions.push(("setfenv", TypeInfo::Boolean));
                global_functions.push(("unpack", TypeInfo::Unknown));
            },
            LuaVersion::Lua52 => {
                // rawlen is available in 5.2+
                global_functions.push(("rawlen", TypeInfo::Number));
            },
            LuaVersion::Lua53 => {
                // rawlen is available in 5.2+
                global_functions.push(("rawlen", TypeInfo::Number));
            },
            LuaVersion::Lua54 => {
                // rawlen is available in 5.2+
                global_functions.push(("rawlen", TypeInfo::Number));
                // warn is new in 5.4
                global_functions.push(("warn", TypeInfo::Unknown));
            }
        }
        
        // Add global module for internal tracking
        let module_info = ModuleInfo {
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source_path: PathBuf::from("stdlib/_G.lua"),
            is_main: false,
            processed: true,
        };
        self.modules.insert("_G".to_string(), module_info);
        
        // Register global functions
        for (fn_name, ret_type) in global_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to _G module exports
            if let Some(module) = self.modules.get_mut("_G") {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures (without module prefix)
            let sig = FunctionSignature {
                name: fn_name.to_string(),
                parameters: Vec::new(), // We could add detailed params later
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(fn_name.to_string(), sig);
        }
        
        // Define global variables
        let mut globals = vec![
            ("_G", TypeInfo::Table),
            ("_VERSION", TypeInfo::String),
        ];
        
        // Version-specific globals
        if matches!(self.lua_version, LuaVersion::Lua54) {
            globals.push(("_EXSTACK", TypeInfo::Table)); // Lua 5.4 stack introspection
        }
        
        for (var_name, var_type) in globals {
            let export = ExportItem {
                name: var_name.to_string(),
                type_info: var_type,
            };
            
            // Add to _G module exports
            if let Some(module) = self.modules.get_mut("_G") {
                module.exports.insert(var_name.to_string(), export);
            }
        }
    }
    
    /// Load bit32 library (Lua 5.2+)
    fn load_bit32_library(&mut self) {
        if !matches!(self.lua_version, LuaVersion::Lua52 | LuaVersion::Lua53 | LuaVersion::Lua54) {
            return; // Skip if not available
        }
        
        let module_name = "bit32";
        
        // Bit32 functions
        let bit32_functions = vec![
            ("arshift", TypeInfo::Number),
            ("band", TypeInfo::Number),
            ("bnot", TypeInfo::Number),
            ("bor", TypeInfo::Number),
            ("btest", TypeInfo::Boolean),
            ("bxor", TypeInfo::Number),
            ("extract", TypeInfo::Number),
            ("lrotate", TypeInfo::Number),
            ("lshift", TypeInfo::Number),
            ("replace", TypeInfo::Number),
            ("rrotate", TypeInfo::Number),
            ("rshift", TypeInfo::Number),
        ];
        
        for (fn_name, ret_type) in bit32_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures
            let full_name = format!("{}.{}", module_name, fn_name);
            let sig = FunctionSignature {
                name: full_name.clone(),
                parameters: Vec::new(),
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(full_name, sig);
        }
    }
    
    /// Load utf8 library (Lua 5.3+)
    fn load_utf8_library(&mut self) {
        if !matches!(self.lua_version, LuaVersion::Lua53 | LuaVersion::Lua54) {
            return; // Skip if not available
        }
        
        let module_name = "utf8";
        
        // UTF8 functions
        let utf8_functions = vec![
            ("char", TypeInfo::String),
            ("codes", TypeInfo::Function),
            ("codepoint", TypeInfo::Number),
            ("len", TypeInfo::Number),
            ("offset", TypeInfo::Number),
        ];
        
        for (fn_name, ret_type) in utf8_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures
            let full_name = format!("{}.{}", module_name, fn_name);
            let sig = FunctionSignature {
                name: full_name.clone(),
                parameters: Vec::new(),
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(full_name, sig);
        }
        
        // UTF8 patterns
        if let Some(module) = self.modules.get_mut(module_name) {
            let pattern_export = ExportItem {
                name: "charpattern".to_string(),
                type_info: TypeInfo::String,
            };
            module.exports.insert("charpattern".to_string(), pattern_export);
        }
    }
    
    /// Load standard string library functions
    fn load_string_library(&mut self) {
        let module_name = "string";
        
        // Common string functions with their return types
        let string_functions = vec![
            ("byte", TypeInfo::Number),
            ("char", TypeInfo::String),
            ("dump", TypeInfo::String),
            ("find", TypeInfo::Number),
            ("format", TypeInfo::String),
            ("gmatch", TypeInfo::Function),
            ("gsub", TypeInfo::String),
            ("len", TypeInfo::Number),
            ("lower", TypeInfo::String),
            ("match", TypeInfo::String),
            ("rep", TypeInfo::String),
            ("reverse", TypeInfo::String),
            ("sub", TypeInfo::String),
            ("upper", TypeInfo::String),
        ];
        
        for (fn_name, ret_type) in string_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures
            let full_name = format!("{}.{}", module_name, fn_name);
            let sig = FunctionSignature {
                name: full_name.clone(),
                parameters: Vec::new(), // We could add detailed params later
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(full_name, sig);
        }
    }
    
    /// Load standard table library functions
    fn load_table_library(&mut self) {
        let module_name = "table";
        
        // Common table functions with their return types (available in all versions)
        let mut table_functions = vec![
            ("concat", TypeInfo::String),
            ("insert", TypeInfo::Unknown),
            ("remove", TypeInfo::Unknown),
            ("sort", TypeInfo::Unknown),
        ];
        
        // Version-specific table functions
        match self.lua_version {
            LuaVersion::Lua51 => {
                // maxn is only in 5.1
                table_functions.push(("maxn", TypeInfo::Number));
                
                // unpack is global in 5.1
                // table.unpack doesn't exist in 5.1
            },
            LuaVersion::Lua52 | LuaVersion::Lua53 | LuaVersion::Lua54 => {
                // pack and unpack are in 5.2+
                table_functions.push(("pack", TypeInfo::Table));
                table_functions.push(("unpack", TypeInfo::Unknown));
            }
        }
        
        // move was added in 5.3
        if matches!(self.lua_version, LuaVersion::Lua53 | LuaVersion::Lua54) {
            table_functions.push(("move", TypeInfo::Table));
        }
        
        // Lua 5.4 specific functions
        if matches!(self.lua_version, LuaVersion::Lua54) {
            table_functions.push(("clone", TypeInfo::Table));
        }
        
        for (fn_name, ret_type) in table_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures
            let full_name = format!("{}.{}", module_name, fn_name);
            let sig = FunctionSignature {
                name: full_name.clone(),
                parameters: Vec::new(), // We could add detailed params later
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(full_name, sig);
        }
    }
    
    /// Load standard math library functions
    fn load_math_library(&mut self) {
        let module_name = "math";
        
        // Common math functions with their return types (available in all versions)
        let mut math_functions = vec![
            ("abs", TypeInfo::Number),
            ("acos", TypeInfo::Number),
            ("asin", TypeInfo::Number),
            ("atan", TypeInfo::Number),
            ("ceil", TypeInfo::Number),
            ("cos", TypeInfo::Number),
            ("deg", TypeInfo::Number),
            ("exp", TypeInfo::Number),
            ("floor", TypeInfo::Number),
            ("fmod", TypeInfo::Number),
            ("log", TypeInfo::Number),
            ("max", TypeInfo::Number),
            ("min", TypeInfo::Number),
            ("modf", TypeInfo::Number),
            ("rad", TypeInfo::Number),
            ("random", TypeInfo::Number),
            ("randomseed", TypeInfo::Unknown),
            ("sin", TypeInfo::Number),
            ("sqrt", TypeInfo::Number),
            ("tan", TypeInfo::Number),
        ];
        
        // Version-specific math functions
        match self.lua_version {
            LuaVersion::Lua51 => {
                // Lua 5.1 functions
                math_functions.push(("pow", TypeInfo::Number));
                math_functions.push(("log10", TypeInfo::Number));
            },
            LuaVersion::Lua52 => {
                // Lua 5.2 adds atan2
                math_functions.push(("atan2", TypeInfo::Number));
            },
            LuaVersion::Lua53 => {
                // Lua 5.3 adds ult and tointeger
                math_functions.push(("tointeger", TypeInfo::Number));
                math_functions.push(("type", TypeInfo::String));
                math_functions.push(("ult", TypeInfo::Boolean));
            },
            LuaVersion::Lua54 => {
                // Same as 5.3
                math_functions.push(("tointeger", TypeInfo::Number));
                math_functions.push(("type", TypeInfo::String));
                math_functions.push(("ult", TypeInfo::Boolean));
            }
        }
        
        // Add constants based on version
        let mut math_constants = vec![
            ("pi", TypeInfo::Number),
            ("huge", TypeInfo::Number),
        ];
        
        // Lua 5.3+ constants
        if matches!(self.lua_version, LuaVersion::Lua53 | LuaVersion::Lua54) {
            math_constants.push(("maxinteger", TypeInfo::Number));
            math_constants.push(("mininteger", TypeInfo::Number));
        }
        
        // Process functions
        for (fn_name, ret_type) in math_functions {
            let export = ExportItem {
                name: fn_name.to_string(),
                type_info: ret_type.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(fn_name.to_string(), export);
            }
            
            // Add to function signatures
            let full_name = format!("{}.{}", module_name, fn_name);
            let sig = FunctionSignature {
                name: full_name.clone(),
                parameters: Vec::new(), // We could add detailed params later
                return_types: vec![ret_type],
                description: None,
                is_method: false,
            };
            
            self.type_registry.function_signatures.insert(full_name, sig);
        }
        
        // Process constants
        for (const_name, type_info) in math_constants {
            let export = ExportItem {
                name: const_name.to_string(),
                type_info: type_info.clone(),
            };
            
            // Add to module exports
            if let Some(module) = self.modules.get_mut(module_name) {
                module.exports.insert(const_name.to_string(), export);
            }
        }
    }
    
    /// Detect the project root by looking for init.lua or similar markers
    pub fn detect_project_root(&mut self, starting_path: &Path) -> Option<PathBuf> {
        let mut current_dir = starting_path.to_path_buf();
        if current_dir.is_file() {
            current_dir = current_dir.parent()?.to_path_buf();
        }
        
        // Traverse up the directory tree looking for project markers
        while let Some(parent) = current_dir.parent() {
            // Check for common project root markers
            if current_dir.join("init.lua").exists() || 
               current_dir.join("main.lua").exists() ||
               current_dir.join(".git").exists() ||
               current_dir.join("lua").exists() {
                self.project_root = Some(current_dir.clone());
                
                // Attempt to detect Lua version from project files
                self.detect_lua_version(&current_dir);
                
                return Some(current_dir);
            }
            
            // Move up one directory
            current_dir = parent.to_path_buf();
        }
        
        // If no project root was found, use the starting directory
        self.project_root = Some(starting_path.to_path_buf());
        
        // Try to detect Lua version anyway
        self.detect_lua_version(&starting_path.to_path_buf());
        
        // Detect frameworks
        self.detect_frameworks(&starting_path.to_path_buf());
        
        Some(starting_path.to_path_buf())
    }
    
    /// Attempt to detect Lua version from project files
    pub fn detect_lua_version(&mut self, dir: &Path) {
        // Check for version-specific configuration files and patterns
        
        // 1. Check for .luarc.json file (used by Lua Language Server and others)
        let luarc_path = dir.join(".luarc.json");
        if luarc_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&luarc_path) {
                // Look for runtime.version field
                if content.contains("\"runtime.version\":") || content.contains("\"runtime\": {") {
                    if content.contains("\"5.1\"") || content.contains("\"51\"") || content.contains("\"Lua 5.1\"") {
                        self.lua_version = LuaVersion::Lua51;
                        println!("Detected Lua 5.1 from .luarc.json");
                        return;
                    } else if content.contains("\"5.2\"") || content.contains("\"52\"") || content.contains("\"Lua 5.2\"") {
                        self.lua_version = LuaVersion::Lua52;
                        println!("Detected Lua 5.2 from .luarc.json");
                        return;
                    } else if content.contains("\"5.3\"") || content.contains("\"53\"") || content.contains("\"Lua 5.3\"") {
                        self.lua_version = LuaVersion::Lua53;
                        println!("Detected Lua 5.3 from .luarc.json");
                        return;
                    } else if content.contains("\"5.4\"") || content.contains("\"54\"") || content.contains("\"Lua 5.4\"") {
                        self.lua_version = LuaVersion::Lua54;
                        println!("Detected Lua 5.4 from .luarc.json");
                        return;
                    } else if content.contains("\"LuaJIT\"") || content.contains("\"luajit\"") {
                        // LuaJIT is closest to Lua 5.1 with some 5.2 features
                        self.lua_version = LuaVersion::Lua51; 
                        println!("Detected LuaJIT from .luarc.json (using Lua 5.1 compatibility)");
                        return;
                    }
                }
            }
        }
        
        // 1b. Check for other configuration files that specify Lua version
        
        // Check for .lua-version file (used by some version managers)
        let lua_version_file = dir.join(".lua-version");
        if lua_version_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&lua_version_file) {
                let content = content.trim();
                if let Some(version) = LuaVersion::from_str(content) {
                    self.lua_version = version;
                    println!("Detected Lua {} from .lua-version file", version.as_str());
                    return;
                }
            }
        }
        
        // Check for config.lua - used by some Lua frameworks
        let config_lua = dir.join("config.lua");
        if config_lua.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_lua) {
                if content.contains("lua_version") || content.contains("LUA_VERSION") {
                    if content.contains("= \"5.1\"") || content.contains("= '5.1'") || 
                       content.contains("=\"5.1\"") || content.contains("='5.1'") {
                        self.lua_version = LuaVersion::Lua51;
                        println!("Detected Lua 5.1 from config.lua");
                        return;
                    } else if content.contains("= \"5.2\"") || content.contains("= '5.2'") || 
                              content.contains("=\"5.2\"") || content.contains("='5.2'") {
                        self.lua_version = LuaVersion::Lua52;
                        println!("Detected Lua 5.2 from config.lua");
                        return;
                    } else if content.contains("= \"5.3\"") || content.contains("= '5.3'") || 
                              content.contains("=\"5.3\"") || content.contains("='5.3'") {
                        self.lua_version = LuaVersion::Lua53;
                        println!("Detected Lua 5.3 from config.lua");
                        return;
                    } else if content.contains("= \"5.4\"") || content.contains("= '5.4'") || 
                              content.contains("=\"5.4\"") || content.contains("='5.4'") {
                        self.lua_version = LuaVersion::Lua54;
                        println!("Detected Lua 5.4 from config.lua");
                        return;
                    }
                }
            }
        }
        
        // Check for .luacheckrc (used by Luacheck linter)
        let luacheckrc = dir.join(".luacheckrc");
        if luacheckrc.exists() {
            if let Ok(content) = std::fs::read_to_string(&luacheckrc) {
                // Check for std configuration which indicates version
                if content.contains("std = ") {
                    if content.contains("\"lua51\"") || content.contains("'lua51'") {
                        self.lua_version = LuaVersion::Lua51;
                        println!("Detected Lua 5.1 from .luacheckrc");
                        return;
                    } else if content.contains("\"lua52\"") || content.contains("'lua52'") {
                        self.lua_version = LuaVersion::Lua52;
                        println!("Detected Lua 5.2 from .luacheckrc");
                        return;
                    } else if content.contains("\"lua53\"") || content.contains("'lua53'") {
                        self.lua_version = LuaVersion::Lua53;
                        println!("Detected Lua 5.3 from .luacheckrc");
                        return;
                    } else if content.contains("\"lua54\"") || content.contains("'lua54'") {
                        self.lua_version = LuaVersion::Lua54;
                        println!("Detected Lua 5.4 from .luacheckrc");
                        return;
                    }
                }
            }
        }
        
        // 2. Check for rockspec files (Luarocks package metadata)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rockspec") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Look for lua version in dependencies section
                        if content.contains("lua ~> 5.1") || content.contains("\"lua >= 5.1, < 5.2\"") {
                            self.lua_version = LuaVersion::Lua51;
                            println!("Detected Lua 5.1 from rockspec file");
                            return;
                        } else if content.contains("lua ~> 5.2") || content.contains("\"lua >= 5.2, < 5.3\"") {
                            self.lua_version = LuaVersion::Lua52;
                            println!("Detected Lua 5.2 from rockspec file");
                            return;
                        } else if content.contains("lua ~> 5.3") || content.contains("\"lua >= 5.3, < 5.4\"") {
                            self.lua_version = LuaVersion::Lua53;
                            println!("Detected Lua 5.3 from rockspec file");
                            return;
                        } else if content.contains("lua ~> 5.4") || content.contains("\"lua >= 5.4\"") {
                            self.lua_version = LuaVersion::Lua54;
                            println!("Detected Lua 5.4 from rockspec file");
                            return;
                        }
                    }
                }
            }
        }
        
        // 2. Check for framework-specific files that indicate a particular Lua version
        
        // Neovim - uses Lua 5.1
        if dir.join("lua").exists() && (
           dir.join("plugin").exists() || 
           dir.join("doc").exists() || 
           dir.join("after").exists() || 
           dir.join("ftplugin").exists() ||
           dir.join("autoload").exists()) {
            self.lua_version = LuaVersion::Lua51;
            println!("Detected Lua 5.1 from Neovim plugin structure");
            return;
        }
        
        // LÖVE2D - often uses Lua 5.1 (older) or 5.3+ (newer versions)
        if dir.join("main.lua").exists() && dir.join("conf.lua").exists() {
            // Try to determine LÖVE version from conf.lua
            if let Ok(content) = std::fs::read_to_string(dir.join("conf.lua")) {
                if content.contains("t.version = \"11.") {
                    self.lua_version = LuaVersion::Lua53;
                    println!("Detected Lua 5.3 from LÖVE2D 11.x configuration");
                    return;
                } else {
                    self.lua_version = LuaVersion::Lua51;
                    println!("Detected Lua 5.1 from LÖVE2D configuration");
                    return;
                }
            } else {
                // Default to 5.1 for LÖVE if we can't determine version
                self.lua_version = LuaVersion::Lua51;
                println!("Detected Lua 5.1 from LÖVE2D project structure");
                return;
            }
        }
        
        // WezTerm uses Lua 5.4
        if dir.join("wezterm.lua").exists() || dir.join(".wezterm.lua").exists() {
            self.lua_version = LuaVersion::Lua54;
            println!("Detected Lua 5.4 from WezTerm configuration");
            return;
        }
        
        // Luvit typically uses Lua 5.2
        if dir.join("package.lua").exists() && dir.join("deps").exists() {
            self.lua_version = LuaVersion::Lua52;
            println!("Detected Lua 5.2 from Luvit project structure");
            return;
        }
        
        // 3. Check for explicit version marker in type.lua
        let type_file = dir.join("type.lua");
        if type_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&type_file) {
                if content.contains("lua_version = \"5.1\"") || content.contains("-- Lua 5.1") {
                    self.lua_version = LuaVersion::Lua51;
                    println!("Detected Lua 5.1 from type.lua");
                    return;
                } else if content.contains("lua_version = \"5.2\"") || content.contains("-- Lua 5.2") {
                    self.lua_version = LuaVersion::Lua52;
                    println!("Detected Lua 5.2 from type.lua");
                    return;
                } else if content.contains("lua_version = \"5.3\"") || content.contains("-- Lua 5.3") {
                    self.lua_version = LuaVersion::Lua53;
                    println!("Detected Lua 5.3 from type.lua");
                    return;
                } else if content.contains("lua_version = \"5.4\"") || content.contains("-- Lua 5.4") {
                    self.lua_version = LuaVersion::Lua54;
                    println!("Detected Lua 5.4 from type.lua");
                    return;
                }
            }
        }
        
        // 4. Scan Lua files for version-specific syntax features
        self.detect_version_from_lua_files(dir);
    }
    
    /// Scan Lua files to detect version from syntax
    fn detect_version_from_lua_files(&mut self, dir: &Path) {
        let mut has_goto = false;
        let mut has_bitwise = false;
        let mut has_integer_division = false;
        let mut has_to_close = false;
        
        // Only scan a limited number of files to avoid performance issues
        let max_files = 10;
        let mut scanned = 0;
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        scanned += 1;
                        
                        // Check for version-specific syntax
                        if content.contains("goto ") || content.contains("::") {
                            has_goto = true;
                        }
                        
                        if content.contains(" & ") || content.contains(" | ") || 
                           content.contains(" ~ ") || content.contains(" << ") || 
                           content.contains(" >> ") {
                            has_bitwise = true;
                        }
                        
                        if content.contains(" // ") {
                            has_integer_division = true;
                        }
                        
                        // <close> variable attribute in Lua 5.4
                        if content.contains("<close>") {
                            has_to_close = true;
                        }
                        
                        if scanned >= max_files {
                            break;
                        }
                    }
                }
            }
        }
        
        // Determine version based on syntax features
        if has_to_close {
            self.lua_version = LuaVersion::Lua54;
            println!("Detected Lua 5.4 from syntax features (to-be-closed variables)");
        } else if has_integer_division {
            self.lua_version = LuaVersion::Lua53;
            println!("Detected Lua 5.3 from syntax features (integer division)");
        } else if has_goto || has_bitwise {
            self.lua_version = LuaVersion::Lua52;
            println!("Detected Lua 5.2 from syntax features (goto/bitwise)");
        } else {
            // Default to Lua 5.1 if no newer features are found
            self.lua_version = LuaVersion::Lua51;
            println!("Using Lua 5.1 as default (no specific version detected)");
        }
    }
    
    /// Scan the project for Lua files starting from the root
    pub fn scan_lua_files(&mut self) -> Result<(), String> {
        let root = self.project_root.clone()
            .ok_or_else(|| "Project root not detected".to_string())?;
        
        self.lua_files.clear();
        self._scan_directory_for_lua_files(&root)?;
        
        Ok(())
    }
    
    fn _scan_directory_for_lua_files(&mut self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Skip hidden directories and common exclude patterns
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !dir_name.starts_with(".") && dir_name != "node_modules" && dir_name != "target" {
                    self._scan_directory_for_lua_files(&path)?;
                }
            } else if path.is_file() {
                // Check if it's a Lua file
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "lua" {
                        self.lua_files.push(path.clone());
                        
                        // Check for type.lua specifically
                        if path.file_name().and_then(|n| n.to_str()) == Some("type.lua") {
                            println!("Found type definition file: {}", path.display());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process the type.lua file if it exists
    pub fn process_type_file(&mut self) -> Result<bool, String> {
        if self.type_file_processed {
            return Ok(true);
        }
        
        let root = self.project_root.clone()
            .ok_or_else(|| "Project root not detected".to_string())?;
            
        // Check for primary type.lua in project root
        let type_file = root.join("type.lua");
        let mut processed = false;
        
        if type_file.exists() {
            // Process the main type file
            self.process_single_type_file(&type_file)?;
            processed = true;
        }
        
        // Look for type definitions directory
        let type_dir = root.join("types");
        if type_dir.exists() && type_dir.is_dir() {
            // Process all .lua files in the types directory
            let entries = match fs::read_dir(&type_dir) {
                Ok(entries) => entries,
                Err(e) => return Err(format!("Failed to read types directory: {}", e)),
            };
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
                        println!("Processing additional type file: {}", path.display());
                        self.process_single_type_file(&path)?;
                        processed = true;
                    }
                }
            }
        }
        
        self.type_file_processed = true;
        Ok(processed)
    }
    
    /// Detect frameworks used in the project
    pub fn detect_frameworks(&mut self, dir: &Path) {
        // Use the framework registry to detect frameworks
        if let Some(registry) = &self.framework_registry {
            let detected = registry.detect_framework_usage(dir);
            
            // Store detected frameworks
            self.detected_frameworks.clear();
            for (name, version_opt) in detected {
                if let Some(version) = version_opt {
                    println!("Detected framework: {} {}", name, version);
                    self.detected_frameworks.push((name, version));
                } else if let Some(latest) = registry.get_latest_version(&name) {
                    println!("Detected framework: {} (using latest version {})", name, latest);
                    self.detected_frameworks.push((name, latest));
                }
            }
            
            // Apply framework definitions to the project context
            self.apply_framework_definitions();
        }
    }
    
    /// Apply detected framework definitions to the project context
    pub fn apply_framework_definitions(&mut self) {
        if let Some(registry) = &self.framework_registry {
            for (name, version) in &self.detected_frameworks {
                println!("Applying framework definitions for {} {}", name, version);
                
                // Apply the framework definition to the project context
                if registry.apply_framework_to_context(self, name, version) {
                    println!("Successfully applied {} {} definitions", name, version);
                } else {
                    println!("Failed to apply {} {} definitions", name, version);
                }
            }
        }
    }
    
    /// Process a single type definition file
    fn process_single_type_file(&mut self, file_path: &Path) -> Result<(), String> {
        // Read the type file
        println!("Processing type definition file: {}", file_path.display());
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read type file: {}", e)),
        };
        
        // Parse the type file using our tokenizer and parser
        let mut code_tokenizer = crate::tokenizer::CodeTokenizer::new_with_options(&content, true);
        let tokens = code_tokenizer.tokenize();
        
        let mut code_parser = crate::parser::code_parser::CodeParser::new(tokens);
        let ast = code_parser.parse();
        
        // Extract type definitions from the AST
        self.extract_type_definitions_from_ast(&ast);
        
        Ok(())
    }
    
    /// Extract type definitions from an AST (used for processing type.lua)
    fn extract_type_definitions_from_ast(&mut self, ast: &[crate::parser::ast::CodeASTNode]) {
        use crate::parser::ast::{CodeASTNode, Expression, TypeInfo};
        
        for node in ast {
            match node {
                // Look for class annotations
                CodeASTNode::Comment { text, .. } => {
                    if text.starts_with("---@class ") {
                        // Parse class annotation
                        let class_line = text.trim_start_matches("---@class ").trim();
                        let parts: Vec<&str> = class_line.split_whitespace().collect();
                        if !parts.is_empty() {
                            let class_name = parts[0].to_string();
                            let description = if parts.len() > 1 {
                                Some(parts[1..].join(" "))
                            } else {
                                None
                            };
                            
                            // Create a custom type
                            let custom_type = CustomType {
                                name: class_name.clone(),
                                fields: Vec::new(),
                                methods: HashMap::new(),
                                description,
                                is_alias: false,
                                variants: Vec::new(),
                            };
                            
                            self.type_registry.custom_types.insert(class_name, custom_type);
                        }
                    } else if text.starts_with("---@field ") {
                        // Parse field annotation
                        let field_line = text.trim_start_matches("---@field ").trim();
                        let parts: Vec<&str> = field_line.split_whitespace().collect();
                        
                        if parts.len() >= 2 {
                            let field_name = parts[0].to_string();
                            let optional = field_name.ends_with('?');
                            let field_name = if optional {
                                field_name.trim_end_matches('?').to_string()
                            } else {
                                field_name
                            };
                            
                            let type_name = parts[1].to_string();
                            let description = if parts.len() > 2 {
                                Some(parts[2..].join(" "))
                            } else {
                                None
                            };
                            
                            // Find the custom type to add this field to
                            // This assumes fields come right after the class definition
                            if let Some(last_type) = self.type_registry.custom_types.keys().last() {
                                if let Some(custom_type) = self.type_registry.custom_types.get_mut(last_type) {
                                    // Add the field
                                    let type_info = self.type_name_to_info(&type_name);
                                    let field = TypeField {
                                        name: field_name,
                                        type_info,
                                        description,
                                        optional,
                                    };
                                    custom_type.fields.push(field);
                                }
                            }
                        }
                    } else if text.starts_with("---@alias ") {
                        // Parse alias annotation
                        let alias_line = text.trim_start_matches("---@alias ").trim();
                        let parts: Vec<&str> = alias_line.split_whitespace().collect();
                        
                        if !parts.is_empty() {
                            let alias_name = parts[0].to_string();
                            let description = if parts.len() > 1 {
                                Some(parts[1..].join(" "))
                            } else {
                                None
                            };
                            
                            // Create a custom type alias
                            let custom_type = CustomType {
                                name: alias_name.clone(),
                                fields: Vec::new(),
                                methods: HashMap::new(),
                                description,
                                is_alias: true,
                                variants: Vec::new(),
                            };
                            
                            self.type_registry.custom_types.insert(alias_name, custom_type);
                        }
                    } else if text.starts_with("---|") {
                        // Parse alias variant
                        let variant_line = text.trim_start_matches("---|").trim();
                        let variant = variant_line.trim_matches('\'').trim_matches('"').to_string();
                        
                        // Add to the last alias type
                        if let Some(last_type) = self.type_registry.custom_types.keys().last() {
                            if let Some(custom_type) = self.type_registry.custom_types.get_mut(last_type) {
                                if custom_type.is_alias {
                                    custom_type.variants.push(variant);
                                }
                            }
                        }
                    }
                },
                // Look for function definitions to extract signatures
                CodeASTNode::FunctionDef { name, params, .. } => {
                    // Extract function signature
                    let mut parameters = Vec::new();
                    for param in params {
                        parameters.push(FunctionParameter {
                            name: param.clone(),
                            type_info: TypeInfo::Unknown,
                            description: None,
                            optional: false,
                        });
                    }
                    
                    let is_method = name.contains(':');
                    let signature = FunctionSignature {
                        name: name.clone(),
                        parameters,
                        return_types: Vec::new(),
                        description: None,
                        is_method,
                    };
                    
                    // If it's a method, add it to the appropriate class
                    if is_method {
                        let parts: Vec<&str> = name.split(':').collect();
                        if parts.len() >= 2 {
                            let class_name = parts[0].to_string();
                            let method_name = parts[1].to_string();
                            
                            if let Some(custom_type) = self.type_registry.custom_types.get_mut(&class_name) {
                                custom_type.methods.insert(method_name, signature);
                            }
                        }
                    } else {
                        // Otherwise add it as a standalone function
                        self.type_registry.function_signatures.insert(name.clone(), signature);
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Convert a type name string to a TypeInfo
    fn type_name_to_info(&self, type_name: &str) -> TypeInfo {
        match type_name {
            "string" => TypeInfo::String,
            "number" => TypeInfo::Number,
            "boolean" => TypeInfo::Boolean,
            "table" => TypeInfo::Table,
            "function" => TypeInfo::Function,
            _ => {
                // Check if it's a custom type we know about
                if self.type_registry.custom_types.contains_key(type_name) {
                    TypeInfo::Table  // Treat custom types as tables for now
                } else {
                    TypeInfo::Unknown
                }
            }
        }
    }
    
    /// Build dependency graph between modules
    pub fn build_dependency_graph(&mut self) {
        self.dependency_graph.clear();
        
        for (module_name, module_info) in &self.modules {
            for dependency in &module_info.dependencies {
                // Get or create entry for this dependency
                self.dependency_graph
                    .entry(dependency.required_path.clone())
                    .or_insert_with(HashSet::new)
                    .insert(module_name.clone());
            }
        }
    }

    pub fn add_module(&mut self, name: String, info: ModuleInfo) {
        self.modules.insert(name, info);
    }

    pub fn resolve_type(&self, name: &str) -> Option<TypeInfo> {
        // First check custom types
        if let Some(custom_type) = self.type_registry.custom_types.get(name) {
            // For simplicity, we just return a generic type for now
            // In the future, we could create a more specific TypeInfo for custom types
            return Some(TypeInfo::Table);
        }
        
        // Then check standard types
        self.type_registry.standard_types.get(name).cloned()
    }

    pub fn add_export(&mut self, module_name: &str, export: ExportItem) {
        self.modules
            .entry(module_name.to_string())
            .or_insert_with(|| ModuleInfo {
                exports: HashMap::new(),
                dependencies: Vec::new(),
                source_path: PathBuf::new(),
                is_main: false,
                processed: false,
            })
            .exports
            .insert(export.name.clone(), export);
    }
    
    /// Generate a type.lua file from observed types in the project
    pub fn generate_type_file(&self) -> Result<String, String> {
        if self.custom_types_count() == 0 {
            return Err("No custom types to generate".to_string());
        }
        
        let mut output = String::new();
        
        // Header
        output.push_str("--[[
  Project Type Definition File (Generated)

  This file defines custom types and structures for the project.
  It serves as both documentation and a source of type information
  for the lua_tools annotation system.
  
  Format version: 1.0
]]--\n\n");

        output.push_str("local Types = {}\n\n");
        
        // Classes/Tables
        output.push_str("-- =====================\n");
        output.push_str("-- Class/Table Definitions\n");
        output.push_str("-- =====================\n\n");
        
        for (name, custom_type) in &self.type_registry.custom_types {
            if !custom_type.is_alias {
                output.push_str(&format!("---@class {}\n", name));
                
                // Fields
                for field in &custom_type.fields {
                    let optional_marker = if field.optional { "?" } else { "" };
                    let type_name = self.type_name_for_info(&field.type_info);
                    let description = field.description.as_deref().unwrap_or("");
                    
                    output.push_str(&format!("---@field {}{} {} {}\n", 
                        field.name, optional_marker, type_name, description));
                }
                
                output.push_str(&format!("Types.{} = {{}}\n\n", name));
                
                // Methods
                for (method_name, method) in &custom_type.methods {
                    output.push_str(&self.format_function_signature(method, Some(name)));
                    output.push_str(&format!("function Types.{}:{}(", name, method_name));
                    
                    // Parameters
                    let params = method.parameters.iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    output.push_str(&format!("{}) end\n\n", params));
                }
            }
        }
        
        // Enums/Aliases
        output.push_str("-- =====================\n");
        output.push_str("-- Enum Definitions\n");
        output.push_str("-- =====================\n\n");
        
        for (name, custom_type) in &self.type_registry.custom_types {
            if custom_type.is_alias && !custom_type.variants.is_empty() {
                output.push_str(&format!("---@alias {}\n", name));
                
                for variant in &custom_type.variants {
                    output.push_str(&format!("---| '\"{}\"'\n", variant));
                }
                
                output.push_str(&format!("Types.{} = {{}}\n\n", name));
            }
        }
        
        // Function Signatures
        output.push_str("-- =====================\n");
        output.push_str("-- Function Signatures\n");
        output.push_str("-- =====================\n\n");
        
        for (_, function) in &self.type_registry.function_signatures {
            if !function.is_method {
                output.push_str(&self.format_function_signature(function, None));
                output.push_str(&format!("Types.{} = function(", function.name));
                
                // Parameters
                let params = function.parameters.iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                output.push_str(&format!("{}) end\n\n", params));
            }
        }
        
        output.push_str("return Types");
        
        Ok(output)
    }
    
    /// Count the number of custom types
    pub fn custom_types_count(&self) -> usize {
        self.type_registry.custom_types.len()
    }
    
    /// Format a function signature for the type file
    fn format_function_signature(&self, function: &FunctionSignature, class_name: Option<&str>) -> String {
        let mut output = String::new();
        
        // Description
        if let Some(desc) = &function.description {
            output.push_str(&format!("--- {}\n", desc));
        }
        
        // Parameters
        for param in &function.parameters {
            let optional_marker = if param.optional { "?" } else { "" };
            let type_name = self.type_name_for_info(&param.type_info);
            let description = param.description.as_deref().unwrap_or("");
            
            output.push_str(&format!("---@param {}{} {} {}\n", 
                param.name, optional_marker, type_name, description));
        }
        
        // Return types
        if !function.return_types.is_empty() {
            let return_types = function.return_types.iter()
                .map(|rt| self.type_name_for_info(rt))
                .collect::<Vec<_>>()
                .join(", ");
                
            output.push_str(&format!("---@return {}\n", return_types));
        }
        
        output
    }
    
    /// Get a string representation of a TypeInfo
    fn type_name_for_info(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String => "string".to_string(),
            TypeInfo::Number => "number".to_string(),
            TypeInfo::Boolean => "boolean".to_string(),
            TypeInfo::Table => "table".to_string(),
            TypeInfo::Function => "function".to_string(),
            TypeInfo::Unknown => "any".to_string(),
        }
    }
}
