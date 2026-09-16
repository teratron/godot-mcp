use crate::bridge::BridgeClient;
use crate::cli_runner::CliRunner;
use crate::protocol::{Tool, ToolResult};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ToolManager {
    bridge: BridgeClient,
    cli: CliRunner,
}

impl ToolManager {
    pub fn new(bridge: BridgeClient, cli: CliRunner) -> Self {
        Self { bridge, cli }
    }

    /// Return the list of all registered tools with their input schemas
    pub fn list_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "godot_get_version".into(),
                description: "Retrieves the Godot Engine version and build information.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "godot_get_project_info".into(),
                description: "Inspects project configuration, name, main scene, and enabled features.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_path": {
                            "type": "string",
                            "description": "Optional project root directory. Defaults to active editor project."
                        }
                    }
                }),
            },
            Tool {
                name: "godot_launch_editor".into(),
                description: "Launches the Godot Editor for a designated project directory.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_path": {
                            "type": "string",
                            "description": "Absolute path to the Godot project directory containing project.godot."
                        }
                    },
                    "required": ["project_path"]
                }),
            },
            Tool {
                name: "godot_run_project".into(),
                description: "Runs the Godot project or a specific scene in debug mode.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_path": {
                            "type": "string",
                            "description": "Path to the project root directory."
                        },
                        "scene": {
                            "type": "string",
                            "description": "Optional specific scene path (e.g., 'res://scenes/Main.tscn')."
                        }
                    },
                    "required": ["project_path"]
                }),
            },
            Tool {
                name: "godot_get_scene_tree".into(),
                description: "Retrieves the complete node hierarchy of the active scene open in Godot Editor.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "godot_add_node".into(),
                description: "Adds a new node to the active scene in Godot Editor with undo/redo support.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "parent_node_path": {
                            "type": "string",
                            "description": "Path to parent node (e.g. '.' for root or 'Player/Sprite')."
                        },
                        "node_type": {
                            "type": "string",
                            "description": "Godot class name to instantiate (e.g. 'Sprite2D', 'Camera3D', 'Node')."
                        },
                        "node_name": {
                            "type": "string",
                            "description": "Optional custom name for the new node."
                        }
                    },
                    "required": ["node_type"]
                }),
            },
            Tool {
                name: "godot_remove_node".into(),
                description: "Removes a node from the active scene tree with undo/redo support.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "node_path": {
                            "type": "string",
                            "description": "Path to the node to remove."
                        }
                    },
                    "required": ["node_path"]
                }),
            },
            Tool {
                name: "godot_get_node_properties".into(),
                description: "Inspects all properties, export variables, and types of a target node.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "node_path": {
                            "type": "string",
                            "description": "Path to target node."
                        }
                    },
                    "required": ["node_path"]
                }),
            },
            Tool {
                name: "godot_set_node_properties".into(),
                description: "Modifies exported properties or transforms on a target node in the active scene.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "node_path": {
                            "type": "string",
                            "description": "Path to target node."
                        },
                        "properties": {
                            "type": "object",
                            "description": "Key-value map of property names to new values."
                        }
                    },
                    "required": ["node_path", "properties"]
                }),
            },
            Tool {
                name: "godot_capture_viewport".into(),
                description: "Captures a screenshot of the active Godot 2D/3D editor viewport as a base64 image.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "godot_open_scene".into(),
                description: "Opens a scene file in the Godot Editor by its res:// path.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "scene_path": {
                            "type": "string",
                            "description": "Resource path to scene (e.g. 'res://scenes/level1.tscn')."
                        }
                    },
                    "required": ["scene_path"]
                }),
            },
            Tool {
                name: "godot_save_scene".into(),
                description: "Saves changes to the current open scene in Godot Editor.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "godot_create_script".into(),
                description: "Creates a new GDScript file with standard class header and optional base class.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute or relative file path for the .gd script."
                        },
                        "extends_class": {
                            "type": "string",
                            "description": "Base class to extend (e.g. 'Node2D', 'CharacterBody3D'). Defaults to 'Node'."
                        },
                        "class_name": {
                            "type": "string",
                            "description": "Optional global class_name registration."
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            Tool {
                name: "godot_validate_script".into(),
                description: "Performs syntax checking and static validation on a GDScript file without execution.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "script_path": {
                            "type": "string",
                            "description": "Path to the .gd script file."
                        },
                        "project_path": {
                            "type": "string",
                            "description": "Optional project root path for class resolution."
                        }
                    },
                    "required": ["script_path"]
                }),
            },
        ]
    }

    /// Dispatch tool calls to live bridge or CLI runner
    pub async fn call_tool(self: Arc<Self>, name: &str, args: Value) -> ToolResult {
        match name {
            "godot_get_version" => self.handle_get_version().await,
            "godot_get_project_info" => self.handle_bridge_call("get_project_info", args).await,
            "godot_launch_editor" => self.handle_launch_editor(args).await,
            "godot_run_project" => self.handle_run_project(args).await,
            "godot_get_scene_tree" => self.handle_bridge_call("get_scene_tree", args).await,
            "godot_add_node" => self.handle_bridge_call("add_node", args).await,
            "godot_remove_node" => self.handle_bridge_call("remove_node", args).await,
            "godot_get_node_properties" => {
                self.handle_bridge_call("get_node_properties", args).await
            }
            "godot_set_node_properties" => {
                self.handle_bridge_call("set_node_properties", args).await
            }
            "godot_capture_viewport" => self.handle_capture_viewport().await,
            "godot_open_scene" => self.handle_bridge_call("open_scene", args).await,
            "godot_save_scene" => self.handle_bridge_call("save_scene", args).await,
            "godot_create_script" => self.handle_create_script(args).await,
            "godot_validate_script" => self.handle_validate_script(args).await,
            _ => ToolResult::error(
                format!("Tool '{name}' is not recognized"),
                &["Check tool name spelling against tools/list"],
            ),
        }
    }

    async fn handle_get_version(&self) -> ToolResult {
        // Try live editor bridge first
        if let Ok(result) = self.bridge.send_command("get_version", json!({})).await {
            return ToolResult::text(format!(
                "Godot Engine (Live Editor Connected):\n{}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            ));
        }

        // Fallback to CLI
        match self.cli.get_version().await {
            Ok(ver) => ToolResult::text(format!(
                "Godot Engine (Headless CLI):\nVersion: {ver}\nNote: Editor bridge is offline. Launch Godot Editor and enable Godot MCP Bridge plugin for live manipulation."
            )),
            Err(e) => ToolResult::error(
                format!("Failed to retrieve Godot version: {e}"),
                &[
                    "Verify GODOT_PATH in .env",
                    "Ensure Godot executable is installed and reachable",
                ],
            ),
        }
    }

    async fn handle_bridge_call(&self, command_type: &str, params: Value) -> ToolResult {
        match self.bridge.send_command(command_type, params).await {
            Ok(res) => ToolResult::text(serde_json::to_string_pretty(&res).unwrap_or_default()),
            Err(e) => ToolResult::error(
                format!("{e}"),
                &[
                    "Launch Godot Editor with your project",
                    "Ensure 'Godot MCP Bridge' plugin is enabled in Project Settings -> Plugins",
                    "Verify GODOT_DEBUG_PORT in .env matches Godot bridge port (default: 9333)",
                ],
            ),
        }
    }

    async fn handle_capture_viewport(&self) -> ToolResult {
        match self
            .bridge
            .send_command("capture_viewport", json!({}))
            .await
        {
            Ok(res) => {
                if let (Some(data), Some(mime)) = (
                    res.get("data").and_then(|v| v.as_str()),
                    res.get("mime_type").and_then(|v| v.as_str()),
                ) {
                    ToolResult::image(data, mime)
                } else {
                    ToolResult::error("Invalid image response received from Godot viewport", &[])
                }
            }
            Err(e) => ToolResult::error(
                format!("Failed to capture viewport: {e}"),
                &["Ensure Godot Editor has a scene open in 2D or 3D view"],
            ),
        }
    }

    async fn handle_launch_editor(&self, args: Value) -> ToolResult {
        let project_path_str = match args.get("project_path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter: project_path", &[]),
        };

        let path = PathBuf::from(project_path_str);
        if !path.exists() {
            return ToolResult::error(
                format!("Project directory does not exist: {project_path_str}"),
                &["Verify path spelling and ensure project.godot exists"],
            );
        }

        match self.cli.launch_editor(&path).await {
            Ok(pid) => ToolResult::text(format!(
                "Godot Editor launched successfully for '{}' (PID: {pid}).",
                path.display()
            )),
            Err(e) => ToolResult::error(format!("Failed to launch Godot Editor: {e}"), &[]),
        }
    }

    async fn handle_run_project(&self, args: Value) -> ToolResult {
        let project_path_str = match args.get("project_path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter: project_path", &[]),
        };

        let scene = args.get("scene").and_then(|s| s.as_str());
        let path = PathBuf::from(project_path_str);

        match self.cli.run_project(&path, scene).await {
            Ok(pid) => ToolResult::text(format!(
                "Godot project instance spawned (PID: {pid}) for '{}'.",
                path.display()
            )),
            Err(e) => ToolResult::error(format!("Failed to run project: {e}"), &[]),
        }
    }

    async fn handle_create_script(&self, args: Value) -> ToolResult {
        let file_path_str = match args.get("file_path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter: file_path", &[]),
        };

        // Path validation against directory traversal
        if file_path_str.contains("..") {
            return ToolResult::error(
                "Invalid path: directory traversal with '..' is strictly forbidden",
                &["Provide a path without relative parent references"],
            );
        }

        let extends_class = args
            .get("extends_class")
            .and_then(|c| c.as_str())
            .unwrap_or("Node");

        let class_name_decl = if let Some(cn) = args.get("class_name").and_then(|c| c.as_str()) {
            format!("class_name {cn}\n")
        } else {
            String::new()
        };

        let content =
            format!("extends {extends_class}\n{class_name_decl}\nfunc _ready() -> void:\n\tpass\n");

        let path = Path::new(file_path_str);
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            let _ = fs::create_dir_all(parent);
        }

        match fs::write(path, content) {
            Ok(_) => ToolResult::text(format!(
                "Successfully created GDScript file at '{}' extending '{}'.",
                path.display(),
                extends_class
            )),
            Err(e) => ToolResult::error(format!("Failed to write script file: {e}"), &[]),
        }
    }

    async fn handle_validate_script(&self, args: Value) -> ToolResult {
        let script_path_str = match args.get("script_path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter: script_path", &[]),
        };

        let script_path = PathBuf::from(script_path_str);
        let project_path = args
            .get("project_path")
            .and_then(|p| p.as_str())
            .map(PathBuf::from);

        match self
            .cli
            .validate_script(project_path.as_deref(), &script_path)
            .await
        {
            Ok(msg) => ToolResult::text(msg),
            Err(e) => ToolResult::error(
                format!("Script validation failed: {e}"),
                &[
                    "Check GDScript syntax line numbers",
                    "Ensure base class and node types exist in the engine",
                ],
            ),
        }
    }
}
