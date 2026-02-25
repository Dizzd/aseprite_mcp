use rmcp::schemars;
use serde::Deserialize;

use crate::server::AsepriteServer;

// ============================================================================
// Parameter Structs
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunLuaScriptParams {
    /// The Lua script code to execute in Aseprite
    pub script: String,
    /// Optional sprite file to open before running the script
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExecuteCliParams {
    /// CLI arguments to pass to Aseprite (batch mode is always enabled).
    /// Example: ["sprite.ase", "--save-as", "output.png"]
    pub args: Vec<String>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub async fn run_lua_script(server: &AsepriteServer, p: RunLuaScriptParams) -> Result<String, String> {
    if let Some(ref file_path) = p.file_path {
        server.execute_script_on_file(file_path, &p.script).await
    } else {
        server.execute_script(&p.script).await
    }
}

pub async fn execute_cli(server: &AsepriteServer, p: ExecuteCliParams) -> Result<String, String> {
    match server.run_cli(&p.args).await {
        Ok(output) => {
            if output.success {
                Ok(output.result_text())
            } else {
                Err(output.result_text())
            }
        }
        Err(e) => Err(format!("CLI execution failed: {}", e)),
    }
}
