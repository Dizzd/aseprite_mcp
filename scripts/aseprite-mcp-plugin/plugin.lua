----------------------------------------------------------------------
-- Aseprite MCP Plugin - WebSocket Bridge
-- 
-- This plugin connects a running Aseprite instance to the MCP server
-- via WebSocket for interactive real-time communication.
-- 
-- Install: Copy this folder to Aseprite's scripts/extensions directory.
-- Usage:   The plugin auto-connects when Aseprite starts.
--          It listens for JSON commands from the MCP server.
----------------------------------------------------------------------

-- Configuration
local CONFIG = {
    host = "127.0.0.1",
    port = 9876,
    reconnect_min = 1,
    reconnect_max = 30,
    debug = false,
}

-- Read port from environment or use default
local port = CONFIG.port

local ws = nil
local connected = false

-- Logging helper
local function log(msg)
    if CONFIG.debug then
        print("[MCP Plugin] " .. msg)
    end
end

-- Execute a command received from the MCP server
local function execute_command(cmd)
    local ok, result = pcall(function()
        local action = cmd.action

        if action == "ping" then
            return { status = "pong" }

        elseif action == "get_sprite_info" then
            local spr = app.sprite
            if not spr then
                return { error = "No sprite open" }
            end
            local info = {}
            info.filename = spr.filename
            info.width = spr.width
            info.height = spr.height
            info.colorMode = tostring(spr.colorMode)
            info.numFrames = #spr.frames
            info.numLayers = #spr.layers
            info.numCels = #spr.cels
            info.isModified = spr.isModified
            return info

        elseif action == "run_script" then
            local script_code = cmd.script
            if not script_code then
                return { error = "No script provided" }
            end
            -- Execute the Lua code
            local fn, err = load(script_code)
            if not fn then
                return { error = "Script parse error: " .. tostring(err) }
            end
            local exec_ok, exec_result = pcall(fn)
            if not exec_ok then
                return { error = "Script runtime error: " .. tostring(exec_result) }
            end
            return { status = "ok", result = exec_result }

        elseif action == "create_sprite" then
            local w = cmd.width or 32
            local h = cmd.height or 32
            local spr = Sprite(w, h)
            if cmd.filename then
                spr:saveAs(cmd.filename)
            end
            return { status = "created", width = spr.width, height = spr.height }

        elseif action == "use_tool" then
            local tool_name = cmd.tool or "pencil"
            local points = cmd.points or {}
            local lua_points = {}
            for _, p in ipairs(points) do
                table.insert(lua_points, Point(p.x, p.y))
            end
            local color = Color(
                cmd.color and cmd.color.r or 0,
                cmd.color and cmd.color.g or 0,
                cmd.color and cmd.color.b or 0,
                cmd.color and cmd.color.a or 255
            )
            app.useTool {
                tool = tool_name,
                color = color,
                points = lua_points,
                brush = Brush(cmd.brush_size or 1),
            }
            app.refresh()
            return { status = "drawn", tool = tool_name }

        elseif action == "save" then
            local spr = app.sprite
            if not spr then
                return { error = "No sprite open" }
            end
            if cmd.filename then
                spr:saveCopyAs(cmd.filename)
            else
                spr:saveAs(spr.filename)
            end
            return { status = "saved", filename = spr.filename }

        elseif action == "refresh" then
            app.refresh()
            return { status = "refreshed" }

        else
            return { error = "Unknown action: " .. tostring(action) }
        end
    end)

    if ok then
        return result
    else
        return { error = "Execution error: " .. tostring(result) }
    end
end

-- Connect to the MCP server WebSocket
local function connect()
    if ws then
        ws:close()
    end

    local url = string.format("ws://%s:%d", CONFIG.host, port)
    log("Connecting to " .. url)

    ws = WebSocket {
        url = url,
        onreceive = function(messageType, data)
            if messageType == WebSocketMessageType.OPEN then
                connected = true
                log("Connected to MCP server")
                -- Send hello
                ws:sendText(json.encode({
                    type = "hello",
                    version = app.version.string or "unknown",
                    apiVersion = app.apiVersion or 0,
                }))

            elseif messageType == WebSocketMessageType.TEXT then
                log("Received: " .. data)
                local ok, cmd = pcall(json.decode, data)
                if ok and cmd then
                    local result = execute_command(cmd)
                    result.id = cmd.id  -- Echo back request ID for correlation
                    local response = json.encode(result)
                    log("Sending: " .. response)
                    ws:sendText(response)
                else
                    ws:sendText(json.encode({
                        error = "Invalid JSON",
                        id = nil,
                    }))
                end

            elseif messageType == WebSocketMessageType.CLOSE then
                connected = false
                log("Disconnected from MCP server")

            elseif messageType == WebSocketMessageType.PING then
                -- Auto-handled by Aseprite
            end
        end,
        deflate = false,
        minreconnectwait = CONFIG.reconnect_min,
        maxreconnectwait = CONFIG.reconnect_max,
    }

    ws:connect()
end

-- Initialize plugin
function init(plugin)
    log("Aseprite MCP Plugin initialized")

    -- Register a menu command to manually connect/disconnect
    plugin:newMenuGroup {
        id = "mcp_group",
        title = "MCP Server",
        group = "help_about",
    }

    plugin:newCommand {
        id = "mcp_connect",
        title = "Connect to MCP Server",
        group = "mcp_group",
        onclick = function()
            connect()
            app.alert("MCP: Connecting to " .. CONFIG.host .. ":" .. port)
        end,
    }

    plugin:newCommand {
        id = "mcp_disconnect",
        title = "Disconnect from MCP Server",
        group = "mcp_group",
        onclick = function()
            if ws then
                ws:close()
                connected = false
                app.alert("MCP: Disconnected")
            end
        end,
    }

    plugin:newCommand {
        id = "mcp_status",
        title = "MCP Connection Status",
        group = "mcp_group",
        onclick = function()
            if connected then
                app.alert("MCP: Connected to " .. CONFIG.host .. ":" .. port)
            else
                app.alert("MCP: Not connected")
            end
        end,
    }

    -- Auto-connect on startup (optional)
    -- Uncomment the line below to auto-connect when Aseprite starts
    -- connect()
end

function exit(plugin)
    if ws then
        ws:close()
    end
    log("Aseprite MCP Plugin exited")
end
