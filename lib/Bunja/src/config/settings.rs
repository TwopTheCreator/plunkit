use super::BunjaLock;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Settings {
    pub lock: Arc<BunjaLock>,
    pub lock_file_path: PathBuf,
    pub runtime_cache_dir: PathBuf,
}

impl Settings {
    pub fn new(lock_file_path: PathBuf) -> Result<Self> {
        let lock = BunjaLock::load(&lock_file_path)?;
        lock.validate()?;

        let runtime_cache_dir = PathBuf::from(&lock.global_settings.cache_dir);

        Ok(Self {
            lock: Arc::new(lock),
            lock_file_path,
            runtime_cache_dir,
        })
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from("bunja.lock")
    }

    pub fn reload(&mut self) -> Result<()> {
        let lock = BunjaLock::load(&self.lock_file_path)?;
        lock.validate()?;

        self.runtime_cache_dir = PathBuf::from(&lock.global_settings.cache_dir);
        self.lock = Arc::new(lock);

        Ok(())
    }

    pub fn get_cache_path(&self, key: &str) -> PathBuf {
        self.runtime_cache_dir.join(key)
    }
}
