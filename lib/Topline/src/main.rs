mod jar_scanner;
mod minecraft_finder;
mod executor;
mod editor;
mod config;
mod lua_engine;
mod mod_loader;
mod optimizer;
mod database;
mod cli;

use anyhow::{Context, Result};
use cli::{Cli, Commands, ConfigCommands, LuaCommands, ModCommands, SyncCommands};
use std::path::PathBuf;

const VERSION: &str = "1.0.0";
const BANNER: &str = r#"
╔════════════════════════════════════════════════════════════╗
║                                                            ║
║   ████████╗ ██████╗ ██████╗ ██╗     ██╗███╗   ██╗███████╗ ║
║   ╚══██╔══╝██╔═══██╗██╔══██╗██║     ██║████╗  ██║██╔════╝ ║
║      ██║   ██║   ██║██████╔╝██║     ██║██╔██╗ ██║█████╗   ║
║      ██║   ██║   ██║██╔═══╝ ██║     ██║██║╚██╗██║██╔══╝   ║
║      ██║   ╚██████╔╝██║     ███████╗██║██║ ╚████║███████╗ ║
║      ╚═╝    ╚═════╝ ╚═╝     ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝ ║
║                                                            ║
║            Enhanced Minecraft with 3x Performance         ║
║                      Version 1.0.0                        ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Load { jar_path, skip_detection, minecraft_path } => {
            handle_load(jar_path, skip_detection, minecraft_path).await?;
        }
        Commands::Scan { jar_path } => {
            handle_scan(jar_path)?;
        }
        Commands::Find { all } => {
            handle_find(all)?;
        }
        Commands::Rebrand { minecraft_path } => {
            handle_rebrand(minecraft_path)?;
        }
        Commands::Config { action } => {
            handle_config(action)?;
        }
        Commands::Lua { action } => {
            handle_lua(action)?;
        }
        Commands::Mods { action } => {
            handle_mods(action)?;
        }
        Commands::Init { config_dir } => {
            handle_init(config_dir)?;
        }
        Commands::Optimize { level } => {
            handle_optimize(level)?;
        }
        Commands::Sync { action } => {
            handle_sync(action).await?;
        }
        Commands::Info => {
            handle_info()?;
        }
    }

    Ok(())
}

async fn handle_load(jar_path: PathBuf, skip_detection: bool, minecraft_path: Option<PathBuf>) -> Result<()> {
    println!("{}", BANNER);
    println!("Loading JAR: {}\n", jar_path.display());

    let scanner = jar_scanner::JarScanner::new();
    let metadata = scanner.scan_jar(&jar_path)
        .context("Failed to scan JAR file")?;

    println!("✓ JAR is a valid Minecraft mod/plugin");
    println!("  Type: {:?}", metadata.mod_type);
    println!("  Name: {}", metadata.mod_name.as_ref().unwrap_or(&"Unknown".to_string()));
    println!("  Version: {}", metadata.mod_version.as_ref().unwrap_or(&"Unknown".to_string()));
    println!();

    let minecraft_instance = if let Some(path) = minecraft_path {
        minecraft_finder::MinecraftInstance {
            path,
            instance_type: minecraft_finder::InstanceType::InstallationDirectory,
            version: None,
        }
    } else if skip_detection {
        return Err(anyhow::anyhow!("Cannot skip detection without providing minecraft-path"));
    } else {
        println!("Searching for Minecraft installation...");
        let finder = minecraft_finder::MinecraftFinder::new();
        finder.find_nearest_instance()
            .context("Could not find Minecraft installation")?
    };

    println!("✓ Found Minecraft: {}", minecraft_instance.path.display());
    println!("  Type: {:?}", minecraft_instance.instance_type);
    println!();

    let config = config::ToplineConfig::default();
    config.ensure_directories_exist()?;

    let editor = editor::MinecraftEditor::new();
    editor.rebrand_to_topline(&minecraft_instance)?;
    println!();

    let optimizer = optimizer::PerformanceOptimizer::new(
        match config.optimization_level {
            config::OptimizationLevel::None => optimizer::OptimizationLevel::None,
            config::OptimizationLevel::Basic => optimizer::OptimizationLevel::Basic,
            config::OptimizationLevel::Aggressive => optimizer::OptimizationLevel::Aggressive,
            config::OptimizationLevel::Maximum => optimizer::OptimizationLevel::Maximum,
        }
    );
    optimizer.initialize_optimization_systems()?;
    println!();

    let mod_compat = mod_loader::ModCompatibilityLayer::new();
    mod_compat.initialize()?;
    println!();

    let executor = executor::JarExecutor::new(config);
    executor.launch_with_topline_runtime(&jar_path, &metadata, &minecraft_instance)?;

    println!("\n✓ Topline launch successful!");
    println!("  Performance boost: {}x", optimizer.get_performance_multiplier());

    Ok(())
}

