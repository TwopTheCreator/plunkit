use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToplineConfig {
    pub plugin_directory: PathBuf,
    pub mod_directory: PathBuf,
    pub lua_plugin_directory: PathBuf,
    pub config_directory: PathBuf,
    pub cache_directory: PathBuf,

    pub enable_fabric_mods: bool,
    pub enable_forge_mods: bool,
    pub enable_lua_plugins: bool,

    pub performance_mode: PerformanceMode,
    pub memory_allocation_gb: usize,

    pub optimization_level: OptimizationLevel,
    pub enable_async_loading: bool,
    pub enable_parallel_processing: bool,
    pub max_worker_threads: usize,

    pub auto_update_mods: bool,
    pub check_compatibility: bool,

    pub log_level: LogLevel,
    pub enable_metrics: bool,

    pub supabase_sync: bool,
    pub backup_configs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceMode {
    Balanced,
    MaxPerformance,
    PowerSaver,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Default for ToplineConfig {
    fn default() -> Self {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Topline");

        Self {
            plugin_directory: base_dir.join("plugins"),
            mod_directory: base_dir.join("mods"),
            lua_plugin_directory: base_dir.join("lua_plugins"),
            config_directory: base_dir.join("config"),
            cache_directory: base_dir.join("cache"),

            enable_fabric_mods: true,
            enable_forge_mods: true,
            enable_lua_plugins: true,

            performance_mode: PerformanceMode::MaxPerformance,
            memory_allocation_gb: 4,

            optimization_level: OptimizationLevel::Maximum,
            enable_async_loading: true,
            enable_parallel_processing: true,
            max_worker_threads: num_cpus::get(),

            auto_update_mods: false,
            check_compatibility: true,

            log_level: LogLevel::Info,
            enable_metrics: true,

            supabase_sync: false,
            backup_configs: true,
        }
    }
}

impl ToplineConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;

        let config: ToplineConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path.as_ref(), content)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn create_default_config<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::default();
        config.save_to_file(path)?;
        Ok(config)
    }

    pub fn ensure_directories_exist(&self) -> Result<()> {
        fs::create_dir_all(&self.plugin_directory)?;
        fs::create_dir_all(&self.mod_directory)?;
        fs::create_dir_all(&self.lua_plugin_directory)?;
        fs::create_dir_all(&self.config_directory)?;
        fs::create_dir_all(&self.cache_directory)?;
        Ok(())
    }

    pub fn get_performance_multiplier(&self) -> f32 {
        match self.optimization_level {
            OptimizationLevel::None => 1.0,
            OptimizationLevel::Basic => 1.5,
            OptimizationLevel::Aggressive => 2.5,
            OptimizationLevel::Maximum => 3.0,
        }
    }
}

pub struct EditorConfigGenerator;

impl EditorConfigGenerator {
    pub fn generate_rust_config_file<P: AsRef<Path>>(path: P, config: &ToplineConfig) -> Result<()> {
        let rust_config = format!(
r#"// topline.editorconfig.rs
// Topline Configuration File
// This file is auto-generated but can be customized

use std::path::PathBuf;

pub struct ToplineEditorConfig {{
    // Directory Configuration
    pub plugin_directory: PathBuf,
    pub mod_directory: PathBuf,
    pub lua_plugin_directory: PathBuf,
    pub config_directory: PathBuf,
    pub cache_directory: PathBuf,

    // Feature Flags
    pub enable_fabric_mods: bool,
    pub enable_forge_mods: bool,
    pub enable_lua_plugins: bool,

    // Performance Settings
    pub memory_allocation_gb: usize,
    pub optimization_level: usize,  // 0-3 (None, Basic, Aggressive, Maximum)
    pub enable_async_loading: bool,
    pub enable_parallel_processing: bool,
    pub max_worker_threads: usize,

    // Advanced Features
    pub auto_update_mods: bool,
    pub check_compatibility: bool,
    pub enable_metrics: bool,
    pub supabase_sync: bool,
    pub backup_configs: bool,
}}

impl ToplineEditorConfig {{
    pub fn load() -> Self {{
        Self {{
            plugin_directory: PathBuf::from("{}"),
            mod_directory: PathBuf::from("{}"),
            lua_plugin_directory: PathBuf::from("{}"),
            config_directory: PathBuf::from("{}"),
            cache_directory: PathBuf::from("{}"),

            enable_fabric_mods: {},
            enable_forge_mods: {},
            enable_lua_plugins: {},

            memory_allocation_gb: {},
            optimization_level: {},
            enable_async_loading: {},
            enable_parallel_processing: {},
            max_worker_threads: {},

            auto_update_mods: {},
            check_compatibility: {},
            enable_metrics: {},
            supabase_sync: {},
            backup_configs: {},
        }}
    }}

    pub fn apply_custom_settings(&mut self) {{
        // Add your custom configuration logic here

        // Example: Override plugin directory
        // self.plugin_directory = PathBuf::from("/custom/path/to/plugins");

        // Example: Increase memory allocation
        // self.memory_allocation_gb = 8;

        // Example: Disable certain features
        // self.enable_forge_mods = false;
    }}
}}
"#,
            config.plugin_directory.display(),
            config.mod_directory.display(),
            config.lua_plugin_directory.display(),
            config.config_directory.display(),
            config.cache_directory.display(),
            config.enable_fabric_mods,
            config.enable_forge_mods,
            config.enable_lua_plugins,
            config.memory_allocation_gb,
            config.optimization_level as usize,
            config.enable_async_loading,
            config.enable_parallel_processing,
            config.max_worker_threads,
            config.auto_update_mods,
            config.check_compatibility,
            config.enable_metrics,
            config.supabase_sync,
            config.backup_configs,
        );

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, rust_config)
            .context("Failed to write Rust config file")?;

        Ok(())
    }
}

use num_cpus;
