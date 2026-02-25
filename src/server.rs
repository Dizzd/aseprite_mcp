use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext},
    model::*,
    tool, tool_router,
    service::RequestContext,
};
use rmcp::handler::server::tool::Parameters;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

use crate::aseprite::{AsepriteRunner, ScriptOutput};
use crate::tools;

// ============================================================================
// AsepriteServer
// ============================================================================

#[derive(Debug, Clone)]
pub struct AsepriteServer {
    runner: Arc<AsepriteRunner>,
    /// Default output directory for generated files. Read from ASEPRITE_OUTPUT_DIR env var.
    /// When set, relative output paths are resolved against this directory.
    output_dir: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

// ============================================================================
// Tool Routing — thin wrappers that delegate to tool modules
// ============================================================================

#[tool_router]
impl AsepriteServer {
    pub fn new() -> anyhow::Result<Self> {
        let runner = Arc::new(AsepriteRunner::new()?);
        let output_dir = std::env::var("ASEPRITE_OUTPUT_DIR").ok().map(|dir| {
            let path = PathBuf::from(&dir);
            if !path.exists() {
                info!("Creating output directory: {}", path.display());
                std::fs::create_dir_all(&path).ok();
            }
            info!("Output directory set to: {}", path.display());
            path
        });
        Ok(Self {
            runner,
            output_dir,
            tool_router: Self::tool_router(),
        })
    }

    // ========================================================================
    // Sprite Management Tools
    // ========================================================================

    #[tool(description = "Create a new sprite file with specified dimensions and color mode. Supports .aseprite, .png, .gif and other formats.")]
    async fn create_sprite(
        &self,
        params: Parameters<tools::sprite::CreateSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::create_sprite(self, params.0).await
    }

    #[tool(description = "Get comprehensive information about a sprite file: dimensions, color mode, layers, frames, tags, slices, and palette size.")]
    async fn get_sprite_info(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::sprite::get_sprite_info(self, params.0).await
    }

    #[tool(description = "Resize a sprite to specified width and height in pixels.")]
    async fn resize_sprite(
        &self,
        params: Parameters<tools::sprite::ResizeSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::resize_sprite(self, params.0).await
    }

    #[tool(description = "Crop a sprite to a rectangular region defined by x, y, width, and height.")]
    async fn crop_sprite(
        &self,
        params: Parameters<tools::sprite::CropSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::crop_sprite(self, params.0).await
    }

    #[tool(description = "Flip a sprite horizontally or vertically.")]
    async fn flip_sprite(
        &self,
        params: Parameters<tools::sprite::FlipSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::flip_sprite(self, params.0).await
    }

    #[tool(description = "Rotate a sprite by 90, 180, or 270 degrees.")]
    async fn rotate_sprite(
        &self,
        params: Parameters<tools::sprite::RotateSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::rotate_sprite(self, params.0).await
    }

    #[tool(description = "Change the canvas size by specifying left, top, right, bottom padding. Positive values expand, negative values shrink.")]
    async fn canvas_size(
        &self,
        params: Parameters<tools::sprite::CanvasSizeParams>,
    ) -> Result<String, String> {
        tools::sprite::canvas_size(self, params.0).await
    }

    #[tool(description = "Duplicate a sprite to a new file, preserving all layers, frames, tags, and slices.")]
    async fn duplicate_sprite(
        &self,
        params: Parameters<tools::sprite::DuplicateSpriteParams>,
    ) -> Result<String, String> {
        tools::sprite::duplicate_sprite(self, params.0).await
    }

    #[tool(description = "Auto-crop a sprite, trimming transparent borders to fit the content tightly. Essential for optimizing game sprite sizes.")]
    async fn auto_crop_sprite(
        &self,
        params: Parameters<tools::sprite::AutoCropParams>,
    ) -> Result<String, String> {
        tools::sprite::auto_crop_sprite(self, params.0).await
    }

    #[tool(description = "Change sprite color mode to 'rgb', 'grayscale', or 'indexed'. Useful for optimizing game assets.")]
    async fn change_color_mode(
        &self,
        params: Parameters<tools::sprite::ChangeColorModeParams>,
    ) -> Result<String, String> {
        tools::sprite::change_color_mode(self, params.0).await
    }