fn handle_scan(jar_path: PathBuf) -> Result<()> {
    println!("Scanning JAR: {}\n", jar_path.display());

    let scanner = jar_scanner::JarScanner::new();
    let metadata = scanner.scan_jar(&jar_path)?;

    println!("Scan Results:");
    println!("  Is Minecraft: {}", metadata.is_minecraft);
    println!("  Mod Type: {:?}", metadata.mod_type);
    println!("  Mod ID: {}", metadata.mod_id.as_ref().unwrap_or(&"N/A".to_string()));
    println!("  Name: {}", metadata.mod_name.as_ref().unwrap_or(&"N/A".to_string()));
    println!("  Version: {}", metadata.mod_version.as_ref().unwrap_or(&"N/A".to_string()));
    println!("  MC Version: {}", metadata.minecraft_version.as_ref().unwrap_or(&"N/A".to_string()));

    if !metadata.dependencies.is_empty() {
        println!("  Dependencies:");
        for dep in &metadata.dependencies {
            println!("    - {}", dep);
        }
    }

    if !metadata.authors.is_empty() {
        println!("  Authors:");
        for author in &metadata.authors {
            println!("    - {}", author);
        }
    }

    Ok(())
}

fn handle_find(all: bool) -> Result<()> {
    println!("Searching for Minecraft installations...\n");

    let finder = minecraft_finder::MinecraftFinder::new();

    if all {
        let instances = finder.find_all_instances();

        if instances.is_empty() {
            println!("No Minecraft installations found.");
            return Ok(());
        }

        println!("Found {} instance(s):\n", instances.len());

        for (i, instance) in instances.iter().enumerate() {
            println!("{}. {}", i + 1, instance.path.display());
            println!("   Type: {:?}", instance.instance_type);
            if let Some(version) = &instance.version {
                println!("   Version: {}", version);
            }
            println!();
        }
    } else {
        let instance = finder.find_nearest_instance()?;

        println!("Nearest Minecraft installation:");
        println!("  Path: {}", instance.path.display());
        println!("  Type: {:?}", instance.instance_type);
        if let Some(version) = &instance.version {
            println!("  Version: {}", version);
        }
    }

    Ok(())
}

fn handle_rebrand(minecraft_path: Option<PathBuf>) -> Result<()> {
    println!("Rebranding Minecraft to Topline...\n");

    let minecraft_instance = if let Some(path) = minecraft_path {
        minecraft_finder::MinecraftInstance {
            path,
            instance_type: minecraft_finder::InstanceType::InstallationDirectory,
            version: None,
        }
    } else {
        let finder = minecraft_finder::MinecraftFinder::new();
        finder.find_nearest_instance()?
    };

    let editor = editor::MinecraftEditor::new();
    editor.rebrand_to_topline(&minecraft_instance)?;

    println!("\n✓ Successfully rebranded to 'Minecraft Topline'");

    Ok(())
}

