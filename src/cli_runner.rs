use crate::error::{McpError, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Clone, Debug)]
pub struct CliRunner {
    godot_binary: PathBuf,
}

impl CliRunner {
    pub fn new(godot_binary: PathBuf) -> Self {
        Self { godot_binary }
    }

    /// Retrieve engine version directly via CLI
    pub async fn get_version(&self) -> Result<String> {
        let output = Command::new(&self.godot_binary)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                McpError::Process(format!(
                    "Failed to invoke Godot executable at '{}': {e}",
                    self.godot_binary.display()
                ))
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(McpError::Process(format!(
                "Godot --version exited with error: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    /// Launch Godot Editor for a designated project directory
    pub async fn launch_editor(&self, project_path: &Path) -> Result<u32> {
        let mut cmd = Command::new(&self.godot_binary);
        cmd.arg("--editor")
            .arg("--path")
            .arg(project_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| McpError::Process(format!("Failed to spawn Godot Editor process: {e}")))?;

        let pid = child
            .id()
            .ok_or_else(|| McpError::Process("Failed to obtain Godot process ID".into()))?;

        Ok(pid)
    }

    /// Run the game project or specific scene
    pub async fn run_project(&self, project_path: &Path, scene: Option<&str>) -> Result<u32> {
        let mut cmd = Command::new(&self.godot_binary);
        cmd.arg("--path").arg(project_path);

        if let Some(scene_file) = scene {
            cmd.arg(scene_file);
        }

        let child = cmd
            .spawn()
            .map_err(|e| McpError::Process(format!("Failed to spawn Godot project: {e}")))?;

        let pid = child
            .id()
            .ok_or_else(|| McpError::Process("Failed to obtain project process ID".into()))?;

        Ok(pid)
    }

    /// Validate a GDScript file syntax using headless Godot check
    pub async fn validate_script(
        &self,
        project_path: Option<&Path>,
        script_path: &Path,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.godot_binary);
        cmd.arg("--headless").arg("--check-only");

        if let Some(proj) = project_path {
            cmd.arg("--path").arg(proj);
        }

        cmd.arg("--script").arg(script_path);

        let output = cmd.output().await.map_err(|e| {
            McpError::Process(format!("Failed to run Godot script validation: {e}"))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() && stderr.trim().is_empty() {
            Ok("GDScript validation passed successfully with no errors.".into())
        } else {
            let combined = format!("Stdout: {stdout}\nStderr: {stderr}");
            Err(McpError::Validation(combined.trim().to_string()))
        }
    }
}

