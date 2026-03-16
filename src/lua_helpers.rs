//! Reusable Lua script snippets for Aseprite MCP server.
//! These constants and functions help reduce code duplication across tool modules.

use crate::aseprite::lua_string;

// ============================================================================
// Layer Operations
// ============================================================================

/// Find a layer by name (searches groups recursively).
/// Usage: Call `find_layer(spr.layers, "layer_name")` to get the layer object.
pub const LUA_FIND_LAYER: &str = r#"
local function find_layer(lyrs, name)
    for i, l in ipairs(lyrs) do
        if l.name == name then return l end
        if l.isGroup and l.layers then
            local found = find_layer(l.layers, name)
            if found then return found end
        end
    end
    return nil
end"#;

/// Lua snippet to select a target layer by name.
/// Requires LUA_FIND_LAYER to be included first.
/// Sets `app.layer = target_layer` if found, otherwise prints error and returns.
pub fn lua_select_layer(layer_name: &str, error_on_missing: bool) -> String {
    let name = lua_string(layer_name);
    if error_on_missing {
        format!(
            r#"
local target_layer = find_layer(spr.layers, {name})
if not target_layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
app.layer = target_layer"#,
            name = name
        )
    } else {
        format!(
            r#"
local target_layer = find_layer(spr.layers, {name})
if target_layer then app.layer = target_layer end"#,
            name = name
        )
    }
}

/// Lua snippet to get layer by name and validate it exists.
/// Requires LUA_FIND_LAYER to be included first.
/// Returns error JSON if layer not found.
pub fn lua_require_layer(layer_name: &str) -> String {
    let name = lua_string(layer_name);
    format!(
        r#"
{}
local layer = find_layer(spr.layers, {})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {}}}))
    return
end"#,
        LUA_FIND_LAYER,
        name,
        name
    )
}

// ============================================================================
// Helper Functions for Building Lua Scripts
// ============================================================================

/// Parse blend mode string to Aseprite Lua constant.
pub fn parse_blend_mode(mode: &str) -> &'static str {
    match mode.to_lowercase().as_str() {
        "normal" => "BlendMode.NORMAL",
        "multiply" => "BlendMode.MULTIPLY",
        "screen" => "BlendMode.SCREEN",
        "overlay" => "BlendMode.OVERLAY",
        "darken" => "BlendMode.DARKEN",
        "lighten" => "BlendMode.LIGHTEN",
        "color_dodge" => "BlendMode.COLOR_DODGE",
        "color_burn" => "BlendMode.COLOR_BURN",
        "hard_light" => "BlendMode.HARD_LIGHT",
        "soft_light" => "BlendMode.SOFT_LIGHT",
        "difference" => "BlendMode.DIFFERENCE",
        "exclusion" => "BlendMode.EXCLUSION",
        "addition" => "BlendMode.ADDITION",
        "subtract" => "BlendMode.SUBTRACT",
        "divide" => "BlendMode.DIVIDE",
        _ => "BlendMode.NORMAL",
    }
}

/// Parse animation direction string to Aseprite Lua constant.
pub fn parse_ani_dir(dir: Option<&str>) -> &'static str {
    match dir {
        Some("reverse") => "AniDir.REVERSE",
        Some("ping_pong") => "AniDir.PING_PONG",
        Some("ping_pong_reverse") => "AniDir.PING_PONG_REVERSE",
        _ => "AniDir.FORWARD",
    }
}

/// Parse selection mode string to Lua method name.
pub fn parse_selection_mode(mode: Option<&str>) -> &'static str {
    match mode {
        Some("add") => "add",
        Some("subtract") => "subtract",
        Some("intersect") => "intersect",
        _ => "select",
    }
}

/// Parse color mode string to Aseprite format string.
pub fn parse_color_mode(mode: Option<&str>) -> &'static str {
    match mode {
        Some("grayscale") => "gray",
        Some("indexed") => "indexed",
        _ => "rgb",
    }
}
