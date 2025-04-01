--[[
  LÖVE2D Framework Type Definitions for v11.5

  This file provides type definitions for the LÖVE 2D game framework.
  It enhances type checking and auto-completion for LÖVE game development.
  
  LÖVE version: 11.5
  lua_version = "5.3" -- LÖVE 11+ uses Lua 5.3
]]--

local love = {}

-- =====================
-- Core Types
-- =====================

---@class Image Represents a drawable image
---@field getWidth fun(): number Get the width of the image
---@field getHeight fun(): number Get the height of the image
---@field getDimensions fun(): number, number Get the dimensions of the image
---@field getPixel fun(x: number, y: number): number, number, number, number Get the color at a pixel
---@field setFilter fun(min: string, mag: string) Set the filter mode
---@field getFilter fun(): string, string Get the filter mode
---@field setWrap fun(horiz: string, vert: string) Set the wrap mode
---@field getWrap fun(): string, string Get the wrap mode
---@field replacePixels fun(data: ImageData, x: number, y: number, dx: number, dy: number, sx: number, sy: number) Replace pixels in the image
love.Image = {}

---@class Quad Represents a quadrilateral with texture coordinates
---@field getViewport fun(): number, number, number, number Get the viewport of the quad
---@field setViewport fun(x: number, y: number, width: number, height: number) Set the viewport of the quad
love.Quad = {}

---@class Font Represents a font object for rendering text
---@field getWidth fun(text: string): number Get the width of the text when rendered with this font
---@field getHeight fun(): number Get the height of the font
---@field getAscent fun(): number Get the ascent of the font
---@field getDescent fun(): number Get the descent of the font
---@field getBaseline fun(): number Get the baseline of the font
---@field hasGlyphs fun(text: string): boolean Check if the font has glyphs for the given text
---@field getKerning fun(leftchar: string, rightchar: string): number Get the kerning between two characters
love.Font = {}

---@class Canvas Represents a canvas for offscreen rendering
---@field getWidth fun(): number Get the width of the canvas
---@field getHeight fun(): number Get the height of the canvas
---@field getDimensions fun(): number, number Get the dimensions of the canvas
---@field renderTo fun(callback: function) Render to the canvas
---@field newImageData fun(): ImageData Create an image data from the canvas
---@field getFormat fun(): string Get the format of the canvas
---@field getMSAA fun(): number Get the MSAA value of the canvas
---@field generateMipmaps fun() Generate mipmaps for the canvas
---@field getDepthValues fun(x: number, y: number, width: number, height: number): number[] Get depth values from the canvas
love.Canvas = {}

---@class Shader Represents a graphics shader
---@field send fun(name: string, value: any) Send a value to the shader
---@field getWarnings fun(): string Get any warnings generated from the shader
---@field sendColor fun(name: string, r: number, g: number, b: number, a: number) Send a color to the shader
---@field hasUniform fun(name: string): boolean Check if the shader has a uniform
---@field getExternVariable fun(name: string): string Get an extern variable from the shader
love.Shader = {}

---@class Source Represents an audio source
---@field play fun() Play the source
---@field stop fun() Stop the source
---@field pause fun() Pause the source
---@field isPlaying fun(): boolean Check if the source is playing
---@field isPaused fun(): boolean Check if the source is paused
---@field isStopped fun(): boolean Check if the source is stopped
---@field setVolume fun(volume: number) Set the volume of the source
---@field getVolume fun(): number Get the volume of the source
---@field setPitch fun(pitch: number) Set the pitch of the source
---@field getPitch fun(): number Get the pitch of the source
---@field seek fun(position: number, unit: string) Seek to a position in the source
---@field tell fun(unit: string): number Get the current position in the source
---@field setLooping fun(loop: boolean) Set whether the source should loop
---@field isLooping fun(): boolean Check if the source is looping
---@field setFilter fun(filter: table) Set a filter for the source
---@field getFilter fun(): table Get the filter for the source
---@field getChannelCount fun(): number Get the number of channels in the source
love.Source = {}

-- =====================
-- Modules
-- =====================

-- Graphics Module

