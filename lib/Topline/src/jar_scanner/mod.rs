use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarMetadata {
    pub is_minecraft: bool,
    pub mod_type: Option<ModType>,
    pub minecraft_version: Option<String>,
    pub mod_id: Option<String>,
    pub mod_name: Option<String>,
    pub mod_version: Option<String>,
    pub dependencies: Vec<String>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModType {
    Fabric,
    Forge,
    Vanilla,
    Plugin,
    Unknown,
}

pub struct JarScanner;

impl JarScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_jar<P: AsRef<Path>>(&self, path: P) -> Result<JarMetadata> {
        let file = File::open(path.as_ref())
            .context("Failed to open JAR file")?;

        let mut archive = ZipArchive::new(file)
            .context("Failed to read JAR archive")?;

        let mut metadata = JarMetadata {
            is_minecraft: false,
            mod_type: None,
            minecraft_version: None,
            mod_id: None,
            mod_name: None,
            mod_version: None,
            dependencies: Vec::new(),
            authors: Vec::new(),
        };

        let has_fabric_json = self.check_file_exists(&mut archive, "fabric.mod.json");
        let has_mods_toml = self.check_file_exists(&mut archive, "META-INF/mods.toml");
        let has_mcmod_info = self.check_file_exists(&mut archive, "mcmod.info");
        let has_plugin_yml = self.check_file_exists(&mut archive, "plugin.yml");

        if has_fabric_json {
            metadata.is_minecraft = true;
            metadata.mod_type = Some(ModType::Fabric);
            self.parse_fabric_metadata(&mut archive, &mut metadata)?;
        } else if has_mods_toml {
            metadata.is_minecraft = true;
            metadata.mod_type = Some(ModType::Forge);
            self.parse_forge_metadata(&mut archive, &mut metadata)?;
        } else if has_mcmod_info {
            metadata.is_minecraft = true;
            metadata.mod_type = Some(ModType::Forge);
            self.parse_mcmod_info(&mut archive, &mut metadata)?;
        } else if has_plugin_yml {
            metadata.is_minecraft = true;
            metadata.mod_type = Some(ModType::Plugin);
            self.parse_plugin_yml(&mut archive, &mut metadata)?;
        } else {
            let has_minecraft_classes = self.check_minecraft_classes(&mut archive);
            if has_minecraft_classes {
                metadata.is_minecraft = true;
                metadata.mod_type = Some(ModType::Vanilla);
            }
        }

        if !metadata.is_minecraft {
            return Err(anyhow!("This JAR file is not a Minecraft mod or plugin"));
        }

        Ok(metadata)
    }

    fn check_file_exists(&self, archive: &mut ZipArchive<File>, name: &str) -> bool {
        archive.by_name(name).is_ok()
    }

    fn check_minecraft_classes(&self, archive: &mut ZipArchive<File>) -> bool {
        let minecraft_indicators = vec![
            "net/minecraft/",
            "com/mojang/",
            "net/minecraftforge/",
            "cpw/mods/",
        ];

        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name();
                for indicator in &minecraft_indicators {
                    if name.starts_with(indicator) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn parse_fabric_metadata(&self, archive: &mut ZipArchive<File>, metadata: &mut JarMetadata) -> Result<()> {
        let mut file = archive.by_name("fabric.mod.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json: serde_json::Value = serde_json::from_str(&contents)?;

        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            metadata.mod_id = Some(id.to_string());
        }

        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            metadata.mod_name = Some(name.to_string());
        }

        if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
            metadata.mod_version = Some(version.to_string());
        }

        if let Some(depends) = json.get("depends").and_then(|v| v.as_object()) {
            if let Some(mc_version) = depends.get("minecraft").and_then(|v| v.as_str()) {
                metadata.minecraft_version = Some(mc_version.to_string());
            }
            for (dep, _) in depends {
                if dep != "minecraft" && dep != "java" && dep != "fabricloader" {
                    metadata.dependencies.push(dep.clone());
                }
            }
        }

        if let Some(authors) = json.get("authors").and_then(|v| v.as_array()) {
            for author in authors {
                if let Some(name) = author.as_str() {
                    metadata.authors.push(name.to_string());
                } else if let Some(obj) = author.as_object() {
                    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                        metadata.authors.push(name.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_forge_metadata(&self, archive: &mut ZipArchive<File>, metadata: &mut JarMetadata) -> Result<()> {
        let mut file = archive.by_name("META-INF/mods.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let toml: toml::Value = toml::from_str(&contents)?;

        if let Some(mods) = toml.get("mods").and_then(|v| v.as_array()) {
            if let Some(first_mod) = mods.get(0) {
                if let Some(mod_id) = first_mod.get("modId").and_then(|v| v.as_str()) {
                    metadata.mod_id = Some(mod_id.to_string());
                }
                if let Some(display_name) = first_mod.get("displayName").and_then(|v| v.as_str()) {
                    metadata.mod_name = Some(display_name.to_string());
                }
                if let Some(version) = first_mod.get("version").and_then(|v| v.as_str()) {
                    metadata.mod_version = Some(version.to_string());
                }
                if let Some(authors) = first_mod.get("authors").and_then(|v| v.as_str()) {
                    metadata.authors.push(authors.to_string());
                }
            }
        }

        if let Some(dependencies) = toml.get("dependencies").and_then(|v| v.as_table()) {
            for (mod_id, deps) in dependencies {
                if let Some(deps_array) = deps.as_array() {
                    for dep in deps_array {
                        if let Some(dep_id) = dep.get("modId").and_then(|v| v.as_str()) {
                            if dep_id == "minecraft" {
                                if let Some(version_range) = dep.get("versionRange").and_then(|v| v.as_str()) {
                                    metadata.minecraft_version = Some(version_range.to_string());
                                }
                            } else if dep_id != "forge" {
                                metadata.dependencies.push(dep_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_mcmod_info(&self, archive: &mut ZipArchive<File>, metadata: &mut JarMetadata) -> Result<()> {
        let mut file = archive.by_name("mcmod.info")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json: serde_json::Value = serde_json::from_str(&contents)?;

        let mod_list = if json.is_array() {
            json.as_array().unwrap()
        } else if let Some(mod_list) = json.get("modList").and_then(|v| v.as_array()) {
            mod_list
        } else {
            return Ok(());
        };

        if let Some(first_mod) = mod_list.get(0) {
            if let Some(mod_id) = first_mod.get("modid").and_then(|v| v.as_str()) {
                metadata.mod_id = Some(mod_id.to_string());
            }
            if let Some(name) = first_mod.get("name").and_then(|v| v.as_str()) {
                metadata.mod_name = Some(name.to_string());
            }
            if let Some(version) = first_mod.get("version").and_then(|v| v.as_str()) {
                metadata.mod_version = Some(version.to_string());
            }
            if let Some(mc_version) = first_mod.get("mcversion").and_then(|v| v.as_str()) {
                metadata.minecraft_version = Some(mc_version.to_string());
            }
        }

        Ok(())
    }

    fn parse_plugin_yml(&self, archive: &mut ZipArchive<File>, metadata: &mut JarMetadata) -> Result<()> {
        let mut file = archive.by_name("plugin.yml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let yaml: serde_json::Value = serde_yaml::from_str(&contents)
            .unwrap_or_else(|_| serde_json::json!({}));

        if let Some(name) = yaml.get("name").and_then(|v| v.as_str()) {
            metadata.mod_id = Some(name.to_lowercase().replace(" ", "_"));
            metadata.mod_name = Some(name.to_string());
        }

        if let Some(version) = yaml.get("version").and_then(|v| v.as_str()) {
            metadata.mod_version = Some(version.to_string());
        }

        if let Some(api_version) = yaml.get("api-version").and_then(|v| v.as_str()) {
            metadata.minecraft_version = Some(format!("1.{}", api_version));
        }

        if let Some(depend) = yaml.get("depend").and_then(|v| v.as_array()) {
            for dep in depend {
                if let Some(dep_str) = dep.as_str() {
                    metadata.dependencies.push(dep_str.to_string());
                }
            }
        }

        Ok(())
    }
}

use serde_yaml;
