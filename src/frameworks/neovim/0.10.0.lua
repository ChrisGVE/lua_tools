--[[
  Neovim API Type Definitions for v0.10.0

  This file provides type definitions for the Neovim API.
  It enhances type checking and auto-completion for Neovim plugin development.
  
  Neovim version: 0.10.0
  lua_version = "5.1" -- Neovim uses Lua 5.1
]]--

local vim = {}

-- =====================
-- Core Vim API Types
-- =====================

---@class Buffer Represents a Neovim buffer
---@field id number Buffer handle ID
---@field name string Buffer name
---@field lines table List of lines in the buffer
vim.Buffer = {}

---@class Window Represents a Neovim window
---@field id number Window handle ID
---@field buffer Buffer The buffer displayed in this window
---@field height number Window height
---@field width number Window width
vim.Window = {}

---@class Tabpage Represents a Neovim tabpage
---@field id number Tabpage handle ID
---@field windows table List of windows in this tabpage
vim.Tabpage = {}

-- =====================
-- Vim Namespace (0.10 additions)
-- =====================

--- Get the value of a Vim option
---@param name string The option name
---@return any
vim.opt_get = function(name) end

--- Set the value of a Vim option
---@param name string The option name
---@param value any The option value
vim.opt_set = function(name, value) end

--- Evaluate a Vimscript expression
---@param expr string The expression to evaluate
---@return any
vim.eval = function(expr) end

--- Execute a Vim command
---@param cmd string The command to execute
vim.cmd = function(cmd) end

--- Version information (new in 0.10)
---@class Version
---@field major number Major version number
---@field minor number Minor version number
---@field patch number Patch version number
---@field prerelease string|nil Prerelease version string
---@field build string|nil Build metadata
vim.version = {
  major = 0,
  minor = 10,
  patch = 0,
}

--- Log a message at the specified level
---@param message string The message to log
---@param level? number The log level (0-4)
vim.log.info = function(message, level) end

--- Get information about a buffer
---@param bufnr? number The buffer number (0 for current)
---@return table
vim.fn.getbufinfo = function(bufnr) end

-- =====================
-- Vim API Functions (0.10 additions)
-- =====================

vim.api = {}

--- Get the current buffer
---@return Buffer
vim.api.nvim_get_current_buf = function() end

--- Get a list of all buffers
---@return Buffer[]
vim.api.nvim_list_bufs = function() end

--- Get buffer lines
---@param buffer number Buffer handle
---@param start number Start line (0-indexed)
---@param end_ number End line (exclusive)
---@param strict_indexing boolean Error on out-of-bounds
---@return string[]
vim.api.nvim_buf_get_lines = function(buffer, start, end_, strict_indexing) end

--- Set buffer lines
---@param buffer number Buffer handle
---@param start number Start line (0-indexed)
---@param end_ number End line (exclusive)
---@param strict_indexing boolean Error on out-of-bounds
---@param lines string[] The lines to set
vim.api.nvim_buf_set_lines = function(buffer, start, end_, strict_indexing, lines) end

--- Create a new buffer
---@param listed boolean Whether the buffer should be listed
---@param scratch boolean Whether the buffer is a scratch buffer
---@return number
vim.api.nvim_create_buf = function(listed, scratch) end

--- Get current window
---@return number
vim.api.nvim_get_current_win = function() end

--- Get a list of all windows
---@return number[]
vim.api.nvim_list_wins = function() end

--- Create a new window
---@param buffer number Buffer handle
---@param enter boolean Whether to enter the window
---@param config table Configuration
---@return number
vim.api.nvim_open_win = function(buffer, enter, config) end

--- Create a user command
---@param name string Command name
---@param command string|function Command replacement or function
---@param opts table Command options
vim.api.nvim_create_user_command = function(name, command, opts) end

--- Set a keymap
---@param mode string Mode (n, i, v, x, etc.)
---@param lhs string Left-hand side of the mapping
---@param rhs string|function Right-hand side of the mapping
---@param opts table? Optional settings
vim.api.nvim_set_keymap = function(mode, lhs, rhs, opts) end

--- Create an autocommand
---@param event string|string[] Event name or list of events
---@param opts table Options
---@return number
vim.api.nvim_create_autocmd = function(event, opts) end

--- Create an autocommand group
---@param name string Group name
---@param opts table Options
---@return number
vim.api.nvim_create_augroup = function(name, opts) end

--- Get all autocommands
---@param opts table? Options
---@return table[]
vim.api.nvim_get_autocmds = function(opts) end

--- Delete an autocommand
---@param id number Autocommand ID
vim.api.nvim_del_autocmd = function(id) end

--- Execute Lua code
---@param code string Lua code
---@param args table? Arguments
---@return any
vim.api.nvim_exec_lua = function(code, args) end

-- New in 0.10:

--- Create a floating text with virtual text properties
---@param bufnr number Buffer number
---@param ns_id number Namespace ID
---@param opts table Options
---@return number
vim.api.nvim_create_virtual_text = function(bufnr, ns_id, opts) end

--- Get information about runtime files
---@param name string Pattern to match
---@param opts table? Options
---@return table[]
vim.api.nvim_get_runtime_file = function(name, opts) end

--- Clear all UI highlights
vim.api.nvim_clear_highlights = function() end

-- =====================
-- Vim Functions (vim.fn)
-- =====================

vim.fn = {}

--- Get the current line
---@return string
vim.fn.getline = function() end

--- Get a list of buffers
---@return number[]
vim.fn.bufnr = function() end

--- Expand a file path
---@param expr string The expression to expand
---@param nosuf boolean Whether to ignore suffixes
---@param list boolean Whether to return a list
---@return string|string[]
vim.fn.expand = function(expr, nosuf, list) end

--- Get the current directory
---@return string
vim.fn.getcwd = function() end

--- Check if a buffer exists
---@param bufnr number Buffer number
---@return number
vim.fn.bufexists = function(bufnr) end

--- Check if a buffer is loaded
---@param bufnr number Buffer number
---@return number
vim.fn.bufloaded = function(bufnr) end

return vim