---@class GraphicsModule
---@field newImage fun(filename: string): Image Create a new image
---@field newQuad fun(x: number, y: number, width: number, height: number, iw: number, ih: number): Quad Create a new quad
---@field newFont fun(filename: string, size: number): Font Create a new font
---@field newCanvas fun(width: number, height: number, settings: table): Canvas Create a new canvas
---@field newShader fun(code: string): Shader Create a new shader
---@field setColor fun(r: number, g: number, b: number, a: number) Set the current color
---@field getColor fun(): number, number, number, number Get the current color
---@field clear fun(r: number, g: number, b: number, a: number) Clear the screen with a color
---@field present fun() Present the screen
---@field draw fun(drawable: table, x: number, y: number, r: number, sx: number, sy: number, ox: number, oy: number, kx: number, ky: number) Draw a drawable
---@field print fun(text: string, x: number, y: number, r: number, sx: number, sy: number, ox: number, oy: number, kx: number, ky: number) Print text
---@field printf fun(text: string, x: number, y: number, limit: number, align: string, r: number, sx: number, sy: number, ox: number, oy: number, kx: number, ky: number) Print formatted text
---@field rectangle fun(mode: string, x: number, y: number, width: number, height: number, rx: number, ry: number, segments: number) Draw a rectangle
---@field circle fun(mode: string, x: number, y: number, radius: number, segments: number) Draw a circle
---@field ellipse fun(mode: string, x: number, y: number, radiusx: number, radiusy: number, segments: number) Draw an ellipse
---@field line fun(x1: number, y1: number, x2: number, y2: number, ...) Draw a line
---@field polygon fun(mode: string, ...) Draw a polygon
---@field points fun(...) Draw points
---@field setCanvas fun(canvas: Canvas) Set the current canvas
---@field getCanvas fun(): Canvas Get the current canvas
---@field setShader fun(shader: Shader) Set the current shader
---@field getShader fun(): Shader Get the current shader
---@field setBlendMode fun(mode: string) Set the blend mode
---@field getBlendMode fun(): string Get the blend mode
---@field setFont fun(font: Font) Set the current font
---@field getFont fun(): Font Get the current font
---@field setLineWidth fun(width: number) Set the line width
---@field getLineWidth fun(): number Get the line width
---@field setLineStyle fun(style: string) Set the line style
---@field getLineStyle fun(): string Get the line style
---@field setPointSize fun(size: number) Set the point size
---@field getPointSize fun(): number Get the point size
---@field reset fun() Reset all graphics settings
---@field captureScreenshot fun(filename: string) Capture a screenshot
---@field getTextureTypes fun(): table Get the available texture types
---@field getRendererInfo fun(): string, string, number, number, number, number Get information about the renderer
love.graphics = {}

-- Audio Module

---@class AudioModule
---@field newSource fun(filename: string, type: string): Source Create a new audio source
---@field play fun(source: Source) Play an audio source
---@field stop fun(source: Source) Stop an audio source
---@field pause fun(source: Source) Pause an audio source
---@field setVolume fun(volume: number) Set the master volume
---@field getVolume fun(): number Get the master volume
---@field newQueueableSource fun(samplerate: number, bitdepth: number, channels: number, buffercount: number): Source Create a new queueable source
---@field setDistanceModel fun(model: string) Set the distance model
---@field getDistanceModel fun(): string Get the distance model
---@field setDopplerScale fun(scale: number) Set the doppler scale
---@field getDopplerScale fun(): number Get the doppler scale
---@field isEffectsSupported fun(): boolean Check if effects are supported
love.audio = {}

-- Input Module

---@class KeyboardModule
---@field isDown fun(key: string, ...): boolean Check if a key is down
---@field isScancodeDown fun(scancode: string, ...): boolean Check if a scancode is down
---@field setKeyRepeat fun(enable: boolean) Enable or disable key repeat
---@field hasKeyRepeat fun(): boolean Check if key repeat is enabled
---@field hasTextInput fun(): boolean Check if text input is enabled
---@field setTextInput fun(enable: boolean, x: number, y: number, w: number, h: number) Enable or disable text input
---@field getKeyFromScancode fun(scancode: string): string Get the key from a scancode
---@field getScancodeFromKey fun(key: string): string Get the scancode from a key
love.keyboard = {}

---@class MouseModule
---@field getPosition fun(): number, number Get the position of the mouse
---@field isDown fun(button: number, ...): boolean Check if a mouse button is down
---@field setVisible fun(visible: boolean) Set the visibility of the mouse cursor
---@field isVisible fun(): boolean Check if the mouse cursor is visible
---@field newCursor fun(imageData: ImageData, hotx: number, hoty: number): Cursor Create a new cursor
---@field setCursor fun(cursor: Cursor) Set the mouse cursor
---@field getCursor fun(): Cursor Get the mouse cursor
---@field getX fun(): number Get the x-coordinate of the mouse
---@field getY fun(): number Get the y-coordinate of the mouse
---@field isGrabbed fun(): boolean Check if the mouse is grabbed
---@field setGrabbed fun(grab: boolean) Set whether the mouse is grabbed
---@field setRelativeMode fun(enable: boolean) Set whether relative mode is enabled
---@field getRelativeMode fun(): boolean Get whether relative mode is enabled
love.mouse = {}

