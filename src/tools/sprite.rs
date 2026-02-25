use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::{lua_path, lua_string};
use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSpriteParams {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Output file path (e.g. "my_sprite.aseprite", "art/player.png")
    pub output_path: String,
    /// Color mode: "rgb", "grayscale", or "indexed" (default: "rgb")
    pub color_mode: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SpriteFileParams {
    /// Path to the sprite file
    pub file_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ResizeSpriteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// New width in pixels
    pub width: u32,
    /// New height in pixels
    pub height: u32,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CropSpriteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// X coordinate of the crop region
    pub x: i32,
    /// Y coordinate of the crop region
    pub y: i32,
    /// Width of the crop region
    pub width: u32,
    /// Height of the crop region
    pub height: u32,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlipSpriteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Flip direction: "horizontal" or "vertical"
    pub direction: String,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RotateSpriteParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Rotation angle in degrees (90, 180, or 270)
    pub angle: u32,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CanvasSizeParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Left padding (positive=expand, negative=shrink)
    pub left: i32,
    /// Top padding
    pub top: i32,
    /// Right padding
    pub right: i32,
    /// Bottom padding
    pub bottom: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DuplicateSpriteParams {
    /// Path to the source sprite file
    pub file_path: String,
    /// Path to save the duplicate (e.g. "player_copy.aseprite")
    pub output_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AutoCropParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChangeColorModeParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Target color mode: "rgb", "grayscale", or "indexed"
    pub color_mode: String,
    /// Save to a different path (if omitted, overwrites the original)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReverseFramesParams {
    /// Path to the sprite file
    pub file_path: String,
    /// First frame number (1-based) of the range to reverse. Defaults to 1.
    pub from_frame: Option<u32>,
    /// Last frame number (1-based) of the range to reverse. Defaults to last frame.
    pub to_frame: Option<u32>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn create_sprite(server: &AsepriteServer, p: CreateSpriteParams) -> Result<String, String> {
    if p.width == 0 || p.height == 0 {
        return Err("Width and height must be greater than 0".to_string());
    }
    if p.output_path.trim().is_empty() {
        return Err("Output path cannot be empty".to_string());
    }
    let color_mode = match p.color_mode.as_deref() {
        Some("grayscale") => "ColorMode.GRAYSCALE",
        Some("indexed") => "ColorMode.INDEXED",
        _ => "ColorMode.RGB",
    };
    let output = lua_path(&server.resolve_output_path(&p.output_path));

    let script = format!(
        r#"local spr = Sprite({w}, {h}, {cm})
spr:saveAs({out})
local result = {{}}
result.width = spr.width
result.height = spr.height
result.filename = spr.filename
result.colorMode = tostring(spr.colorMode)
print(json.encode(result))"#,
        w = p.width,
        h = p.height,
        cm = color_mode,
        out = output,
    );

    server.execute_script(&script).await
}

pub async fn get_sprite_info(server: &AsepriteServer, p: SpriteFileParams) -> Result<String, String> {
    let script = r#"local spr = app.sprite
if not spr then
    print(json.encode({error = "No sprite loaded"}))
    return
end

local layers = {}
local function collect_layers(lyrs, depth)
    for i, layer in ipairs(lyrs) do
        local l = {}
        l.name = layer.name
        l.isVisible = layer.isVisible
        l.isEditable = layer.isEditable
        l.isGroup = layer.isGroup
        l.stackIndex = layer.stackIndex
        l.depth = depth
        if layer.opacity then l.opacity = layer.opacity end
        if layer.blendMode then l.blendMode = tostring(layer.blendMode) end
        l.isTilemap = layer.isTilemap or false
        l.isBackground = layer.isBackground or false
        l.isReference = layer.isReference or false
        table.insert(layers, l)
        if layer.isGroup and layer.layers then
            collect_layers(layer.layers, depth + 1)
        end
    end
end
collect_layers(spr.layers, 0)

local frames = {}
for i, frame in ipairs(spr.frames) do
    local f = {}
    f.frameNumber = frame.frameNumber
    f.duration = frame.duration
    table.insert(frames, f)
end

local tags = {}
for i, tag in ipairs(spr.tags) do
    local t = {}
    t.name = tag.name
    t.fromFrame = tag.fromFrame.frameNumber
    t.toFrame = tag.toFrame.frameNumber
    t.frames = tag.frames
    t.aniDir = tostring(tag.aniDir)
    t.repeats = tag.repeats
    table.insert(tags, t)
end

local slices = {}
for i, slice in ipairs(spr.slices) do
    local s = {}
    s.name = slice.name
    if slice.bounds then
        s.bounds = {
            x = slice.bounds.x,
            y = slice.bounds.y,
            width = slice.bounds.width,
            height = slice.bounds.height
        }
    end
    table.insert(slices, s)
end

local pal = spr.palettes[1]
local paletteSize = pal and #pal or 0

local result = {}
result.filename = spr.filename
result.width = spr.width
result.height = spr.height
result.colorMode = tostring(spr.colorMode)
result.numFrames = #spr.frames
result.numLayers = #layers
result.numCels = #spr.cels
result.numTags = #spr.tags
result.numSlices = #spr.slices
result.paletteSize = paletteSize
result.isModified = spr.isModified
result.gridBounds = {
    x = spr.gridBounds.x,
    y = spr.gridBounds.y,
    width = spr.gridBounds.width,
    height = spr.gridBounds.height
}
result.pixelRatio = {
    width = spr.pixelRatio.width,
    height = spr.pixelRatio.height
}
result.layers = layers
result.frames = frames
result.tags = tags
result.slices = slices
print(json.encode(result))"#;

    server.execute_script_on_file(&p.file_path, script).await
}

pub async fn resize_sprite(server: &AsepriteServer, p: ResizeSpriteParams) -> Result<String, String> {
    let output = server.resolve_output_path(p.output_path.as_deref().unwrap_or(&p.file_path));
    let script = format!(
        r#"local spr = app.sprite
spr:resize({w}, {h})
spr:saveCopyAs({out})
local result = {{}}
result.width = spr.width
result.height = spr.height
result.filename = {out}
result.status = "resized"
print(json.encode(result))"#,
        w = p.width,
        h = p.height,
        out = lua_path(&output)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn crop_sprite(server: &AsepriteServer, p: CropSpriteParams) -> Result<String, String> {
    let output = server.resolve_output_path(p.output_path.as_deref().unwrap_or(&p.file_path));
    let script = format!(
        r#"local spr = app.sprite
spr:crop({x}, {y}, {w}, {h})
spr:saveCopyAs({out})
local result = {{}}
result.width = spr.width
result.height = spr.height
result.status = "cropped"
print(json.encode(result))"#,
        x = p.x,
        y = p.y,
        w = p.width,
        h = p.height,
        out = lua_path(&output)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn flip_sprite(server: &AsepriteServer, p: FlipSpriteParams) -> Result<String, String> {
    let output = server.resolve_output_path(p.output_path.as_deref().unwrap_or(&p.file_path));
    match p.direction.to_lowercase().as_str() {
        "horizontal" | "vertical" => {}
        _ => return Err("direction must be 'horizontal' or 'vertical'".to_string()),
    };
    let script = format!(
        r#"local spr = app.sprite
app.command.Flip {{
    ui = false,
    target = "canvas",
    orientation = {orient}
}}
spr:saveCopyAs({out})
print(json.encode({{status = "flipped", direction = {dir}}}))"#,
        orient = lua_string(match p.direction.to_lowercase().as_str() {
            "horizontal" => "horizontal",
            _ => "vertical",
        }),
        out = lua_path(&output),
        dir = lua_string(&p.direction)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn rotate_sprite(server: &AsepriteServer, p: RotateSpriteParams) -> Result<String, String> {
    let output = server.resolve_output_path(p.output_path.as_deref().unwrap_or(&p.file_path));
    if p.angle != 90 && p.angle != 180 && p.angle != 270 {
        return Err("angle must be 90, 180, or 270".to_string());
    }
    let script = format!(
        r#"local spr = app.sprite
app.command.Rotate {{
    ui = false,
    angle = {angle},
    rotsprite = false
}}
spr:saveCopyAs({out})
print(json.encode({{status = "rotated", angle = {angle}, width = spr.width, height = spr.height}}))"#,
        angle = p.angle,
        out = lua_path(&output)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn canvas_size(server: &AsepriteServer, p: CanvasSizeParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
local newW = spr.width + {left} + {right}
local newH = spr.height + {top} + {bottom}
app.command.CanvasSize {{
    ui = false,
    left = {left},
    top = {top},
    right = {right},
    bottom = {bottom}
}}
spr:saveAs(spr.filename)
local result = {{}}
result.width = spr.width
result.height = spr.height
result.status = "canvas_resized"
print(json.encode(result))"#,
        left = p.left,
        top = p.top,
        right = p.right,
        bottom = p.bottom
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn duplicate_sprite(server: &AsepriteServer, p: DuplicateSpriteParams) -> Result<String, String> {
    let output = lua_path(&server.resolve_output_path(&p.output_path));
    let script = format!(
        r#"local spr = app.sprite
local copy = Sprite(spr)
copy:saveAs({out})
local result = {{}}
result.width = copy.width
result.height = copy.height
result.filename = copy.filename
result.numLayers = #copy.layers
result.numFrames = #copy.frames
result.status = "duplicated"
print(json.encode(result))"#,
        out = output
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn auto_crop_sprite(server: &AsepriteServer, p: AutoCropParams) -> Result<String, String> {
    let save_code = if let Some(ref output) = p.output_path {
        let out = lua_path(&server.resolve_output_path(output));
        format!("spr:saveCopyAs({})", out)
    } else {
        "spr:saveAs(spr.filename)".to_string()
    };

    let script = format!(
        r#"local spr = app.sprite
local oldW, oldH = spr.width, spr.height
app.command.AutocropSprite()
{save}
local result = {{}}
result.oldWidth = oldW
result.oldHeight = oldH
result.width = spr.width
result.height = spr.height
result.status = "auto_cropped"
print(json.encode(result))"#,
        save = save_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn change_color_mode(server: &AsepriteServer, p: ChangeColorModeParams) -> Result<String, String> {
    let format_str = match p.color_mode.to_lowercase().as_str() {
        "rgb" => "rgb",
        "grayscale" => "gray",
        "indexed" => "indexed",
        _ => return Err("color_mode must be 'rgb', 'grayscale', or 'indexed'".to_string()),
    };

    let save_code = if let Some(ref output) = p.output_path {
        let out = lua_path(&server.resolve_output_path(output));
        format!("spr:saveCopyAs({})", out)
    } else {
        "spr:saveAs(spr.filename)".to_string()
    };

    let script = format!(
        r#"local spr = app.sprite
app.command.ChangePixelFormat {{
    ui = false,
    format = "{format}"
}}
{save}
local result = {{}}
result.colorMode = tostring(spr.colorMode)
result.width = spr.width
result.height = spr.height
result.status = "color_mode_changed"
print(json.encode(result))"#,
        format = format_str,
        save = save_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn reverse_frames(server: &AsepriteServer, p: ReverseFramesParams) -> Result<String, String> {
    let from = p.from_frame.unwrap_or(1);
    let to_code = if let Some(to) = p.to_frame {
        format!("local toFrame = {}", to)
    } else {
        "local toFrame = #spr.frames".to_string()
    };

    let script = format!(
        r#"local spr = app.sprite
local fromFrame = {from}
{to_code}
-- Select the frame range
app.frame = spr.frames[fromFrame]
local range = app.range
range:clear()
for i = fromFrame, toFrame do
    range:contains(spr.frames[i])
end
app.command.ReverseFrames()
spr:saveAs(spr.filename)
local result = {{}}
result.fromFrame = fromFrame
result.toFrame = toFrame
result.numFrames = #spr.frames
result.status = "reversed"
print(json.encode(result))"#,
        from = from,
        to_code = to_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