    #[tool(description = "Reverse the order of frames in a sprite or within a frame range. Useful for creating reverse animations (e.g. walk backward from walk forward).")]
    async fn reverse_frames(
        &self,
        params: Parameters<tools::sprite::ReverseFramesParams>,
    ) -> Result<String, String> {
        tools::sprite::reverse_frames(self, params.0).await
    }

    // ========================================================================
    // Layer Management Tools
    // ========================================================================

    #[tool(description = "List all layers in a sprite file with name, visibility, opacity, blend mode, and hierarchy information.")]
    async fn list_layers(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::layer::list_layers(self, &params.0.file_path).await
    }

    #[tool(description = "Add a new layer or group layer to a sprite. Optionally specify where to insert it.")]
    async fn add_layer(
        &self,
        params: Parameters<tools::layer::AddLayerParams>,
    ) -> Result<String, String> {
        tools::layer::add_layer(self, params.0).await
    }

    #[tool(description = "Remove/delete a layer from a sprite by its name.")]
    async fn remove_layer(
        &self,
        params: Parameters<tools::layer::RemoveLayerParams>,
    ) -> Result<String, String> {
        tools::layer::remove_layer(self, params.0).await
    }

    #[tool(description = "Modify layer properties: rename, set visibility, opacity (0-255), or blend mode.")]
    async fn set_layer_property(
        &self,
        params: Parameters<tools::layer::SetLayerPropertyParams>,
    ) -> Result<String, String> {
        tools::layer::set_layer_property(self, params.0).await
    }

    #[tool(description = "Duplicate a layer (and all its cels) within a sprite. Optionally rename the new layer.")]
    async fn duplicate_layer(
        &self,
        params: Parameters<tools::layer::DuplicateLayerParams>,
    ) -> Result<String, String> {
        tools::layer::duplicate_layer(self, params.0).await
    }

    #[tool(description = "Merge a layer down into the layer below it, combining their contents.")]
    async fn merge_down_layer(
        &self,
        params: Parameters<tools::layer::MergeDownLayerParams>,
    ) -> Result<String, String> {
        tools::layer::merge_down_layer(self, params.0).await
    }

    #[tool(description = "Flatten all visible layers into a single layer. Useful for final export preparation.")]
    async fn flatten_layers(
        &self,
        params: Parameters<tools::layer::FlattenLayersParams>,
    ) -> Result<String, String> {
        tools::layer::flatten_layers(self, params.0).await
    }

    // ========================================================================
    // Frame Management Tools
    // ========================================================================

    #[tool(description = "List all frames in a sprite with frame numbers and durations in seconds.")]
    async fn list_frames(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::frame::list_frames(self, &params.0.file_path).await
    }

    #[tool(description = "Add one or more frames to a sprite. Can create copies of the current frame or empty frames.")]
    async fn add_frame(
        &self,
        params: Parameters<tools::frame::AddFrameParams>,
    ) -> Result<String, String> {
        tools::frame::add_frame(self, params.0).await
    }

    #[tool(description = "Remove a specific frame from a sprite by frame number (1-based).")]
    async fn remove_frame(
        &self,
        params: Parameters<tools::frame::RemoveFrameParams>,
    ) -> Result<String, String> {
        tools::frame::remove_frame(self, params.0).await
    }

    #[tool(description = "Set the duration of a specific frame in milliseconds.")]
    async fn set_frame_duration(
        &self,
        params: Parameters<tools::frame::SetFrameDurationParams>,
    ) -> Result<String, String> {
        tools::frame::set_frame_duration(self, params.0).await
    }

    // ========================================================================
    // Tag Management Tools
    // ========================================================================

    #[tool(description = "List all animation tags in a sprite with name, frame range, direction, and repeat count.")]
    async fn list_tags(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::tag::list_tags(self, &params.0.file_path).await
    }

    #[tool(description = "Create a new animation tag spanning a range of frames with optional direction and color.")]
    async fn create_tag(
        &self,
        params: Parameters<tools::tag::CreateTagParams>,
    ) -> Result<String, String> {
        tools::tag::create_tag(self, params.0).await
    }

    #[tool(description = "Delete an animation tag from a sprite by its name.")]
    async fn delete_tag(
        &self,
        params: Parameters<tools::tag::DeleteTagParams>,
    ) -> Result<String, String> {
        tools::tag::delete_tag(self, params.0).await
    }

    // ========================================================================
    // Slice Management Tools (Game Dev — hitboxes, 9-slice UI, pivots)
    // ========================================================================

