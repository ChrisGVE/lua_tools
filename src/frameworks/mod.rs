// src/frameworks/mod.rs
//
// Framework registry for Lua tools - provides access to framework-specific
// type definitions and API information.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::project_context::{LuaVersion, ProjectContext};

/// Framework definition with version information
pub struct FrameworkVersion {
    /// Name of the framework
    pub name: String,
    /// Version identifier
    pub version: String,
    /// Lua version used by this framework version
    pub lua_version: LuaVersion,
    /// Brief description of the framework
    pub description: String,
    /// Path to the definition file
    pub definition_path: Option<PathBuf>,
    /// Frameworks this depends on
    pub dependencies: Vec<String>,
}

impl FrameworkVersion {
    pub fn new(name: &str, version: &str, lua_version: LuaVersion) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            lua_version,
            description: String::new(),
            definition_path: None,
            dependencies: Vec::new(),
        }
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    pub fn with_dependencies(mut self, dependencies: Vec<&str>) -> Self {
        self.dependencies = dependencies.iter().map(|d| d.to_string()).collect();
        self
    }
}

/// Registry for framework definitions
pub struct FrameworkRegistry {
    /// Available frameworks, indexed by name:version
    frameworks: HashMap<String, FrameworkVersion>,
    /// Framework versions by name
    versions: HashMap<String, Vec<String>>,
    /// Base directory for prepackaged framework definitions
    base_dir: PathBuf,
    /// User-specific framework directory
    user_dir: Option<PathBuf>,
    /// Project-specific framework directory
    project_dir: Option<PathBuf>,
}

impl FrameworkRegistry {
    /// Create a new framework registry
    pub fn new() -> Self {
        let mut registry = Self {
            frameworks: HashMap::new(),
            versions: HashMap::new(),
            base_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/frameworks"),
            user_dir: None,
            project_dir: None,
        };
        
        // Initialize with built-in frameworks
        registry.initialize_builtin_frameworks();
        registry.discover_frameworks();
        
        registry
    }
    
    /// Initialize the registry with built-in framework definitions
    fn initialize_builtin_frameworks(&mut self) {
        // Neovim
        self.register_framework(
            FrameworkVersion::new("neovim", "0.9.0", LuaVersion::Lua51)
                .with_description("Neovim API for version 0.9.x")
        );
        
        self.register_framework(
            FrameworkVersion::new("neovim", "0.10.0", LuaVersion::Lua51)
                .with_description("Neovim API for version 0.10.x")
        );
        
        self.register_framework(
            FrameworkVersion::new("neovim", "0.11.0", LuaVersion::Lua51)
                .with_description("Neovim API for version 0.11.x")
        );
        
        // WezTerm
        self.register_framework(
            FrameworkVersion::new("wezterm", "20230712", LuaVersion::Lua54)
                .with_description("WezTerm API (July 2023 release)")
        );
        
        self.register_framework(
            FrameworkVersion::new("wezterm", "20240222", LuaVersion::Lua54)
                .with_description("WezTerm API (February 2024 release)")
        );
        
        // LÖVE2D
        self.register_framework(
            FrameworkVersion::new("love2d", "11.4", LuaVersion::Lua53)
                .with_description("LÖVE2D API for version 11.4")
        );
        
        self.register_framework(
            FrameworkVersion::new("love2d", "11.5", LuaVersion::Lua53)
                .with_description("LÖVE2D API for version 11.5")
        );
        
        // Yazi
        self.register_framework(
            FrameworkVersion::new("yazi", "0.1.5", LuaVersion::Lua54)
                .with_description("Yazi file manager API")
        );
    }
    
    /// Register a framework version in the registry
    fn register_framework(&mut self, framework: FrameworkVersion) {
        let key = format!("{}:{}", framework.name, framework.version);
        
        // Update the versions list for this framework
        self.versions
            .entry(framework.name.clone())
            .or_insert_with(Vec::new)
            .push(framework.version.clone());
            
        // Add to the frameworks map
        self.frameworks.insert(key, framework);
    }
    
    /// Discover framework definitions in standard locations
    pub fn discover_frameworks(&mut self) {
        // Check built-in frameworks directory
        self.discover_in_directory(&self.base_dir);
        
        // Check user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let user_frameworks = config_dir.join("lua_tools/frameworks");
            if user_frameworks.exists() {
                self.user_dir = Some(user_frameworks.clone());
                self.discover_in_directory(&user_frameworks);
            }
        }
        
