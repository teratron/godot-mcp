@tool
class_name GodotMcpBridgeServer
extends Node

## Port on which the WebSocket bridge server listens.
const DEFAULT_PORT: int = 9333

var server: TCPServer = TCPServer.new()
var peers: Array[WebSocketPeer] = []
var undo_redo: EditorUndoRedoManager = null
var port: int = DEFAULT_PORT

func _ready() -> void:
    read_configuration()
    start_server()

func read_configuration() -> void:
    # Read custom port from project settings if defined
    if ProjectSettings.has_setting("godot_mcp/bridge_port"):
        port = int(ProjectSettings.get_setting("godot_mcp/bridge_port"))
    elif OS.has_environment("GODOT_DEBUG_PORT"):
        var env_port = OS.get_environment("GODOT_DEBUG_PORT")
        if env_port.is_valid_int():
            port = int(env_port)

func start_server() -> void:
    var err = server.listen(port)
    if err == OK:
        print("[Godot MCP] Bridge server listening on port: ", port)
    else:
        printerr("[Godot MCP] Failed to start bridge server on port ", port, ": ", error_string(err))

func stop_server() -> void:
    for peer in peers:
        peer.close()
    peers.clear()
    server.stop()
    print("[Godot MCP] Bridge server stopped")

func _process(_delta: float) -> void:
    if not server.is_listening():
        return

    # Check for incoming TCP connections and upgrade to WebSocket
    while server.is_connection_available():
        var tcp_conn: StreamPeerTCP = server.take_connection()
        if tcp_conn != null:
            var ws_peer: WebSocketPeer = WebSocketPeer.new()
            var err = ws_peer.accept_stream(tcp_conn)
            if err == OK:
                peers.append(ws_peer)
                print("[Godot MCP] Client connected")
            else:
                printerr("[Godot MCP] Failed to accept WebSocket connection: ", error_string(err))

    # Poll connected peers
    var i: int = peers.size() - 1
    while i >= 0:
        var peer = peers[i]
        peer.poll()
        var state = peer.get_ready_state()

        if state == WebSocketPeer.STATE_OPEN:
            while peer.get_available_packet_count() > 0:
                var packet = peer.get_packet()
                var message_str = packet.get_string_from_utf8()
                handle_message(peer, message_str)
        elif state == WebSocketPeer.STATE_CLOSED:
            peers.remove_at(i)
            print("[Godot MCP] Client disconnected")

        i -= 1

func handle_message(peer: WebSocketPeer, raw_msg: String) -> void:
    var json = JSON.new()
    var parse_err = json.parse(raw_msg)
    if parse_err != OK:
        send_error(peer, "", "Invalid JSON format: " + json.get_error_message())
        return

    var data = json.get_data()
    if not data is Dictionary:
        send_error(peer, "", "Message payload must be a JSON dictionary")
        return

    var cmd_id: String = data.get("commandId", "")
    var cmd_type: String = data.get("type", "")

    # A statically-typed `Dictionary` assignment throws a script error (not a
    # catchable exception) if "params" is present but not an object, which
    # aborts this function before any response is sent, leaving the caller
    # hanging until its own timeout. Coerce defensively instead.
    var raw_params = data.get("params", {})
    var params: Dictionary = raw_params if raw_params is Dictionary else {}
    if not raw_params is Dictionary and raw_params != null:
        send_error(peer, cmd_id, "Field 'params' must be a JSON object, got: " + type_string(typeof(raw_params)))
        return

    dispatch_command(peer, cmd_id, cmd_type, params)

