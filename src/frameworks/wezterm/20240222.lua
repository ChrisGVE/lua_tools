--[[
  WezTerm API Type Definitions for 20240222 release

  This file provides type definitions for the WezTerm terminal emulator.
  It enhances type checking and auto-completion for WezTerm configuration.
  
  WezTerm version: 20240222
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
---@field window_decorations string Window decorations (TITLE | RESIZE | NONE)
---@field use_fancy_tab_bar boolean Whether to use the fancy tab bar
---@field window_padding table Padding for windows
wezterm.Config = {}

---@class Window A WezTerm window
---@field active_tab Tab The active tab
---@field tabs Tab[] List of tabs
---@field window_id number Window ID
---@field set_title fun(title: string) Set the window title
---@field get_dimensions fun(): table Get the dimensions of the window
---@field maximize fun() Maximize the window
---@field toggle_fullscreen fun() Toggle fullscreen mode
---@field toast_notification fun(title: string, message: string, options: table) Show a toast notification
wezterm.Window = {}

---@class Tab A tab in WezTerm
---@field active_pane Pane The active pane
---@field panes Pane[] List of panes
---@field tab_id number Tab ID
---@field set_title fun(title: string) Set the tab title
---@field get_title fun(): string Get the tab title
---@field activate fun() Activate this tab
wezterm.Tab = {}

---@class Pane A pane in WezTerm
---@field pane_id number Pane ID
---@field title string The pane title
---@field is_active boolean Whether the pane is active
---@field send_text fun(text: string) Send text to the pane
---@field split fun(direction: string, options: table): Pane Split the pane
---@field get_dimensions fun(): table Get the dimensions of the pane
---@field get_cursor_position fun(): number, number Get the cursor position
---@field get_foreground_process_name fun(): string Get the name of the foreground process
---@field get_foreground_process_info fun(): table Get info about the foreground process
---@field has_unseen_output fun(): boolean Whether the pane has unseen output
wezterm.Pane = {}

-- =====================
-- Module Namespaces (new in 20240222)
-- =====================

--- GUI module with functions related to the GUI
wezterm.gui = {}

--- Get all GUI windows
---@return Window[]
wezterm.gui.gui_windows = function() end

--- Get all workspace names
---@return string[]
wezterm.gui.get_workspace_names = function() end

--- Multiplexer module for session management
wezterm.mux = {}

--- Get all mux windows
---@return MuxWindow[]
wezterm.mux.all_windows = function() end

--- Get the current pane
---@return Pane
wezterm.mux.get_active_pane = function() end

--- Connect to a mux server
---@param domain string Domain name
---@return MuxDomain
wezterm.mux.connect_or_start = function(domain) end

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

--- Get the current time
---@return table Time object
wezterm.time = function() end

--- Get the current strftime
---@param format string Format string
---@param time table? Time object (defaults to now)
---@return string Formatted time string
wezterm.strftime = function(format, time) end

--- Create a keyboard event
---@param event string Event type
---@param key string Key
---@param mods string[] Modifiers
---@return table Keyboard event
wezterm.action = function(event, key, mods) end

return wezterm