        // Project-specific directory is set when processing a project
    }
    
    /// Set the project-specific frameworks directory
    pub fn set_project_dir(&mut self, project_root: &Path) {
        let project_frameworks = project_root.join(".lua_tools/frameworks");
        if project_frameworks.exists() {
            self.project_dir = Some(project_frameworks.clone());
            self.discover_in_directory(&project_frameworks);
        }
    }
    
    /// Scan a directory for framework definitions
    fn discover_in_directory(&mut self, dir: &Path) {
        if !dir.exists() || !dir.is_dir() {
            return;
        }
        
        // Iterate through subdirectories (framework names)
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let framework_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // Skip if not a valid framework name
                    if framework_name.is_empty() || framework_name.starts_with('.') {
                        continue;
                    }
                    
                    // Scan this framework directory for version files
                    self.discover_framework_versions(&framework_name, &path);
                }
            }
        }
    }
    
    /// Scan a framework directory for version definition files
    fn discover_framework_versions(&mut self, framework_name: &str, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
                    // Extract version from filename (without extension)
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        // Try to determine Lua version from the file content
                        let lua_version = self.detect_lua_version_from_file(&path)
                            .unwrap_or(LuaVersion::Lua54); // Default to 5.4 if not specified
                        
                        // Create framework version
                        let mut framework = FrameworkVersion::new(
                            framework_name, 
                            filename, 
                            lua_version
                        );
                        
                        // Set the definition path
                        framework.definition_path = Some(path.clone());
                        
                        // Extract description from file if possible
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(desc) = extract_description_from_content(&content) {
                                framework.description = desc;
                            }
                        }
                        
                        // Register this framework version
                        self.register_framework(framework);
                    }
                }
            }
        }
    }
    
    /// Detect Lua version from a framework definition file
    fn detect_lua_version_from_file(&self, path: &Path) -> Option<LuaVersion> {
        if let Ok(content) = fs::read_to_string(path) {
            // Look for Lua version marker in the content
            if content.contains("lua_version") || content.contains("LUA_VERSION") {
                if content.contains("\"5.1\"") || content.contains("'5.1'") {
                    return Some(LuaVersion::Lua51);
                } else if content.contains("\"5.2\"") || content.contains("'5.2'") {
                    return Some(LuaVersion::Lua52);
                } else if content.contains("\"5.3\"") || content.contains("'5.3'") {
                    return Some(LuaVersion::Lua53);
                } else if content.contains("\"5.4\"") || content.contains("'5.4'") {
                    return Some(LuaVersion::Lua54);
                }
            }
        }
        None
    }
    
    /// Get all available framework names
    pub fn get_framework_names(&self) -> Vec<String> {
        self.versions.keys().cloned().collect()
    }
    
    /// Get all available versions for a framework
    pub fn get_framework_versions(&self, name: &str) -> Vec<String> {
        self.versions.get(name)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get the latest version for a framework
    pub fn get_latest_version(&self, name: &str) -> Option<String> {
        let versions = self.get_framework_versions(name);
        if versions.is_empty() {
            return None;
        }
        
        // Simple version comparison (assumes semantic versioning or similar)
        // A more robust solution would use a proper version parsing library
        let mut latest = versions[0].clone();
        for version in &versions[1..] {
            if version_is_newer(version, &latest) {
                latest = version.clone();
            }
        }
        
        Some(latest)
    }
    
    /// Get a framework definition by name and version
    pub fn get_framework(&self, name: &str, version: &str) -> Option<&FrameworkVersion> {
        let key = format!("{}:{}", name, version);
        self.frameworks.get(&key)
    }
    
    /// Get the latest version of a framework
    pub fn get_latest_framework(&self, name: &str) -> Option<&FrameworkVersion> {
        let version = self.get_latest_version(name)?;
        self.get_framework(name, &version)
    }
    
    /// Read a framework definition content
    pub fn read_framework_definition(&self, name: &str, version: &str) -> Option<String> {
        // Get the framework
        let framework = self.get_framework(name, version)?;
        
        // Get the definition path
        let definition_path = match &framework.definition_path {
            Some(path) => path.clone(),
            None => {
                // Check built-in definition
                let filename = format!("{}.lua", version);
                self.base_dir.join(name).join(filename)
            }
        };
        
        // Read the file
        fs::read_to_string(definition_path).ok()
    }
    
    /// Detect if a directory is using a specific framework
    pub fn detect_framework_usage(&self, dir: &Path) -> Vec<(String, Option<String>)> {
        let mut results = Vec::new();
        
        // Check for Neovim
        if self.is_neovim_project(dir) {
            let version = self.detect_neovim_version(dir);
            results.push(("neovim".to_string(), version));
        }
        
        // Check for WezTerm
        if self.is_wezterm_project(dir) {
            let version = self.detect_wezterm_version(dir);
            results.push(("wezterm".to_string(), version));
        }
        
        // Check for LÖVE2D
        if self.is_love2d_project(dir) {
            let version = self.detect_love2d_version(dir);
            results.push(("love2d".to_string(), version));
        }
        
        // Check for Yazi
        if self.is_yazi_project(dir) {
            let version = self.detect_yazi_version(dir);
            results.push(("yazi".to_string(), version));
        }
        
        // Attempt to detect additional frameworks from dependencies
        self.detect_framework_from_dependencies(dir, &mut results);
        
        results
    }
    
    /// Detect frameworks by analyzing dependencies and require statements
    fn detect_framework_from_dependencies(&self, dir: &Path, results: &mut Vec<(String, Option<String>)>) {
        // Skip if directory doesn't exist
        if !dir.exists() || !dir.is_dir() {
            return;
        }
        
        // Scan Lua files for require statements
        let mut frameworks_detected = std::collections::HashSet::new();
        let max_files_to_scan = 20;
        let mut scanned = 0;
        
        // Walk the directory recursively
        let walker = walkdir::WalkDir::new(dir)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| {
                let path = e.path();
                // Skip hidden directories, node_modules, etc.
                !path.to_string_lossy().contains("node_modules") &&
                !path.to_string_lossy().contains("/.git/") &&
                !path.file_name().map_or(false, |n| n.to_string_lossy().starts_with('.'))
            });
        
        for entry in walker.filter_map(|e| e.ok()) {
            if scanned >= max_files_to_scan {
                break;
            }
            
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("lua") {
                continue;
            }
            
            // Read file content
            if let Ok(content) = fs::read_to_string(path) {
                scanned += 1;
                
                // Check for require patterns
                if self.scan_for_framework_imports(&content, &mut frameworks_detected) {
                    // If we found definitive framework imports, we can stop scanning
                    break;
                }
            }
        }
        
        // Add detected frameworks to results
        for framework in frameworks_detected {
            // Avoid duplicates
            if !results.iter().any(|(name, _)| name == &framework) {
                let version = match framework.as_str() {
                    "neovim" => self.detect_neovim_version(dir),
                    "wezterm" => self.detect_wezterm_version(dir),
                    "love2d" => self.detect_love2d_version(dir),
                    "yazi" => self.detect_yazi_version(dir),
                    _ => self.get_latest_version(&framework),
                };
                results.push((framework, version));
            }
        }
    }
    
    /// Scan file content for framework imports and require statements
    fn scan_for_framework_imports(&self, content: &str, detected: &mut std::collections::HashSet<String>) -> bool {
        // Framework-specific modules and patterns
        let framework_patterns = [
            // Neovim
            ("require'nvim", "neovim"),
            ("require 'nvim", "neovim"),
            ("require(\"nvim", "neovim"),
            ("require('nvim", "neovim"),
            ("vim.api.", "neovim"),
            ("vim.fn.", "neovim"),
            ("vim.cmd", "neovim"),
            ("vim.keymap", "neovim"),
            ("vim.ui.", "neovim"),
            ("vim.undo.", "neovim"),
            
            // WezTerm
            ("require'wezterm", "wezterm"),
            ("require 'wezterm", "wezterm"),
            ("require(\"wezterm", "wezterm"),
            ("require('wezterm", "wezterm"),
            ("wezterm.action", "wezterm"),
            ("wezterm.format", "wezterm"),
            
            // LÖVE2D
            ("love.graphics", "love2d"),
            ("love.audio", "love2d"),
            ("love.event", "love2d"),
            ("love.load", "love2d"),
            ("love.update", "love2d"),
            ("love.draw", "love2d"),
            
            // Yazi
            ("require'yazi", "yazi"),
            ("require 'yazi", "yazi"),
            ("require(\"yazi", "yazi"),
            ("require('yazi", "yazi"),
            ("ya.manager", "yazi"),
            ("ya.preview", "yazi")
        ];
        
        let lines = content.lines().collect::<Vec<_>>();
        
        // First check if we have a definitive framework marker
        for line in &lines {
            for (pattern, framework) in &framework_patterns {
                if line.contains(pattern) {
                    detected.insert(framework.to_string());
                    return true; // Found definitive marker
                }
            }
        }
        
        // Look for module structures that indicate frameworks
        if content.contains("function love.") {
            detected.insert("love2d".to_string());
            return true;
        }
        
        return false;
    }
    
    /// Check if a directory is a Neovim plugin project
    fn is_neovim_project(&self, dir: &Path) -> bool {
        // Common Neovim plugin structure
        dir.join("lua").exists() && (
            dir.join("plugin").exists() || 
            dir.join("doc").exists() || 
            dir.join("after").exists() || 
            dir.join("ftplugin").exists() ||
            dir.join("autoload").exists()
        )
    }
    
    /// Try to detect Neovim version from project files
    fn detect_neovim_version(&self, dir: &Path) -> Option<String> {
        // Check for explicit version in documentation
        let doc_files = vec!["README.md", "doc/help.txt", "doc/plugin.txt", "plugin/plugin.lua", "plugin/init.lua"];
        for file in doc_files {
            let file_path = dir.join(file);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for version requirements with more patterns
                    if content.contains("requires Neovim 0.11") || 
                       content.contains("requires nvim 0.11") || 
                       content.contains("requires \"neovim >= 0.11") || 
                       content.contains("requires 'neovim >= 0.11") ||
                       content.contains("neovim >= 0.11") ||
                       content.contains("nvim >= 0.11") {
                        return Some("0.11.0".to_string());
                    } else if content.contains("requires Neovim 0.10") || 
                       content.contains("requires nvim 0.10") || 
                       content.contains("requires \"neovim >= 0.10") || 
                       content.contains("requires 'neovim >= 0.10") ||
                       content.contains("neovim >= 0.10") ||
                       content.contains("nvim >= 0.10") {
                        return Some("0.10.0".to_string());
                    } else if content.contains("requires Neovim 0.9") || 
                            content.contains("requires nvim 0.9") ||
                            content.contains("requires \"neovim >= 0.9") ||
                            content.contains("requires 'neovim >= 0.9") ||
                            content.contains("neovim >= 0.9") ||
                            content.contains("nvim >= 0.9") {
                        return Some("0.9.0".to_string());
                    } else if content.contains("requires Neovim 0.8") || 
                            content.contains("requires nvim 0.8") ||
                            content.contains("neovim >= 0.8") ||
                            content.contains("nvim >= 0.8") {
                        // Though we don't have 0.8 definitions, fallback to 0.9
                        return Some("0.9.0".to_string());
                    }
                }
            }
        }
        
        // Check package.json for plugin requirements (for TypeScript/Node.js based plugins)
        let package_json = dir.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if content.contains("\"engines\"") && content.contains("\"nvim\"") {
                    if content.contains("\"nvim\": \">=0.11") || content.contains("\"nvim\":{") && content.contains("\">=0.11") {
                        return Some("0.11.0".to_string());
                    } else if content.contains("\"nvim\": \">=0.10") || content.contains("\"nvim\":{") && content.contains("\">=0.10") {
                        return Some("0.10.0".to_string());
                    } else if content.contains("\"nvim\": \">=0.9") || content.contains("\"nvim\":{") && content.contains("\">=0.9") {
                        return Some("0.9.0".to_string());
                    }
                }
            }
        }
        
        // Check for rockspec file
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rockspec") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("dependencies") {
                        if content.contains("nvim >= 0.11") || content.contains("neovim >= 0.11") {
                            return Some("0.11.0".to_string());
                        } else if content.contains("nvim >= 0.10") || content.contains("neovim >= 0.10") {
                            return Some("0.10.0".to_string());
                        } else if content.contains("nvim >= 0.9") || content.contains("neovim >= 0.9") {
                            return Some("0.9.0".to_string());
                        }
                        }
                    }
                }
            }
        }
        
        // More sophisticated API detection from Lua files
        if let Some(lua_dir) = self.find_lua_dir(dir) {
            // Check for 0.11-specific APIs
            let neovim_0_11_apis = vec![
                "vim.api.nvim_ui_attach_ext",
                "vim.ui.select",
                "vim.ui.input",
                "vim.keymap.set",
                "vim.undo.",
                "vim.api.nvim_get_namespaces"
            ];
            
            if self.scan_for_neovim_api_usage(&lua_dir, neovim_0_11_apis) {
                return Some("0.11.0".to_string());
            }
            
            // Check for 0.10-specific APIs
            let neovim_0_10_apis = vec![
                "vim.version", 
                "vim.api.nvim_create_autocmd",
                "vim.fs.",
                "vim.system(",
                "vim.iter(",
                "vim.print(",
                "vim.json."
            ];
            
            if self.scan_for_neovim_api_usage(&lua_dir, neovim_0_10_apis) {
                return Some("0.10.0".to_string());
            }
            
            // Check for 0.9-specific APIs that aren't in 0.8
            let neovim_0_9_apis = vec![
                "vim.api.nvim_create_autocmd",
                "vim.api.nvim_set_hl",
                "vim.api.nvim_get_hl",
                "vim.diagnostic.",
                "vim.uv."
            ];
            
            if self.scan_for_neovim_api_usage(&lua_dir, neovim_0_9_apis) {
                return Some("0.9.0".to_string());
            }
        }
        
        // Check plugin version naming
        // Some plugins have version numbers in their releases or filenames related to Neovim versions
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && (path.extension().and_then(|e| e.to_str()) == Some("lua") || 
                                     path.extension().and_then(|e| e.to_str()) == Some("md")) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if filename.contains("nvim-0.11") || filename.contains("nvim_0.11") || 
                       filename.contains("neovim-0.11") || filename.contains("neovim_0.11") {
                        return Some("0.11.0".to_string());
                    } else if filename.contains("nvim-0.10") || filename.contains("nvim_0.10") || 
                       filename.contains("neovim-0.10") || filename.contains("neovim_0.10") {
                        return Some("0.10.0".to_string());
                    } else if filename.contains("nvim-0.9") || filename.contains("nvim_0.9") || 
                              filename.contains("neovim-0.9") || filename.contains("neovim_0.9") {
                        return Some("0.9.0".to_string());
                    }
                }
            }
        }
        
        // Default to latest version
        self.get_latest_version("neovim")
    }
    
    /// Find the main Lua directory in a project
    fn find_lua_dir(&self, dir: &Path) -> Option<PathBuf> {
        let lua_dir = dir.join("lua");
        if lua_dir.exists() && lua_dir.is_dir() {
            return Some(lua_dir);
        }
        None
    }
    
    /// Scan Lua files in a directory for specific API usage
    fn scan_for_neovim_api_usage(&self, dir: &Path, patterns: Vec<&str>) -> bool {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively scan subdirectories
                    if self.scan_for_neovim_api_usage(&path, patterns.clone()) {
                        return true;
                    }
                } else if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
                    // Check file content for patterns
                    if let Ok(content) = fs::read_to_string(&path) {
                        for pattern in &patterns {
                            if content.contains(pattern) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    
    /// Check if a directory is a WezTerm configuration project
    fn is_wezterm_project(&self, dir: &Path) -> bool {
        dir.join("wezterm.lua").exists() || dir.join(".wezterm.lua").exists()
    }
    
    /// Try to detect WezTerm version from project files
    fn detect_wezterm_version(&self, dir: &Path) -> Option<String> {
        let config_files = vec!["wezterm.lua", ".wezterm.lua"];
        for file in config_files {
            let file_path = dir.join(file);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for direct version specifications
                    if content.contains("-- WezTerm version: 20240222") || 
                       content.contains("-- wezterm_version = \"20240222") || 
                       content.contains("-- wezterm version: 20240222") {
                        return Some("20240222".to_string());
                    } else if content.contains("-- WezTerm version: 20230712") || 
                              content.contains("-- wezterm_version = \"20230712") || 
                              content.contains("-- wezterm version: 20230712") {
                        return Some("20230712".to_string());
                    }
                    
                    // Look for version-specific features
                    if content.contains("wezterm.mux") && content.contains("wezterm.gui") {
                        // Features introduced in 20240222
                        return Some("20240222".to_string());
                    }
                    
                    // Check for other features specific to 20240222
                    if content.contains("wezterm.color.parse") || 
                       content.contains("wezterm.color.gradient") || 
                       content.contains("wezterm.procinfo") || 
                       content.contains("background_blur_radius") {
                        return Some("20240222".to_string());
                    }
                    
                    // Check for APIs that existed in 20230712
                    if content.contains("wezterm.action") || 
                       content.contains("wezterm.format") {
                        // These features existed in 20230712
                        // but we only return this if we haven't already identified newer features
                        return Some("20230712".to_string());
                    }
                }
            }
        }
        
        // Check if there are any comments mentioning specific WezTerm versions in any Lua files
        let mut max_files = 5;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        max_files -= 1;
                        if content.contains("WezTerm 20240222") || content.contains("wezterm 20240222") {
                            return Some("20240222".to_string());
                        } else if content.contains("WezTerm 20230712") || content.contains("wezterm 20230712") {
                            return Some("20230712".to_string());
                        }
                        
                        if max_files <= 0 {
                            break;
                        }
                    }
                }
            }
        }
        
        // Default to latest version
        self.get_latest_version("wezterm")
    }
    
    /// Check if a directory is a LÖVE2D project
    fn is_love2d_project(&self, dir: &Path) -> bool {
        dir.join("main.lua").exists() && dir.join("conf.lua").exists()
    }
    
    /// Try to detect LÖVE2D version from project files
    fn detect_love2d_version(&self, dir: &Path) -> Option<String> {
        // Check conf.lua for explicit version
        let conf_path = dir.join("conf.lua");
        if conf_path.exists() {
            if let Ok(content) = fs::read_to_string(&conf_path) {
                // Look for LÖVE version in configuration with more patterns
                if content.contains("t.version = \"11.5") || 
                   content.contains("t.version = '11.5") {
                    return Some("11.5".to_string());
                } else if content.contains("t.version = \"11.4") || 
                           content.contains("t.version = '11.4") {
                    return Some("11.4".to_string());
                } else if content.contains("t.version = \"11.3") || 
                           content.contains("t.version = '11.3") {
                    // Fall back to 11.4 for 11.3 (which we don't have specific support for)
                    return Some("11.4".to_string());
                } else if content.contains("t.version = \"11.") || 
                           content.contains("t.version = '11.") {
                    return Some("11.4".to_string()); // Default for LÖVE 11.x
                }
                
                // Check for feature requirements mentioned in comments
                let lines: Vec<&str> = content.lines().collect();
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with("--") {
                        if trimmed.contains("LÖVE 11.5") || trimmed.contains("LOVE 11.5") {
                            return Some("11.5".to_string());
                        } else if trimmed.contains("LÖVE 11.4") || trimmed.contains("LOVE 11.4") {
                            return Some("11.4".to_string());
                        }
                    }
                }
            }
        }
        
        // Check main.lua for version hints
        let main_path = dir.join("main.lua");
        if main_path.exists() {
            if let Ok(content) = fs::read_to_string(&main_path) {
                // Check for features specific to LÖVE 11.5
                if content.contains("love.graphics.stencil(") || 
                   content.contains("love.graphics.getTextureTypes(") || 
                   content.contains("love.graphics.getRendererInfo(") {
                    return Some("11.5".to_string());
                }
                
                // Check for comments specifying version
                let lines: Vec<&str> = content.lines().collect();
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with("--") {
                        if trimmed.contains("LÖVE 11.5") || trimmed.contains("LOVE 11.5") {
                            return Some("11.5".to_string());
                        } else if trimmed.contains("LÖVE 11.4") || trimmed.contains("LOVE 11.4") {
                            return Some("11.4".to_string());
                        }
                    }
                }
            }
        }
        
        // Check README or other documentation
        let doc_files = vec!["README.md", "README.txt", "readme.md", "readme.txt", "docs/README.md"];
        for file in doc_files {
            let doc_path = dir.join(file);
            if doc_path.exists() {
                if let Ok(content) = fs::read_to_string(&doc_path) {
                    if content.contains("LÖVE 11.5") || content.contains("LOVE 11.5") {
                        return Some("11.5".to_string());
                    } else if content.contains("LÖVE 11.4") || content.contains("LOVE 11.4") {
                        return Some("11.4".to_string());
                    }
                }
            }
        }
        
        // Check .github/workflows for CI configurations that might specify version
        let workflow_dir = dir.join(".github/workflows");
        if workflow_dir.exists() {
            if let Ok(entries) = fs::read_dir(workflow_dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("love-11.5") || content.contains("love:11.5") {
                            return Some("11.5".to_string());
                        } else if content.contains("love-11.4") || content.contains("love:11.4") {
                            return Some("11.4".to_string());
                        }
                    }
                }
            }
        }
        
        // Default to latest version
        self.get_latest_version("love2d")
    }
    
    /// Check if a directory is a Yazi configuration project
    fn is_yazi_project(&self, dir: &Path) -> bool {
        dir.join("yazi").exists() && dir.join("yazi").join("init.lua").exists()
    }
    
    /// Try to detect Yazi version from project files
    fn detect_yazi_version(&self, dir: &Path) -> Option<String> {
        // Check for Yazi configuration files
        let config_files = vec![
            "yazi/init.lua", 
            "yazi/keymap.lua", 
            "yazi/theme.lua", 
            "yazi/config.lua"
        ];
        
        // Scan config files for version-specific features or comments
        for file in config_files {
            let file_path = dir.join(file);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Check for explicit version markers
                    if content.contains("-- Yazi version: 0.1.5") || 
                       content.contains("-- yazi_version = \"0.1.5") {
                        return Some("0.1.5".to_string());
                    }
                    
                    // Look for 0.1.5 features
                    if content.contains("ya.manager.select_by") || 
                       content.contains("ya.preview.archive") {
                        return Some("0.1.5".to_string());
                    }
                }
            }
        }
        
        // Check for version in README
        let readme_files = vec!["README.md", "readme.md", "README.txt", "readme.txt"];
        for file in readme_files {
            let file_path = dir.join(file);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if content.contains("Yazi 0.1.5") {
                        return Some("0.1.5".to_string());
                    }
                }
            }
        }
        
        // Default to latest version as Yazi is relatively new
        self.get_latest_version("yazi")
    }
    
    /// Apply a framework's type definitions to a project context
    pub fn apply_framework_to_context(&self, context: &mut ProjectContext, name: &str, version: &str) -> bool {
        // Get the framework definition
        let definition = match self.read_framework_definition(name, version) {
            Some(content) => content,
            None => return false,
        };
        
        // Create a temporary file with the definition
        let temp_dir = tempfile::tempdir().ok()?;
        let temp_file = temp_dir.path().join(format!("{}.lua", name));
        
        if fs::write(&temp_file, definition).is_err() {
            return false;
        }
        
        // Process the definition file using the project context
        context.process_single_type_file(&temp_file).is_ok()
    }
}

