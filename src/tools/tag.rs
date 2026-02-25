use rmcp::schemars;
use serde::Deserialize;

use crate::aseprite::lua_string;
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
// Tool Implementations
// ============================================================================

pub async fn list_tags(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
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
    server.execute_script_on_file(file_path, script).await
}

pub async fn create_tag(server: &AsepriteServer, p: CreateTagParams) -> Result<String, String> {
    let ani_dir = match p.ani_dir.as_deref() {
        Some("reverse") => "AniDir.REVERSE",
        Some("ping_pong") => "AniDir.PING_PONG",
        Some("ping_pong_reverse") => "AniDir.PING_PONG_REVERSE",
        _ => "AniDir.FORWARD",
    };
    let color_code = if let Some(ref color) = p.color {
        let (r, g, b) = parse_hex_color(color);
        format!("tag.color = Color({}, {}, {})\n", r, g, b)
    } else {
        String::new()
    };

    let script = format!(
        r#"local spr = app.sprite
local tag = spr:newTag({from}, {to})
tag.name = {name}
tag.aniDir = {ani}
{color}
spr:saveAs(spr.filename)
local result = {{}}
result.name = tag.name
result.fromFrame = tag.fromFrame.frameNumber
result.toFrame = tag.toFrame.frameNumber
result.aniDir = tostring(tag.aniDir)
result.status = "created"
print(json.encode(result))"#,
        from = p.from_frame,
        to = p.to_frame,
        name = lua_string(&p.name),
        ani = ani_dir,
        color = color_code
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn delete_tag(server: &AsepriteServer, p: DeleteTagParams) -> Result<String, String> {
    let script = format!(
        r#"local spr = app.sprite
spr:deleteTag({name})
spr:saveAs(spr.filename)
print(json.encode({{status = "deleted", tag = {name}}}))"#,
        name = lua_string(&p.name)
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
