use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunjaLock {
    pub version: String,
    pub domains: HashMap<String, AssetDomain>,
    #[serde(default)]
    pub global_settings: GlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
    #[serde(default = "default_max_cache_size")]
    pub max_cache_size_mb: u64,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
    #[serde(default)]
    pub enable_compression: bool,
    #[serde(default = "default_workers")]
    pub worker_threads: usize,
    #[serde(default = "default_port")]
    pub server_port: u16,
    #[serde(default)]
    pub enable_https: bool,
    #[serde(default)]
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDomain {
    pub provider: DomainProvider,
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub transformations: Vec<Transformation>,
    #[serde(default)]
    pub fallback_domains: Vec<String>,
    #[serde(default)]
    pub rate_limit: Option<RateLimit>,
    #[serde(default)]
    pub retry_strategy: RetryStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainProvider {
    Pexels,
    Unsplash,
    Cloudinary,
    S3,
    Custom,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub name: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_backoff")]
    pub backoff_ms: u64,
    #[serde(default)]
    pub exponential_backoff: bool,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
            max_cache_size_mb: default_max_cache_size(),
            cache_ttl_seconds: default_cache_ttl(),
            enable_compression: false,
            worker_threads: default_workers(),
            server_port: default_port(),
            enable_https: false,
            log_level: "info".to_string(),
        }
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            backoff_ms: default_backoff(),
            exponential_backoff: true,
        }
    }
}

fn default_cache_dir() -> String {
    ".bunja_cache".to_string()
}

fn default_max_cache_size() -> u64 {
    1024
}

fn default_cache_ttl() -> u64 {
    86400
}

fn default_workers() -> usize {
    4
}

fn default_port() -> u16 {
    8080
}

fn default_max_retries() -> u32 {
    3
}

fn default_backoff() -> u64 {
    1000
}

impl BunjaLock {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read bunja.lock file")?;

        let lock: BunjaLock = toml::from_str(&content)
            .context("Failed to parse bunja.lock file")?;

        Ok(lock)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_else(|_| Self::default())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize bunja.lock")?;

        fs::write(path.as_ref(), content)
            .context("Failed to write bunja.lock file")?;

        Ok(())
    }

    pub fn get_domain(&self, name: &str) -> Option<&AssetDomain> {
        self.domains.get(name)
    }

    pub fn add_domain(&mut self, name: String, domain: AssetDomain) {
        self.domains.insert(name, domain);
    }

    pub fn remove_domain(&mut self, name: &str) -> Option<AssetDomain> {
        self.domains.remove(name)
    }

    pub fn validate(&self) -> Result<()> {
        if self.domains.is_empty() {
            anyhow::bail!("No asset domains configured");
        }

        for (name, domain) in &self.domains {
            if domain.base_url.is_empty() {
                anyhow::bail!("Domain '{}' has empty base_url", name);
            }
        }

        Ok(())
    }
}

impl Default for BunjaLock {
    fn default() -> Self {
        let mut domains = HashMap::new();

        domains.insert(
            "pexels".to_string(),
            AssetDomain {
                provider: DomainProvider::Pexels,
                base_url: "https://api.pexels.com/v1".to_string(),
                api_key: None,
                headers: HashMap::new(),
                transformations: vec![],
                fallback_domains: vec![],
                rate_limit: Some(RateLimit {
                    requests_per_second: 20,
                    burst_size: 50,
                }),
                retry_strategy: RetryStrategy::default(),
            },
        );

        domains.insert(
            "unsplash".to_string(),
            AssetDomain {
                provider: DomainProvider::Unsplash,
                base_url: "https://api.unsplash.com".to_string(),
                api_key: None,
                headers: HashMap::new(),
                transformations: vec![],
                fallback_domains: vec![],
                rate_limit: Some(RateLimit {
                    requests_per_second: 50,
                    burst_size: 100,
                }),
                retry_strategy: RetryStrategy::default(),
            },
        );

        Self {
            version: "1.0.0".to_string(),
            domains,
            global_settings: GlobalSettings::default(),
        }
    }
}