/// Extract description from framework definition file content
fn extract_description_from_content(content: &str) -> Option<String> {
    // Look for description in header comment
    if let Some(header_end) = content.find("]]--") {
        let header = &content[..header_end];
        if let Some(desc_line) = header.lines().find(|line| 
            line.trim().starts_with("This") || 
            line.trim().starts_with("A ") || 
            line.trim().starts_with("Provides") ||
            line.trim().contains("API")
        ) {
            return Some(desc_line.trim().to_string());
        }
    }
    None
}

/// Compare two version strings to determine which is newer
/// This is a more robust implementation that handles various versioning schemes
fn version_is_newer(version1: &str, version2: &str) -> bool {
    // Special case for date-based versions (like WezTerm's YYYYMMDD format)
    if version1.len() == 8 && version2.len() == 8 && 
       version1.chars().all(|c| c.is_digit(10)) && 
       version2.chars().all(|c| c.is_digit(10)) {
        return version1 > version2;
    }
    
    // Handle semver with prefixes (v1.2.3)
    let clean_v1 = version1.trim_start_matches('v');
    let clean_v2 = version2.trim_start_matches('v');
    
    // Try different separators (., _, -, etc.)
    let separators = ['.', '-', '_', ' '];
    let mut components1 = Vec::new();
    let mut components2 = Vec::new();
    
    // Find the first separator that produces a valid split
    for &sep in &separators {
        let split1: Vec<&str> = clean_v1.split(sep).collect();
        let split2: Vec<&str> = clean_v2.split(sep).collect();
        
        if split1.len() > 1 || split2.len() > 1 {
            components1 = split1;
            components2 = split2;
            break;
        }
    }
    
    // If no separator worked, treat as single components
    if components1.is_empty() {
        components1 = vec![clean_v1];
        components2 = vec![clean_v2];
    }
    
    // Compare components
    for (i, c1) in components1.iter().enumerate() {
        // If we've run out of components in version2, version1 is newer
        // But only if the additional component is not "0" (1.2.3 > 1.2)
        if i >= components2.len() {
            return c1 != &"0";
        }
        
        let c2 = components2[i];
        
        // Try to parse components as integers for numeric comparison
        match (c1.parse::<u64>(), c2.parse::<u64>()) {
            (Ok(n1), Ok(n2)) => {
                if n1 > n2 {
                    return true;
                } else if n1 < n2 {
                    return false;
                }
                // If equal, continue to the next component
            },
            _ => {
                // Special handling for prerelease suffixes
                if c1.starts_with(|c: char| c.is_digit(10)) && !c2.starts_with(|c: char| c.is_digit(10)) {
                    // Numeric is newer than alpha/beta/etc (1.0 > 1.0-beta)
                    return true;
                } else if !c1.starts_with(|c: char| c.is_digit(10)) && c2.starts_with(|c: char| c.is_digit(10)) {
                    // Alpha/beta/etc is older than numeric (1.0-beta < 1.0)
                    return false;
                }
                
                // Prerelease order: dev < alpha < beta < rc < (nothing)
                let prerelease_order = |s: &str| -> u8 {
                    let lower = s.to_lowercase();
                    if lower.contains("dev") { 1 }
                    else if lower.contains("alpha") { 2 } 
                    else if lower.contains("beta") { 3 }
                    else if lower.contains("rc") { 4 }
                    else { 5 }
                };
                
                let order1 = prerelease_order(c1);
                let order2 = prerelease_order(c2);
                
                if order1 != order2 {
                    return order1 > order2;
                }
                
                // If all else fails, compare as strings
                if c1 > c2 {
                    return true;
                } else if c1 < c2 {
                    return false;
                }
            }
        }
    }
    
    // If we've exhausted version1's components and version2 has more, it might be newer
    // But only if those extra components aren't all zeros (1.2 == 1.2.0)
    if components1.len() < components2.len() {
        return !components2[components1.len()..].iter().all(|c| c == &"0");
    }
    
    // If all components are equal, the versions are equal
    false
}

