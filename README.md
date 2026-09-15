# Godot MCP Server

A high-performance, cross-platform Model Context Protocol (MCP) server written in Rust that provides deep, bidirectional control over the Godot Engine (4.x+) for AI coding assistants.

Designed for uncompromising execution speed, zero runtime dependencies, and seamless operation across **Windows**, **Linux**, and **macOS**.

## Key Features

- **Blazing Fast Native Rust Core**: Built on Tokio async I/O with zero-overhead JSON-RPC handling and sub-millisecond bridge latency. No Node.js or Python runtime required.
- **Cross-Platform Compatibility**: Single standalone executable for Windows (`godot-mcp.exe`), Linux (`godot-mcp`), and macOS (`godot-mcp` with Apple Silicon and Intel support).
- **Dual-Mode Engine Architecture**: Operates via a live WebSocket bridge in active Godot Editor sessions, with automatic fallback to headless CLI execution when the editor is closed.
- **Full Scene & Node Control**: Query, instantiate, configure, re-parent, and inspect 2D and 3D scenes with full property reflection.
- **Visual Feedback & Multimodal Inspection**: Capture viewport frames, active camera views, or isolated node renderings directly into base64 images for multimodal LLMs.
- **Live Scripting & Diagnostics**: Read, create, update, and validate GDScript files with real-time parser error detection and symbol resolution.
- **Safe Editor Integration**: All editor mutations route through `EditorUndoRedoManager`, giving human developers full undo/redo control (`Ctrl+Z`) over AI-generated changes.
- **Runtime Debugging & Log Streaming**: Launch game scenes, stream standard output and errors in real time, monitor breakpoints, and capture runtime stack traces.

## Architecture

The system consists of two tightly coupled components:

1. **Godot Editor Bridge Plugin (`addons/godot_mcp/`)**:
   A lightweight, zero-dependency GDScript plugin running inside the Godot Editor. It opens a local WebSocket server (default port `9333`) providing real-time scene tree inspection, editor command execution, and viewport screenshot streaming.

2. **Core MCP Server (Rust)**:
   A high-speed binary server implementing the Model Context Protocol over stdio. It manages bidirectional communication between the AI assistant and the Godot Editor bridge, or invokes Godot in headless CLI mode when offline operations are requested.

## Prerequisites

- Godot Engine 4.3+ (Standard or .NET builds, verified on 4.8.dev)
- Rust toolchain 1.80+ (for building from source; precompiled binaries require no toolchain)

## Configuration

The server reads environment variables from a local `.env` file or from the host system:

```env
# Path to the primary Godot executable
# Windows: "C:\Program Files\Godot\Godot.exe"
# Linux: "/usr/bin/godot"
# macOS: "/Applications/Godot.app/Contents/MacOS/Godot"
GODOT_PATH="C:\Program Files\Godot\Godot.exe"

# Path to the Godot console wrapper (Windows) for synchronous log capturing
GODOT_CONSOLE_PATH="C:\Program Files\Godot\Godot_console.exe"

# Communication port between MCP server and Godot Editor bridge
GODOT_DEBUG_PORT="9333"

# Optional remote MCP URL when running in HTTP/SSE transport mode
GODOT_MCP_URL=""
```

## Installation

### 1. Build the MCP Server

Build the optimized release binary for your platform:

```bash
# Build optimized native binary
cargo build --release

# The compiled binary is located at:
# Windows: target/release/godot-mcp.exe
# Linux:   target/release/godot-mcp
# macOS:   target/release/godot-mcp
```

### 2. Godot Editor Plugin Setup

1. Copy or symlink the `addons/godot_mcp` directory into your target Godot project's `addons/` folder:

   ```
   your_godot_project/
   └── addons/
       └── godot_mcp/
           ├── plugin.cfg
           ├── plugin.gd
           └── bridge_server.gd
   ```

2. Open your project in Godot Editor.
3. Go to **Project -> Project Settings -> Plugins** and enable **Godot MCP Bridge**.
4. The plugin automatically begins listening on the port configured in `.env` (default: `9333`).

### 3. MCP Client Configuration

#### Claude Desktop

Add the server to your `claude_desktop_config.json`:

Windows:

```json
{
  "mcpServers": {
    "godot": {
      "command": "C:/path/to/godot-mcp/target/release/godot-mcp.exe",
      "args": [],
      "env": {
        "GODOT_PATH": "C:\\Program Files\\Godot\\Godot.exe",
        "GODOT_CONSOLE_PATH": "C:\\Program Files\\Godot\\Godot_console.exe",
        "GODOT_DEBUG_PORT": "9333"
      }
    }
  }
}
```

