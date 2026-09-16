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

    /// Resolve the effective Godot binary path.
    ///
    /// `GODOT_PATH` should always be set explicitly. Official Godot builds
    /// for Windows are portable, version-named executables (e.g.
    /// `Godot_v4.8.2-stable_win64.exe`) that users extract wherever they
    /// like, so there is no single standard install location to hardcode.
    /// The scan below is a best-effort convenience for a handful of common
    /// locations — it is not a substitute for configuring `GODOT_PATH`.
    pub fn resolve_godot_path(&self) -> PathBuf {
        if let Some(path) = self.godot_path.as_ref().filter(|p| p.exists()) {
            return path.clone();
        }

        if let Some(found) = Self::scan_for_godot_binary() {
            return found;
        }

        // Fallback to searching in PATH
        PathBuf::from("godot")
    }

    /// Scan a few common Windows install roots (Program Files, Local AppData)
    /// for an executable whose name starts with `Godot` and ends in `.exe`,
    /// looking one level into subdirectories to account for version-named
    /// folders produced by extracting the official zip release.
    #[cfg(target_os = "windows")]
    fn scan_for_godot_binary() -> Option<PathBuf> {
        let roots = [
            std::env::var("ProgramFiles").ok(),
            std::env::var("ProgramFiles(x86)").ok(),
            std::env::var("LOCALAPPDATA")
                .ok()
                .map(|p| format!("{p}\\Programs")),
        ];

        roots
            .into_iter()
            .flatten()
            .map(PathBuf::from)
            .find_map(|root| Self::find_godot_exe_in(&root, 2))
    }

    #[cfg(target_os = "windows")]
    fn find_godot_exe_in(dir: &std::path::Path, depth: u8) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        let mut subdirs = Vec::new();
        let mut matches = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let lower = name.to_lowercase();
            if lower.starts_with("godot") && lower.ends_with(".exe") && !lower.contains("console") {
                matches.push(path);
            }
        }

        // Prefer the lexicographically last match: for versioned filenames
        // like `Godot_v4.8.2-stable_win64.exe` this tends to pick the newest.
        matches.sort();
        if let Some(found) = matches.pop() {
            return Some(found);
        }

        if depth == 0 {
            return None;
        }
        subdirs
            .into_iter()
            .find_map(|subdir| Self::find_godot_exe_in(&subdir, depth - 1))
    }

    /// macOS installs are conventionally a drag-and-dropped `.app` bundle,
    /// so a fixed path is a reasonable (if not guaranteed) default.
    #[cfg(target_os = "macos")]
    fn scan_for_godot_binary() -> Option<PathBuf> {
        [
            "/Applications/Godot.app/Contents/MacOS/Godot",
            "/Applications/Godot_mono.app/Contents/MacOS/Godot",
        ]
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
    }

    /// Linux installs commonly land in one of these locations via a package
    /// manager or Flatpak.
    #[cfg(target_os = "linux")]
    fn scan_for_godot_binary() -> Option<PathBuf> {
        [
            "/usr/bin/godot",
            "/usr/local/bin/godot",
            "/var/lib/flatpak/exports/bin/org.godotengine.Godot",
        ]
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn scan_for_godot_binary() -> Option<PathBuf> {
        None
    }
}
