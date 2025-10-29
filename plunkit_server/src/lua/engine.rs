use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

pub struct LuaEngine {
    lua: Arc<RwLock<Lua>>,
}

impl LuaEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        Ok(Self {
            lua: Arc::new(RwLock::new(lua)),
        })
    }

    pub async fn load_script(&self, script: &str) -> Result<()> {
        let lua = self.lua.write().await;
        lua.load(script).exec()?;
        Ok(())
    }

    pub async fn setup_api(&self) -> Result<()> {
        let lua = self.lua.write().await;

        let globals = lua.globals();

        let plunkit_table = lua.create_table()?;

        let log_fn = lua.create_function(|_, msg: String| {
            tracing::info!("Lua: {}", msg);
            Ok(())
        })?;
        plunkit_table.set("log", log_fn)?;

        let get_block_fn = lua.create_async_function(|_, (x, y, z): (i32, i32, i32)| async move {
            Ok(0)
        })?;
        plunkit_table.set("get_block", get_block_fn)?;

        let set_block_fn = lua.create_async_function(
            |_, (x, y, z, block_id): (i32, i32, i32, i32)| async move {
                Ok(())
            },
        )?;
        plunkit_table.set("set_block", set_block_fn)?;

        let spawn_entity_fn = lua.create_async_function(
            |_, (entity_type, x, y, z): (String, f64, f64, f64)| async move {
                Ok(0)
            },
        )?;
        plunkit_table.set("spawn_entity", spawn_entity_fn)?;

        let broadcast_fn = lua.create_async_function(|_, message: String| async move {
            tracing::info!("Broadcast: {}", message);
            Ok(())
        })?;
        plunkit_table.set("broadcast", broadcast_fn)?;

        let get_players_fn = lua.create_async_function(|_, ()| async move {
            Ok(Vec::<String>::new())
        })?;
        plunkit_table.set("get_players", get_players_fn)?;

        let teleport_player_fn = lua.create_async_function(
            |_, (player, x, y, z): (String, f64, f64, f64)| async move {
                Ok(())
            },
        )?;
        plunkit_table.set("teleport_player", teleport_player_fn)?;

        let give_item_fn = lua.create_async_function(
            |_, (player, item, count): (String, String, i32)| async move {
                Ok(())
            },
        )?;
        plunkit_table.set("give_item", give_item_fn)?;

        globals.set("plunkit", plunkit_table)?;

        Ok(())
    }

    pub async fn call_function(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
        let lua = self.lua.read().await;

        let globals = lua.globals();
        let func: Function = globals.get(name)?;

        let results = func.call::<_, mlua::MultiValue>(mlua::MultiValue::from_vec(args))?;

        Ok(results.into_vec())
    }

    pub async fn call_hook(&self, hook_name: &str, args: Vec<Value>) -> Result<()> {
        let lua = self.lua.read().await;

        let globals = lua.globals();

        if let Ok(hooks_table) = globals.get::<_, Table>("_hooks") {
            if let Ok(hook_list) = hooks_table.get::<_, Table>(hook_name) {
                for pair in hook_list.pairs::<Value, Function>() {
                    let (_, func) = pair?;
                    if let Err(e) = func.call::<_, ()>(mlua::MultiValue::from_vec(args.clone())) {
                        tracing::error!("Error calling hook {}: {}", hook_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn register_hook(&self, hook_name: &str, callback: Function) -> Result<()> {
        let lua = self.lua.write().await;

        let globals = lua.globals();

        let hooks_table = if let Ok(table) = globals.get::<_, Table>("_hooks") {
            table
        } else {
            let table = lua.create_table()?;
            globals.set("_hooks", table.clone())?;
            table
        };

        let hook_list = if let Ok(list) = hooks_table.get::<_, Table>(hook_name) {
            list
        } else {
            let list = lua.create_table()?;
            hooks_table.set(hook_name, list.clone())?;
            list
        };

        let len = hook_list.len()? + 1;
        hook_list.set(len, callback)?;

        Ok(())
    }

    pub async fn execute_code(&self, code: &str) -> Result<Value> {
        let lua = self.lua.read().await;
        let result = lua.load(code).eval()?;
        Ok(result)
    }
}

impl Default for LuaEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