func dispatch_command(peer: WebSocketPeer, cmd_id: String, cmd_type: String, params: Dictionary) -> void:
    match cmd_type:
        "ping":
            send_success(peer, cmd_id, {"pong": true, "timestamp": Time.get_unix_time_from_system()})
        "get_version":
            get_version(peer, cmd_id)
        "get_project_info":
            get_project_info(peer, cmd_id)
        "get_scene_tree":
            get_scene_tree(peer, cmd_id, params)
        "get_open_scenes":
            get_open_scenes(peer, cmd_id)
        "add_node":
            add_node(peer, cmd_id, params)
        "remove_node":
            remove_node(peer, cmd_id, params)
        "get_node_properties":
            get_node_properties(peer, cmd_id, params)
        "set_node_properties":
            set_node_properties(peer, cmd_id, params)
        "capture_viewport":
            capture_viewport(peer, cmd_id)
        "open_scene":
            open_scene(peer, cmd_id, params)
        "save_scene":
            save_scene(peer, cmd_id)
        "run_project":
            run_project(peer, cmd_id, params)
        "stop_project":
            stop_project(peer, cmd_id)
        "get_run_status":
            get_run_status(peer, cmd_id)
        "resave_resources":
            resave_resources(peer, cmd_id, params)
        _:
            send_error(peer, cmd_id, "Unknown command type: " + cmd_type)

func get_version(peer: WebSocketPeer, cmd_id: String) -> void:
    var info = Engine.get_version_info()
    send_success(peer, cmd_id, {
        "version_string": Engine.get_version_info()["string"],
        "major": info["major"],
        "minor": info["minor"],
        "patch": info["patch"],
        "status": info["status"]
    })

func get_project_info(peer: WebSocketPeer, cmd_id: String) -> void:
    var project_name: String = ProjectSettings.get_setting("application/config/name", "")
    var main_scene: String = ProjectSettings.get_setting("application/run/main_scene", "")
    send_success(peer, cmd_id, {
        "name": project_name,
        "main_scene": main_scene,
        "features": ProjectSettings.get_setting("application/config/features", [])
    })

