use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::lua_string;
use crate::lua_helpers::parse_ani_dir;
use crate::server::AsepriteServer;
use crate::utils::parse_hex_color;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTagParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Tag name
    pub name: String,
    /// First frame number (1-based)
    pub from_frame: u32,
    /// Last frame number (1-based)
    pub to_frame: u32,
    /// Animation direction: "forward", "reverse", "ping_pong", "ping_pong_reverse" (default: "forward")
    pub ani_dir: Option<String>,
    /// Tag color as hex string (e.g. "#ff0000")
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteTagParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Tag name to delete
    pub name: String,
}

// ============================================================================
// Common Lua Scripts
// ============================================================================

const LUA_LIST_TAGS: &str = r#"local spr = app.sprite
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
print(json.encode({tags = tags, total = #tags}))"#;

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn list_tags(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    server.execute_script_on_file(file_path, LUA_LIST_TAGS).await
}

pub async fn create_tag(server: &AsepriteServer, p: CreateTagParams) -> Result<String, String> {
    let ani_dir = parse_ani_dir(p.ani_dir.as_deref());
    let color_code = p.color.as_ref().map(|color| {
        let (r, g, b) = parse_hex_color(color);
        format!("tag.color = Color({}, {}, {})\n", r, g, b)
    }).unwrap_or_default();

    let script = format!(
        r#"local spr = app.sprite
local tag = spr:newTag({}, {})
tag.name = {}
tag.aniDir = {}
{}
spr:saveAs(spr.filename)
local result = {{}}
result.name = tag.name
result.fromFrame = tag.fromFrame.frameNumber
result.toFrame = tag.toFrame.frameNumber
result.aniDir = tostring(tag.aniDir)
result.status = "created"
print(json.encode(result))"#,
        p.from_frame,
        p.to_frame,
        lua_string(&p.name),
        ani_dir,
        color_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn delete_tag(server: &AsepriteServer, p: DeleteTagParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
spr:deleteTag({})
spr:saveAs(spr.filename)
print(json.encode({{status = "deleted", tag = {}}}))"#,
        lua_string(&p.name),
        lua_string(&p.name)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
