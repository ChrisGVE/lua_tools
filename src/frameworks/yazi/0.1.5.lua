--[[
  Yazi File Manager Type Definitions

  This file provides type definitions for the Yazi file manager.
  It enhances type checking and auto-completion for Yazi customization.
  
  Yazi version: 0.1.5
  lua_version = "5.4" -- Yazi uses Lua 5.4
]]--

local ya = {}

-- =====================
-- Core Types
-- =====================

---@class File Represents a file in Yazi
---@field name string The name of the file
---@field path string The path to the file
---@field url string The URL of the file
---@field size number The size of the file in bytes
---@field mimetype string The MIME type of the file
---@field is_dir boolean Whether the file is a directory
---@field is_hidden boolean Whether the file is hidden
---@field is_readonly boolean Whether the file is read-only
---@field is_selected boolean Whether the file is selected
---@field ctime number The creation time of the file
---@field mtime number The modification time of the file
---@field atime number The access time of the file
---@field mode number The file mode
ya.File = {}

---@class Manager The file manager
---@field cwd string The current working directory
---@field files File[] The list of files in the current directory
---@field current_file File The currently selected file
---@field cd fun(path: string) Change the current directory
---@field reload fun() Reload the current directory
---@field select fun(idx: number) Select a file by index
---@field select_all fun() Select all files
---@field unselect_all fun() Unselect all files
---@field toggle_select fun(idx: number) Toggle the selection of a file
---@field invert_select fun() Invert the selection
---@field copy fun(cut: boolean) Copy or cut the selected files
---@field paste fun() Paste the copied files
---@field delete fun() Delete the selected files
---@field create fun(name: string, is_dir: boolean) Create a file or directory
---@field rename fun(name: string) Rename the selected file
---@field search fun(pattern: string) Search for files
---@field filter fun(pattern: string) Filter the files
---@field sort fun(method: string, reverse: boolean) Sort the files
---@field peek fun() Peek the selected file
---@field open fun() Open the selected file
---@field parent fun() Go to the parent directory
---@field preview fun() Preview the selected file
ya.Manager = {}

---@class Input Input handling
---@field bind fun(key: string, mode: string, action: function) Bind a key to an action
---@field send fun(key: string) Send a key event
---@field unbind fun(key: string, mode: string) Unbind a key
ya.Input = {}

---@class Ui UI components
---@field preview fun(file: File) Show a preview of a file
---@field status fun(status: string) Set the status message
---@field message fun(message: string, level: string) Show a message
---@field notify fun(message: string, level: string) Show a notification
---@field quit fun() Quit Yazi
ya.Ui = {}

---@class App The application
---@field config table The configuration
---@field version string The version of Yazi
---@field name string The name of the application
---@field clipboard table The clipboard
---@field manager Manager The file manager
---@field ui Ui The UI
---@field keybinds table The keybindings
ya.App = {}

-- =====================
-- Core API
-- =====================

--- Get the current manager
---@return Manager
ya.manager = ya.Manager

--- Get the UI
---@return Ui
ya.ui = ya.Ui

--- Get the app
---@return App
ya.app = ya.App

--- Get the input
---@return Input
ya.input = ya.Input

-- =====================
-- Functions
-- =====================

--- Execute a shell command
---@param cmd string The command to execute
---@param env table The environment variables
---@param callback fun(ok: boolean, stdout: string, stderr: string) The callback function
---@return boolean, string, string Whether the command succeeded, stdout, stderr
ya.exec = function(cmd, env, callback) end

--- Run a shell command
---@param cmd string The command to execute
---@param env table The environment variables
---@return boolean Whether the command succeeded
ya.run = function(cmd, env) end

--- Synchronize the state
---@param force boolean Whether to force synchronization
ya.sync = function(force) end

--- Get the configuration value
---@param key string The key to get
---@return any The value
ya.get_config = function(key) end

--- Set the configuration value
---@param key string The key to set
---@param value any The value to set
ya.set_config = function(key, value) end

--- Get the keymap
---@return table The keymap
ya.get_keymap = function() end

--- Set the keymap
---@param keymap table The keymap
ya.set_keymap = function(keymap) end

--- Get the theme
---@return table The theme
ya.get_theme = function() end

--- Set the theme
---@param theme table The theme
ya.set_theme = function(theme) end

--- Show a notification
---@param message string The message to show
---@param level string The level of the notification (info, warn, error)
ya.notify = function(message, level) end

--- Log a message
---@param message string The message to log
---@param level string The level of the message (debug, info, warn, error)
ya.log = function(message, level) end

return ya