Linux / macOS:

```json
{
  "mcpServers": {
    "godot": {
      "command": "/path/to/godot-mcp/target/release/godot-mcp",
      "args": [],
      "env": {
        "GODOT_PATH": "/usr/bin/godot",
        "GODOT_DEBUG_PORT": "9333"
      }
    }
  }
}
```

#### Cursor / Antigravity / VS Code

Configure the server in your MCP settings (`.cursor/mcp.json` or `.gemini/antigravity/mcp/godot.json`):

```json
{
  "mcpServers": {
    "godot": {
      "command": "/path/to/godot-mcp/target/release/godot-mcp.exe",
      "args": []
    }
  }
}
```

## Autonomous AI Setup Prompt

Copy and paste this prompt to allow an AI assistant to detect your environment and install the Godot MCP server automatically:

```text
You are an expert systems engineer tasked with configuring and verifying the Godot MCP server on this machine.

Follow this exact execution plan:
1. Environment Detection:
   - Identify the host operating system (Windows, Linux, or macOS).
   - Inspect `.env` in the repository root and verify that `GODOT_PATH` points to a valid Godot executable.
   - Run `<GODOT_PATH> --version` to confirm Godot is operational.
2. Build the Server Binary:
   - Run `cargo build --release` in the project root.
   - Confirm that the binary exists (`target/release/godot-mcp.exe` on Windows, `target/release/godot-mcp` on Linux/macOS).
3. Install the Godot Addon:
   - Identify the user's active Godot project directory.
   - Copy `addons/godot_mcp` into `<target_project>/addons/godot_mcp`.
   - Ensure the plugin is enabled in `<target_project>/project.godot` under `[editor_plugins]` with `enabled=PackedStringArray("res://addons/godot_mcp/plugin.cfg")`.
4. Register the MCP Client:
   - Locate the active MCP client configuration file (e.g., Claude Desktop, Cursor, or Antigravity).
   - Register the server under the key "godot" with the full path to the compiled binary and the required environment variables.
5. End-to-End Verification:
   - Execute the test suite using `cargo test`.
   - Trigger a basic verification call (e.g., `godot_get_version` or `godot_get_scene_tree`).
   - Confirm healthy communication and report the detected version and tool list.
```

## Quality Assurance & Development

The codebase enforces strict testing, formatting, and linting standards:

### Running Tests

```bash
# Run all unit and integration tests
cargo test

# Run tests with output logging
cargo test -- --nocapture
```

### Code Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Static Analysis & Lints

```bash
# Run Clippy with zero warnings allowed
cargo clippy --all-targets --all-features -- -D warnings
```

### GDScript Addon Validation

```bash
# Validate GDScript syntax using Godot CLI
godot --headless --check-only --script addons/godot_mcp/plugin.gd
```

## Available MCP Tools

### Project & Engine Control

- `godot_get_version`: Returns the engine version and build metadata.
- `godot_get_project_info`: Inspects `project.godot` settings, render pipeline, and autoloads.
- `godot_launch_editor`: Launches the Godot Editor for a designated project path.
- `godot_run_project`: Runs the active project or a specific scene in debug mode.
- `godot_stop_project`: Stops any active debug instance.

### Scene Management

- `godot_create_scene`: Creates a new `.tscn` scene with a specified root node type and script.
- `godot_save_scene`: Saves changes to an open or specified scene.
- `godot_get_scene_tree`: Retrieves the hierarchical node tree of the current open scene.
- `godot_open_scene`: Opens a scene file in the editor.

### Node Operations

- `godot_add_node`: Adds a new child node to a specified parent path in the scene tree.
- `godot_remove_node`: Removes a node by its node path.
- `godot_set_node_properties`: Modifies exported properties, transforms, or materials on a target node.
- `godot_get_node_properties`: Inspects all properties, types, and current values of a target node.

### Scripting & Resources

- `godot_create_script`: Generates a new GDScript with standard templates and class definitions.
- `godot_attach_script`: Attaches an existing script to a target node.
- `godot_validate_script`: Runs static analysis and syntax checks on a GDScript file without execution.
- `godot_resave_resources`: Forces re-import and cache invalidation for updated assets and resources.

### Multimodal & Visual Inspection

- `godot_capture_viewport`: Captures an image of the current 2D/3D editor viewport as a base64 PNG.
- `godot_capture_camera`: Captures the view from an in-scene Camera2D or Camera3D node.

### Diagnostics & Console

- `godot_get_debug_log`: Retrieves recent console outputs, engine warnings, and script errors.
- `godot_get_stack_trace`: Fetches stack trace details when execution is paused on error.

## License

MIT License. See `LICENSE` for details.
