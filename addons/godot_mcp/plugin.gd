@tool
extends EditorPlugin

var bridge_node: GodotMcpBridgeServer = null

func _enter_tree() -> void:
    print("[Godot MCP] Initializing Godot MCP plugin...")
    bridge_node = GodotMcpBridgeServer.new()
    bridge_node.name = "GodotMcpBridgeServer"
    bridge_node.undo_redo = get_undo_redo()
    add_child(bridge_node)
    print("[Godot MCP] Plugin activated successfully")

func _exit_tree() -> void:
    if bridge_node != null:
        bridge_node.stop_server()
        bridge_node.queue_free()
        bridge_node = null
    print("[Godot MCP] Plugin deactivated")