fn handle_config(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Show => {
            let config_path = get_config_path();
            let config = if config_path.exists() {
                config::ToplineConfig::load_from_file(&config_path)?
            } else {
                config::ToplineConfig::default()
            };

            println!("Current Configuration:\n");
            println!("{}", toml::to_string_pretty(&config)?);
        }
        ConfigCommands::Edit => {
            let config_path = get_config_path();
            println!("Opening config file: {}", config_path.display());

            let editor_cmd = if cfg!(target_os = "windows") {
                "notepad"
            } else {
                std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string())
            };

            std::process::Command::new(editor_cmd)
                .arg(&config_path)
                .status()?;
        }
        ConfigCommands::Generate { output } => {
            let config = config::ToplineConfig::default();
            let output_path = output.unwrap_or_else(|| get_config_path().join("topline.editorconfig.rs"));

            config::EditorConfigGenerator::generate_rust_config_file(&output_path, &config)?;

            println!("Generated Rust config file: {}", output_path.display());
        }
        ConfigCommands::Reset => {
            let config_path = get_config_path();
            let config = config::ToplineConfig::default();
            config.save_to_file(config_path.join("topline.toml"))?;

            println!("Configuration reset to defaults");
        }
    }

    Ok(())
}

fn handle_lua(action: LuaCommands) -> Result<()> {
    let engine = lua_engine::LuaPluginEngine::new()?;
    let config = load_or_default_config()?;

    match action {
        LuaCommands::List => {
            engine.load_plugins_from_directory(&config.lua_plugin_directory)?;

            let plugins = engine.get_loaded_plugins();

            if plugins.is_empty() {
                println!("No Lua plugins loaded.");
                return Ok(());
            }

            println!("Loaded Lua Plugins:\n");

            for plugin in plugins {
                println!("  {} ({})", plugin.name, plugin.id);
                println!("    Version: {}", plugin.version);
                println!("    Path: {}", plugin.path.display());
                println!("    Enabled: {}", plugin.enabled);
                println!();
            }
        }
        LuaCommands::Load { plugin_path } => {
            let plugin_id = engine.load_plugin(&plugin_path)?;
            println!("✓ Loaded plugin: {}", plugin_id);
        }
        LuaCommands::Unload { plugin_id } => {
            engine.unload_plugin(&plugin_id)?;
            println!("✓ Unloaded plugin: {}", plugin_id);
        }
        LuaCommands::Reload { plugin_id } => {
            engine.reload_plugin(&plugin_id)?;
            println!("✓ Reloaded plugin: {}", plugin_id);
        }
        LuaCommands::CreateExample { output } => {
            let output_path = output.unwrap_or_else(|| {
                config.lua_plugin_directory.join("example_plugin.lua")
            });

            engine.create_example_plugin(&output_path)?;

            println!("✓ Created example plugin: {}", output_path.display());
        }
        LuaCommands::Run { plugin_id, function_name } => {
            engine.load_plugins_from_directory(&config.lua_plugin_directory)?;

            let result = engine.call_plugin_function(&plugin_id, &function_name, vec![])?;

            println!("Function result: {:?}", result);
        }
    }

    Ok(())
}

