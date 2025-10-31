use crate::asset::AssetFetcher;
use crate::cache::CacheManager;
use crate::config::{AssetDomain, BunjaLock, DomainProvider, Settings};
use crate::server::BunjaServer;
use crate::translator::TranslationEngine;
use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn init_command(path: Option<PathBuf>) -> Result<()> {
    let lock_path = path.unwrap_or_else(|| PathBuf::from("bunja.lock"));

    if lock_path.exists() {
        anyhow::bail!("bunja.lock already exists at {:?}", lock_path);
    }

    let lock = BunjaLock::default();

    lock.save(&lock_path)
        .context("Failed to save bunja.lock file")?;

    info!("Initialized bunja.lock at {:?}", lock_path);
    info!("Default domains configured:");
    for domain_name in lock.domains.keys() {
        info!("  - {}", domain_name);
    }

    Ok(())
}

pub async fn add_domain_command(
    name: String,
    provider: String,
    base_url: String,
    api_key: Option<String>,
) -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let mut lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    let provider_type = match provider.to_lowercase().as_str() {
        "pexels" => DomainProvider::Pexels,
        "unsplash" => DomainProvider::Unsplash,
        "cloudinary" => DomainProvider::Cloudinary,
        "s3" => DomainProvider::S3,
        "custom" => DomainProvider::Custom,
        "local" => DomainProvider::Local,
        _ => anyhow::bail!("Unknown provider: {}. Use one of: pexels, unsplash, cloudinary, s3, custom, local", provider),
    };

    let domain = AssetDomain {
        provider: provider_type,
        base_url,
        api_key,
        headers: Default::default(),
        transformations: vec![],
        fallback_domains: vec![],
        rate_limit: None,
        retry_strategy: Default::default(),
    };

    lock.add_domain(name.clone(), domain);

    lock.save(&lock_path)
        .context("Failed to save bunja.lock")?;

    info!("Added domain '{}' to bunja.lock", name);

    Ok(())
}

pub async fn remove_domain_command(name: String) -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let mut lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock")?;

    if lock.remove_domain(&name).is_some() {
        lock.save(&lock_path)
            .context("Failed to save bunja.lock")?;

        info!("Removed domain '{}' from bunja.lock", name);
    } else {
        anyhow::bail!("Domain '{}' not found in bunja.lock", name);
    }

    Ok(())
}

pub async fn list_domains_command() -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    if lock.domains.is_empty() {
        info!("No domains configured.");
        return Ok(());
    }

    info!("Configured domains:");
    for (name, domain) in &lock.domains {
        info!("  {} ({:?})", name, domain.provider);
        info!("    Base URL: {}", domain.base_url);
        if domain.api_key.is_some() {
            info!("    API Key: [configured]");
        }
    }

    Ok(())
}

pub async fn serve_command(port: Option<u16>) -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let mut lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    if let Some(port) = port {
        lock.global_settings.server_port = port;
    }

    let lock = Arc::new(lock);

    let cache_manager = Arc::new(
        CacheManager::new(&lock.global_settings)
            .await
            .context("Failed to initialize cache manager")?
    );

    let fetcher = Arc::new(AssetFetcher::new(
        Arc::clone(&lock),
        Arc::clone(&cache_manager),
    ));

    let server = BunjaServer::new(lock, cache_manager, fetcher);

    server.run().await
}

pub async fn translate_command(input: PathBuf, output: PathBuf) -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = Arc::new(
        BunjaLock::load(&lock_path)
            .context("Failed to load bunja.lock. Run 'bunja init' first.")?
    );

    let cache_manager = Arc::new(
        CacheManager::new(&lock.global_settings)
            .await
            .context("Failed to initialize cache manager")?
    );

    let engine = TranslationEngine::new(lock, cache_manager);

    engine.translate_file(&input, &output).await
        .context("Translation failed")?;

    info!("Translation complete!");

    Ok(())
}

pub async fn prefetch_command(directory: PathBuf) -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = Arc::new(
        BunjaLock::load(&lock_path)
            .context("Failed to load bunja.lock. Run 'bunja init' first.")?
    );

    let cache_manager = Arc::new(
        CacheManager::new(&lock.global_settings)
            .await
            .context("Failed to initialize cache manager")?
    );

    let engine = TranslationEngine::new(lock, cache_manager);

    info!("Prefetching assets from directory: {:?}", directory);

    engine.prefetch_directory(&directory).await
        .context("Prefetch failed")?;

    info!("Prefetch complete!");

    Ok(())
}

pub async fn cache_stats_command() -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    let cache_manager = CacheManager::new(&lock.global_settings)
        .await
        .context("Failed to initialize cache manager")?;

    let stats = cache_manager.get_stats().await;

    info!("Cache Statistics:");
    info!("  Total entries: {}", stats.total_entries);
    info!("  Total size: {} MB", stats.total_size_bytes / (1024 * 1024));
    info!("  Max size: {} MB", stats.max_size_bytes / (1024 * 1024));
    info!(
        "  Usage: {:.2}%",
        (stats.total_size_bytes as f64 / stats.max_size_bytes as f64 * 100.0)
    );

    Ok(())
}

pub async fn cache_clear_command() -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    let cache_manager = CacheManager::new(&lock.global_settings)
        .await
        .context("Failed to initialize cache manager")?;

    cache_manager.clear().await
        .context("Failed to clear cache")?;

    info!("Cache cleared successfully");

    Ok(())
}

pub async fn validate_command() -> Result<()> {
    let lock_path = PathBuf::from("bunja.lock");

    let lock = BunjaLock::load(&lock_path)
        .context("Failed to load bunja.lock. Run 'bunja init' first.")?;

    lock.validate()
        .context("Validation failed")?;

    info!("bunja.lock is valid!");
    info!("  Version: {}", lock.version);
    info!("  Domains: {}", lock.domains.len());
    info!("  Cache directory: {}", lock.global_settings.cache_dir);
    info!("  Server port: {}", lock.global_settings.server_port);

    Ok(())
}
