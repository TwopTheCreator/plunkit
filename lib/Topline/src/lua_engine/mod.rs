use anyhow::{Context, Result, anyhow};
use mlua::{Lua, Table, Function, Value as LuaValue};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;

pub struct LuaPluginEngine {
    lua: Arc<RwLock<Lua>>,
    loaded_plugins: Arc<DashMap<String, LuaPlugin>>,
}

#[derive(Debug, Clone)]
pub struct LuaPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub enabled: bool,
}

impl LuaPluginEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        let engine = Self {
            lua: Arc::new(RwLock::new(lua)),
            loaded_plugins: Arc::new(DashMap::new()),
        };

        engine.setup_sandbox()?;

        Ok(engine)
    }

    fn setup_sandbox(&self) -> Result<()> {
        let lua = self.lua.write();

        lua.load(r#"
            topline = {}
            topline.version = "1.0.0"

            topline.log = function(message)
                print("[Topline] " .. tostring(message))
            end

            topline.register_event = function(event_name, callback)
                if not topline.events then
                    topline.events = {}
                end
                if not topline.events[event_name] then
                    topline.events[event_name] = {}
                end
                table.insert(topline.events[event_name], callback)
            end

            topline.trigger_event = function(event_name, ...)
                if topline.events and topline.events[event_name] then
                    for _, callback in ipairs(topline.events[event_name]) do
                        callback(...)
                    end
                end
            end

            topline.register_command = function(command_name, callback)
                if not topline.commands then
                    topline.commands = {}
                end
                topline.commands[command_name] = callback
            end

            topline.execute_command = function(command_name, ...)
                if topline.commands and topline.commands[command_name] then
                    return topline.commands[command_name](...)
                end
                return nil
            end

            minecraft = {}

            minecraft.get_player = function(name)
                return {
                    name = name,
                    health = 20,
                    max_health = 20,
                    position = {x = 0, y = 64, z = 0}
                }
            end

            minecraft.get_world = function()
                return {
                    name = "world",
                    time = 0,
                    weather = "clear"
                }
            end

            minecraft.spawn_entity = function(entity_type, x, y, z)
                topline.log("Spawning " .. entity_type .. " at " .. x .. ", " .. y .. ", " .. z)
                return true
            end

            minecraft.set_block = function(x, y, z, block_type)
                topline.log("Setting block at " .. x .. ", " .. y .. ", " .. z .. " to " .. block_type)
                return true
            end

            minecraft.get_block = function(x, y, z)
                return "air"
            end
        "#).exec()?;

        Ok(())
    }

    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();
        let lua = self.lua.write();

        let content = fs::read_to_string(path)
            .context("Failed to read Lua plugin file")?;

        lua.load(&content).exec()
            .context("Failed to execute Lua plugin")?;

        let globals = lua.globals();
        let plugin_info: Table = globals.get("plugin")
            .context("Plugin must define a 'plugin' table")?;

        let id: String = plugin_info.get("id")
            .context("Plugin must have an 'id' field")?;
        let name: String = plugin_info.get("name")
            .context("Plugin must have a 'name' field")?;
        let version: String = plugin_info.get("version")
            .unwrap_or_else(|_| "1.0.0".to_string());

        let plugin = LuaPlugin {
            id: id.clone(),
            name,
            version,
            path: path.to_path_buf(),
            enabled: true,
        };

        if let Ok(on_load): Result<Function, _> = plugin_info.get("on_load") {
            on_load.call::<_, ()>(())
                .context("Failed to call plugin on_load function")?;
        }

        self.loaded_plugins.insert(id.clone(), plugin);

        println!("Loaded Lua plugin: {} ({})", id, path.display());

        Ok(id)
    }

    pub fn load_plugins_from_directory<P: AsRef<Path>>(&self, directory: P) -> Result<Vec<String>> {
        let directory = directory.as_ref();

        if !directory.exists() {
            fs::create_dir_all(directory)?;
            return Ok(Vec::new());
        }

        let mut loaded_ids = Vec::new();

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("lua") {
                match self.load_plugin(&path) {
                    Ok(id) => loaded_ids.push(id),
                    Err(e) => eprintln!("Failed to load plugin {}: {}", path.display(), e),
                }
            }
        }

        Ok(loaded_ids)
    }

    pub fn call_plugin_function(&self, plugin_id: &str, function_name: &str, args: Vec<LuaValue>) -> Result<LuaValue> {
        if !self.loaded_plugins.contains_key(plugin_id) {
            return Err(anyhow!("Plugin not loaded: {}", plugin_id));
        }

        let lua = self.lua.write();
        let globals = lua.globals();

        let plugin_table: Table = globals.get("plugin")
            .context("Plugin table not found")?;

        let function: Function = plugin_table.get(function_name)
            .context(format!("Function {} not found in plugin", function_name))?;

        let result = function.call::<_, LuaValue>(mlua::MultiValue::from_vec(args))
            .context("Failed to call plugin function")?;

        Ok(result)
    }

    pub fn trigger_event(&self, event_name: &str, args: Vec<LuaValue>) -> Result<()> {
        let lua = self.lua.write();
        let globals = lua.globals();

        let topline: Table = globals.get("topline")?;
        let trigger_event: Function = topline.get("trigger_event")?;

        let mut call_args = vec![lua.create_string(event_name)?.into()];
        call_args.extend(args);

        trigger_event.call::<_, ()>(mlua::MultiValue::from_vec(call_args))?;

        Ok(())
    }

    pub fn get_loaded_plugins(&self) -> Vec<LuaPlugin> {
        self.loaded_plugins.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        if let Some((_, plugin)) = self.loaded_plugins.remove(plugin_id) {
            println!("Unloaded plugin: {}", plugin.name);
            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_id))
        }
    }

    pub fn reload_plugin(&self, plugin_id: &str) -> Result<()> {
        let path = self.loaded_plugins.get(plugin_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

        self.unload_plugin(plugin_id)?;
        self.load_plugin(&path)?;

        Ok(())
    }

    pub fn create_example_plugin<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let example_plugin = r#"
-- Example Topline Lua Plugin
plugin = {
    id = "example_plugin",
    name = "Example Plugin",
    version = "1.0.0",
    author = "Topline Team",
    description = "An example plugin demonstrating Topline's Lua API"
}

function plugin.on_load()
    topline.log("Example plugin loaded!")

    topline.register_event("player_join", function(player_name)
        topline.log("Player joined: " .. player_name)
    end)

    topline.register_command("hello", function(player_name)
        topline.log("Hello from " .. player_name .. "!")
        return "Hello, " .. player_name .. "!"
    end)
end

function plugin.on_tick()
end

function plugin.on_player_join(player_name)
    local player = minecraft.get_player(player_name)
    topline.log("Player " .. player.name .. " joined with " .. player.health .. " health")
end

function plugin.custom_function(arg1, arg2)
    return arg1 + arg2
end

topline.log("Example plugin initialized")
"#;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, example_plugin)
            .context("Failed to write example plugin")?;

        Ok(())
    }
}
