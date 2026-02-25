use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::aseprite::lua_string;
use crate::lua_helpers::{LUA_FIND_LAYER, lua_select_layer};
use crate::server::AsepriteServer;
use crate::utils::{parse_hex_color_with_alpha, validate_hex_color};

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DrawPixelsParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Array of pixel data: [{"x": 0, "y": 0, "color": "#ff0000"}, ...]
    pub pixels: Vec<PixelData>,
    /// Target layer name (if omitted, uses active layer)
    pub layer: Option<String>,
    /// Target frame number, 1-based (if omitted, uses frame 1)
    pub frame: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PixelData {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Color as hex string (e.g. "#ff0000", "#ff000080" with alpha)
    pub color: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UseToolParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Tool name: "pencil", "line", "rectangle", "filled_rectangle", "ellipse",
    /// "filled_ellipse", "paint_bucket", "spray", "eraser", "contour", "polygon"
    pub tool: String,
    /// Array of points: [{"x": 0, "y": 0}, ...] defining the tool stroke
    pub points: Vec<PointData>,
    /// Foreground color as hex string (e.g. "#ff0000")
    pub color: String,
    /// Brush size (default: 1)
    pub brush_size: Option<u32>,
    /// Opacity 0-255 (default: 255)
    pub opacity: Option<u32>,
    /// Target layer name (if omitted, uses active layer)
    pub layer: Option<String>,
    /// Target frame number, 1-based (if omitted, uses frame 1)
    pub frame: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PointData {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPixelDataParams {
    /// Path to the sprite file
    pub file_path: String,
    /// X coordinate of the region start
    pub x: u32,
    /// Y coordinate of the region start
    pub y: u32,
    /// Width of the region to read
    pub width: u32,
    /// Height of the region to read
    pub height: u32,
    /// Target layer name (if omitted, uses flattened image)
    pub layer: Option<String>,
    /// Target frame number, 1-based (if omitted, uses frame 1)
    pub frame: Option<u32>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn draw_pixels(server: &AsepriteServer, p: DrawPixelsParams) -> Result<String, String> {
    if p.pixels.is_empty() {
        return Err("Pixels array cannot be empty".to_string());
    }
    for px in &p.pixels {
        validate_hex_color(&px.color)
            .map_err(|e| format!("Invalid pixel color '{}': {}", px.color, e))?;
    }
    let frame_num = p.frame.unwrap_or(1);

    let layer_select = if let Some(ref layer_name) = p.layer {
        format!("{}{}", LUA_FIND_LAYER, lua_select_layer(layer_name, true))
    } else {
        String::new()
    };

    // Build pixel drawing code using Image:drawPixel for much better performance
    // than calling app.useTool per pixel
    let mut pixel_code = String::new();
    for px in &p.pixels {
        let (r, g, b, a) = parse_hex_color_with_alpha(&px.color);
        pixel_code.push_str(&format!(
            "    img:drawPixel({}, {}, app.pixelColor.rgba({}, {}, {}, {}))\n",
            px.x, px.y, r, g, b, a
        ));
    }

    let script = format!(
        r#"local spr = app.sprite
app.frame = spr.frames[{frame}]
{layer_select}

app.transaction("Draw Pixels", function()
    local cel = app.cel
    if not cel then
        cel = spr:newCel(app.layer, app.frame)
    end
    local img = cel.image
    local pos = cel.position
{pixel_code}
end)
spr:saveAs(spr.filename)
print(json.encode({{status = "drawn", pixelCount = {count}}}))"#,
        frame = frame_num,
        layer_select = layer_select,
        pixel_code = pixel_code,
        count = p.pixels.len()
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn use_tool(server: &AsepriteServer, p: UseToolParams) -> Result<String, String> {
    if p.points.is_empty() {
        return Err("Points array cannot be empty".to_string());
    }
    validate_hex_color(&p.color).map_err(|e| format!("Invalid color '{}': {}", p.color, e))?;
    let frame_num = p.frame.unwrap_or(1);
    let brush_size = p.brush_size.unwrap_or(1);
    let opacity = p.opacity.unwrap_or(255).min(255);
    let (r, g, b, a) = parse_hex_color_with_alpha(&p.color);

    let points_lua: Vec<String> = p
        .points
        .iter()
        .map(|pt| format!("Point({}, {})", pt.x, pt.y))
        .collect();
    let points_str = points_lua.join(", ");

    let layer_select = if let Some(ref layer_name) = p.layer {
        format!("{}{}", LUA_FIND_LAYER, lua_select_layer(layer_name, false))
    } else {
        String::new()
    };

    let script = format!(
        r#"local spr = app.sprite
app.frame = spr.frames[{frame}]
{layer_select}

app.transaction("Use Tool", function()
    app.useTool{{
        tool = {tool},
        color = Color({r}, {g}, {b}, {a}),
        brush = Brush({{size = {bs}}}),
        points = {{ {points} }},
        opacity = {opacity},
        cel = app.cel
    }}
end)
spr:saveAs(spr.filename)
print(json.encode({{status = "drawn", tool = {tool}}}))"#,
        frame = frame_num,
        layer_select = layer_select,
        tool = lua_string(&p.tool),
        r = r,
        g = g,
        b = b,
        a = a,
        bs = brush_size,
        points = points_str,
        opacity = opacity
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn get_pixel_data(server: &AsepriteServer, p: GetPixelDataParams) -> Result<String, String> {
    let frame_num = p.frame.unwrap_or(1);

    let image_source = if let Some(ref layer_name) = p.layer {
        format!(
            r#"
{find_layer}
local target_layer = find_layer(spr.layers, {name})
if not target_layer then
    print(json.encode({{error = "Layer not found"}}))
    return
end
local cel = target_layer:cel(spr.frames[{frame}])
if not cel then
    print(json.encode({{error = "No cel at this frame"}}))
    return
end
local img = cel.image
local offX = cel.position.x
local offY = cel.position.y"#,
            find_layer = LUA_FIND_LAYER,
            name = lua_string(layer_name),
            frame = frame_num
        )
    } else {
        format!(
            r#"
local flat = Image(spr.spec)
flat:drawSprite(spr, {frame})
local img = flat
local offX = 0
local offY = 0"#,
            frame = frame_num
        )
    };

    let script = format!(
        r##"local spr = app.sprite
{image_source}

local pixels = {{}}
for py = {y}, {y} + {h} - 1 do
    for px = {x}, {x} + {w} - 1 do
        local ix = px - offX
        local iy = py - offY
        local p = {{}}
        p.x = px
        p.y = py
        if ix >= 0 and ix < img.width and iy >= 0 and iy < img.height then
            local pv = img:getPixel(ix, iy)
            local r = app.pixelColor.rgbaR(pv)
            local g = app.pixelColor.rgbaG(pv)
            local b = app.pixelColor.rgbaB(pv)
            local a = app.pixelColor.rgbaA(pv)
            p.color = string.format("#%02x%02x%02x%02x", r, g, b, a)
        else
            p.color = "#00000000"
        end
        table.insert(pixels, p)
    end
end
print(json.encode({{pixels = pixels, width = {w}, height = {h}}}))"##,
        image_source = image_source,
        x = p.x,
        y = p.y,
        w = p.width,
        h = p.height
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
