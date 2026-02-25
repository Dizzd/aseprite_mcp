use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::lua_string;
use crate::lua_helpers::LUA_FIND_LAYER;
use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListCelsParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Filter by layer name (optional)
    pub layer: Option<String>,
    /// Filter by frame number, 1-based (optional)
    pub frame: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveCelParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Layer name of the cel to move
    pub layer: String,
    /// Frame number (1-based) of the cel to move
    pub frame: u32,
    /// New X position on the canvas
    pub x: i32,
    /// New Y position on the canvas
    pub y: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetCelOpacityParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Layer name of the cel
    pub layer: String,
    /// Frame number (1-based) of the cel
    pub frame: u32,
    /// Opacity value (0-255)
    pub opacity: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClearCelParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Layer name of the cel to clear
    pub layer: String,
    /// Frame number (1-based) of the cel to clear
    pub frame: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NewCelParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Layer name where the new cel should be created
    pub layer: String,
    /// Frame number (1-based) for the new cel
    pub frame: u32,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn list_cels(server: &AsepriteServer, p: ListCelsParams) -> Result<String, String> {
    let filter_code = if let Some(ref layer) = p.layer {
        format!(
            r#"
{find_layer}
local target_layer = find_layer(spr.layers, {name})
if not target_layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end"#,
            find_layer = LUA_FIND_LAYER,
            name = lua_string(layer)
        )
    } else {
        String::new()
    };

    let frame_filter = if let Some(frame) = p.frame {
        format!("local target_frame = {}", frame)
    } else {
        "local target_frame = nil".to_string()
    };

    let script = format!(
        r#"local spr = app.sprite
{filter_code}
{frame_filter}
local cels = {{}}
for i, cel in ipairs(spr.cels) do
    local include = true
    if target_layer and cel.layer ~= target_layer then include = false end
    if target_frame and cel.frameNumber ~= target_frame then include = false end
    if include then
        local c = {{}}
        c.layer = cel.layer.name
        c.frame = cel.frameNumber
        c.x = cel.position.x
        c.y = cel.position.y
        c.width = cel.image.width
        c.height = cel.image.height
        c.opacity = cel.opacity
        c.zIndex = cel.zIndex
        if cel.data and cel.data ~= "" then c.data = cel.data end
        table.insert(cels, c)
    end
end
print(json.encode({{cels = cels, total = #cels}}))"#,
        filter_code = filter_code,
        frame_filter = frame_filter
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn move_cel(server: &AsepriteServer, p: MoveCelParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
local cel = layer:cel({frame})
if not cel then
    print(json.encode({{error = "No cel at frame " .. {frame} .. " on layer " .. {name}}}))
    return
end
cel.position = Point({x}, {y})
spr:saveAs(spr.filename)
local result = {{}}
result.layer = cel.layer.name
result.frame = cel.frameNumber
result.x = cel.position.x
result.y = cel.position.y
result.status = "moved"
print(json.encode(result))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.layer),
        frame = p.frame,
        x = p.x,
        y = p.y
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn set_cel_opacity(server: &AsepriteServer, p: SetCelOpacityParams) -> Result<String, String> {
    let opacity = p.opacity.min(255);
    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
local cel = layer:cel({frame})
if not cel then
    print(json.encode({{error = "No cel at frame " .. {frame} .. " on layer " .. {name}}}))
    return
end
cel.opacity = {opacity}
spr:saveAs(spr.filename)
local result = {{}}
result.layer = cel.layer.name
result.frame = cel.frameNumber
result.opacity = cel.opacity
result.status = "updated"
print(json.encode(result))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.layer),
        frame = p.frame,
        opacity = opacity
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn clear_cel(server: &AsepriteServer, p: ClearCelParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
local cel = layer:cel({frame})
if cel then
    spr:deleteCel(cel)
end
spr:saveAs(spr.filename)
print(json.encode({{status = "cleared", layer = {name}, frame = {frame}}}))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.layer),
        frame = p.frame
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn new_cel(server: &AsepriteServer, p: NewCelParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
local frame = spr.frames[{frame}]
if not frame then
    print(json.encode({{error = "Frame {frame} does not exist"}}))
    return
end
local cel = spr:newCel(layer, frame)
spr:saveAs(spr.filename)
local result = {{}}
result.layer = cel.layer.name
result.frame = cel.frameNumber
result.x = cel.position.x
result.y = cel.position.y
result.width = cel.image.width
result.height = cel.image.height
result.opacity = cel.opacity
result.status = "created"
print(json.encode(result))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.layer),
        frame = p.frame
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
