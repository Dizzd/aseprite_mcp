# Aseprite MCP Server

A **Model Context Protocol (MCP) server** written in Rust that bridges AI assistants (Claude, GPT, etc.) with the **Aseprite** pixel art editor. This allows AI to create, edit, and export pixel art sprites and animations through natural language instructions.

## Features

### 44 MCP Tools for Full Aseprite Control

| Category | Tools | Description |
|----------|-------|-------------|
| **Sprite** | `create_sprite`, `get_sprite_info`, `resize_sprite`, `crop_sprite`, `flip_sprite`, `rotate_sprite`, `canvas_size`, `duplicate_sprite`, `auto_crop_sprite`, `change_color_mode`, `reverse_frames` | Create, inspect, transform, duplicate, and optimize sprites |
| **Layers** | `list_layers`, `add_layer`, `remove_layer`, `set_layer_property`, `duplicate_layer`, `merge_down_layer`, `flatten_layers` | Full layer management with duplicate, merge, and flatten |
| **Frames** | `list_frames`, `add_frame`, `remove_frame`, `set_frame_duration` | Animation frame management |
| **Tags** | `list_tags`, `create_tag`, `delete_tag` | Animation tag/sequence management |
| **Slices** | `list_slices`, `create_slice`, `delete_slice` | Named regions for game engines: hitboxes, 9-slice UI, pivot points |
| **Cels** | `list_cels`, `move_cel`, `set_cel_opacity`, `clear_cel`, `new_cel` | Fine-grained cel (layer×frame) management for animation |
| **Drawing** | `draw_pixels`, `use_tool`, `get_pixel_data` | Pixel-level drawing and reading with all Aseprite tools |
| **Palette** | `get_palette`, `set_palette_color`, `resize_palette`, `load_palette`, `save_palette`, `color_quantization` | Full palette management with load/save and auto-quantization |
| **Selection** | `select_region`, `deselect`, `select_all`, `invert_selection`, `select_by_color` | Advanced selection operations including color-based selection |
| **Export** | `export_sprite`, `export_spritesheet` | Export to multiple formats and spritesheet with JSON metadata |
| **Effects** | `replace_color`, `outline` | Color replacement and outline effects |
| **Filters** | `brightness_contrast`, `hue_saturation`, `invert_color`, `despeckle` | Image filters for color adjustment and noise reduction |
| **Advanced** | `run_lua_script`, `execute_cli` | Direct Lua scripting and CLI access |

## Architecture

```
┌─────────────┐     stdio (MCP)     ┌──────────────────┐     CLI      ┌──────────┐
│ AI Assistant │◄──────────────────►│ aseprite_mcp     │────────────►│ Aseprite │
│ (Claude etc) │                    │ (Rust MCP Server) │  Lua scripts │ (batch)  │
└─────────────┘                    └──────────────────┘             └──────────┘
```

The server communicates with AI assistants via the MCP protocol over stdio, and controls Aseprite by:
1. Generating Lua scripts dynamically based on tool parameters
2. Executing them via `aseprite --batch --script script.lua`
3. Parsing the JSON output from Aseprite's stdout
4. Returning structured results to the AI assistant

## Installation

### Prerequisites

- **Rust** (1.85+ with edition 2024 support)
- **Aseprite** installed on your system

### Build

```bash
cargo build --release
```

The binary will be at `target/release/aseprite_mcp.exe` (Windows) or `target/release/aseprite_mcp` (Linux/macOS).

### Configure Aseprite Path

The server automatically searches for Aseprite in common locations:

- **Windows**: `C:\Program Files\Aseprite\Aseprite.exe`, Steam path
- **macOS**: `/Applications/Aseprite.app/Contents/MacOS/aseprite`
- **Linux**: PATH, Steam path

To specify a custom path, set the `ASEPRITE_PATH` environment variable:

```bash
export ASEPRITE_PATH="/path/to/aseprite"
```

