use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::{Value, json};
use crate::minecraft_finder::MinecraftInstance;

pub struct MinecraftEditor;

impl MinecraftEditor {
    pub fn new() -> Self {
        Self
    }

    pub fn rebrand_to_topline(&self, minecraft_instance: &MinecraftInstance) -> Result<()> {
        println!("Rebranding Minecraft instance to 'Minecraft Topline'...");

        let base_path = self.get_base_path(minecraft_instance);

        self.update_launcher_profiles(&base_path)?;

        self.create_topline_profile(&base_path)?;

        self.update_window_title(&base_path)?;

        println!("Successfully rebranded to 'Minecraft Topline'!");

        Ok(())
    }

    fn get_base_path(&self, minecraft_instance: &MinecraftInstance) -> PathBuf {
        match minecraft_instance.instance_type {
            crate::minecraft_finder::InstanceType::InstallationDirectory => {
                minecraft_instance.path.clone()
            }
            _ => {
                minecraft_instance.path.parent()
                    .unwrap_or(minecraft_instance.path.as_path())
                    .to_path_buf()
            }
        }
    }

    fn update_launcher_profiles(&self, base_path: &Path) -> Result<()> {
        let profiles_path = base_path.join("launcher_profiles.json");

        if !profiles_path.exists() {
            println!("Creating new launcher_profiles.json...");
            let default_profiles = json!({
                "profiles": {
                    "Topline": {
                        "name": "Minecraft Topline",
                        "type": "latest-release",
                        "created": chrono::Utc::now().to_rfc3339(),
                        "lastUsed": chrono::Utc::now().to_rfc3339(),
                        "icon": "Furnace"
                    }
                },
                "settings": {
                    "enableSnapshots": false,
                    "enableAdvanced": true
                },
                "version": 3
            });

            fs::write(&profiles_path, serde_json::to_string_pretty(&default_profiles)?)
                .context("Failed to create launcher profiles")?;

            return Ok(());
        }

        let content = fs::read_to_string(&profiles_path)
            .context("Failed to read launcher profiles")?;

        let mut profiles: Value = serde_json::from_str(&content)
            .context("Failed to parse launcher profiles")?;

        if let Some(profiles_obj) = profiles.get_mut("profiles") {
            if let Some(profiles_map) = profiles_obj.as_object_mut() {
                for (_, profile) in profiles_map.iter_mut() {
                    if let Some(name) = profile.get_mut("name") {
                        if let Some(name_str) = name.as_str() {
                            if !name_str.contains("Topline") {
                                *name = json!(format!("{} (Topline)", name_str));
                            }
                        }
                    }
                }

                profiles_map.insert(
                    "Topline".to_string(),
                    json!({
                        "name": "Minecraft Topline",
                        "type": "latest-release",
                        "created": chrono::Utc::now().to_rfc3339(),
                        "lastUsed": chrono::Utc::now().to_rfc3339(),
                        "icon": "Furnace",
                        "javaArgs": "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200"
                    })
                );
            }
        }

        fs::write(&profiles_path, serde_json::to_string_pretty(&profiles)?)
            .context("Failed to write launcher profiles")?;

        println!("Updated launcher profiles with Topline branding");

        Ok(())
    }

    fn create_topline_profile(&self, base_path: &Path) -> Result<()> {
        let topline_dir = base_path.join(".topline");
        fs::create_dir_all(&topline_dir)
            .context("Failed to create .topline directory")?;

        let branding_file = topline_dir.join("branding.json");
        let branding_data = json!({
            "name": "Minecraft Topline",
            "version": "1.0.0",
            "description": "Enhanced Minecraft with 3x performance boost and Lua plugin support",
            "features": [
                "3x Performance Optimization",
                "Lua Plugin Support",
                "Fabric Mod Compatibility",
                "Forge Mod Compatibility",
                "Custom Configuration System"
            ],
            "branded_at": chrono::Utc::now().to_rfc3339()
        });

        fs::write(&branding_file, serde_json::to_string_pretty(&branding_data)?)
            .context("Failed to write branding file")?;

        println!("Created Topline profile at: {}", topline_dir.display());

        Ok(())
    }

    fn update_window_title(&self, base_path: &Path) -> Result<()> {
        let options_txt = base_path.join("options.txt");

        if options_txt.exists() {
            let content = fs::read_to_string(&options_txt)?;
            let mut new_content = String::new();
            let mut found_title = false;

            for line in content.lines() {
                if line.starts_with("lastServer") {
                    new_content.push_str(&format!("{}|Topline\n", line));
                    found_title = true;
                } else {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }

            if !found_title {
                new_content.push_str("\n# Topline Branding\n");
            }

            fs::write(&options_txt, new_content)?;
        }

        Ok(())
    }

    pub fn setup_topline_directories(&self, base_path: &Path) -> Result<ToplineDirectories> {
        let topline_root = base_path.join(".topline");
        fs::create_dir_all(&topline_root)?;

        let plugins_dir = topline_root.join("plugins");
        let mods_dir = topline_root.join("mods");
        let lua_plugins_dir = topline_root.join("lua_plugins");
        let config_dir = topline_root.join("config");
        let cache_dir = topline_root.join("cache");

        for dir in &[&plugins_dir, &mods_dir, &lua_plugins_dir, &config_dir, &cache_dir] {
            fs::create_dir_all(dir)?;
        }

        let readme = topline_root.join("README.txt");
        fs::write(&readme, "Topline Minecraft Enhancement System\n\
            \n\
            Directories:\n\
            - plugins/      : Server plugins (Bukkit/Spigot/Paper)\n\
            - mods/         : Client/Server mods (Fabric/Forge)\n\
            - lua_plugins/  : Custom Lua plugins\n\
            - config/       : Configuration files\n\
            - cache/        : Cached data\n\
            \n\
            Edit topline.editorconfig.rs to customize your setup.\n")?;

        println!("Created Topline directory structure at: {}", topline_root.display());

        Ok(ToplineDirectories {
            root: topline_root,
            plugins: plugins_dir,
            mods: mods_dir,
            lua_plugins: lua_plugins_dir,
            config: config_dir,
            cache: cache_dir,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ToplineDirectories {
    pub root: PathBuf,
    pub plugins: PathBuf,
    pub mods: PathBuf,
    pub lua_plugins: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
}
