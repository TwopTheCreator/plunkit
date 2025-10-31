use crate::config::{AssetDomain, DomainProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>>;
    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String>;
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(provider_type: &DomainProvider) -> Box<dyn Provider> {
        match provider_type {
            DomainProvider::Pexels => Box::new(PexelsProvider::new()),
            DomainProvider::Unsplash => Box::new(UnsplashProvider::new()),
            DomainProvider::Cloudinary => Box::new(CloudinaryProvider::new()),
            DomainProvider::S3 => Box::new(S3Provider::new()),
            DomainProvider::Custom => Box::new(CustomProvider::new()),
            DomainProvider::Local => Box::new(LocalProvider::new()),
        }
    }
}

pub struct PexelsProvider {
    client: Client,
}

impl PexelsProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Provider for PexelsProvider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let url = self.resolve_url(asset_path, domain).await?;

        let mut request = self.client.get(&url);

        if let Some(api_key) = &domain.api_key {
            request = request.header("Authorization", api_key);
        }

        for (key, value) in &domain.headers {
            request = request.header(key, value);
        }

        let response = request.send().await
            .context("Failed to fetch asset from Pexels")?;

        if !response.status().is_success() {
            anyhow::bail!("Pexels API returned status: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        Ok(bytes.to_vec())
    }

    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String> {
        if asset_path.starts_with("http://") || asset_path.starts_with("https://") {
            return Ok(asset_path.to_string());
        }

        let search_url = format!("{}/search?query={}&per_page=1", domain.base_url, asset_path);

        let mut request = self.client.get(&search_url);

        if let Some(api_key) = &domain.api_key {
            request = request.header("Authorization", api_key);
        }

        let response = request.send().await
            .context("Failed to search Pexels")?;

        let json: Value = response.json().await
            .context("Failed to parse Pexels response")?;

        let photo_url = json["photos"][0]["src"]["original"]
            .as_str()
            .context("No photo found in Pexels response")?;

        Ok(photo_url.to_string())
    }
}

pub struct UnsplashProvider {
    client: Client,
}

impl UnsplashProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Provider for UnsplashProvider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let url = self.resolve_url(asset_path, domain).await?;

        let mut request = self.client.get(&url);

        for (key, value) in &domain.headers {
            request = request.header(key, value);
        }

        let response = request.send().await
            .context("Failed to fetch asset from Unsplash")?;

        if !response.status().is_success() {
            anyhow::bail!("Unsplash API returned status: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        Ok(bytes.to_vec())
    }

    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String> {
        if asset_path.starts_with("http://") || asset_path.starts_with("https://") {
            return Ok(asset_path.to_string());
        }

        let search_url = format!("{}/search/photos?query={}&per_page=1", domain.base_url, asset_path);

        let mut request = self.client.get(&search_url);

        if let Some(api_key) = &domain.api_key {
            request = request.header("Authorization", format!("Client-ID {}", api_key));
        }

        let response = request.send().await
            .context("Failed to search Unsplash")?;

        let json: Value = response.json().await
            .context("Failed to parse Unsplash response")?;

        let photo_url = json["results"][0]["urls"]["regular"]
            .as_str()
            .context("No photo found in Unsplash response")?;

        Ok(photo_url.to_string())
    }
}

pub struct CloudinaryProvider {
    client: Client,
}

impl CloudinaryProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Provider for CloudinaryProvider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let url = self.resolve_url(asset_path, domain).await?;

        let mut request = self.client.get(&url);

        for (key, value) in &domain.headers {
            request = request.header(key, value);
        }

        let response = request.send().await
            .context("Failed to fetch asset from Cloudinary")?;

        if !response.status().is_success() {
            anyhow::bail!("Cloudinary returned status: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        Ok(bytes.to_vec())
    }

    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String> {
        let mut url = format!("{}/{}", domain.base_url.trim_end_matches('/'), asset_path.trim_start_matches('/'));

        if !domain.transformations.is_empty() {
            let transforms: Vec<String> = domain.transformations
                .iter()
                .map(|t| {
                    let params: Vec<String> = t.parameters
                        .iter()
                        .map(|(k, v)| format!("{}_{}", k, v))
                        .collect();
                    params.join(",")
                })
                .collect();

            let transform_str = transforms.join("/");
            url = format!("{}/{}/{}", domain.base_url.trim_end_matches('/'), transform_str, asset_path.trim_start_matches('/'));
        }

        Ok(url)
    }
}

pub struct S3Provider {
    client: Client,
}

impl S3Provider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Provider for S3Provider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let url = self.resolve_url(asset_path, domain).await?;

        let mut request = self.client.get(&url);

        for (key, value) in &domain.headers {
            request = request.header(key, value);
        }

        let response = request.send().await
            .context("Failed to fetch asset from S3")?;

        if !response.status().is_success() {
            anyhow::bail!("S3 returned status: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        Ok(bytes.to_vec())
    }

    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String> {
        Ok(format!("{}/{}", domain.base_url.trim_end_matches('/'), asset_path.trim_start_matches('/')))
    }
}

pub struct CustomProvider {
    client: Client,
}

impl CustomProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Provider for CustomProvider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let url = self.resolve_url(asset_path, domain).await?;

        let mut request = self.client.get(&url);

        if let Some(api_key) = &domain.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        for (key, value) in &domain.headers {
            request = request.header(key, value);
        }

        let response = request.send().await
            .context("Failed to fetch asset from custom provider")?;

        if !response.status().is_success() {
            anyhow::bail!("Custom provider returned status: {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        Ok(bytes.to_vec())
    }

    async fn resolve_url(&self, asset_path: &str, domain: &AssetDomain) -> Result<String> {
        Ok(format!("{}/{}", domain.base_url.trim_end_matches('/'), asset_path.trim_start_matches('/')))
    }
}

pub struct LocalProvider;

impl LocalProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for LocalProvider {
    async fn fetch_asset(&self, asset_path: &str, domain: &AssetDomain) -> Result<Vec<u8>> {
        let full_path = format!("{}/{}", domain.base_url.trim_end_matches('/'), asset_path.trim_start_matches('/'));

        let bytes = tokio::fs::read(&full_path).await
            .context(format!("Failed to read local file: {}", full_path))?;

        Ok(bytes)
    }

    async fn resolve_url(&self, asset_path: &str, _domain: &AssetDomain) -> Result<String> {
        Ok(asset_path.to_string())
    }
}
