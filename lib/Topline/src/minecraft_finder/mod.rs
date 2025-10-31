use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct MinecraftInstance {
    pub path: PathBuf,
    pub instance_type: InstanceType,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceType {
    JavaExecutable,
    LauncherExecutable,
    WasmBinary,
    InstallationDirectory,
}

pub struct MinecraftFinder;

impl MinecraftFinder {
    pub fn new() -> Self {
        Self
    }

    pub fn find_nearest_instance(&self) -> Result<MinecraftInstance> {
        let search_paths = self.get_search_paths();

        for base_path in search_paths {
            if !base_path.exists() {
                continue;
            }

            if let Ok(instance) = self.search_directory(&base_path) {
                return Ok(instance);
            }
        }

        Err(anyhow!("No Minecraft installation found on this system"))
    }

    pub fn find_all_instances(&self) -> Vec<MinecraftInstance> {
        let search_paths = self.get_search_paths();
        let mut instances = Vec::new();

        for base_path in search_paths {
            if !base_path.exists() {
                continue;
            }

            if let Ok(found_instances) = self.search_all_in_directory(&base_path) {
                instances.extend(found_instances);
            }
        }

        instances
    }

    fn get_search_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if cfg!(target_os = "windows") {
            if let Ok(appdata) = std::env::var("APPDATA") {
                paths.push(PathBuf::from(appdata).join(".minecraft"));
            }
            if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                paths.push(PathBuf::from(localappdata).join("Packages").join("Microsoft.MinecraftUWP_8wekyb3d8bbwe"));
            }
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                paths.push(PathBuf::from(program_files).join("Minecraft Launcher"));
                paths.push(PathBuf::from(program_files).join("Minecraft"));
            }
            if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
                paths.push(PathBuf::from(program_files_x86).join("Minecraft Launcher"));
                paths.push(PathBuf::from(program_files_x86).join("Minecraft"));
            }
        } else if cfg!(target_os = "macos") {
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join("Library").join("Application Support").join("minecraft"));
                paths.push(PathBuf::from("/Applications/Minecraft.app"));
            }
        } else {
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join(".minecraft"));
                paths.push(home.join(".local").join("share").join("minecraft"));
            }
            paths.push(PathBuf::from("/usr/share/minecraft"));
            paths.push(PathBuf::from("/opt/minecraft"));
        }

        if let Ok(current_dir) = std::env::current_dir() {
            paths.push(current_dir);
        }

        paths
    }

    fn search_directory(&self, base_path: &Path) -> Result<MinecraftInstance> {
        for entry in WalkDir::new(base_path)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if self.is_minecraft_executable(path) {
                return Ok(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::JavaExecutable,
                    version: self.detect_version(path),
                });
            }

            if self.is_launcher_executable(path) {
                return Ok(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::LauncherExecutable,
                    version: self.detect_version(path),
                });
            }

            if self.is_wasm_binary(path) {
                return Ok(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::WasmBinary,
                    version: self.detect_version(path),
                });
            }

            if self.is_minecraft_directory(path) {
                return Ok(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::InstallationDirectory,
                    version: self.detect_version(path),
                });
            }
        }

        Err(anyhow!("No Minecraft instance found in {}", base_path.display()))
    }

    fn search_all_in_directory(&self, base_path: &Path) -> Result<Vec<MinecraftInstance>> {
        let mut instances = Vec::new();

        for entry in WalkDir::new(base_path)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if self.is_minecraft_executable(path) {
                instances.push(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::JavaExecutable,
                    version: self.detect_version(path),
                });
            } else if self.is_launcher_executable(path) {
                instances.push(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::LauncherExecutable,
                    version: self.detect_version(path),
                });
            } else if self.is_wasm_binary(path) {
                instances.push(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::WasmBinary,
                    version: self.detect_version(path),
                });
            } else if self.is_minecraft_directory(path) {
                instances.push(MinecraftInstance {
                    path: path.to_path_buf(),
                    instance_type: InstanceType::InstallationDirectory,
                    version: self.detect_version(path),
                });
            }
        }

        Ok(instances)
    }

    fn is_minecraft_executable(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        file_name.to_lowercase().contains("minecraft") &&
            (file_name.ends_with(".exe") ||
             file_name.ends_with(".jar") ||
             !file_name.contains('.'))
    }

    fn is_launcher_executable(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        (file_name.to_lowercase().contains("launcher") ||
         file_name == "MinecraftLauncher.exe" ||
         file_name == "minecraft-launcher") &&
            file_name.to_lowercase().contains("minecraft")
    }

    fn is_wasm_binary(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        file_name.to_lowercase().contains("minecraft") &&
            file_name.ends_with(".wasm")
    }

    fn is_minecraft_directory(&self, path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if dir_name != ".minecraft" && !dir_name.to_lowercase().contains("minecraft") {
            return false;
        }

        let has_versions = path.join("versions").exists();
        let has_assets = path.join("assets").exists();
        let has_launcher_profiles = path.join("launcher_profiles.json").exists();

        has_versions || has_assets || has_launcher_profiles
    }

    fn detect_version(&self, path: &Path) -> Option<String> {
        if path.is_dir() {
            let version_manifest = path.join("versions");
            if version_manifest.exists() {
                if let Ok(entries) = fs::read_dir(&version_manifest) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if entry.path().is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_instance_path(&self, instance: &MinecraftInstance) -> PathBuf {
        match instance.instance_type {
            InstanceType::InstallationDirectory => instance.path.clone(),
            InstanceType::JavaExecutable |
            InstanceType::LauncherExecutable |
            InstanceType::WasmBinary => {
                instance.path.parent()
                    .unwrap_or(instance.path.as_path())
                    .to_path_buf()
            }
        }
    }
}
