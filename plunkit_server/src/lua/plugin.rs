use super::engine::LuaEngine;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

pub struct Plugin {
    pub metadata: PluginMetadata,
    pub engine: Arc<LuaEngine>,
    pub enabled: bool,
}

impl Plugin {
    pub async fn new(metadata: PluginMetadata, script: &str) -> Result<Self> {
        let engine = Arc::new(LuaEngine::new()?);

        engine.setup_api().await?;

        engine.load_script(script).await?;

        Ok(Self {
            metadata,
            engine,
            enabled: true,
        })
    }

    pub async fn enable(&mut self) -> Result<()> {
        if !self.enabled {
            self.engine.call_function("on_enable", vec![]).await?;
            self.enabled = true;
        }
        Ok(())
    }

    pub async fn disable(&mut self) -> Result<()> {
        if self.enabled {
            self.engine.call_function("on_disable", vec![]).await?;
            self.enabled = false;
        }
        Ok(())
    }

    pub async fn on_player_join(&self, player_name: &str) -> Result<()> {
        if self.enabled {
            self.engine
                .call_function(
                    "on_player_join",
                    vec![mlua::Value::String(
                        self.engine.lua.read().await.create_string(player_name)?,
                    )],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn on_player_chat(&self, player_name: &str, message: &str) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let lua = self.engine.lua.read().await;
        let results = self
            .engine
            .call_function(
                "on_player_chat",
                vec![
                    mlua::Value::String(lua.create_string(player_name)?),
                    mlua::Value::String(lua.create_string(message)?),
                ],
            )
            .await?;

        if let Some(mlua::Value::Boolean(cancelled)) = results.first() {
            Ok(*cancelled)
        } else {
            Ok(false)
        }
    }

    pub async fn on_block_break(
        &self,
        player_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let lua = self.engine.lua.read().await;
        let results = self
            .engine
            .call_function(
                "on_block_break",
                vec![
                    mlua::Value::String(lua.create_string(player_name)?),
                    mlua::Value::Integer(x),
                    mlua::Value::Integer(y),
                    mlua::Value::Integer(z),
                ],
            )
            .await?;

        if let Some(mlua::Value::Boolean(cancelled)) = results.first() {
            Ok(*cancelled)
        } else {
            Ok(false)
        }
    }

    pub async fn on_block_place(
        &self,
        player_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_id: i32,
    ) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let lua = self.engine.lua.read().await;
        let results = self
            .engine
            .call_function(
                "on_block_place",
                vec![
                    mlua::Value::String(lua.create_string(player_name)?),
                    mlua::Value::Integer(x),
                    mlua::Value::Integer(y),
                    mlua::Value::Integer(z),
                    mlua::Value::Integer(block_id),
                ],
            )
            .await?;

        if let Some(mlua::Value::Boolean(cancelled)) = results.first() {
            Ok(*cancelled)
        } else {
            Ok(false)
        }
    }
}

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Arc<RwLock<Plugin>>>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_plugin(
        &self,
        metadata: PluginMetadata,
        script: &str,
    ) -> Result<()> {
        let plugin = Arc::new(RwLock::new(Plugin::new(metadata.clone(), script).await?));

        let mut plugins = self.plugins.write().await;
        plugins.insert(metadata.name.clone(), plugin);

        Ok(())
    }

    pub async fn unload_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(plugin_arc) = plugins.get(name) {
            let mut plugin = plugin_arc.write().await;
            plugin.disable().await?;
        }

        plugins.remove(name);
        Ok(())
    }

    pub async fn enable_plugin(&self, name: &str) -> Result<()> {
        let plugins = self.plugins.read().await;

        if let Some(plugin_arc) = plugins.get(name) {
            let mut plugin = plugin_arc.write().await;
            plugin.enable().await?;
        }

        Ok(())
    }

    pub async fn disable_plugin(&self, name: &str) -> Result<()> {
        let plugins = self.plugins.read().await;

        if let Some(plugin_arc) = plugins.get(name) {
            let mut plugin = plugin_arc.write().await;
            plugin.disable().await?;
        }

        Ok(())
    }

    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;

        let mut metadata_list = Vec::new();
        for plugin_arc in plugins.values() {
            let plugin = plugin_arc.read().await;
            metadata_list.push(plugin.metadata.clone());
        }

        metadata_list
    }

    pub async fn on_player_join(&self, player_name: &str) -> Result<()> {
        let plugins = self.plugins.read().await;

        for plugin_arc in plugins.values() {
            let plugin = plugin_arc.read().await;
            if let Err(e) = plugin.on_player_join(player_name).await {
                tracing::error!("Error in plugin {}: {}", plugin.metadata.name, e);
            }
        }

        Ok(())
    }

    pub async fn on_player_chat(&self, player_name: &str, message: &str) -> Result<bool> {
        let plugins = self.plugins.read().await;

        for plugin_arc in plugins.values() {
            let plugin = plugin_arc.read().await;
            match plugin.on_player_chat(player_name, message).await {
                Ok(true) => return Ok(true),
                Err(e) => tracing::error!("Error in plugin {}: {}", plugin.metadata.name, e),
                _ => {}
            }
        }

        Ok(false)
    }

    pub async fn on_block_break(
        &self,
        player_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<bool> {
        let plugins = self.plugins.read().await;

        for plugin_arc in plugins.values() {
            let plugin = plugin_arc.read().await;
            match plugin.on_block_break(player_name, x, y, z).await {
                Ok(true) => return Ok(true),
                Err(e) => tracing::error!("Error in plugin {}: {}", plugin.metadata.name, e),
                _ => {}
            }
        }

        Ok(false)
    }

    pub async fn on_block_place(
        &self,
        player_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_id: i32,
    ) -> Result<bool> {
        let plugins = self.plugins.read().await;

        for plugin_arc in plugins.values() {
            let plugin = plugin_arc.read().await;
            match plugin.on_block_place(player_name, x, y, z, block_id).await {
                Ok(true) => return Ok(true),
                Err(e) => tracing::error!("Error in plugin {}: {}", plugin.metadata.name, e),
                _ => {}
            }
        }

        Ok(false)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
