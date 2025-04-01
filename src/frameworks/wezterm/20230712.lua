--[[
  WezTerm API Type Definitions for 20230712 release

  This file provides type definitions for the WezTerm terminal emulator.
  It enhances type checking and auto-completion for WezTerm configuration.
  
  WezTerm version: 20230712
  lua_version = "5.4" -- WezTerm uses Lua 5.4
]]--

local wezterm = {}

-- =====================
-- Core Types
-- =====================

---@class Config The WezTerm configuration object
---@field color_scheme string Name of the color scheme
---@field font string Font to use
---@field font_size number Size of the font
---@field window_background_opacity number Background opacity (0.0-1.0)
---@field enable_tab_bar boolean Whether to enable the tab bar
---@field hide_tab_bar_if_only_one_tab boolean Whether to hide the tab bar if only one tab is open
wezterm.Config = {}

---@class Window A WezTerm window
---@field active_tab Tab The active tab
---@field tabs Tab[] List of tabs
---@field window_id number Window ID
---@field set_title fun(title: string) Set the window title
---@field get_dimensions fun(): table Get the dimensions of the window
---@field maximize fun() Maximize the window
---@field toggle_fullscreen fun() Toggle fullscreen mode
wezterm.Window = {}

---@class Tab A tab in WezTerm
---@field active_pane Pane The active pane
---@field panes Pane[] List of panes
---@field tab_id number Tab ID
---@field set_title fun(title: string) Set the tab title
---@field get_title fun(): string Get the tab title
wezterm.Tab = {}

---@class Pane A pane in WezTerm
---@field pane_id number Pane ID
---@field title string The pane title
---@field is_active boolean Whether the pane is active
---@field send_text fun(text: string) Send text to the pane
---@field split fun(direction: string): Pane Split the pane
---@field get_dimensions fun(): table Get the dimensions of the pane
---@field get_cursor_position fun(): number, number Get the cursor position
wezterm.Pane = {}

-- =====================
-- API Functions
-- =====================

--- Load the configuration
---@return Config
wezterm.config_builder = function() end

--- Return the default configuration
---@return Config
wezterm.default_config = function() end

--- Get the configuration file's directory
---@return string
wezterm.config_dir = function() end

--- Format a string with colors and attributes
---@param elements table[] Text elements with formatting
---@return string The formatted string
wezterm.format = function(elements) end

--- Get the active window
---@return Window
wezterm.active_window = function() end

--- Spawn a command in a new tab
---@param args table Command arguments
---@param options table? Spawn options
---@return Pane
wezterm.spawn_tab = function(args, options) end

--- Spawn a command in a new window
---@param args table Command arguments
---@param options table? Spawn options
---@return Window
wezterm.spawn_window = function(args, options) end

--- Get information about installed fonts
---@return table[]
wezterm.font_with_fallback = function() end

-- =====================
-- Helper functions
-- =====================

--- Split the provided text by newlines
---@param text string Text to split
---@return string[]
wezterm.split_by_newlines = function(text) end

--- Concatenate path segments
---@param segments string[] Path segments
---@return string
wezterm.join_path = function(segments) end

--- Execute a command and return the output
---@param cmd string[] Command and arguments
---@param options table? Execution options
---@return string Output from the command
wezterm.run_child_process = function(cmd, options) end

--- Log a message
---@param message string The message to log
wezterm.log_info = function(message) end

return wezterm