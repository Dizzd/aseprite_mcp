use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::aseprite::lua_path;
use crate::server::AsepriteServer;
use crate::utils::parse_hex_color_with_alpha;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPaletteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Maximum number of colors to return (default: all)
    pub max_colors: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetPaletteColorParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Array of palette entries: [{"index": 0, "color": "#ff0000"}, ...]
    pub colors: Vec<PaletteEntry>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PaletteEntry {
    /// Palette index
    pub index: u32,
    /// Color as hex string (e.g. "#ff0000")
    pub color: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ResizePaletteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// New palette size (number of colors)
    pub size: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadPaletteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Path to the palette file to load (.gpl, .pal, .act, .col, .png, etc.)
    pub palette_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SavePaletteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Output path for the palette file (e.g. "palette.gpl", "colors.pal", "palette.png")
    pub output_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ColorQuantizationParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Maximum number of colors in the quantized palette (2-256, default: 256)
    pub max_colors: Option<u32>,
    /// Use alpha channel in quantization (default: false)
    pub with_alpha: Option<bool>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn get_palette(server: &AsepriteServer, p: GetPaletteParams) -> Result<String, String> {
    let max_str = if let Some(max) = p.max_colors {
        format!("local maxColors = {}", max)
    } else {
        "local maxColors = #pal".to_string()
    };

    let script = format!(
        r##"local spr = app.sprite
local pal = spr.palettes[1]
{max_str}
local colors = {{}}
local count = math.min(maxColors, #pal)
for i = 0, count - 1 do
    local c = pal:getColor(i)
    local entry = {{}}
    entry.index = i
    entry.color = string.format("#%02x%02x%02x%02x", c.red, c.green, c.blue, c.alpha)
    entry.red = c.red
    entry.green = c.green
    entry.blue = c.blue
    entry.alpha = c.alpha
    table.insert(colors, entry)
end
print(json.encode({{colors = colors, total = #pal}}))"##,
        max_str = max_str
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn set_palette_color(server: &AsepriteServer, p: SetPaletteColorParams) -> Result<String, String> {
    let mut set_code = String::new();
    for entry in &p.colors {
        let (r, g, b, a) = parse_hex_color_with_alpha(&entry.color);
        set_code.push_str(&format!(
            "    pal:setColor({}, Color({}, {}, {}, {}))\n",
            entry.index, r, g, b, a
        ));
    }

    let script = format!(
        r#"local spr = app.sprite
local pal = spr.palettes[1]
app.transaction("Set Palette Colors", function()
{set_code}
end)
spr:saveAs(spr.filename)
print(json.encode({{status = "updated", colorsSet = {count}}}))"#,
        set_code = set_code,
        count = p.colors.len()
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn resize_palette(server: &AsepriteServer, p: ResizePaletteParams) -> Result<String, String> {
    if p.size == 0 {
        return Err("Palette size must be greater than 0".to_string());
    }
    let script = format!(
        r#"local spr = app.sprite
local pal = spr.palettes[1]
local oldSize = #pal
app.command.PaletteSize {{
    ui = false,
    size = {size}
}}
spr:saveAs(spr.filename)
pal = spr.palettes[1]
print(json.encode({{status = "resized", oldSize = oldSize, newSize = #pal}}))"#,
        size = p.size
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn load_palette(server: &AsepriteServer, p: LoadPaletteParams) -> Result<String, String> {
    let pal_path = lua_path(&p.palette_path);
    let script = format!(
        r#"local spr = app.sprite
spr:loadPalette({path})
spr:saveAs(spr.filename)
local pal = spr.palettes[1]
print(json.encode({{status = "loaded", paletteSize = #pal}}))"#,
        path = pal_path
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn save_palette(server: &AsepriteServer, p: SavePaletteParams) -> Result<String, String> {
    let out = lua_path(&server.resolve_output_path(&p.output_path));
    let script = format!(
        r#"local spr = app.sprite
local pal = spr.palettes[1]
pal:saveAs({out})
print(json.encode({{status = "saved", paletteSize = #pal, filename = {out}}}))"#,
        out = out
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn color_quantization(server: &AsepriteServer, p: ColorQuantizationParams) -> Result<String, String> {
    let max_colors = p.max_colors.unwrap_or(256).clamp(2, 256);
    let with_alpha = p.with_alpha.unwrap_or(false);
    let script = format!(
        r#"local spr = app.sprite
app.command.ColorQuantization {{
    ui = false,
    withAlpha = {alpha},
    maxColors = {max_colors}
}}
spr:saveAs(spr.filename)
local pal = spr.palettes[1]
print(json.encode({{status = "quantized", paletteSize = #pal, maxColors = {max_colors}}}))"#,
        alpha = if with_alpha { "true" } else { "false" },
        max_colors = max_colors
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
