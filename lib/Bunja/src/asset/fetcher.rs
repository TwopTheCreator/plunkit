use crate::asset::providers::{Provider, ProviderFactory};
use crate::cache::CacheManager;
use crate::config::{AssetDomain, BunjaLock, RetryStrategy};
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct AssetFetcher {
    lock: Arc<BunjaLock>,
    cache_manager: Arc<CacheManager>,
}

impl AssetFetcher {
    pub fn new(lock: Arc<BunjaLock>, cache_manager: Arc<CacheManager>) -> Self {
        Self {
            lock,
            cache_manager,
        }
    }

    pub async fn fetch(
        &self,
        domain_name: &str,
        asset_path: &str,
    ) -> Result<Vec<u8>> {
        let domain = self
            .lock
            .get_domain(domain_name)
            .context(format!("Domain '{}' not found in configuration", domain_name))?;

        let cache_key = self.generate_cache_key(domain_name, asset_path);

        if let Some(cached) = self.cache_manager.get(&cache_key).await {
            debug!("Cache hit for key: {}", cache_key);
            return Ok(cached);
        }

        debug!("Cache miss for key: {}", cache_key);

        let data = self.fetch_with_retry(domain, asset_path).await?;

        self.cache_manager.set(&cache_key, &data).await?;

        Ok(data)
    }

    async fn fetch_with_retry(
        &self,
        domain: &AssetDomain,
        asset_path: &str,
    ) -> Result<Vec<u8>> {
        let provider = ProviderFactory::create(&domain.provider);
        let retry_strategy = &domain.retry_strategy;

        let mut last_error = None;

        for attempt in 0..=retry_strategy.max_retries {
            if attempt > 0 {
                let backoff = if retry_strategy.exponential_backoff {
                    retry_strategy.backoff_ms * 2_u64.pow(attempt - 1)
                } else {
                    retry_strategy.backoff_ms
                };

                warn!(
                    "Retrying fetch (attempt {}/{}), waiting {}ms",
                    attempt, retry_strategy.max_retries, backoff
                );

                sleep(Duration::from_millis(backoff)).await;
            }

            match provider.fetch_asset(asset_path, domain).await {
                Ok(data) => {
                    if attempt > 0 {
                        info!("Successfully fetched asset after {} retries", attempt);
                    }
                    return Ok(data);
                }
                Err(e) => {
                    error!("Fetch attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    if !domain.fallback_domains.is_empty() && attempt == retry_strategy.max_retries {
                        return self.try_fallback_domains(domain, asset_path).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All fetch attempts failed")))
    }

    async fn try_fallback_domains(
        &self,
        _original_domain: &AssetDomain,
        asset_path: &str,
    ) -> Result<Vec<u8>> {
        info!("Attempting fallback domains");

        for fallback_name in &_original_domain.fallback_domains {
            if let Some(fallback_domain) = self.lock.get_domain(fallback_name) {
                let provider = ProviderFactory::create(&fallback_domain.provider);

                match provider.fetch_asset(asset_path, fallback_domain).await {
                    Ok(data) => {
                        info!("Successfully fetched from fallback domain: {}", fallback_name);
                        return Ok(data);
                    }
                    Err(e) => {
                        warn!("Fallback domain '{}' failed: {}", fallback_name, e);
                    }
                }
            }
        }

        anyhow::bail!("All fallback domains failed")
    }

    fn generate_cache_key(&self, domain_name: &str, asset_path: &str) -> String {
        use sha2::{Digest, Sha256};

        let input = format!("{}:{}", domain_name, asset_path);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    pub async fn prefetch(
        &self,
        domain_name: &str,
        asset_paths: Vec<String>,
    ) -> Result<Vec<Result<()>>> {
        let mut handles = vec![];

        for asset_path in asset_paths {
            let fetcher = self.clone();
            let domain_name = domain_name.to_string();

            let handle = tokio::spawn(async move {
                fetcher.fetch(&domain_name, &asset_path).await.map(|_| ())
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.context("Task join error")?);
        }

        Ok(results)
    }
}

impl Clone for AssetFetcher {
    fn clone(&self) -> Self {
        Self {
            lock: Arc::clone(&self.lock),
            cache_manager: Arc::clone(&self.cache_manager),
        }
    }
}