    #[tool(description = "List all slices in a sprite with bounds, 9-slice center, pivot point, and user data. Slices define named regions for game engines (hitboxes, UI elements, anchors).")]
    async fn list_slices(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::slice::list_slices(self, &params.0.file_path).await
    }

    #[tool(description = "Create a new slice (named region) in a sprite. Supports 9-slice center rect for UI scaling, pivot point for anchor/origin, and custom data for game metadata.")]
    async fn create_slice(
        &self,
        params: Parameters<tools::slice::CreateSliceParams>,
    ) -> Result<String, String> {
        tools::slice::create_slice(self, params.0).await
    }

    #[tool(description = "Delete a slice from a sprite by its name.")]
    async fn delete_slice(
        &self,
        params: Parameters<tools::slice::DeleteSliceParams>,
    ) -> Result<String, String> {
        tools::slice::delete_slice(self, params.0).await
    }

    // ========================================================================
    // Cel Management Tools
    // ========================================================================

    #[tool(description = "List all cels in a sprite with layer, frame, position, size, and opacity. Optionally filter by layer name or frame number.")]
    async fn list_cels(
        &self,
        params: Parameters<tools::cel::ListCelsParams>,
    ) -> Result<String, String> {
        tools::cel::list_cels(self, params.0).await
    }

    #[tool(description = "Move a cel to a new position (x, y) on the canvas. Useful for animation offset adjustments.")]
    async fn move_cel(
        &self,
        params: Parameters<tools::cel::MoveCelParams>,
    ) -> Result<String, String> {
        tools::cel::move_cel(self, params.0).await
    }

    #[tool(description = "Set the opacity (0-255) of a specific cel.")]
    async fn set_cel_opacity(
        &self,
        params: Parameters<tools::cel::SetCelOpacityParams>,
    ) -> Result<String, String> {
        tools::cel::set_cel_opacity(self, params.0).await
    }

    #[tool(description = "Clear (delete) a cel at a specific layer and frame, making that cell empty/transparent.")]
    async fn clear_cel(
        &self,
        params: Parameters<tools::cel::ClearCelParams>,
    ) -> Result<String, String> {
        tools::cel::clear_cel(self, params.0).await
    }

    #[tool(description = "Create a new empty cel at a specific layer and frame.")]
    async fn new_cel(
        &self,
        params: Parameters<tools::cel::NewCelParams>,
    ) -> Result<String, String> {
        tools::cel::new_cel(self, params.0).await
    }

    // ========================================================================
    // Drawing Tools
    // ========================================================================

    #[tool(description = "Draw individual pixels on a sprite at specified coordinates with given colors (hex format like '#ff0000'). Optionally target a specific layer and frame.")]
    async fn draw_pixels(
        &self,
        params: Parameters<tools::drawing::DrawPixelsParams>,
    ) -> Result<String, String> {
        tools::drawing::draw_pixels(self, params.0).await
    }

    #[tool(description = "Use an Aseprite drawing tool (pencil, line, rectangle, filled_rectangle, ellipse, filled_ellipse, paint_bucket, spray, eraser) with specified points, color, brush size, and opacity.")]
    async fn use_tool(
        &self,
        params: Parameters<tools::drawing::UseToolParams>,
    ) -> Result<String, String> {
        tools::drawing::use_tool(self, params.0).await
    }

    #[tool(description = "Read pixel color data from a rectangular region of a sprite. Returns an array of pixel colors in hex format.")]
    async fn get_pixel_data(
        &self,
        params: Parameters<tools::drawing::GetPixelDataParams>,
    ) -> Result<String, String> {
        tools::drawing::get_pixel_data(self, params.0).await
    }

    // ========================================================================
    // Palette Tools
    // ========================================================================

    #[tool(description = "Get the color palette of a sprite as an array of hex color values with their indices.")]
    async fn get_palette(
        &self,
        params: Parameters<tools::palette::GetPaletteParams>,
    ) -> Result<String, String> {
        tools::palette::get_palette(self, params.0).await
    }

    #[tool(description = "Set one or more colors in the sprite's palette by index. Colors should be hex strings like '#ff0000'.")]
    async fn set_palette_color(
        &self,
        params: Parameters<tools::palette::SetPaletteColorParams>,
    ) -> Result<String, String> {
        tools::palette::set_palette_color(self, params.0).await
    }

