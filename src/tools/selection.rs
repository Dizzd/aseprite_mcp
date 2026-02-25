use rmcp::schemars;
use serde::Deserialize;

use crate::server::AsepriteServer;
use crate::utils::parse_hex_color;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SelectRegionParams {
    /// Path to the sprite file
    pub file_path: String,
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Selection mode: "replace", "add", "subtract", "intersect" (default: "replace")
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SelectByColorParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Color to select in hex format (e.g. "#ff0000")
    pub color: String,
    /// Tolerance for color matching (0-255, default: 0)
    pub tolerance: Option<u32>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn select_region(server: &AsepriteServer, p: SelectRegionParams) -> Result<String, String> {
    let mode_fn = match p.mode.as_deref() {
        Some("add") => "add",
        Some("subtract") => "subtract",
        Some("intersect") => "intersect",
        _ => "select",
    };
    let script = format!(
        r#"local spr = app.sprite
local sel = spr.selection
sel:{mode}(Rectangle({x}, {y}, {w}, {h}))
spr:saveAs(spr.filename)
local result = {{}}
result.status = "selected"
result.bounds = {{
    x = sel.bounds.x,
    y = sel.bounds.y,
    width = sel.bounds.width,
    height = sel.bounds.height
}}
result.isEmpty = sel.isEmpty
print(json.encode(result))"#,
        mode = mode_fn,
        x = p.x,
        y = p.y,
        w = p.width,
        h = p.height
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn deselect(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
spr.selection:deselect()
spr:saveAs(spr.filename)
print(json.encode({status = "deselected"}))"#;
    server.execute_script_on_file(file_path, script).await
}

pub async fn select_all(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
app.command.MaskAll()
spr:saveAs(spr.filename)
local sel = spr.selection
local result = {}
result.status = "selected_all"
result.bounds = {
    x = sel.bounds.x,
    y = sel.bounds.y,
    width = sel.bounds.width,
    height = sel.bounds.height
}
print(json.encode(result))"#;
    server.execute_script_on_file(file_path, script).await
}

pub async fn invert_selection(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
app.command.InvertMask()
spr:saveAs(spr.filename)
local sel = spr.selection
local result = {}
result.status = "inverted"
result.isEmpty = sel.isEmpty
if not sel.isEmpty then
    result.bounds = {
        x = sel.bounds.x,
        y = sel.bounds.y,
        width = sel.bounds.width,
        height = sel.bounds.height
    }
end
print(json.encode(result))"#;
    server.execute_script_on_file(file_path, script).await
}

pub async fn select_by_color(server: &AsepriteServer, p: SelectByColorParams) -> Result<String, String> {
    let (r, g, b) = parse_hex_color(&p.color);
    let tolerance = p.tolerance.unwrap_or(0).min(255);
    let color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    let script = format!(
        r#"local spr = app.sprite
app.fgColor = Color({r}, {g}, {b})
app.command.MaskByColor {{
    ui = false,
    tolerance = {tolerance}
}}
spr:saveAs(spr.filename)
local sel = spr.selection
local result = {{}}
result.status = "selected_by_color"
result.color = "{color_hex}"
result.tolerance = {tolerance}
result.isEmpty = sel.isEmpty
if not sel.isEmpty then
    result.bounds = {{
        x = sel.bounds.x,
        y = sel.bounds.y,
        width = sel.bounds.width,
        height = sel.bounds.height
    }}
end
print(json.encode(result))"#,
        r = r,
        g = g,
        b = b,
        tolerance = tolerance,
        color_hex = color_hex
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
