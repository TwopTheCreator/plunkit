use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;

pub struct CacheStorage {
    cache_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CacheMetadata {
    pub size: u64,
    pub created_at: SystemTime,
}

impl CacheStorage {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.get_path(key);

        fs::read(&path).await
            .context(format!("Failed to read cache file: {:?}", path))
    }

    pub async fn set(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.get_path(key);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create cache subdirectory")?;
        }

        fs::write(&path, data).await
            .context(format!("Failed to write cache file: {:?}", path))
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let path = self.get_path(key);

        if path.exists() {
            fs::remove_file(&path).await
                .context(format!("Failed to delete cache file: {:?}", path))?;
        }

        Ok(())
    }

    pub async fn get_metadata(&self, key: &str) -> Result<CacheMetadata> {
        let path = self.get_path(key);

        let metadata = fs::metadata(&path).await
            .context(format!("Failed to get cache file metadata: {:?}", path))?;

        let created_at = metadata.created()
            .or_else(|_| metadata.modified())
            .unwrap_or_else(|_| SystemTime::now());

        Ok(CacheMetadata {
            size: metadata.len(),
            created_at,
        })
    }

    pub async fn list_all(&self) -> Result<Vec<(String, CacheMetadata)>> {
        let mut entries = vec![];

        self.scan_directory(&self.cache_dir, &mut entries).await?;

        Ok(entries)
    }

    async fn scan_directory(
        &self,
        dir: &PathBuf,
        entries: &mut Vec<(String, CacheMetadata)>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut read_dir = fs::read_dir(dir).await
            .context("Failed to read cache directory")?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path, entries).await?;
            } else if path.is_file() {
                if let Some(key) = self.path_to_key(&path) {
                    if let Ok(metadata) = self.get_metadata(&key).await {
                        entries.push((key, metadata));
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).await
                .context("Failed to clear cache directory")?;

            fs::create_dir_all(&self.cache_dir).await
                .context("Failed to recreate cache directory")?;
        }

        Ok(())
    }

    fn get_path(&self, key: &str) -> PathBuf {
        let subdir = &key[..2.min(key.len())];
        self.cache_dir.join(subdir).join(key)
    }

    fn path_to_key(&self, path: &PathBuf) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}
