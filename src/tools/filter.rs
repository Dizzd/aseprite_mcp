use rmcp::schemars;
use serde::Deserialize;

use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BrightnessContrastParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Brightness adjustment (-100 to 100)
    pub brightness: i32,
    /// Contrast adjustment (-100 to 100)
    pub contrast: i32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HueSaturationParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Hue shift in degrees (-180 to 180)
    pub hue: i32,
    /// Saturation adjustment (-100 to 100)
    pub saturation: i32,
    /// Lightness adjustment (-100 to 100)
    pub lightness: Option<i32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InvertColorParams {
    /// Path to the sprite file
    pub file_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DespeckleParams {
    /// Path to the sprite file
    pub file_path: String,
    /// Width of the median filter matrix (default: 3)
    pub width: Option<u32>,
    /// Height of the median filter matrix (default: 3)
    pub height: Option<u32>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn brightness_contrast(
    server: &AsepriteServer,
    p: BrightnessContrastParams,
) -> Result<String, String> {
    let brightness = p.brightness.clamp(-100, 100);
    let contrast = p.contrast.clamp(-100, 100);
    let script = format!(
        r#"local spr = app.sprite
app.command.BrightnessContrast {{
    ui = false,
    brightness = {brightness},
    contrast = {contrast}
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "applied", filter = "brightness_contrast", brightness = {brightness}, contrast = {contrast}}}))"#,
        brightness = brightness,
        contrast = contrast
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn hue_saturation(
    server: &AsepriteServer,
    p: HueSaturationParams,
) -> Result<String, String> {
    let hue = p.hue.clamp(-180, 180);
    let saturation = p.saturation.clamp(-100, 100);
    let lightness = p.lightness.unwrap_or(0).clamp(-100, 100);
    let script = format!(
        r#"local spr = app.sprite
app.command.HueSaturation {{
    ui = false,
    hue = {hue},
    saturation = {saturation},
    lightness = {lightness},
    mode = "hsl"
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "applied", filter = "hue_saturation", hue = {hue}, saturation = {saturation}, lightness = {lightness}}}))"#,
        hue = hue,
        saturation = saturation,
        lightness = lightness
    );
    server.execute_script_on_file(&p.file_path, &script).await
}

pub async fn invert_color(
    server: &AsepriteServer,
    p: InvertColorParams,
) -> Result<String, String> {
    let script = r#"local spr = app.sprite
app.command.InvertColor {
    ui = false
}
spr:saveAs(spr.filename)
print(json.encode({status = "applied", filter = "invert_color"}))"#;
    server.execute_script_on_file(&p.file_path, script).await
}

pub async fn despeckle(server: &AsepriteServer, p: DespeckleParams) -> Result<String, String> {
    let width = p.width.unwrap_or(3).max(1);
    let height = p.height.unwrap_or(3).max(1);
    let script = format!(
        r#"local spr = app.sprite
app.command.Despeckle {{
    ui = false,
    width = {width},
    height = {height}
}}
spr:saveAs(spr.filename)
print(json.encode({{status = "applied", filter = "despeckle", width = {width}, height = {height}}}))"#,
        width = width,
        height = height
    );
    server.execute_script_on_file(&p.file_path, &script).await
}