/// Create a framework definition file with specified version
pub fn create_framework_template(name: &str, version: &str, lua_version: LuaVersion) -> Option<String> {
    match name.to_lowercase().as_str() {
        "neovim" => Some(create_neovim_template(version, lua_version)),
        "wezterm" => Some(create_wezterm_template(version, lua_version)),
        "love2d" => Some(create_love2d_template(version, lua_version)),
        "yazi" => Some(create_yazi_template(version, lua_version)),
        _ => None,
    }
}

/// Create a Neovim framework definition template
fn create_neovim_template(version: &str, _lua_version: LuaVersion) -> String {
    format!(r#"--[[
  Neovim API Type Definitions

  This file provides type definitions for the Neovim API.
  It enhances type checking and auto-completion for Neovim plugin development.
  
  Neovim version: {}
  lua_version = "5.1" -- Neovim uses Lua 5.1
]]--

local Neovim = {{}}

-- =====================
-- Core API Types
-- =====================

---@class Buffer Represents a Neovim buffer
---@field id number Buffer handle ID
---@field name string Buffer name
---@field lines table List of lines in the buffer
---@field options table Buffer-local options
Neovim.Buffer = {{}}

---@class Window Represents a Neovim window
---@field id number Window handle ID
---@field buffer Buffer The buffer displayed in this window
---@field height number Window height
---@field width number Window width
---@field options table Window-local options
Neovim.Window = {{}}

