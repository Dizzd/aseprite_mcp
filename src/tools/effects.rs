use rmcp::schemars;
use serde::Deserialize;

use crate::lua_helpers::{LUA_FIND_LAYER, lua_select_layer};
use crate::server::AsepriteServer;
use crate::utils::{clamp_u32, parse_hex_color};

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReplaceColorParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Source color as hex string
    pub from_color: String,
    /// Target color as hex string
    pub to_color: String,
    /// Tolerance (0-255, default: 0)
    pub tolerance: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OutlineParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Outline color as hex string (e.g. "#000000")
    pub color: String,
    /// Target layer name (if omitted, uses active layer)
    pub layer: Option<String>,
    /// Target frame number, 1-based (if omitted, uses frame 1)
    pub frame: Option<u32>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn replace_color(server: &AsepriteServer, p: ReplaceColorParams) -> Result<String, String> {
    let (fr, fg, fb) = parse_hex_color(&p.from_color);
    let (tr, tg, tb) = parse_hex_color(&p.to_color);
    let tolerance = clamp_u32(p.tolerance.unwrap_or(0), 0, 255);

    let script = format!(
        r#"local spr = app.sprite
app.command.ReplaceColor {{
    ui = false,
    from = Color({}, {}, {}),
    to = Color({}, {}, {}),
    tolerance = {}
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "replaced", from = "{}", to = "{}"}}))"#,
        fr, fg, fb,
        tr, tg, tb,
        tolerance,
        p.from_color,
        p.to_color
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn outline(server: &AsepriteServer, p: OutlineParams) -> Result<String, String> {
    let frame_num = p.frame.unwrap_or(1);
    let (r, g, b) = parse_hex_color(&p.color);

    let layer_select = p.layer.as_ref()
        .map(|name| format!("{}{}", LUA_FIND_LAYER, lua_select_layer(name, false)))
        .unwrap_or_default();

    let script = format!(
        r#"local spr = app.sprite
app.frame = spr.frames[{}]
{}
app.command.Outline {{
    ui = false,
    color = Color({}, {}, {})
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "outlined"}}))"#,
        frame_num,
        layer_select,
        r, g, b
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
