use rmcp::schemars;
use serde::Deserialize;

use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddFrameParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Number of frames to add (default: 1)
    pub count: Option<u32>,
    /// If true, add empty frames instead of copying the current frame
    pub empty: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveFrameParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Frame number to remove (1-based)
    pub frame_number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetFrameDurationParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Frame number (1-based)
    pub frame_number: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn list_frames(server: &AsepriteServer, file_path: &str) -> Result<String, String> {
    let script = r#"local spr = app.sprite
local frames = {}
for i, frame in ipairs(spr.frames) do
    local f = {}
    f.frameNumber = frame.frameNumber
    f.duration = frame.duration
    table.insert(frames, f)
end
print(json.encode({frames = frames, total = #frames}))"#;
    server.execute_script_on_file(file_path, script).await
}

pub async fn add_frame(server: &AsepriteServer, p: AddFrameParams) -> Result<String, String> {
    let count = p.count.unwrap_or(1);
    let empty = p.empty.unwrap_or(false);
    let frame_fn = if empty { "newEmptyFrame" } else { "newFrame" };

    let script = format!(
        r#"local spr = app.sprite
for i = 1, {count} do
    spr:{frame_fn}(#spr.frames + 1)
end
spr:saveAs(spr.filename)
print(json.encode({{status = "added", count = {count}, totalFrames = #spr.frames}}))"#,
        count = count,
        frame_fn = frame_fn
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn remove_frame(server: &AsepriteServer, p: RemoveFrameParams) -> Result<String, String> {
    let frame_num = p.frame_number;
    let script = format!(
        r#"local spr = app.sprite
if {fnum} > #spr.frames then
    print(json.encode({{error = "Frame number out of range"}}))
    return
end
spr:deleteFrame({fnum})
spr:saveAs(spr.filename)
print(json.encode({{status = "deleted", frameNumber = {fnum}, totalFrames = #spr.frames}}))"#,
        fnum = frame_num
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn set_frame_duration(server: &AsepriteServer, p: SetFrameDurationParams) -> Result<String, String> {
    let duration_sec = p.duration_ms as f64 / 1000.0;
    let script = format!(
        r#"local spr = app.sprite
local frame = spr.frames[{frame}]
if not frame then
    print(json.encode({{error = "Frame not found"}}))
    return
end
frame.duration = {dur}
spr:saveAs(spr.filename)
print(json.encode({{status = "updated", frameNumber = {frame}, duration = {dur}}}))"#,
        frame = p.frame_number,
        dur = duration_sec
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
