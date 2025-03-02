# Lua Tools - CLI Utilities for Lua Code Annotation and Header Generation

## Purpose

Lua Tools is a set of command-line utilities designed to assist developers working with Lua by:

1. **lua_commenter**: Automatically adding comments and Lua LSP annotations to functions and classes in Lua source files without altering code structure or formatting.
2. **lua_header**: Extracting and generating Lua header files containing public function signatures and relevant documentation.

These tools enhance code maintainability and provide better integration with Lua Language Server features.

---

## User Instructions

### **Installation & Setup**

To build the project, run:

```sh
cargo build --release
```

This will generate two executables:

- `target/release/lua_commenter`
- `target/release/lua_header`

To make them globally accessible, copy them to a directory in your system's `$PATH`.

```sh
cp target/release/lua_commenter target/release/lua_header /usr/local/bin/
```

### **Usage**

#### **1. lua_commenter** - Adds comments and Lua LSP annotations to Lua source files.

```sh
lua_commenter [options] <input-files>
```

##### **Options:**

- `-o, --output <pattern>` → Define output filename pattern (e.g., `annotated_{}` for `file.lua` → `annotated_file.lua`).
- `-w, --overwrite` → Modify files in-place instead of creating new ones.
- `-r, --recursive` → Recursively process all `.lua` files in the specified directory.

##### **Example:**

```sh
lua_commenter -o annotated_{} examples/example.lua
```

This will generate `annotated_example.lua` with added comments.

#### **2. lua_header** - Extracts public API definitions from a Lua module.

```sh
lua_header [options] <input-files>
```

##### **Options:**

- `-r, --recursive` → Recursively process all `.lua` files in the specified directory.

##### **Example:**

```sh
lua_header examples/example.lua
```

This generates `example.header.lua`, containing only public function signatures with comments.

---

## Design Principles

### **1. Non-intrusive Code Annotation**

- The tool does not alter the structure, formatting, or logic of the Lua code.
- It only **adds missing comments and annotations** without overwriting existing ones.

### **2. Lua LSP Annotation Support**

- Uses `---@function`, `---@param`, and `---@return` annotations.
- Omits `---@param` if no parameters exist.
- Avoids unnecessary `---@return` annotations for functions without a return value.

### **3. Heuristics Used**

#### **Function Annotation Heuristics**

- Function signatures are detected via regex.
- If a function has parameters, `---@param` annotations are added.
- If a function returns a table (`return {}`), it is inferred as `---@return table`.

#### **Class & Object Detection Heuristics**

- Detects Lua objects declared as `MyClass = {}` and annotates them as `---@class MyClass`.
- Detects object methods (`MyClass.method = function(...)`) and adds proper function annotations.

#### **Header File Extraction Heuristics**

- Extracts only **public** functions from a Lua module.
- Removes implementation details and keeps only function signatures.
- Maintains module dependencies using `require("...")` format.

---

## Reference

For more details on Lua LSP annotations, refer to the official documentation:
[Lua Language Server Annotations](https://luals.github.io/wiki/annotations/)

## Future Enhancements

- **Support for detecting more complex return types** using deeper code analysis.
- **Configuration options** to allow customization of annotations and output formatting.
- **Integration with Lua Language Server for real-time annotation support.**

---

This toolset aims to **automate tedious documentation tasks** while maintaining **code readability and integrity**. 🚀
