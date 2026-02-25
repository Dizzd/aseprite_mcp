/// Reusable Lua function for finding a layer by name (searches groups recursively).
/// After including this snippet, call: `find_layer(spr.layers, "name")`
pub const LUA_FIND_LAYER: &str = r#"
local function find_layer(lyrs, name)
    for i, l in ipairs(lyrs) do
        if l.name == name then return l end
        if l.isGroup and l.layers then
            local found = find_layer(l.layers, name)
            if found then return found end
        end
    end
    return nil
end"#;

/// Lua snippet to select a target layer by name. Uses `find_layer` (must include LUA_FIND_LAYER first).
/// Sets `app.layer = target_layer` if found, otherwise prints error JSON and returns.
pub fn lua_select_layer(layer_name: &str, error_on_missing: bool) -> String {
    let name = crate::aseprite::lua_string(layer_name);
    if error_on_missing {
        format!(
            r#"
local target_layer = find_layer(spr.layers, {name})
if not target_layer then
    print(json.encode({{error = "Layer not found: " .. {name}}}))
    return
end
app.layer = target_layer"#,
            name = name
        )
    } else {
        format!(
            r#"
local target_layer = find_layer(spr.layers, {name})
if target_layer then app.layer = target_layer end"#,
            name = name
        )
    }
}