---@class Tabpage Represents a Neovim tabpage
---@field id number Tabpage handle ID
---@field windows table List of windows in this tabpage
Neovim.Tabpage = {{}}

-- =====================
-- API Functions
-- =====================

-- Buffer Operations

--- Gets current buffer
---@return Buffer
Neovim.api.get_current_buf = function() end

--- Gets buffer line
---@param buffer Buffer Buffer handle
---@param index number Line index (0-based)
---@return string
Neovim.api.buf_get_line = function(buffer, index) end

--- Sets buffer line
---@param buffer Buffer Buffer handle
---@param index number Line index (0-based)
---@param line string New line content
---@return boolean
Neovim.api.buf_set_line = function(buffer, index, line) end

-- Command Operations

--- Execute a Vim command
---@param command string Command to execute
Neovim.api.command = function(command) end

return Neovim
"#, version)
}

/// Create a WezTerm framework definition template
fn create_wezterm_template(version: &str, _lua_version: LuaVersion) -> String {
    format!(r#"--[[
  WezTerm API Type Definitions

  This file provides type definitions for the WezTerm terminal emulator.
  It enhances type checking and auto-completion for WezTerm configuration.
  
  WezTerm version: {}
  lua_version = "5.4" -- WezTerm uses Lua 5.4
]]--

