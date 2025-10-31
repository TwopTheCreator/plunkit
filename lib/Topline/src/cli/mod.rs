use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "topline")]
#[command(about = "Topline - Enhanced Minecraft Mod Loader with 3x Performance", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Load {
        #[arg(help = "Path to the JAR file to load")]
        jar_path: PathBuf,

        #[arg(short, long, help = "Skip Minecraft instance detection")]
        skip_detection: bool,

        #[arg(short, long, help = "Use specific Minecraft instance path")]
        minecraft_path: Option<PathBuf>,
    },

    Scan {
        #[arg(help = "Path to the JAR file to scan")]
        jar_path: PathBuf,
    },

    Find {
        #[arg(short, long, help = "Show all instances, not just the nearest")]
        all: bool,
    },

    Rebrand {
        #[arg(short, long, help = "Path to Minecraft instance")]
        minecraft_path: Option<PathBuf>,
    },

    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    Lua {
        #[command(subcommand)]
        action: LuaCommands,
    },

    Mods {
        #[command(subcommand)]
        action: ModCommands,
    },

    Init {
        #[arg(short, long, help = "Custom configuration directory")]
        config_dir: Option<PathBuf>,
    },

    Optimize {
        #[arg(short, long, help = "Optimization level (1-4)")]
        level: Option<u8>,
    },

    Sync {
        #[command(subcommand)]
        action: SyncCommands,
    },

    Info,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Show,

    Edit,

    Generate {
        #[arg(short, long, help = "Output path for generated config")]
        output: Option<PathBuf>,
    },

    Reset,
}

#[derive(Subcommand)]
pub enum LuaCommands {
    List,

    Load {
        #[arg(help = "Path to Lua plugin file")]
        plugin_path: PathBuf,
    },

    Unload {
        #[arg(help = "Plugin ID to unload")]
        plugin_id: String,
    },

    Reload {
        #[arg(help = "Plugin ID to reload")]
        plugin_id: String,
    },

    CreateExample {
        #[arg(short, long, help = "Output path for example plugin")]
        output: Option<PathBuf>,
    },

    Run {
        #[arg(help = "Plugin ID")]
        plugin_id: String,

        #[arg(help = "Function name to call")]
        function_name: String,
    },
}

#[derive(Subcommand)]
pub enum ModCommands {
    List,

    Scan {
        #[arg(help = "Directory to scan for mods")]
        directory: PathBuf,
    },

    Enable {
        #[arg(help = "Mod ID to enable")]
        mod_id: String,
    },

    Disable {
        #[arg(help = "Mod ID to disable")]
        mod_id: String,
    },

    Info {
        #[arg(help = "Mod ID to show info for")]
        mod_id: String,
    },

    Check {
        #[arg(help = "JAR file to check compatibility")]
        jar_path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum SyncCommands {
    Push {
        #[arg(short, long, help = "Sync mods")]
        mods: bool,

        #[arg(short, long, help = "Sync configs")]
        configs: bool,

        #[arg(short, long, help = "Sync Lua plugins")]
        lua: bool,

        #[arg(short, long, help = "Sync all")]
        all: bool,
    },

    Pull {
        #[arg(short, long, help = "Pull mods")]
        mods: bool,

        #[arg(short, long, help = "Pull configs")]
        configs: bool,

        #[arg(short, long, help = "Pull Lua plugins")]
        lua: bool,

        #[arg(short, long, help = "Pull all")]
        all: bool,
    },

    Status,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