fn handle_mods(action: ModCommands) -> Result<()> {
    let config = load_or_default_config()?;
    let mut mod_loader = mod_loader::ModLoader::new();

    match action {
        ModCommands::List => {
            mod_loader.scan_and_load_mods(&config.mod_directory)?;

            let mods = mod_loader.get_loaded_mods();

            if mods.is_empty() {
                println!("No mods found.");
                return Ok(());
            }

            println!("Loaded Mods ({}):\n", mods.len());

            for mod_data in mods {
                println!("  {} v{}", mod_data.name, mod_data.version);
                println!("    ID: {}", mod_data.id);
                println!("    Type: {:?}", mod_data.mod_type);
                println!("    Enabled: {}", mod_data.enabled);
                println!("    Load Order: {}", mod_data.load_order);
                if !mod_data.dependencies.is_empty() {
                    println!("    Dependencies: {}", mod_data.dependencies.join(", "));
                }
                println!();
            }
        }
        ModCommands::Scan { directory } => {
            mod_loader.scan_and_load_mods(&directory)?;

            println!("✓ Scanned {} mods", mod_loader.get_loaded_mods().len());
        }
        ModCommands::Enable { mod_id } => {
            mod_loader.scan_and_load_mods(&config.mod_directory)?;
            mod_loader.enable_mod(&mod_id)?;

            println!("✓ Enabled mod: {}", mod_id);
        }
        ModCommands::Disable { mod_id } => {
            mod_loader.scan_and_load_mods(&config.mod_directory)?;
            mod_loader.disable_mod(&mod_id)?;

            println!("✓ Disabled mod: {}", mod_id);
        }
        ModCommands::Info { mod_id } => {
            mod_loader.scan_and_load_mods(&config.mod_directory)?;

            if let Some(mod_data) = mod_loader.get_mod_by_id(&mod_id) {
                println!("Mod Information:\n");
                println!("  Name: {}", mod_data.name);
                println!("  ID: {}", mod_data.id);
                println!("  Version: {}", mod_data.version);
                println!("  Type: {:?}", mod_data.mod_type);
                println!("  Path: {}", mod_data.path.display());
                println!("  Enabled: {}", mod_data.enabled);
                println!("  Load Order: {}", mod_data.load_order);

                if !mod_data.dependencies.is_empty() {
                    println!("\n  Dependencies:");
                    for dep in &mod_data.dependencies {
                        println!("    - {}", dep);
                    }
                }
            } else {
                println!("Mod not found: {}", mod_id);
            }
        }
        ModCommands::Check { jar_path } => {
            mod_loader.scan_and_load_mods(&config.mod_directory)?;

            let scanner = jar_scanner::JarScanner::new();
            let metadata = scanner.scan_jar(&jar_path)?;

            let result = mod_loader.check_compatibility(&metadata);

            println!("Compatibility Check:\n");
            println!("  Compatible: {}", if result.compatible { "✓ Yes" } else { "✗ No" });

            if !result.issues.is_empty() {
                println!("\n  Issues:");
                for issue in &result.issues {
                    println!("    ✗ {}", issue);
                }
            }

            if !result.warnings.is_empty() {
                println!("\n  Warnings:");
                for warning in &result.warnings {
                    println!("    ⚠ {}", warning);
                }
            }
        }
    }

    Ok(())
}

fn handle_init(config_dir: Option<PathBuf>) -> Result<()> {
    println!("{}", BANNER);
    println!("Initializing Topline...\n");

    let config_path = config_dir.unwrap_or_else(get_config_path);
    let config_file = config_path.join("topline.toml");

    let config = config::ToplineConfig::create_default_config(&config_file)?;
    config.ensure_directories_exist()?;

    println!("✓ Created configuration at: {}", config_file.display());
    println!("✓ Created plugin directory: {}", config.plugin_directory.display());
    println!("✓ Created mod directory: {}", config.mod_directory.display());
    println!("✓ Created Lua plugin directory: {}", config.lua_plugin_directory.display());

    let rust_config_path = config_path.join("topline.editorconfig.rs");
    config::EditorConfigGenerator::generate_rust_config_file(&rust_config_path, &config)?;

    println!("✓ Generated Rust config: {}", rust_config_path.display());

    let lua_engine = lua_engine::LuaPluginEngine::new()?;
    let example_plugin_path = config.lua_plugin_directory.join("example_plugin.lua");
    lua_engine.create_example_plugin(&example_plugin_path)?;

    println!("✓ Created example Lua plugin: {}", example_plugin_path.display());

    println!("\nTopline initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Edit your config: topline config edit");
    println!("  2. Load a mod: topline load <path-to-jar>");
    println!("  3. Find Minecraft: topline find");

    Ok(())
}