local wezterm = {{}}

-- =====================
-- Core Types
-- =====================

---@class Config The WezTerm configuration object
---@field color_scheme string Name of the color scheme
---@field font string Font to use
---@field font_size number Size of the font
---@field window_background_opacity number Background opacity (0.0-1.0)
wezterm.Config = {{}}

---@class Window A WezTerm window
---@field active_tab Tab The active tab
---@field tabs Tab[] List of tabs
---@field set_title fun(title: string) Set the window title
wezterm.Window = {{}}

---@class Tab A tab in WezTerm
---@field active_pane Pane The active pane
---@field panes Pane[] List of panes
---@field set_title fun(title: string) Set the tab title
wezterm.Tab = {{}}

---@class Pane A pane in WezTerm
---@field title string The pane title
---@field send_text fun(text: string) Send text to the pane
---@field split fun(direction: string): Pane Split the pane
wezterm.Pane = {{}}

-- =====================
-- API Functions
-- =====================

--- Load the configuration
---@return Config
wezterm.load_config = function() end

--- Override the configuration with provided values
---@param config Config The configuration to override
---@return Config
wezterm.config_builder = function(config) end

--- Return the default configuration
---@return Config
wezterm.default_config = function() end

return wezterm
"#, version)
}

/// Create a LÖVE2D framework definition template
fn create_love2d_template(version: &str, _lua_version: LuaVersion) -> String {
    format!(r#"--[[
  LÖVE2D Framework Type Definitions

  This file provides type definitions for the LÖVE 2D game framework.
  It enhances type checking and auto-completion for LÖVE game development.
  
  LÖVE version: {}
  lua_version = "5.3" -- LÖVE 11+ uses Lua 5.3
]]--

