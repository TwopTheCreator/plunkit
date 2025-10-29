pub mod wasm_runtime;

pub use wasm_runtime::{WasmRuntime, RuntimeContext};

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct SandboxManager {
    sandboxes: Arc<RwLock<HashMap<String, Arc<WasmRuntime>>>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_sandbox(&self, world_id: String, wasm_bytes: &[u8]) -> Result<()> {
        let runtime = Arc::new(WasmRuntime::new(world_id.clone()).await?);
        runtime.load_module(wasm_bytes).await?;

        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.insert(world_id, runtime);

        Ok(())
    }

    pub async fn get_sandbox(&self, world_id: &str) -> Option<Arc<WasmRuntime>> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes.get(world_id).cloned()
    }

    pub async fn remove_sandbox(&self, world_id: &str) -> Result<()> {
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.remove(world_id);
        Ok(())
    }

    pub async fn list_sandboxes(&self) -> Vec<String> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes.keys().cloned().collect()
    }

    pub async fn tick_all(&self) -> Result<()> {
        let sandboxes = self.sandboxes.read().await;

        for (world_id, runtime) in sandboxes.iter() {
            if let Err(e) = runtime.tick().await {
                tracing::error!("Error ticking sandbox {}: {}", world_id, e);
            }
        }

        Ok(())
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}