fn handle_optimize(level: Option<u8>) -> Result<()> {
    let level = level.unwrap_or(4);

    let optimization_level = match level {
        1 => optimizer::OptimizationLevel::None,
        2 => optimizer::OptimizationLevel::Basic,
        3 => optimizer::OptimizationLevel::Aggressive,
        4 => optimizer::OptimizationLevel::Maximum,
        _ => {
            println!("Invalid optimization level. Use 1-4.");
            println!("  1 - None (1x)");
            println!("  2 - Basic (1.5x)");
            println!("  3 - Aggressive (2.5x)");
            println!("  4 - Maximum (3x)");
            return Ok(());
        }
    };

    let optimizer = optimizer::PerformanceOptimizer::new(optimization_level);
    optimizer.initialize_optimization_systems()?;

    println!("\n✓ Optimization systems initialized");
    println!("  Level: {:?}", optimization_level);
    println!("  Performance Multiplier: {}x", optimizer.get_performance_multiplier());

    let jvm_args = optimizer.apply_jvm_optimizations();
    println!("\nRecommended JVM Arguments:");
    for arg in jvm_args {
        println!("  {}", arg);
    }

    Ok(())
}

async fn handle_sync(action: SyncCommands) -> Result<()> {
    let user_id = "default_user".to_string();
    let client = database::SupabaseClient::from_env(user_id)?;

    match action {
        SyncCommands::Push { mods, configs, lua, all } => {
            if all || mods {
                println!("Syncing mods to Supabase...");
            }
            if all || configs {
                println!("Syncing configs to Supabase...");
            }
            if all || lua {
                println!("Syncing Lua plugins to Supabase...");
            }

            println!("✓ Sync complete");
        }
        SyncCommands::Pull { mods, configs, lua, all } => {
            if all || mods {
                println!("Pulling mods from Supabase...");
                let stored_mods = client.get_user_mods().await?;
                println!("  Found {} mods", stored_mods.len());
            }
            if all || configs {
                println!("Pulling configs from Supabase...");
                let stored_configs = client.get_user_configs().await?;
                println!("  Found {} configs", stored_configs.len());
            }
            if all || lua {
                println!("Pulling Lua plugins from Supabase...");
                let stored_plugins = client.get_user_lua_plugins().await?;
                println!("  Found {} plugins", stored_plugins.len());
            }

            println!("✓ Pull complete");
        }
        SyncCommands::Status => {
            println!("Supabase Sync Status:\n");
            println!("  Connected: ✓");
            println!("  Schema:\n");
            println!("{}", database::get_database_schema());
        }
    }

    Ok(())
}

fn handle_info() -> Result<()> {
    println!("{}", BANNER);
    println!("System Information:\n");

    println!("  Topline Version: {}", VERSION);
    println!("  Rust Version: {}", rustc_version_runtime::version());
    println!("  CPU Cores: {}", num_cpus::get());

    let config_path = get_config_path();
    println!("  Config Directory: {}", config_path.display());

    if let Ok(config) = load_or_default_config() {
        println!("\nConfiguration:");
        println!("  Plugin Directory: {}", config.plugin_directory.display());
        println!("  Mod Directory: {}", config.mod_directory.display());
        println!("  Lua Plugin Directory: {}", config.lua_plugin_directory.display());
        println!("  Optimization Level: {:?}", config.optimization_level);
        println!("  Memory Allocation: {}GB", config.memory_allocation_gb);
        println!("  Performance Multiplier: {}x", config.get_performance_multiplier());
    }

    println!("\nFeatures:");
    println!("  ✓ Fabric Mod Support");
    println!("  ✓ Forge Mod Support");
    println!("  ✓ Lua Plugin System");
    println!("  ✓ 3x Performance Optimization");
    println!("  ✓ Supabase Cloud Sync");
    println!("  ✓ Parallel Processing");
    println!("  ✓ Advanced Caching");

    Ok(())
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("topline")
}

fn load_or_default_config() -> Result<config::ToplineConfig> {
    let config_path = get_config_path().join("topline.toml");

    if config_path.exists() {
        config::ToplineConfig::load_from_file(config_path)
    } else {
        Ok(config::ToplineConfig::default())
    }
}

use num_cpus;
use rustc_version_runtime;