local love = {{}}

-- =====================
-- Core Types
-- =====================

---@class Image Represents a drawable image
---@field getWidth fun(): number Get the width of the image
---@field getHeight fun(): number Get the height of the image
---@field getDimensions fun(): number, number Get the dimensions of the image
love.Image = {{}}

---@class Quad Represents a quadrilateral with texture coordinates
---@field getViewport fun(): number, number, number, number Get the viewport of the quad
---@field setViewport fun(x: number, y: number, width: number, height: number) Set the viewport of the quad
love.Quad = {{}}

---@class Font Represents a font object for rendering text
---@field getWidth fun(text: string): number Get the width of the text when rendered with this font
---@field getHeight fun(): number Get the height of the font
---@field getAscent fun(): number Get the ascent of the font
love.Font = {{}}

---@class Canvas Represents a canvas for offscreen rendering
---@field getWidth fun(): number Get the width of the canvas
---@field getHeight fun(): number Get the height of the canvas
---@field renderTo fun(callback: function) Render to the canvas
love.Canvas = {{}}

-- =====================
-- Modules
-- =====================

-- Graphics Module

---@class GraphicsModule
---@field newImage fun(filename: string): Image Create a new image
---@field newQuad fun(x: number, y: number, width: number, height: number, iw: number, ih: number): Quad Create a new quad
---@field newFont fun(filename: string, size: number): Font Create a new font
---@field newCanvas fun(width: number, height: number): Canvas Create a new canvas
---@field print fun(text: string, x: number, y: number) Print text
---@field rectangle fun(mode: string, x: number, y: number, width: number, height: number) Draw a rectangle
---@field circle fun(mode: string, x: number, y: number, radius: number) Draw a circle
love.graphics = {{}}