-- File Module

---@class FileModule
---@field getDirectoryItems fun(directory: string): string[] Get the items in a directory
---@field getInfo fun(path: string, filtertype: string): table Get information about a file or directory
---@field createDirectory fun(name: string): boolean Create a directory
---@field remove fun(name: string): boolean Remove a file or directory
---@field read fun(name: string, size: number): string, number Read data from a file
---@field write fun(name: string, data: string, size: number): boolean, string Write data to a file
---@field getSourceBaseDirectory fun(): string Get the base directory of the source
---@field getSaveDirectory fun(): string Get the save directory for the game
---@field newFile fun(filename: string, mode: string): File Create a new file
---@field newFileData fun(contents: string, name: string, decoder: string): FileData Create new file data
---@field mount fun(archive: string, mountpoint: string, appendToPath: boolean): boolean Mount an archive
---@field unmount fun(archive: string): boolean Unmount an archive
---@field setSymlinksEnabled fun(enable: boolean) Enable or disable symlinks
---@field areSymlinksEnabled fun(): boolean Check if symlinks are enabled
love.filesystem = {}

-- System Module

---@class SystemModule
---@field getOS fun(): string Get the name of the operating system
---@field getProcessorCount fun(): number Get the number of logical CPUs
---@field getPowerInfo fun(): string, number, number Get information about the system's power status
---@field openURL fun(url: string): boolean Open a URL with the user's web browser
---@field setClipboardText fun(text: string) Set the text in the clipboard
---@field getClipboardText fun(): string Get the text in the clipboard
---@field hasClipboardText fun(): boolean Check if there is text in the clipboard
---@field vibrate fun(seconds: number) Vibrate the device for a specified amount of time
---@field getNetworkInfo fun(): boolean, string, string Get information about the network connection
love.system = {}

-- =====================
-- Main Callbacks
-- =====================

--- Called before the first update and draw
---@param arg any[] Command line arguments
function love.load(arg) end

--- Called every frame to update the game state
---@param dt number Time passed since the last update
function love.update(dt) end

--- Called every frame to draw on the screen
function love.draw() end

--- Called when a key is pressed
---@param key string The key that was pressed
---@param scancode string The scancode of the key
---@param isrepeat boolean Whether this is a repeat keypress
function love.keypressed(key, scancode, isrepeat) end

--- Called when a key is released
---@param key string The key that was released
---@param scancode string The scancode of the key
function love.keyreleased(key, scancode) end

--- Called when text is entered
---@param text string The text entered
function love.textinput(text) end

--- Called when a mouse button is pressed
---@param x number The x-coordinate of the mouse
---@param y number The y-coordinate of the mouse
---@param button number The button that was pressed
---@param istouch boolean Whether the press came from a touch
---@param presses number The number of presses
function love.mousepressed(x, y, button, istouch, presses) end

--- Called when a mouse button is released
---@param x number The x-coordinate of the mouse
---@param y number The y-coordinate of the mouse
---@param button number The button that was released
---@param istouch boolean Whether the release came from a touch
---@param presses number The number of presses
function love.mousereleased(x, y, button, istouch, presses) end

--- Called when the mouse is moved
---@param x number The x-coordinate of the mouse
---@param y number The y-coordinate of the mouse
---@param dx number The x-coordinate movement since the last call
---@param dy number The y-coordinate movement since the last call
---@param istouch boolean Whether the movement came from a touch
function love.mousemoved(x, y, dx, dy, istouch) end

--- Called when the window is resized
---@param width number The new width of the window
---@param height number The new height of the window
function love.resize(width, height) end

--- Called when the window is focused or unfocused
---@param focused boolean Whether the window is focused
function love.focus(focused) end

--- Called when the window is minimized or restored
---@param visible boolean Whether the window is visible
function love.visible(visible) end

--- Called when the program is quitting
---@return boolean Whether to abort quitting
function love.quit() end

--- Called when an error occurs
---@param msg string The error message
---@param traceback string The traceback
function love.errorhandler(msg, traceback) end

return love