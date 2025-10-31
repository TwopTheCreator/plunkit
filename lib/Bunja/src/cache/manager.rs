use crate::cache::eviction::LruEviction;
use crate::cache::storage::CacheStorage;
use crate::config::GlobalSettings;
use anyhow::{Context, Result};
use log::{debug, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CacheManager {
    storage: Arc<CacheStorage>,
    eviction: Arc<RwLock<LruEviction>>,
    max_size_bytes: u64,
    ttl_seconds: u64,
}

impl CacheManager {
    pub async fn new(settings: &GlobalSettings) -> Result<Self> {
        let cache_dir = PathBuf::from(&settings.cache_dir);

        tokio::fs::create_dir_all(&cache_dir).await
            .context("Failed to create cache directory")?;

        let storage = Arc::new(CacheStorage::new(cache_dir));
        let eviction = Arc::new(RwLock::new(LruEviction::new()));

        let max_size_bytes = settings.max_cache_size_mb * 1024 * 1024;

        let manager = Self {
            storage,
            eviction,
            max_size_bytes,
            ttl_seconds: settings.cache_ttl_seconds,
        };

        manager.load_existing_cache().await?;

        Ok(manager)
    }

    async fn load_existing_cache(&self) -> Result<()> {
        let entries = self.storage.list_all().await?;

        let mut eviction = self.eviction.write().await;

        for (key, metadata) in entries {
            eviction.record_access(&key, metadata.size);
        }

        info!("Loaded {} cache entries", eviction.len());

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if self.is_expired(key).await {
            debug!("Cache entry expired: {}", key);
            let _ = self.delete(key).await;
            return None;
        }

        match self.storage.get(key).await {
            Ok(data) => {
                let mut eviction = self.eviction.write().await;
                eviction.record_access(key, data.len() as u64);
                Some(data)
            }
            Err(_) => None,
        }
    }

    pub async fn set(&self, key: &str, data: &[u8]) -> Result<()> {
        let data_size = data.len() as u64;

        self.ensure_space(data_size).await?;

        self.storage.set(key, data).await?;

        let mut eviction = self.eviction.write().await;
        eviction.record_access(key, data_size);

        debug!("Cached entry: {} ({} bytes)", key, data_size);

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.storage.delete(key).await?;

        let mut eviction = self.eviction.write().await;
        eviction.remove(key);

        debug!("Deleted cache entry: {}", key);

        Ok(())
    }

    async fn ensure_space(&self, required_space: u64) -> Result<()> {
        let current_size = self.get_total_size().await;

        if current_size + required_space <= self.max_size_bytes {
            return Ok(());
        }

        let space_needed = (current_size + required_space) - self.max_size_bytes;

        info!(
            "Cache full. Current: {} MB, Need to free: {} MB",
            current_size / (1024 * 1024),
            space_needed / (1024 * 1024)
        );

        let mut freed_space = 0u64;
        let eviction = self.eviction.read().await;
        let victims = eviction.get_eviction_candidates(space_needed);

        drop(eviction);

        for victim in victims {
            if let Ok(metadata) = self.storage.get_metadata(&victim).await {
                let _ = self.delete(&victim).await;
                freed_space += metadata.size;

                if freed_space >= space_needed {
                    break;
                }
            }
        }

        info!("Freed {} MB of cache space", freed_space / (1024 * 1024));

        Ok(())
    }

    async fn get_total_size(&self) -> u64 {
        let eviction = self.eviction.read().await;
        eviction.total_size()
    }

    async fn is_expired(&self, key: &str) -> bool {
        if let Ok(metadata) = self.storage.get_metadata(key).await {
            let age = std::time::SystemTime::now()
                .duration_since(metadata.created_at)
                .unwrap_or_default()
                .as_secs();

            return age > self.ttl_seconds;
        }

        true
    }

    pub async fn clear(&self) -> Result<()> {
        self.storage.clear().await?;

        let mut eviction = self.eviction.write().await;
        eviction.clear();

        info!("Cache cleared");

        Ok(())
    }

    pub async fn get_stats(&self) -> CacheStats {
        let eviction = self.eviction.read().await;

        CacheStats {
            total_entries: eviction.len(),
            total_size_bytes: eviction.total_size(),
            max_size_bytes: self.max_size_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub max_size_bytes: u64,
}