-- Audio Module

---@class AudioModule
---@field newSource fun(filename: string, type: string): Source Create a new audio source
---@field play fun(source: Source) Play an audio source
---@field stop fun(source: Source) Stop an audio source
---@field pause fun(source: Source) Pause an audio source
love.audio = {{}}

-- Input Module

---@class KeyboardModule
---@field isDown fun(key: string): boolean Check if a key is down
---@field isScancodeDown fun(scancode: string): boolean Check if a scancode is down
---@field setKeyRepeat fun(enable: boolean) Enable or disable key repeat
love.keyboard = {{}}

---@class MouseModule
---@field getPosition fun(): number, number Get the position of the mouse
---@field isDown fun(button: number): boolean Check if a mouse button is down
---@field setVisible fun(visible: boolean) Set the visibility of the mouse cursor
love.mouse = {{}}

return love
"#, version)
}

/// Create a Yazi framework definition template
fn create_yazi_template(version: &str, _lua_version: LuaVersion) -> String {
    format!(r#"--[[
  Yazi File Manager Type Definitions

  This file provides type definitions for the Yazi file manager.
  It enhances type checking and auto-completion for Yazi customization.
  
  Yazi version: {}
  lua_version = "5.4" -- Yazi uses Lua 5.4
]]--

local yazi = {{}}

-- =====================
-- Core Types
-- =====================

---@class File Represents a file in Yazi
---@field name string The name of the file
---@field path string The path to the file
---@field size number The size of the file in bytes
---@field mimetype string The MIME type of the file
---@field is_dir boolean Whether the file is a directory
yazi.File = {{}}

---@class Manager The file manager
---@field files File[] The list of files in the current directory
---@field current_file File The currently selected file
---@field cd fun(path: string) Change the current directory
---@field select fun(index: number) Select a file by index
---@field copy fun(files: File[]) Copy files
---@field cut fun(files: File[]) Cut files
---@field paste fun() Paste files
---@field delete fun(files: File[]) Delete files
yazi.Manager = {{}}

---@class Input Input handling
---@field bind fun(key: string, mode: string, action: function) Bind a key to an action
---@field send fun(key: string) Send a key event
---@field unbind fun(key: string, mode: string) Unbind a key
yazi.Input = {{}}

-- =====================
-- API Functions
-- =====================

--- Get the current manager
---@return Manager
yazi.manager = function() end

--- Show a notification
---@param message string The message to show
---@param level string The level of the notification (info, warn, error)
yazi.notify = function(message, level) end

--- Run a shell command
---@param command string The command to run
---@param callback function The callback to run when the command completes
yazi.run = function(command, callback) end

return yazi
"#, version)
}