func get_scene_tree(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var scene_path: String = params.get("scene_path", "")
    var root: Node = null

    if scene_path.is_empty():
        root = EditorInterface.get_edited_scene_root()
        if root == null:
            send_error(peer, cmd_id, "No active scene is currently open in the editor")
            return
    else:
        # get_open_scenes() and get_open_scene_roots() are parallel arrays
        # (same order, same length) per the Godot editor API.
        var open_paths: PackedStringArray = EditorInterface.get_open_scenes()
        var idx: int = open_paths.find(scene_path)
        if idx == -1:
            send_error(peer, cmd_id, "Scene is not currently open in the editor: " + scene_path + ". Use get_open_scenes to list open scenes, or open_scene to open it first.")
            return
        root = EditorInterface.get_open_scene_roots()[idx]

    var tree_data = serialize_node(root)
    send_success(peer, cmd_id, {"tree": tree_data, "scene_path": scene_path if not scene_path.is_empty() else String(root.scene_file_path)})

func get_open_scenes(peer: WebSocketPeer, cmd_id: String) -> void:
    send_success(peer, cmd_id, {
        "open_scenes": EditorInterface.get_open_scenes(),
        "active_scene": EditorInterface.get_edited_scene_root().scene_file_path if EditorInterface.get_edited_scene_root() != null else ""
    })

func serialize_node(node: Node) -> Dictionary:
    var children_data: Array[Dictionary] = []
    for child in node.get_children():
        children_data.append(serialize_node(child))

    return {
        "name": node.name,
        "class": node.get_class(),
        "path": String(node.get_path()),
        "children": children_data
    }

func add_node(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var parent_path: String = params.get("parent_node_path", "")
    var node_type: String = params.get("node_type", "Node")
    var node_name: String = params.get("node_name", "")

    var root = EditorInterface.get_edited_scene_root()
    if root == null:
        send_error(peer, cmd_id, "No active scene open in the editor")
        return

    var parent_node: Node = root if parent_path.is_empty() or parent_path == "." else root.get_node_or_null(parent_path)
    if parent_node == null:
        send_error(peer, cmd_id, "Parent node not found at path: " + parent_path)
        return

    if not ClassDB.class_exists(node_type):
        send_error(peer, cmd_id, "Unknown Godot node type: " + node_type)
        return

    var new_instance = ClassDB.instantiate(node_type)
    if new_instance == null or not new_instance is Node:
        send_error(peer, cmd_id, "Failed to instantiate node of class: " + node_type)
        return

    var new_node: Node = new_instance as Node
    if not node_name.is_empty():
        new_node.name = node_name

    if undo_redo != null:
        undo_redo.create_action("Add Node " + new_node.name)
        undo_redo.add_do_method(parent_node, "add_child", new_node)
        undo_redo.add_do_property(new_node, "owner", root)
        undo_redo.add_do_reference(new_node)
        undo_redo.add_undo_method(parent_node, "remove_child", new_node)
        undo_redo.commit_action()
    else:
        parent_node.add_child(new_node)
        new_node.owner = root

    send_success(peer, cmd_id, {
        "name": new_node.name,
        "class": new_node.get_class(),
        "path": String(new_node.get_path())
    })

func remove_node(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var node_path: String = params.get("node_path", "")
    var root = EditorInterface.get_edited_scene_root()
    if root == null:
        send_error(peer, cmd_id, "No active scene open in the editor")
        return

    var target_node: Node = root.get_node_or_null(node_path)
    if target_node == null:
        send_error(peer, cmd_id, "Node not found at path: " + node_path)
        return

    var parent = target_node.get_parent()
    if parent == null:
        send_error(peer, cmd_id, "Cannot remove the root node directly")
        return

    if undo_redo != null:
        undo_redo.create_action("Remove Node " + target_node.name)
        undo_redo.add_do_method(parent, "remove_child", target_node)
        undo_redo.add_undo_method(parent, "add_child", target_node)
        undo_redo.add_undo_property(target_node, "owner", root)
        undo_redo.add_undo_reference(target_node)
        undo_redo.commit_action()
    else:
        parent.remove_child(target_node)
        target_node.queue_free()

    send_success(peer, cmd_id, {"removed": true, "path": node_path})

func get_node_properties(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var node_path: String = params.get("node_path", "")
    var root = EditorInterface.get_edited_scene_root()
    if root == null:
        send_error(peer, cmd_id, "No active scene open in the editor")
        return

    var target_node: Node = root if node_path.is_empty() or node_path == "." else root.get_node_or_null(node_path)
    if target_node == null:
        send_error(peer, cmd_id, "Node not found at path: " + node_path)
        return

    var props: Dictionary = {}
    for prop in target_node.get_property_list():
        var prop_name = prop["name"]
        if prop["usage"] & PROPERTY_USAGE_EDITOR:
            props[prop_name] = str(target_node.get(prop_name))

    send_success(peer, cmd_id, {"node_path": node_path, "properties": props})

func set_node_properties(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var node_path: String = params.get("node_path", "")
    var properties: Dictionary = params.get("properties", {})
    var root = EditorInterface.get_edited_scene_root()
    if root == null:
        send_error(peer, cmd_id, "No active scene open in the editor")
        return

    var target_node: Node = root if node_path.is_empty() or node_path == "." else root.get_node_or_null(node_path)
    if target_node == null:
        send_error(peer, cmd_id, "Node not found at path: " + node_path)
        return

    if undo_redo != null:
        undo_redo.create_action("Set Properties on " + target_node.name)
        for key in properties.keys():
            var val = properties[key]
            var old_val = target_node.get(key)
            undo_redo.add_do_property(target_node, key, val)
            undo_redo.add_undo_property(target_node, key, old_val)
        undo_redo.commit_action()
    else:
        for key in properties.keys():
            target_node.set(key, properties[key])

    send_success(peer, cmd_id, {"updated": true, "node_path": node_path})

func capture_viewport(peer: WebSocketPeer, cmd_id: String) -> void:
    var vp = EditorInterface.get_editor_viewport_2d()
    if vp == null:
        vp = EditorInterface.get_editor_viewport_3d(0)

    if vp == null:
        send_error(peer, cmd_id, "Unable to access editor viewport")
        return

    var img = vp.get_texture().get_image()
    if img == null:
        send_error(peer, cmd_id, "Failed to capture image from viewport texture")
        return

    var png_buffer = img.save_png_to_buffer()
    var base64_str = Marshalls.raw_to_base64(png_buffer)
    send_success(peer, cmd_id, {
        "mime_type": "image/png",
        "data": base64_str,
        "width": img.get_width(),
        "height": img.get_height()
    })

func open_scene(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var scene_path: String = params.get("scene_path", "")
    if not FileAccess.file_exists(scene_path):
        send_error(peer, cmd_id, "Scene file does not exist: " + scene_path)
        return

    EditorInterface.open_scene_from_path(scene_path)
    send_success(peer, cmd_id, {"opened": true, "scene_path": scene_path})

func save_scene(peer: WebSocketPeer, cmd_id: String) -> void:
    var err = EditorInterface.save_scene()
    if err == OK:
        send_success(peer, cmd_id, {"saved": true})
    else:
        send_error(peer, cmd_id, "Failed to save scene: " + error_string(err))

func run_project(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var scene_path: String = params.get("scene_path", "")

    if not scene_path.is_empty():
        if not FileAccess.file_exists(scene_path):
            send_error(peer, cmd_id, "Scene file does not exist: " + scene_path)
            return
        EditorInterface.play_custom_scene(scene_path)
    else:
        EditorInterface.play_main_scene()

    send_success(peer, cmd_id, {
        "running": EditorInterface.is_playing_scene(),
        "scene": EditorInterface.get_playing_scene()
    })

func stop_project(peer: WebSocketPeer, cmd_id: String) -> void:
    EditorInterface.stop_playing_scene()
    send_success(peer, cmd_id, {"stopped": true})

func get_run_status(peer: WebSocketPeer, cmd_id: String) -> void:
    send_success(peer, cmd_id, {
        "running": EditorInterface.is_playing_scene(),
        "scene": EditorInterface.get_playing_scene()
    })

func resave_resources(peer: WebSocketPeer, cmd_id: String, params: Dictionary) -> void:
    var fs: EditorFileSystem = EditorInterface.get_resource_filesystem()

    if fs.is_importing():
        send_error(peer, cmd_id, "A resource import is already in progress in the editor; retry once it finishes")
        return

    var requested: Array = params.get("paths", [])
    var existing_paths: PackedStringArray = PackedStringArray()
    var missing_paths: PackedStringArray = PackedStringArray()
    for p in requested:
        var p_str: String = String(p)
        if FileAccess.file_exists(p_str):
            existing_paths.append(p_str)
        else:
            missing_paths.append(p_str)

    if requested.is_empty():
        # No specific paths: rescan the whole project for new/changed/removed
        # files. This runs on a background thread, so it is not finished by
        # the time this response is sent.
        fs.scan()
        send_success(peer, cmd_id, {"mode": "full_scan", "scanning": fs.is_scanning()})
        return

    if existing_paths.is_empty():
        send_error(peer, cmd_id, "None of the given paths exist: " + ", ".join(missing_paths))
        return

    # Unlike scan(), reimport_files() runs on the main thread and blocks
    # until it finishes, so the reimport is genuinely done by the time we
    # respond.
    fs.reimport_files(existing_paths)
    send_success(peer, cmd_id, {
        "mode": "reimport",
        "reimported": existing_paths,
        "skipped_missing": missing_paths
    })

func send_success(peer: WebSocketPeer, cmd_id: String, result: Dictionary) -> void:
    var response = {
        "commandId": cmd_id,
        "status": "success",
        "result": result
    }
    peer.send_text(JSON.stringify(response))

func send_error(peer: WebSocketPeer, cmd_id: String, err_msg: String) -> void:
    var response = {
        "commandId": cmd_id,
        "status": "error",
        "error": err_msg
    }
    peer.send_text(JSON.stringify(response))

