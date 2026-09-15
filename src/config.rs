use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "godot-mcp",
    about = "High-performance Model Context Protocol (MCP) server for the Godot Game Engine",
    version
)]
pub struct Config {
    /// Path to the primary Godot executable
    #[arg(long, env = "GODOT_PATH")]
    pub godot_path: Option<PathBuf>,

    /// Path to the Godot console wrapper executable (Windows)
    #[arg(long, env = "GODOT_CONSOLE_PATH")]
    pub godot_console_path: Option<PathBuf>,

    /// WebSocket port for Godot Editor bridge communication
    #[arg(long, env = "GODOT_DEBUG_PORT", default_value = "9333")]
    pub debug_port: u16,

    /// Optional remote MCP URL when running in HTTP/SSE transport mode
    #[arg(long, env = "GODOT_MCP_URL")]
    pub mcp_url: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        // Load .env file if present
        let _ = dotenvy::dotenv();
        Config::parse()
    }

    /// Resolve effective Godot binary path
    pub fn resolve_godot_path(&self) -> PathBuf {
        if let Some(path) = self.godot_path.as_ref().filter(|p| p.exists()) {
            return path.clone();
        }

        // Search in common default system locations
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                r"D:\Program Files\Godot\Godot.exe",
                r"C:\Program Files\Godot\Godot.exe",
                r"C:\Program Files\Godot_v4\Godot.exe",
            ];
            for candidate in candidates {
                let p = PathBuf::from(candidate);
                if p.exists() {
                    return p;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                "/Applications/Godot.app/Contents/MacOS/Godot",
                "/Applications/Godot_mono.app/Contents/MacOS/Godot",
            ];
            for candidate in candidates {
                let p = PathBuf::from(candidate);
                if p.exists() {
                    return p;
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = ["/usr/bin/godot", "/usr/local/bin/godot"];
            for candidate in candidates {
                let p = PathBuf::from(candidate);
                if p.exists() {
                    return p;
                }
            }
        }

        // Fallback to searching in PATH
        PathBuf::from("godot")
    }
}