    #[tool(description = "Resize the color palette to a specific number of colors.")]
    async fn resize_palette(
        &self,
        params: Parameters<tools::palette::ResizePaletteParams>,
    ) -> Result<String, String> {
        tools::palette::resize_palette(self, params.0).await
    }

    #[tool(description = "Load a palette from a file (.gpl, .pal, .act, .col, .png) and apply it to the sprite.")]
    async fn load_palette(
        &self,
        params: Parameters<tools::palette::LoadPaletteParams>,
    ) -> Result<String, String> {
        tools::palette::load_palette(self, params.0).await
    }

    #[tool(description = "Save the sprite's current palette to a file (.gpl, .pal, .act, .png).")]
    async fn save_palette(
        &self,
        params: Parameters<tools::palette::SavePaletteParams>,
    ) -> Result<String, String> {
        tools::palette::save_palette(self, params.0).await
    }

    #[tool(description = "Automatically generate an optimized palette from sprite colors using color quantization. Great for reducing color count for indexed-mode game sprites.")]
    async fn color_quantization(
        &self,
        params: Parameters<tools::palette::ColorQuantizationParams>,
    ) -> Result<String, String> {
        tools::palette::color_quantization(self, params.0).await
    }

    // ========================================================================
    // Selection Tools
    // ========================================================================

    #[tool(description = "Select a rectangular region in a sprite. Mode can be 'replace', 'add', 'subtract', or 'intersect'.")]
    async fn select_region(
        &self,
        params: Parameters<tools::selection::SelectRegionParams>,
    ) -> Result<String, String> {
        tools::selection::select_region(self, params.0).await
    }

    #[tool(description = "Deselect / clear any active selection in a sprite.")]
    async fn deselect(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::selection::deselect(self, &params.0.file_path).await
    }

    #[tool(description = "Select the entire sprite canvas.")]
    async fn select_all(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::selection::select_all(self, &params.0.file_path).await
    }

    #[tool(description = "Invert the current selection (selected becomes unselected and vice versa).")]
    async fn invert_selection(
        &self,
        params: Parameters<tools::sprite::SpriteFileParams>,
    ) -> Result<String, String> {
        tools::selection::invert_selection(self, &params.0.file_path).await
    }

    #[tool(description = "Select all pixels of a specific color with optional tolerance. Useful for selecting and modifying specific color regions.")]
    async fn select_by_color(
        &self,
        params: Parameters<tools::selection::SelectByColorParams>,
    ) -> Result<String, String> {
        tools::selection::select_by_color(self, params.0).await
    }

    // ========================================================================
    // Export Tools
    // ========================================================================

    #[tool(description = "Export a sprite to a different format (png, gif, jpg, bmp, webp, etc.) with optional scale factor and layer/tag filtering.")]
    async fn export_sprite(
        &self,
        params: Parameters<tools::export::ExportSpriteParams>,
    ) -> Result<String, String> {
        tools::export::export_sprite(self, params.0).await
    }

    #[tool(description = "Export a sprite as a spritesheet image with optional JSON metadata. Supports horizontal, vertical, rows, columns, and packed layouts.")]
    async fn export_spritesheet(
        &self,
        params: Parameters<tools::export::ExportSpritesheetParams>,
    ) -> Result<String, String> {
        tools::export::export_spritesheet(self, params.0).await
    }

    // ========================================================================
    // Color Operations
    // ========================================================================

    #[tool(description = "Replace all occurrences of one color with another color throughout the sprite, with optional tolerance.")]
    async fn replace_color(
        &self,
        params: Parameters<tools::effects::ReplaceColorParams>,
    ) -> Result<String, String> {
        tools::effects::replace_color(self, params.0).await
    }

    #[tool(description = "Apply an outline effect around non-transparent pixels with a specified color.")]
    async fn outline(
        &self,
        params: Parameters<tools::effects::OutlineParams>,
    ) -> Result<String, String> {
        tools::effects::outline(self, params.0).await
    }

    // ========================================================================
    // Filter Tools
    // ========================================================================

    #[tool(description = "Adjust brightness and contrast of a sprite. Values range from -100 to 100.")]
    async fn brightness_contrast(
        &self,
        params: Parameters<tools::filter::BrightnessContrastParams>,
    ) -> Result<String, String> {
        tools::filter::brightness_contrast(self, params.0).await
    }

