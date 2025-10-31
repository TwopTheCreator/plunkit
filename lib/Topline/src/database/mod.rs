use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use postgrest::Postgrest;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMod {
    pub id: Option<i32>,
    pub user_id: String,
    pub mod_id: String,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_type: String,
    pub enabled: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    pub id: Option<i32>,
    pub user_id: String,
    pub config_name: String,
    pub config_data: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLuaPlugin {
    pub id: Option<i32>,
    pub user_id: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_code: String,
    pub enabled: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct SupabaseClient {
    client: Postgrest,
    user_id: String,
}

impl SupabaseClient {
    pub fn new(supabase_url: &str, api_key: &str, user_id: String) -> Result<Self> {
        let client = Postgrest::new(format!("{}/rest/v1", supabase_url))
            .insert_header("apikey", api_key)
            .insert_header("Authorization", format!("Bearer {}", api_key));

        Ok(Self { client, user_id })
    }

    pub fn from_env(user_id: String) -> Result<Self> {
        dotenv::dotenv().ok();

        let supabase_url = std::env::var("SUPABASE_URL")
            .context("SUPABASE_URL not set in environment")?;
        let api_key = std::env::var("SUPABASE_ANON_KEY")
            .context("SUPABASE_ANON_KEY not set in environment")?;

        Self::new(&supabase_url, &api_key, user_id)
    }

    pub async fn sync_mod(&self, mod_data: &StoredMod) -> Result<()> {
        let response = self.client
            .from("topline_mods")
            .insert(serde_json::to_string(&mod_data)?)
            .execute()
            .await
            .context("Failed to sync mod to Supabase")?;

        if response.status().is_success() {
            println!("Synced mod {} to Supabase", mod_data.mod_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to sync mod: {}", response.status()))
        }
    }

    pub async fn get_user_mods(&self) -> Result<Vec<StoredMod>> {
        let response = self.client
            .from("topline_mods")
            .eq("user_id", &self.user_id)
            .select("*")
            .execute()
            .await
            .context("Failed to fetch mods from Supabase")?;

        if response.status().is_success() {
            let body = response.text().await?;
            let mods: Vec<StoredMod> = serde_json::from_str(&body)?;
            Ok(mods)
        } else {
            Err(anyhow::anyhow!("Failed to fetch mods: {}", response.status()))
        }
    }

    pub async fn sync_config(&self, config_data: &StoredConfig) -> Result<()> {
        let response = self.client
            .from("topline_configs")
            .insert(serde_json::to_string(&config_data)?)
            .execute()
            .await
            .context("Failed to sync config to Supabase")?;

        if response.status().is_success() {
            println!("Synced config {} to Supabase", config_data.config_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to sync config: {}", response.status()))
        }
    }

    pub async fn get_user_configs(&self) -> Result<Vec<StoredConfig>> {
        let response = self.client
            .from("topline_configs")
            .eq("user_id", &self.user_id)
            .select("*")
            .execute()
            .await
            .context("Failed to fetch configs from Supabase")?;

        if response.status().is_success() {
            let body = response.text().await?;
            let configs: Vec<StoredConfig> = serde_json::from_str(&body)?;
            Ok(configs)
        } else {
            Err(anyhow::anyhow!("Failed to fetch configs: {}", response.status()))
        }
    }

    pub async fn sync_lua_plugin(&self, plugin_data: &StoredLuaPlugin) -> Result<()> {
        let response = self.client
            .from("topline_lua_plugins")
            .insert(serde_json::to_string(&plugin_data)?)
            .execute()
            .await
            .context("Failed to sync Lua plugin to Supabase")?;

        if response.status().is_success() {
            println!("Synced Lua plugin {} to Supabase", plugin_data.plugin_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to sync Lua plugin: {}", response.status()))
        }
    }

    pub async fn get_user_lua_plugins(&self) -> Result<Vec<StoredLuaPlugin>> {
        let response = self.client
            .from("topline_lua_plugins")
            .eq("user_id", &self.user_id)
            .select("*")
            .execute()
            .await
            .context("Failed to fetch Lua plugins from Supabase")?;

        if response.status().is_success() {
            let body = response.text().await?;
            let plugins: Vec<StoredLuaPlugin> = serde_json::from_str(&body)?;
            Ok(plugins)
        } else {
            Err(anyhow::anyhow!("Failed to fetch Lua plugins: {}", response.status()))
        }
    }

    pub async fn delete_mod(&self, mod_id: &str) -> Result<()> {
        let response = self.client
            .from("topline_mods")
            .eq("user_id", &self.user_id)
            .eq("mod_id", mod_id)
            .delete()
            .execute()
            .await
            .context("Failed to delete mod from Supabase")?;

        if response.status().is_success() {
            println!("Deleted mod {} from Supabase", mod_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to delete mod: {}", response.status()))
        }
    }

    pub async fn backup_config(&self, config_name: &str, config_path: &PathBuf) -> Result<()> {
        let config_content = std::fs::read_to_string(config_path)?;

        let stored_config = StoredConfig {
            id: None,
            user_id: self.user_id.clone(),
            config_name: config_name.to_string(),
            config_data: serde_json::json!({ "content": config_content }),
            created_at: None,
            updated_at: None,
        };

        self.sync_config(&stored_config).await?;

        Ok(())
    }

    pub async fn restore_config(&self, config_name: &str, destination: &PathBuf) -> Result<()> {
        let configs = self.get_user_configs().await?;

        let config = configs.iter()
            .find(|c| c.config_name == config_name)
            .ok_or_else(|| anyhow::anyhow!("Config not found: {}", config_name))?;

        if let Some(content) = config.config_data.get("content").and_then(|v| v.as_str()) {
            std::fs::write(destination, content)?;
            println!("Restored config {} to {}", config_name, destination.display());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid config data"))
        }
    }
}

pub fn get_database_schema() -> &'static str {
    r#"
-- Topline Supabase Schema

-- Mods table
CREATE TABLE IF NOT EXISTS topline_mods (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    mod_id TEXT NOT NULL,
    mod_name TEXT NOT NULL,
    mod_version TEXT NOT NULL,
    mod_type TEXT NOT NULL,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, mod_id)
);

-- Configs table
CREATE TABLE IF NOT EXISTS topline_configs (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    config_name TEXT NOT NULL,
    config_data JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, config_name)
);

-- Lua plugins table
CREATE TABLE IF NOT EXISTS topline_lua_plugins (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    plugin_name TEXT NOT NULL,
    plugin_code TEXT NOT NULL,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, plugin_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_topline_mods_user_id ON topline_mods(user_id);
CREATE INDEX IF NOT EXISTS idx_topline_configs_user_id ON topline_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_topline_lua_plugins_user_id ON topline_lua_plugins(user_id);

-- Row Level Security would be configured through Supabase dashboard
"#
}