## MCP Client Configuration

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "aseprite": {
      "command": "path/to/aseprite_mcp.exe",
      "env": {
        "ASEPRITE_PATH": "C:\\Program Files\\Aseprite\\Aseprite.exe"
      }
    }
  }
}
```

### VS Code (Copilot)

Add to your `.vscode/mcp.json`:

```json
{
  "servers": {
    "aseprite": {
      "type": "stdio",
      "command": "path/to/aseprite_mcp.exe",
      "env": {
        "ASEPRITE_PATH": "C:\\Program Files\\Aseprite\\Aseprite.exe"
      }
    }
  }
}
```

### Cursor

Add to your Cursor MCP settings:

```json
{
  "mcpServers": {
    "aseprite": {
      "command": "path/to/aseprite_mcp.exe",
      "env": {
        "ASEPRITE_PATH": "C:\\Program Files\\Aseprite\\Aseprite.exe"
      }
    }
  }
}
```

## Tool Examples

### Create a Sprite
```
Create a 64x64 pixel art sprite with indexed color mode and save it as player.aseprite
```
Tool call: `create_sprite(width=64, height=64, output_path="player.aseprite", color_mode="indexed")`

### Draw Pixels
```
Draw a red pixel at (10, 10) and a blue pixel at (20, 20) on the player sprite
```
Tool call: `draw_pixels(file_path="player.aseprite", pixels=[{x:10,y:10,color:"#ff0000"}, {x:20,y:20,color:"#0000ff"}])`

### Use Drawing Tools
```
Draw a yellow line from (0,0) to (63,63) on the player sprite
```
Tool call: `use_tool(file_path="player.aseprite", tool="line", points=[{x:0,y:0},{x:63,y:63}], color="#ffff00")`

### Export Spritesheet
```
Export the animation as a spritesheet with JSON data
```
Tool call: `export_spritesheet(file_path="player.aseprite", output_image="sheet.png", output_data="sheet.json", sheet_type="rows", columns=4)`

### Run Custom Lua Script
```
Run a custom script to create a checkerboard pattern
```
Tool call: `run_lua_script(script="...", file_path="player.aseprite")`

## Available Drawing Tools

The `use_tool` command supports all Aseprite tools:

| Tool ID | Description |
|---------|-------------|
| `pencil` | Freehand pixel drawing |
| `line` | Straight line |
| `rectangle` | Rectangle outline |
| `filled_rectangle` | Filled rectangle |
| `ellipse` | Ellipse outline |
| `filled_ellipse` | Filled ellipse |
| `paint_bucket` | Flood fill |
| `spray` | Spray/airbrush |
| `eraser` | Erase pixels |
| `contour` | Draw contour |
| `polygon` | Draw polygon |

## Color Format

Colors use hex string format:
- `#rrggbb` — RGB (e.g., `#ff0000` for red)
- `#rrggbbaa` — RGBA with alpha (e.g., `#ff000080` for semi-transparent red)

## WebSocket Plugin (Optional)

For real-time interactive control of a running Aseprite instance, an optional WebSocket plugin is included in `scripts/aseprite-mcp-plugin/`.

### Install the Plugin

1. Copy the `scripts/aseprite-mcp-plugin/` folder to your Aseprite extensions directory:
   - Windows: `%APPDATA%\Aseprite\extensions\`
   - macOS: `~/Library/Application Support/Aseprite/extensions/`
   - Linux: `~/.config/aseprite/extensions/`
2. Restart Aseprite
3. Go to `Help > MCP Server > Connect to MCP Server`

## Project Structure

```
aseprite_mcp/
├── Cargo.toml                          # Rust dependencies
├── README.md                           # This file
├── src/
│   ├── main.rs                         # Entry point, MCP transport setup
│   ├── server.rs                       # MCP server, tool routing & ServerHandler
│   ├── aseprite.rs                     # Aseprite CLI runner (process execution)
│   ├── lua_helpers.rs                  # Reusable Lua snippets (find_layer, etc.)
│   ├── utils.rs                        # Color parsing & validation utilities
│   └── tools/                          # Tool implementations (one file per domain)
│       ├── mod.rs                      # Module re-exports
│       ├── sprite.rs                   # Sprite management (create, info, resize, crop, flip, rotate, canvas, duplicate, auto_crop, color_mode, reverse)
│       ├── layer.rs                    # Layer management (list, add, remove, set properties, duplicate, merge, flatten)
│       ├── frame.rs                    # Frame management (list, add, remove, set duration)
│       ├── tag.rs                      # Animation tag management (list, create, delete)
│       ├── slice.rs                    # Slice management (list, create, delete — 9-slice, pivots, hitboxes)
│       ├── cel.rs                      # Cel management (list, move, opacity, clear, new)
│       ├── drawing.rs                  # Drawing tools (draw_pixels, use_tool, get_pixel_data)
│       ├── palette.rs                  # Palette management (get, set, resize, load, save, quantize)
│       ├── selection.rs                # Selection operations (region, all, invert, by_color, deselect)
│       ├── export.rs                   # Export tools (export_sprite, export_spritesheet)
│       ├── effects.rs                  # Effects (replace_color, outline)
│       ├── filter.rs                   # Image filters (brightness_contrast, hue_saturation, invert, despeckle)
│       └── scripting.rs                # Direct Lua & CLI execution
└── scripts/
    └── aseprite-mcp-plugin/            # Optional Aseprite WebSocket plugin
        ├── package.json
        └── plugin.lua
```

## Tech Stack

- **Rust** (edition 2024)
- **rmcp** 0.3 — MCP protocol framework
- **tokio** — Async runtime
- **serde/serde_json** — JSON serialization
- **tracing** — Structured logging

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ASEPRITE_PATH` | Full path to Aseprite executable | Auto-detected |
| `ASEPRITE_OUTPUT_DIR` | Default output directory for generated files | Working directory |
| `RUST_LOG` | Log level (`trace`, `debug`, `info`, `warn`, `error`) | `info` |

## License

MIT