    #[tool(description = "Adjust hue, saturation, and lightness of a sprite. Hue: -180 to 180 degrees, Saturation/Lightness: -100 to 100.")]
    async fn hue_saturation(
        &self,
        params: Parameters<tools::filter::HueSaturationParams>,
    ) -> Result<String, String> {
        tools::filter::hue_saturation(self, params.0).await
    }

    #[tool(description = "Invert all colors in a sprite (negative effect).")]
    async fn invert_color(
        &self,
        params: Parameters<tools::filter::InvertColorParams>,
    ) -> Result<String, String> {
        tools::filter::invert_color(self, params.0).await
    }

    #[tool(description = "Apply a despeckle (median) filter to reduce noise in pixel art. Adjustable matrix size.")]
    async fn despeckle(
        &self,
        params: Parameters<tools::filter::DespeckleParams>,
    ) -> Result<String, String> {
        tools::filter::despeckle(self, params.0).await
    }

    // ========================================================================
    // Script & Command Execution
    // ========================================================================

    #[tool(description = "Execute arbitrary Lua code in Aseprite's scripting environment. The script has full access to the Aseprite API. Use print() to return data. Optionally specify a sprite file to open first.")]
    async fn run_lua_script(
        &self,
        params: Parameters<tools::scripting::RunLuaScriptParams>,
    ) -> Result<String, String> {
        tools::scripting::run_lua_script(self, params.0).await
    }

    #[tool(description = "Run Aseprite in batch mode with custom CLI arguments. Useful for complex export operations, format conversions, and operations best expressed as CLI commands.")]
    async fn execute_cli(
        &self,
        params: Parameters<tools::scripting::ExecuteCliParams>,
    ) -> Result<String, String> {
        tools::scripting::execute_cli(self, params.0).await
    }
}

// ============================================================================
// Public Helper Methods — used by tool modules
// ============================================================================

impl AsepriteServer {
    /// Execute a Lua script without opening a file first.
    pub async fn execute_script(&self, script: &str) -> Result<String, String> {
        match self.runner.run_script(script).await {
            Ok(output) => {
                if output.success {
                    Ok(output.result_text())
                } else {
                    error!("Script error: {}", output.stderr);
                    Err(output.result_text())
                }
            }
            Err(e) => {
                error!("Failed to run script: {}", e);
                Err(format!("Failed to execute script: {}", e))
            }
        }
    }

    /// Execute a Lua script with a file loaded first.
    pub async fn execute_script_on_file(
        &self,
        file_path: &str,
        script: &str,
    ) -> Result<String, String> {
        match self.runner.run_script_on_file(file_path, script).await {
            Ok(output) => {
                if output.success {
                    Ok(output.result_text())
                } else {
                    error!("Script error on {}: {}", file_path, output.stderr);
                    Err(output.result_text())
                }
            }
            Err(e) => {
                error!("Failed to run script on {}: {}", file_path, e);
                Err(format!("Failed to execute script: {}", e))
            }
        }
    }

    /// Resolve an output path against the configured output directory.
    /// If `ASEPRITE_OUTPUT_DIR` is set and `path` is relative, it's joined with the output dir.
    /// If `path` is absolute or no output dir is set, returns the path as-is.
    pub fn resolve_output_path(&self, path: &str) -> String {
        if let Some(ref output_dir) = self.output_dir {
            let p = Path::new(path);
            if p.is_relative() {
                return output_dir.join(p).to_string_lossy().to_string();
            }
        }
        path.to_string()
    }

    /// Run Aseprite with raw CLI arguments (batch mode). Exposed for tool modules.
    pub async fn run_cli(&self, args: &[String]) -> anyhow::Result<ScriptOutput> {
        self.runner.run_cli(args).await
    }
}

// ============================================================================
// ServerHandler Implementation
// ============================================================================

impl ServerHandler for AsepriteServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            server_info: Implementation {
                name: "aseprite-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some(
                "Aseprite MCP Server - Bridge AI assistants with the Aseprite pixel art editor. \
                 Control Aseprite via CLI batch mode to create, edit, and export pixel art sprites \
                 and animations. All file paths should be absolute or relative to the working directory. \
                 Colors use hex format: '#rrggbb' or '#rrggbbaa'."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        let ctx = ToolCallContext::new(self, request, context);
        async move { self.tool_router.call(ctx).await }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
        }))
    }
}
