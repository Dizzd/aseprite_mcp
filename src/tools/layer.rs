use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::{lua_path, lua_string};
use crate::lua_helpers::LUA_FIND_LAYER;
use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DuplicateLayerParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name of the layer to duplicate
    pub name: String,
    /// Name for the duplicated layer (optional, defaults to "name Copy")
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeDownLayerParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name of the upper layer to merge down into the layer below
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlattenLayersParams {
    /// Path to the sprite file
    pub file_path: String,
    /// If true, save to a different path instead of overwriting
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddLayerParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name for the new layer
    pub name: String,
    /// Create a group layer instead of a normal layer (default: false)
    pub is_group: Option<bool>,
    /// Insert after this layer name (if omitted, adds at top)
    pub after_layer: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveLayerParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name of the layer to remove
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetLayerPropertyParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name of the layer to modify
    pub name: String,
    /// New name for the layer
    pub new_name: Option<String>,
    /// Set visibility (true=visible, false=hidden)
    pub visible: Option<bool>,
    /// Set opacity (0-255)
    pub opacity: Option<u32>,
    /// Set blend mode ("normal", "multiply", "screen", "overlay", "darken", "lighten", etc.)
    pub blend_mode: Option<String>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn list_layers(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
local layers = {}
local function collect(lyrs, depth, parent_name)
    for i, layer in ipairs(lyrs) do
        local l = {}
        l.name = layer.name
        l.isVisible = layer.isVisible
        l.isEditable = layer.isEditable
        l.isGroup = layer.isGroup
        l.stackIndex = layer.stackIndex
        l.depth = depth
        l.parent = parent_name
        if layer.opacity then l.opacity = layer.opacity end
        if layer.blendMode then l.blendMode = tostring(layer.blendMode) end
        l.isBackground = layer.isBackground or false
        l.isTilemap = layer.isTilemap or false
        l.numCels = #layer.cels
        table.insert(layers, l)
        if layer.isGroup and layer.layers then
            collect(layer.layers, depth + 1, layer.name)
        end
    end
end
collect(spr.layers, 0, nil)
print(json.encode({layers = layers, total = #layers}))"#;
    server.execute_script_on_file(file_path, script).await
}

pub async fn add_layer(server: &AsepriteServer, p: AddLayerParams) -> Result<String, String> {
    let is_group = p.is_group.unwrap_or(false);
    let create_fn = if is_group { "newGroup" } else { "newLayer" };
    let after_code = if let Some(ref after) = p.after_layer {
        format!(
            r#"
local target = nil
for i, l in ipairs(spr.layers) do
    if l.name == {name} then target = l break end
end
if target then
    new_layer.stackIndex = target.stackIndex + 1
end"#,
            name = lua_string(after)
        )
    } else {
        String::new()
    };

    let script = format!(
        r#"local spr = app.sprite
local new_layer = spr:{create_fn}()
new_layer.name = {name}
{after_code}
spr:saveAs(spr.filename)
local result = {{}}
result.name = new_layer.name
result.isGroup = new_layer.isGroup
result.stackIndex = new_layer.stackIndex
result.status = "created"
print(json.encode(result))"#,
        create_fn = create_fn,
        name = lua_string(&p.name),
        after_code = after_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn remove_layer(server: &AsepriteServer, p: RemoveLayerParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
spr:deleteLayer({name})
spr:saveAs(spr.filename)
print(json.encode({{status = "deleted", layer = {name}}}))"#,
        name = lua_string(&p.name)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn set_layer_property(server: &AsepriteServer, p: SetLayerPropertyParams) -> Result<String, String> {
    let mut property_code = String::new();

    if let Some(ref new_name) = p.new_name {
        property_code.push_str(&format!("    layer.name = {}\n", lua_string(new_name)));
    }
    if let Some(visible) = p.visible {
        property_code.push_str(&format!(
            "    layer.isVisible = {}\n",
            if visible { "true" } else { "false" }
        ));
    }
    if let Some(opacity) = p.opacity {
        property_code.push_str(&format!("    layer.opacity = {}\n", opacity.min(255)));
    }
    if let Some(ref blend_mode) = p.blend_mode {
        let bm = match blend_mode.to_lowercase().as_str() {
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
        };
        property_code.push_str(&format!("    layer.blendMode = {}\n", bm));
    }

    if property_code.is_empty() {
        return Err("No properties specified to change".to_string());
    }

    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if layer then
{props}
    spr:saveAs(spr.filename)
    local result = {{}}
    result.name = layer.name
    result.isVisible = layer.isVisible
    if layer.opacity then result.opacity = layer.opacity end
    if layer.blendMode then result.blendMode = tostring(layer.blendMode) end
    result.status = "updated"
    print(json.encode(result))
else
    print(json.encode({{error = "Layer not found: " .. {name}}}))
end"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.name),
        props = property_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn duplicate_layer(server: &AsepriteServer, p: DuplicateLayerParams) -> Result<String, String> {
    let rename_code = if let Some(ref new_name) = p.new_name {
        format!("app.layer.name = {}", lua_string(new_name))
    } else {
        String::new()
    };

    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
app.layer = layer
app.command.DuplicateLayer()
{rename}
spr:saveAs(spr.filename)
local result = {{}}
result.name = app.layer.name
result.isGroup = app.layer.isGroup
result.stackIndex = app.layer.stackIndex
result.status = "duplicated"
print(json.encode(result))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.name),
        rename = rename_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn merge_down_layer(server: &AsepriteServer, p: MergeDownLayerParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
{find_layer}
local layer = find_layer(spr.layers, {name})
if not layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
app.layer = layer
app.command.MergeDownLayer()
spr:saveAs(spr.filename)
local result = {{}}
result.name = app.layer.name
result.status = "merged"
print(json.encode(result))"#,
        find_layer = LUA_FIND_LAYER,
        name = lua_string(&p.name)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn flatten_layers(server: &AsepriteServer, p: FlattenLayersParams) -> Result<String, String> {
    let save_code = if let Some(ref output) = p.output_path {
        let out = lua_path(&server.resolve_output_path(output));
        format!("spr:saveCopyAs({})", out)
    } else {
        "spr:saveAs(spr.filename)".to_string()
    };

    let script = format!(
        r#"local spr = app.sprite
app.command.FlattenLayers()
{save}
local result = {{}}
result.numLayers = #spr.layers
result.status = "flattened"
print(json.encode(result))"#,
        save = save_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
