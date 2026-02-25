use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::lua_string;
use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSliceParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name for the new slice
    pub name: String,
    /// X coordinate of slice bounds
    pub x: i32,
    /// Y coordinate of slice bounds
    pub y: i32,
    /// Width of slice bounds
    pub width: u32,
    /// Height of slice bounds
    pub height: u32,
    /// 9-slice center rectangle (for UI scaling). Format: {x, y, width, height} relative to slice bounds.
    pub center: Option<SliceRect>,
    /// Pivot point for the slice (anchor point for game engines). Format: {x, y} relative to slice bounds.
    pub pivot: Option<SlicePoint>,
    /// User-defined color for the slice in hex (e.g. "#ff0000")
    pub color: Option<String>,
    /// User-defined data string (can store JSON metadata for game engines)
    pub data: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SliceRect {
    /// X offset relative to slice bounds
    pub x: i32,
    /// Y offset relative to slice bounds
    pub y: i32,
    /// Width of center rectangle
    pub width: u32,
    /// Height of center rectangle
    pub height: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SlicePoint {
    /// X coordinate of pivot
    pub x: i32,
    /// Y coordinate of pivot
    pub y: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteSliceParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Name of the slice to delete
    pub name: String,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn list_slices(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r##"local spr = app.sprite
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
    if slice.center then
        s.center = {
            x = slice.center.x,
            y = slice.center.y,
            width = slice.center.width,
            height = slice.center.height
        }
    end
    if slice.pivot then
        s.pivot = {
            x = slice.pivot.x,
            y = slice.pivot.y
        }
    end
    if slice.color then
        s.color = string.format("#%02x%02x%02x%02x", slice.color.red, slice.color.green, slice.color.blue, slice.color.alpha)
    end
    if slice.data and slice.data ~= "" then
        s.data = slice.data
    end
    table.insert(slices, s)
end
print(json.encode({slices = slices, total = #slices}))"##;
    server.execute_script_on_file(file_path, script).await
}

pub async fn create_slice(server: &AsepriteServer, p: CreateSliceParams) -> Result<String, String> {
    let mut extra_code = String::new();

    if let Some(ref center) = p.center {
        extra_code.push_str(&format!(
            "slice.center = Rectangle({}, {}, {}, {})\n",
            center.x, center.y, center.width, center.height
        ));
    }
    if let Some(ref pivot) = p.pivot {
        extra_code.push_str(&format!(
            "slice.pivot = Point({}, {})\n",
            pivot.x, pivot.y
        ));
    }
    if let Some(ref color) = p.color {
        let color_clean = color.trim_start_matches('#');
        if color_clean.len() >= 6 {
            let r = u8::from_str_radix(&color_clean[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&color_clean[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&color_clean[4..6], 16).unwrap_or(0);
            let a = if color_clean.len() >= 8 {
                u8::from_str_radix(&color_clean[6..8], 16).unwrap_or(255)
            } else {
                255
            };
            extra_code.push_str(&format!(
                "slice.color = Color({}, {}, {}, {})\n",
                r, g, b, a
            ));
        }
    }
    if let Some(ref data) = p.data {
        extra_code.push_str(&format!("slice.data = {}\n", lua_string(data)));
    }

    let script = format!(
        r#"local spr = app.sprite
local slice = spr:newSlice(Rectangle({x}, {y}, {w}, {h}))
slice.name = {name}
{extra}
spr:saveAs(spr.filename)
local result = {{}}
result.name = slice.name
result.bounds = {{
    x = slice.bounds.x,
    y = slice.bounds.y,
    width = slice.bounds.width,
    height = slice.bounds.height
}}
if slice.center then
    result.center = {{
        x = slice.center.x,
        y = slice.center.y,
        width = slice.center.width,
        height = slice.center.height
    }}
end
if slice.pivot then
    result.pivot = {{
        x = slice.pivot.x,
        y = slice.pivot.y
    }}
end
result.status = "created"
print(json.encode(result))"#,
        x = p.x,
        y = p.y,
        w = p.width,
        h = p.height,
        name = lua_string(&p.name),
        extra = extra_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn delete_slice(server: &AsepriteServer, p: DeleteSliceParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
spr:deleteSlice({name})
spr:saveAs(spr.filename)
print(json.encode({{status = "deleted", slice = {name}}}))"#,
        name = lua_string(&p.name)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
