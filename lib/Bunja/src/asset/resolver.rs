use crate::asset::providers::ProviderFactory;
use crate::config::BunjaLock;
use anyhow::{Context, Result};
use std::sync::Arc;

pub struct AssetResolver {
    lock: Arc<BunjaLock>,
}

impl AssetResolver {
    pub fn new(lock: Arc<BunjaLock>) -> Self {
        Self { lock }
    }

    pub async fn resolve_url(
        &self,
        domain_name: &str,
        asset_path: &str,
    ) -> Result<String> {
        let domain = self
            .lock
            .get_domain(domain_name)
            .context(format!("Domain '{}' not found", domain_name))?;

        let provider = ProviderFactory::create(&domain.provider);

        provider.resolve_url(asset_path, domain).await
    }

    pub async fn resolve_multiple(
        &self,
        domain_name: &str,
        asset_paths: Vec<String>,
    ) -> Vec<Result<String>> {
        let mut handles = vec![];

        for asset_path in asset_paths {
            let resolver = self.clone();
            let domain_name = domain_name.to_string();

            let handle = tokio::spawn(async move {
                resolver.resolve_url(&domain_name, &asset_path).await
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(anyhow::anyhow!("Task join error: {}", e))),
            }
        }

        results
    }
}

impl Clone for AssetResolver {
    fn clone(&self) -> Self {
        Self {
            lock: Arc::clone(&self.lock),
        }
    }
}
