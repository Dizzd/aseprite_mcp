---
description: Aseprite API reference and project guidelines for the aseprite_mcp MCP server
applyTo: '**/*.rs'
---

# Aseprite MCP Server — Project Context & API Reference

This project (`aseprite_mcp`) is a **Model Context Protocol (MCP) server** written in Rust that bridges AI assistants with the **Aseprite** pixel art editor. It uses the `rmcp` crate for MCP transport over stdio.

## Project Stack

- **Language**: Rust (edition 2024)
- **MCP Framework**: `rmcp` 0.3 with `server`, `macros`, `transport-io` features
- **Async Runtime**: `tokio` 1.46 (full features)
- **HTTP Client**: `reqwest` 0.12 (json feature) — for communicating with Aseprite's WebSocket/HTTP endpoints
- **Serialization**: `serde` 1.0 + `serde_json` 1.0
- **Error Handling**: `anyhow` 1.0
- **Logging**: `tracing` 0.1 + `tracing-subscriber` 0.3

## Coding Guidelines

- Use `anyhow::Result` for error handling in tool implementations.
- Use `#[derive(Serialize, Deserialize)]` from serde for all data structures exchanged with Aseprite.
- Follow MCP tool conventions: each tool should have a clear name, description, and JSON schema for parameters.
- Use `tracing` macros (`info!`, `debug!`, `error!`) for logging, not `println!`.
- Keep tool implementations modular — one file per logical group of tools if the project grows.

---

## Aseprite API Reference

> Full documentation: <https://www.aseprite.org/api/>
> Source: <https://github.com/aseprite/api>
> Aseprite uses **Lua 5.3** as its scripting language.

### Interaction Model

Aseprite can be controlled in two primary ways relevant to this MCP server:

1. **CLI (Command Line Interface)** — Run `aseprite -b --script script.lua` to execute Lua scripts in batch mode.
2. **WebSocket** — Aseprite's Lua scripting API includes a `WebSocket` class that can connect to external servers, enabling bidirectional communication between an MCP server and a running Aseprite instance.
3. **Lua `--script` with `--script-param`** — Pass parameters to scripts via `app.params`.

### CLI Quick Reference

```
aseprite -b                            # Batch mode (no UI)
aseprite -b sprite.ase --save-as out.png
aseprite -b --script myscript.lua
aseprite -b --script-param key=value --script myscript.lua
aseprite -b sprite.ase --scale 2 --save-as out.png
aseprite -b sprite.ase --sheet sheet.png --data sheet.json
aseprite -b --split-layers sprite.ase --save-as "{layer}-{frame}.png"
aseprite -b sprite.ase --color-mode indexed --save-as out.png
aseprite -b --list-layers sprite.ase
aseprite -b --list-tags sprite.ase
aseprite -b --list-slices sprite.ase
```

Key CLI options:
- `--batch` / `-b`: No UI, process and exit
- `--save-as <filename>`: Export sprite (supports `{layer}`, `{frame}`, `{tag}`, `{slice}` placeholders)
- `--sheet <filename>`: Export sprite sheet image
- `--data <filename.json>`: Export sprite sheet JSON data
- `--script <filename.lua>`: Execute a Lua script
- `--script-param name=value`: Pass params to script (accessible via `app.params`)
- `--scale <factor>`: Resize exported images
- `--split-layers`, `--split-tags`, `--split-slices`: Split output by layers/tags/slices
- `--layer <name>`, `--tag <name>`, `--slice <name>`: Filter exports
- `--trim`, `--crop x,y,w,h`: Trim/crop output
- `--color-mode rgb|grayscale|indexed`: Change color mode
- `--palette <filename>`: Change palette
- `--list-layers`, `--list-tags`, `--list-slices`: List metadata

Platform paths:
- Windows: `"C:\Program Files\Aseprite\Aseprite.exe"` or Steam: `"C:\Program Files (x86)\Steam\steamapps\common\Aseprite\Aseprite.exe"`
- macOS: `/Applications/Aseprite.app/Contents/MacOS/aseprite`
- Linux (Steam): `~/.steam/debian-installation/steamapps/common/Aseprite/aseprite`

