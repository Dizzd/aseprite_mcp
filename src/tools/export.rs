use rmcp::schemars;
use serde::Deserialize;

use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportSpriteParams {
    /// Path to the input sprite file
    pub file_path: String,
    /// Output file path with desired format extension (e.g. "output.png", "output.gif")
    pub output_path: String,
    /// Scale factor (e.g. 2 for 2x size)
    pub scale: Option<u32>,
    /// Specific layer name to export (if omitted, exports all visible layers)
    pub layer: Option<String>,
    /// Specific animation tag to export (if omitted, exports all frames)
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportSpritesheetParams {
    /// Path to the input sprite file
    pub file_path: String,
    /// Output image path for the spritesheet (e.g. "sheet.png")
    pub output_image: String,
    /// Output JSON data path (e.g. "sheet.json")
    pub output_data: Option<String>,
    /// Sheet type: "horizontal", "vertical", "rows", "columns", "packed" (default: "rows")
    pub sheet_type: Option<String>,
    /// Number of columns (for "rows" type)
    pub columns: Option<u32>,
    /// Whether to trim empty space from each frame
    pub trim: Option<bool>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn export_sprite(server: &AsepriteServer, p: ExportSpriteParams) -> Result<String, String> {
    let mut args = vec![p.file_path.clone()];
    if let Some(scale) = p.scale {
        args.push("--scale".to_string());
        args.push(scale.to_string());
    }
    if let Some(ref layer) = p.layer {
        args.push("--layer".to_string());
        args.push(layer.clone());
    }
    if let Some(ref tag) = p.tag {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }
    args.push("--save-as".to_string());
    let resolved_output = server.resolve_output_path(&p.output_path);
    args.push(resolved_output.clone());

    match server.run_cli(&args).await {
        Ok(output) => {
            if output.success {
                Ok(format!(
                    "Exported {} -> {}",
                    p.file_path, resolved_output
                ))
            } else {
                Err(output.result_text())
            }
        }
        Err(e) => Err(format!("Export failed: {}", e)),
    }
}

pub async fn export_spritesheet(server: &AsepriteServer, p: ExportSpritesheetParams) -> Result<String, String> {
    let resolved_image = server.resolve_output_path(&p.output_image);
    let resolved_data = p.output_data.as_ref().map(|d| server.resolve_output_path(d));
    let mut args = vec![p.file_path.clone(), "--sheet".to_string(), resolved_image.clone()];

    if let Some(ref data_path) = resolved_data {
        args.push("--data".to_string());
        args.push(data_path.clone());
    }
    if let Some(ref sheet_type) = p.sheet_type {
        args.push("--sheet-type".to_string());
        args.push(sheet_type.clone());
    }
    if let Some(columns) = p.columns {
        args.push("--sheet-columns".to_string());
        args.push(columns.to_string());
    }
    if p.trim.unwrap_or(false) {
        args.push("--trim".to_string());
    }

    match server.run_cli(&args).await {
        Ok(output) => {
            if output.success {
                Ok(format!(
                    "Spritesheet exported: {}{}",
                    resolved_image,
                    resolved_data
                        .map(|d| format!(", data: {}", d))
                        .unwrap_or_default()
                ))
            } else {
                Err(output.result_text())
            }
        }
        Err(e) => Err(format!("Export failed: {}", e)),
    }
}
