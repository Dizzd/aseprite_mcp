use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Default timeout for Aseprite process execution (60 seconds).
const PROCESS_TIMEOUT: Duration = Duration::from_secs(60);

/// Handles execution of Aseprite CLI commands and Lua scripts.
#[derive(Debug)]
pub struct AsepriteRunner {
    exe_path: PathBuf,
    temp_dir: PathBuf,
}

/// Output from an Aseprite CLI or script execution.
#[derive(Debug)]
pub struct ScriptOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl ScriptOutput {
    /// Returns a user-friendly result string.
    pub fn result_text(&self) -> String {
        if self.success {
            if self.stdout.trim().is_empty() {
                "Operation completed successfully.".to_string()
            } else {
                self.stdout.trim().to_string()
            }
        } else {
            let err_msg = if !self.stderr.trim().is_empty() {
                self.stderr.trim()
            } else if !self.stdout.trim().is_empty() {
                self.stdout.trim()
            } else {
                "Unknown error occurred"
            };
            format!("Error: {}", err_msg)
        }
    }
}

impl AsepriteRunner {
    /// Create a new AsepriteRunner, locating the Aseprite executable.
    pub fn new() -> Result<Self> {
        let exe_path = Self::find_aseprite()?;
        let temp_dir = std::env::temp_dir().join("aseprite_mcp");
        // Temp dir creation is fine synchronous — only runs once at startup
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temp directory for Aseprite scripts")?;
        info!("Aseprite MCP: using executable at {}", exe_path.display());
        Ok(Self { exe_path, temp_dir })
    }

    /// Locate the Aseprite executable on the system.
    fn find_aseprite() -> Result<PathBuf> {
        // 1. Check ASEPRITE_PATH environment variable
        if let Ok(path) = std::env::var("ASEPRITE_PATH") {
            let path = PathBuf::from(&path);
            if path.exists() {
                return Ok(path);
            }
            debug!("ASEPRITE_PATH={} does not exist, searching...", path.display());
        }

        // 2. Platform-specific common paths
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                r"C:\Program Files\Aseprite\Aseprite.exe",
                r"C:\Program Files (x86)\Steam\steamapps\common\Aseprite\Aseprite.exe",
                r"C:\Program Files\Steam\steamapps\common\Aseprite\Aseprite.exe",
            ];
            for p in &candidates {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Ok(path);
                }
            }
            // Try `where` command
            if let Ok(output) = std::process::Command::new("where")
                .arg("aseprite")
                .output()
            {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_line) = path_str.lines().next() {
                        let p = PathBuf::from(first_line.trim());
                        if p.exists() {
                            return Ok(p);
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/Aseprite.app/Contents/MacOS/aseprite");
            if path.exists() {
                return Ok(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("which")
                .arg("aseprite")
                .output()
            {
                if output.status.success() {
                    let path =
                        String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let p = PathBuf::from(&path);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let steam_path = format!(
                "{}/.steam/debian-installation/steamapps/common/Aseprite/aseprite",
                home
            );
            let p = PathBuf::from(&steam_path);
            if p.exists() {
                return Ok(p);
            }
        }

        bail!(
            "Could not find Aseprite executable. \
             Please set the ASEPRITE_PATH environment variable to the full path \
             of the Aseprite executable."
        )
    }

    /// Generate a unique temporary script file path.
    fn temp_script_path(&self) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        self.temp_dir
            .join(format!("mcp_{}_{}.lua", ts, count))
    }

    /// Run a Lua script in batch mode (no file opened beforehand).
    pub async fn run_script(&self, lua_code: &str) -> Result<ScriptOutput> {
        let script_path = self.temp_script_path();
        tokio::fs::write(&script_path, lua_code)
            .await
            .context("Failed to write temporary Lua script")?;

        debug!("Running Lua script (no file): {}", script_path.display());

        let result = self
            .execute_with_timeout(
                Command::new(&self.exe_path)
                    .args(["--batch", "--script"])
                    .arg(&script_path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped()),
            )
            .await;

        // Clean up temp file (best-effort)
        if let Err(e) = tokio::fs::remove_file(&script_path).await {
            warn!("Failed to clean up temp script {}: {}", script_path.display(), e);
        }

        result
    }

    /// Run a Lua script with a sprite file opened first.
    pub async fn run_script_on_file(
        &self,
        file_path: &str,
        lua_code: &str,
    ) -> Result<ScriptOutput> {
        let script_path = self.temp_script_path();
        tokio::fs::write(&script_path, lua_code)
            .await
            .context("Failed to write temporary Lua script")?;

        debug!(
            "Running Lua script on file: {} | {}",
            file_path,
            script_path.display()
        );

        let result = self
            .execute_with_timeout(
                Command::new(&self.exe_path)
                    .arg("--batch")
                    .arg(file_path)
                    .arg("--script")
                    .arg(&script_path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped()),
            )
            .await;

        if let Err(e) = tokio::fs::remove_file(&script_path).await {
            warn!("Failed to clean up temp script {}: {}", script_path.display(), e);
        }

        result
    }

    /// Run Aseprite with raw CLI arguments (batch mode).
    pub async fn run_cli(&self, args: &[String]) -> Result<ScriptOutput> {
        debug!("Running Aseprite CLI: {:?}", args);

        self.execute_with_timeout(
            Command::new(&self.exe_path)
                .arg("--batch")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        )
        .await
    }

    /// Execute an Aseprite command with a timeout. Kills the process if it exceeds the limit.
    async fn execute_with_timeout(&self, cmd: &mut Command) -> Result<ScriptOutput> {
        let mut child = cmd.spawn().context("Failed to spawn Aseprite process")?;

        // Take stdout/stderr handles before awaiting, so we can still kill the child on timeout
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        let status = match tokio::time::timeout(PROCESS_TIMEOUT, child.wait()).await {
            Ok(result) => result.context("Failed to wait for Aseprite process")?,
            Err(_) => {
                // Timeout — try to kill the process
                warn!("Aseprite process timed out after {:?}, killing...", PROCESS_TIMEOUT);
                child.kill().await.ok();
                bail!(
                    "Aseprite process timed out after {} seconds. \
                     The operation may be too complex or Aseprite may be unresponsive.",
                    PROCESS_TIMEOUT.as_secs()
                );
            }
        };

        // Read captured output
        let stdout = if let Some(mut handle) = stdout_handle {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            handle.read_to_end(&mut buf).await.unwrap_or(0);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        let stderr = if let Some(mut handle) = stderr_handle {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            handle.read_to_end(&mut buf).await.unwrap_or(0);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        debug!(
            "Aseprite exit={} stdout_len={} stderr_len={}",
            status.code().unwrap_or(-1),
            stdout.len(),
            stderr.len()
        );

        Ok(ScriptOutput {
            stdout,
            stderr,
            success: status.success(),
        })
    }
}

// ============================================================================
// Lua String Helpers
// ============================================================================

/// Escape a string for safe use inside a Lua quoted string literal.
/// Returns the string wrapped in quotes.
pub fn lua_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\0', "\\0");
    format!("\"{}\"", escaped)
}

/// Normalize a file path to use forward slashes (Lua/Aseprite-friendly).
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Create a Lua-safe path string (normalized + escaped).
pub fn lua_path(path: &str) -> String {
    lua_string(&normalize_path(path))
}