---

### Global Namespaces

#### `app` — Main Application Object

| Property / Method | Description |
|---|---|
| `app.sprite` | Active [Sprite](#sprite) object (nil if none) |
| `app.sprites` | Array of all open sprites |
| `app.layer` | Active Layer |
| `app.frame` | Active Frame (can assign frame number) |
| `app.cel` | Active Cel |
| `app.image` | Active Image |
| `app.tag` | Active Tag at current frame |
| `app.tool` | Active Tool |
| `app.brush` | Active Brush |
| `app.editor` | Active Editor |
| `app.window` | Main Window |
| `app.site` | Active Site (image, layer, frame, sprite) |
| `app.range` | Active selection Range |
| `app.fgColor` | Foreground Color |
| `app.bgColor` | Background Color |
| `app.fgTile` | Foreground Tile |
| `app.bgTile` | Background Tile |
| `app.version` | Aseprite Version object |
| `app.apiVersion` | API version number |
| `app.isUIAvailable` | `true` if UI is available (false in `--batch`) |
| `app.params` | Table of `--script-param` values |
| `app.uiScale` | UI scaling factor |
| `app.defaultPalette` | User's default Palette |
| `app.open(filename)` | Open sprite from file, returns Sprite or nil |
| `app.exit()` | Close application |
| `app.transaction(fn)` | Group operations into one undo step |
| `app.transaction(label, fn)` | Same with undo label |
| `app.alert(...)` | Show alert dialog |
| `app.tip(text [, duration])` | Show tooltip in status bar |
| `app.refresh()` | Force UI refresh |
| `app.undo()` | Undo last operation |
| `app.redo()` | Redo last undone operation |
| `app.useTool{...}` | Simulate a tool stroke on canvas |

##### `app.useTool` Parameters
```lua
app.useTool{
  tool = string | Tool,          -- e.g. "pencil", "line", "rectangle", "paint_bucket", etc.
  color = Color,                 -- foreground color
  bgColor = Color,               -- background color
  brush = Brush,
  points = { Point, Point, ... },
  cel = Cel,
  layer = Layer,
  frame = Frame,
  ink = Ink,
  button = MouseButton.LEFT | MouseButton.RIGHT,
  opacity = integer (0-255),
  contiguous = boolean,
  tolerance = integer (0-255),
  freehandAlgorithm = 0 | 1,     -- 0=regular, 1=pixel-perfect
  selection = SelectionMode,
  tilemapMode = TilemapMode,
  tilesetMode = TilesetMode,
}
```

Available tool IDs: `rectangular_marquee`, `elliptical_marquee`, `lasso`, `polygonal_lasso`, `magic_wand`, `pencil`, `spray`, `eraser`, `eyedropper`, `zoom`, `hand`, `move`, `slice`, `paint_bucket`, `gradient`, `line`, `curve`, `rectangle`, `filled_rectangle`, `ellipse`, `filled_ellipse`, `contour`, `polygon`, `blur`, `jumble`.

#### `app.command` — Execute Aseprite Menu Commands

```lua
app.command.CommandName()
app.command.CommandName { param1=value1, param2=value2 }
```

Check if enabled: `app.command.CommandName.enabled`

Key commands (partial list):
- **File**: `NewFile`, `OpenFile`, `SaveFile`, `SaveFileAs`, `SaveFileCopyAs`, `CloseFile`, `ExportSpriteSheet`
- **Edit**: `Undo`, `Redo`, `Copy`, `Cut`, `Paste`, `Clear`, `ClearCel`
- **Sprite**: `SpriteSize`, `CropSprite`, `AutocropSprite`, `CanvasSize`, `ChangePixelFormat`, `DuplicateSprite`, `FlattenLayers`, `Flip`, `Rotate`
- **Layer**: `NewLayer`, `DuplicateLayer`, `RemoveLayer`, `LayerOpacity`, `LayerVisibility`, `MergeDownLayer`
- **Frame**: `NewFrame`, `RemoveFrame`, `FrameProperties`, `ReverseFrames`
- **Selection**: `MaskAll`, `DeselectMask`, `InvertMask`, `ModifySelection`, `MaskByColor`
- **Palette**: `LoadPalette`, `SavePalette`, `PaletteSize`, `ColorQuantization`
- **Filters**: `BrightnessContrast`, `HueSaturation`, `ColorCurve`, `InvertColor`, `ReplaceColor`, `Outline`, `Despeckle`, `ConvolutionMatrix`
- **Tags**: `NewFrameTag`, `RemoveFrameTag`, `FrameTagProperties`, `SetLoopSection`
- **View**: `Zoom`, `FitScreen`, `ShowGrid`, `ShowOnionSkin`, `TiledMode`

Annotations:
- `[*]` — Can have UI interactions, controlled by the `ui` parameter (set `ui=false` for batch mode)
- `[UI]` — Requires UI, cannot be used in `--batch` mode

#### `app.fs` — File System Utilities

Provides path manipulation: `app.fs.pathSeparator`, `app.fs.filePath()`, `app.fs.fileName()`, `app.fs.fileExtension()`, `app.fs.fileTitle()`, `app.fs.joinPath()`, `app.fs.isFile()`, `app.fs.isDirectory()`, `app.fs.fileSize()`, `app.fs.listFiles()`, `app.fs.makeDirectory()`, `app.fs.makeAllDirectories()`, `app.fs.removeDirectory()`.

#### `app.pixelColor` — Low-Level Pixel Color Functions

Functions for RGBA: `app.pixelColor.rgba(r,g,b,a)`, `rgbaR()`, `rgbaG()`, `rgbaB()`, `rgbaA()`.
Functions for Grayscale: `graya(v,a)`, `grayaV()`, `grayaA()`.

#### `json` — JSON Module

`json.decode(string)`, `json.encode(table)`.

---

### Core Classes

#### Sprite

```lua
-- Creating
local spr = Sprite(width, height [, colorMode])
local spr = Sprite(spec)
local spr = Sprite(otherSprite)            -- duplicate
local spr = Sprite{ fromFile="path.ase" }

-- Properties
spr.width, spr.height                      -- dimensions (read/write)
spr.bounds                                 -- Rectangle (read-only)
spr.filename                               -- file path
spr.id                                     -- unique integer ID
spr.isModified                             -- has unsaved changes?
spr.colorMode                              -- ColorMode enum
spr.colorSpace                             -- ColorSpace object
spr.spec                                   -- ImageSpec
spr.transparentColor                       -- index for transparent color
spr.gridBounds                             -- grid Rectangle
spr.pixelRatio                             -- Size
spr.selection                              -- Selection object
spr.color                                  -- user-defined Color (tab color)
spr.data                                   -- user-defined string data
spr.properties                             -- user/extension properties

-- Collections
spr.frames                                 -- array of Frame
spr.layers                                 -- array of Layer
spr.cels                                   -- array of Cel
spr.tags                                   -- array of Tag
spr.slices                                 -- array of Slice
spr.palettes                               -- array of Palette
spr.tilesets                               -- array of Tileset
spr.backgroundLayer                        -- Layer or nil

-- Methods
spr:resize(w, h)
spr:crop(x, y, w, h)   or spr:crop(rectangle)
spr:saveAs(filename)
spr:saveCopyAs(filename)
spr:close()
spr:loadPalette(filename)
spr:setPalette(palette)
spr:assignColorSpace(cs)
spr:convertColorSpace(cs)
spr:newLayer()          --> Layer
spr:newGroup()          --> Layer (group)
spr:deleteLayer(layer | name)
spr:newFrame(frame)     --> Frame (copy)
spr:newEmptyFrame(frameNumber) --> Frame
spr:deleteFrame(frame)
spr:newCel(layer, frame [, image, position]) --> Cel
spr:deleteCel(cel)  or  spr:deleteCel(layer, frame)
spr:newTag(from, to)    --> Tag
spr:deleteTag(tag | name)
spr:newSlice([rect])    --> Slice
spr:deleteSlice(slice | name)
spr:newTileset([grid|rect [, numTiles]]) --> Tileset
spr:deleteTileset(tileset | index)
spr:newTile(tileset [, tileIndex]) --> Tile
spr:deleteTile(tile)  or  spr:deleteTile(tileset, index)
spr:flatten()

-- Events
spr.events:on('change', function(ev) end)
-- Available: 'change', 'filenamechange', 'layerblendmode', 'layername', 'layeropacity', 'layervisibility'
```

#### Image

```lua
-- Creating
local img = Image(width, height [, colorMode])
local img = Image(spec)
local img = Image(otherImage)                  -- clone
local img = Image(otherImage, rectangle)       -- clone region
local img = Image{ fromFile=filename }
local img = image:clone()

-- Properties
img.id, img.version
img.width, img.height, img.bounds
img.colorMode, img.spec
img.cel                                        -- associated Cel or nil
img.context                                    -- GraphicsContext
img.bytes                                      -- raw byte string
img.rowStride, img.bytesPerPixel

-- Methods
img:clear([bounds, color])
img:drawPixel(x, y, color)                     -- no undo!
img:getPixel(x, y) --> integer pixel value
img:drawImage(srcImage [, pos, opacity, blendMode])
img:drawSprite(srcSprite, frameNumber [, pos])
img:isEqual(otherImage) --> bool
img:isEmpty() --> bool
img:isPlain(color) --> bool
img:pixels([rectangle]) --> iterator
img:saveAs(filename)  or  img:saveAs{filename=s, palette=p}
img:resize(w, h)  or  img:resize{width=w, height=h, method="bilinear"|"rotsprite"}
img:shrinkBounds([refColor]) --> Rectangle
img:flip([FlipType])
```

#### Layer

```lua
-- Properties
layer.sprite                     -- parent Sprite
layer.name                       -- string
layer.opacity                    -- 0-255 (nil for groups)
layer.blendMode                  -- BlendMode enum (nil for groups)
layer.layers                     -- child layers (groups only, nil otherwise)
layer.parent                     -- Sprite or group Layer
layer.stackIndex                 -- position in parent stack
layer.uuid                       -- Uuid
layer.isImage, layer.isGroup, layer.isTilemap
layer.isTransparent, layer.isBackground
layer.isEditable, layer.isVisible
layer.isContinuous
layer.isCollapsed, layer.isExpanded
layer.isReference
layer.cels                       -- array of Cel
layer.color                      -- user Color (timeline)
layer.data                       -- user string
layer.properties
layer.tileset                    -- Tileset (tilemap layers only)

-- Methods
layer:cel(frameNumber) --> Cel or nil
```

#### Frame

```lua
frame.sprite
frame.frameNumber                -- 1-based index
frame.duration                   -- in seconds (e.g. 0.1)
frame.previous, frame.next      -- adjacent frames (or nil)
```

#### Cel

```lua
cel.sprite, cel.layer, cel.frame, cel.frameNumber
cel.image                        -- Image (setting replaces image)
cel.bounds                       -- Rectangle (position + image size)
cel.position                     -- Point (top-left in canvas)
cel.opacity                      -- 0-255
cel.zIndex                       -- Z-order offset
cel.color                        -- user Color
cel.data                         -- user string
cel.properties
```

#### Tag

```lua
tag.sprite
tag.name                         -- string
tag.fromFrame, tag.toFrame       -- Frame objects
tag.frames                       -- number of frames in tag
tag.aniDir                       -- AniDir enum
tag.repeats                      -- number of repeats
tag.color                        -- user Color
tag.data                         -- user string
tag.properties
```

#### Palette

```lua
local pal = Palette()
local pal = Palette(ncolors)
local pal = Palette{ fromFile=filename }
local pal = Palette{ fromResource=id }

pal:resize(ncolors)
pal:getColor(index) --> Color
pal:setColor(index, color)
#pal                              -- number of colors
pal.frame                         -- Frame this palette is associated with
pal:saveAs(filename)
```

#### Color

```lua
local c = Color(r, g, b [, a])
local c = Color{ r=int, g=int, b=int, a=int }
local c = Color{ h=number, s=number, v=number, a=int }   -- HSV
local c = Color{ h=number, s=number, l=number, a=int }   -- HSL
local c = Color{ gray=int, a=int }
local c = Color{ index=int }
local c = Color(pixelValue)

-- Properties: c.red, c.green, c.blue, c.alpha
-- c.hsvHue, c.hsvSaturation, c.hsvValue
-- c.hslHue, c.hslSaturation, c.hslLightness
-- c.hue, c.saturation, c.value, c.lightness
-- c.index, c.gray
-- c.rgbaPixel, c.grayPixel
```

#### Selection

```lua
local sel = Selection()
local sel = Selection(rectangle)

sel.bounds                        -- bounding Rectangle
sel.origin                        -- Point
sel.isEmpty                       -- bool

sel:select(rectangle)
sel:selectAll()
sel:deselect()
sel:contains(x, y) or sel:contains(point) --> bool
sel:add(rectangle | selection)
sel:subtract(rectangle | selection)
sel:intersect(rectangle | selection)
```

#### Slice

```lua
slice.sprite, slice.name
slice.bounds                      -- Rectangle
slice.center                      -- Rectangle (9-slice center, or nil)
slice.pivot                       -- Point (or nil)
slice.color                       -- user Color
slice.data                        -- user string
slice.properties
```

#### Rectangle

```lua
local r = Rectangle()
local r = Rectangle(x, y, width, height)
local r = Rectangle{x=n, y=n, width=n, height=n}

r.x, r.y, r.width, r.height
r.origin --> Point
r.size --> Size
r.isEmpty --> bool

r:contains(point | rect) --> bool
r:intersects(rect) --> bool
r:union(rect) --> Rectangle
r:intersect(rect) --> Rectangle
```

#### Point

```lua
local p = Point()
local p = Point(x, y)
local p = Point{x=n, y=n}
p.x, p.y
```

#### Size

```lua
local s = Size()
local s = Size(width, height)
local s = Size{width=n, height=n}
s.width, s.height
```

---

### WebSocket (Aseprite side)

Aseprite's Lua API includes a WebSocket **client** that can connect to external servers. This is the recommended way for an MCP server to communicate with a running Aseprite instance.

```lua
local ws = WebSocket{
  url = "http://127.0.0.1:PORT",
  onreceive = function(messageType, data)
    if messageType == WebSocketMessageType.OPEN then
      -- connection established
    elseif messageType == WebSocketMessageType.TEXT then
      -- received text message
    elseif messageType == WebSocketMessageType.BINARY then
      -- received binary data
    elseif messageType == WebSocketMessageType.CLOSE then
      -- connection closed
    end
  end,
  deflate = false,               -- compression (disable for localhost)
  minreconnectwait = number,     -- seconds
  maxreconnectwait = number,     -- seconds
}

ws:connect()
ws:close()
ws:sendText(str1, str2, ...)     -- send text message
ws:sendBinary(bstr1, bstr2, ...) -- send binary message
ws:sendPing(str)                 -- keep-alive

ws.url                            -- read-only server address
```

`WebSocketMessageType` enum values: `OPEN`, `TEXT`, `BINARY`, `CLOSE`, `PING`, `PONG`, `FRAGMENT`.

**Architecture pattern**: The MCP server (Rust) starts a WebSocket server. An Aseprite Lua plugin/script connects to it. The MCP server sends JSON commands which the Lua script executes and returns results.

---

### Constants / Enums

| Enum | Values |
|---|---|
| `ColorMode` | `RGB`, `GRAYSCALE`, `INDEXED`, `TILEMAP` |
| `BlendMode` | `NORMAL`, `MULTIPLY`, `SCREEN`, `OVERLAY`, `DARKEN`, `LIGHTEN`, `COLOR_DODGE`, `COLOR_BURN`, `HARD_LIGHT`, `SOFT_LIGHT`, `DIFFERENCE`, `EXCLUSION`, `HSL_HUE`, `HSL_SATURATION`, `HSL_COLOR`, `HSL_LUMINOSITY`, `ADDITION`, `SUBTRACT`, `DIVIDE` |
| `AniDir` | `FORWARD`, `REVERSE`, `PING_PONG`, `PING_PONG_REVERSE` |
| `FlipType` | `HORIZONTAL`, `VERTICAL` |
| `RangeType` | `EMPTY`, `LAYERS`, `FRAMES`, `CELS` |
| `Ink` | `SIMPLE`, `ALPHA_COMPOSITING`, `COPY_COLOR`, `LOCK_ALPHA`, `SHADING` |
| `MouseButton` | `LEFT`, `RIGHT`, `MIDDLE`, `X1`, `X2` |
| `SelectionMode` | `REPLACE`, `ADD`, `SUBTRACT`, `INTERSECT` |
| `SpriteSheetType` | `HORIZONTAL`, `VERTICAL`, `ROWS`, `COLUMNS`, `PACKED` |
| `SpriteSheetDataFormat` | `JSON_HASH`, `JSON_ARRAY` |
| `TilemapMode` | `PIXELS`, `TILES` |
| `TilesetMode` | `MANUAL`, `AUTO`, `STACK` |
| `BrushType` | `CIRCLE`, `SQUARE`, `LINE`, `IMAGE` |
| `BrushPattern` | `NONE`, `ORIGIN`, `TARGET` |
| `FilterChannels` | `RED`, `GREEN`, `BLUE`, `ALPHA`, `GRAY`, `INDEX`, `RGB`, `RGBA` |
| `Align` | `LEFT`, `CENTER`, `RIGHT`, `TOP`, `BOTTOM` |

---

### Lua Libraries Available in Aseprite

- Base library (print, type, tostring, pairs, ipairs, etc.)
- Coroutine Manipulation
- String Manipulation
- UTF-8 Support
- Table Manipulation
- Mathematical Functions
- Operating System Facilities (some functions like `os.exit`, `os.tmpname` are not available; `os.execute` and `io.open` will ask for user permissions)
- The Debug Library

---

### Important Notes for MCP Implementation

1. **Batch mode** (`-b`): No UI is available. `app.isUIAvailable` returns `false`. `[UI]`-annotated commands will fail. Use `ui=false` parameter for `[*]`-annotated commands.

2. **Transactions**: Wrap multiple operations in `app.transaction()` to create a single undo step.

3. **Script execution**: Use `aseprite -b --script file.lua` for one-shot operations. For interactive communication, use WebSocket.

4. **Image coordinates**: `Image:getPixel(x,y)` uses coordinates relative to the image (0,0 = top-left of image), NOT the sprite canvas. Use `Cel.position` to convert.

5. **Frame numbering**: Frames are 1-based in Lua (`sprite.frames[1]` is the first frame).

6. **Color values**: Pixel values depend on color mode:
   - RGB: Use `app.pixelColor.rgba(r,g,b,a)` and `rgbaR/G/B/A()` to construct/decompose
   - Grayscale: Use `graya(v,a)` and `grayaV/A()`
   - Indexed: Plain integer index

7. **File formats supported**: `.ase`/`.aseprite` (native), `.png`, `.gif`, `.jpg`, `.bmp`, `.tga`, `.pcx`, `.ico`, `.webp`, `.svg`, `.psd`, `.fli`/`.flc`, and more.

8. **Undo safety**: `Image:drawPixel()` does NOT create undo info. Clone the image, modify the clone, then use `cel.image = clone` for proper undo support.

9. **Script examples**: <https://github.com/aseprite/Aseprite-Script-Examples>
10. **Tests**: <https://github.com/aseprite/aseprite/tree/main/tests/scripts>