use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::lua_string;
use crate::lua_helpers::{LUA_FIND_LAYER, lua_select_layer};
use crate::server::AsepriteServer;
use crate::utils::parse_hex_color;

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
    let tolerance = p.tolerance.unwrap_or(0);

    let script = format!(
        r#"local spr = app.sprite
app.command.ReplaceColor {{
    ui = false,
    from = Color({fr}, {fg}, {fb}),
    to = Color({tr}, {tg}, {tb}),
    tolerance = {tol}
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "replaced", from = {from_s}, to = {to_s}}}))"#,
        fr = fr,
        fg = fg,
        fb = fb,
        tr = tr,
        tg = tg,
        tb = tb,
        tol = tolerance,
        from_s = lua_string(&p.from_color),
        to_s = lua_string(&p.to_color)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn outline(server: &AsepriteServer, p: OutlineParams) -> Result<String, String> {
    let frame_num = p.frame.unwrap_or(1);
    let (r, g, b) = parse_hex_color(&p.color);

    let layer_select = if let Some(ref layer_name) = p.layer {
        format!("{}{}", LUA_FIND_LAYER, lua_select_layer(layer_name, false))
    } else {
        String::new()
    };

    let script = format!(
        r#"local spr = app.sprite
app.frame = spr.frames[{frame}]
{layer_select}
app.command.Outline {{
    ui = false,
    color = Color({r}, {g}, {b})
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "outlined"}}))"#,
        frame = frame_num,
        layer_select = layer_select,
        r = r,
        g = g,
        b = b
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
