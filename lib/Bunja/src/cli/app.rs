use crate::cli::commands::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bunja")]
#[command(about = "Bunja - Asset translation and management system", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new bunja.lock file")]
    Init {
        #[arg(short, long, help = "Path to bunja.lock file")]
        path: Option<PathBuf>,
    },

    #[command(about = "Add a new asset domain")]
    Add {
        #[arg(help = "Domain name")]
        name: String,

        #[arg(help = "Provider type (pexels, unsplash, cloudinary, s3, custom, local)")]
        provider: String,

        #[arg(help = "Base URL for the domain")]
        base_url: String,

        #[arg(short, long, help = "API key (if required)")]
        api_key: Option<String>,
    },

    #[command(about = "Remove an asset domain")]
    Remove {
        #[arg(help = "Domain name to remove")]
        name: String,
    },

    #[command(about = "List all configured domains")]
    List,

    #[command(about = "Start the Bunja HTTP server")]
    Serve {
        #[arg(short, long, help = "Port to serve on (overrides config)")]
        port: Option<u16>,
    },

    #[command(about = "Translate asset calls in a file")]
    Translate {
        #[arg(short, long, help = "Input file path")]
        input: PathBuf,

        #[arg(short, long, help = "Output file path")]
        output: PathBuf,
    },

    #[command(about = "Prefetch assets from a directory")]
    Prefetch {
        #[arg(help = "Directory to scan and prefetch")]
        directory: PathBuf,
    },

    #[command(about = "Cache management commands")]
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },

    #[command(about = "Validate bunja.lock configuration")]
    Validate,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    #[command(about = "Show cache statistics")]
    Stats,

    #[command(about = "Clear the cache")]
    Clear,
}

pub async fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => init_command(path).await,

        Commands::Add {
            name,
            provider,
            base_url,
            api_key,
        } => add_domain_command(name, provider, base_url, api_key).await,

        Commands::Remove { name } => remove_domain_command(name).await,

        Commands::List => list_domains_command().await,

        Commands::Serve { port } => serve_command(port).await,

        Commands::Translate { input, output } => translate_command(input, output).await,

        Commands::Prefetch { directory } => prefetch_command(directory).await,

        Commands::Cache { action } => match action {
            CacheCommands::Stats => cache_stats_command().await,
            CacheCommands::Clear => cache_clear_command().await,
        },

        Commands::Validate => validate_command().await,
    }
}
