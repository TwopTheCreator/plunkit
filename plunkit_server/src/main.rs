mod api;
mod db;
mod lua;
mod protocol;
mod sandbox;
mod server;
mod world;

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing_subscriber;

use api::{create_router, AppState};
use db::Database;
use lua::PluginManager;
use sandbox::SandboxManager;
use world::WorldManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Plunkit Server...");

    let db = Arc::new(Database::from_env()?);
    let world_manager = Arc::new(WorldManager::new());
    let sandbox_manager = Arc::new(SandboxManager::new());
    let plugin_manager = Arc::new(PluginManager::new());

    let state = AppState {
        db: db.clone(),
        world_manager: world_manager.clone(),
        sandbox_manager: sandbox_manager.clone(),
        plugin_manager: plugin_manager.clone(),
    };

    let world_manager_tick = world_manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(50));
        loop {
            ticker.tick().await;
            if let Err(e) = world_manager_tick.tick_all().await {
                tracing::error!("World tick error: {}", e);
            }
        }
    });

    let sandbox_manager_tick = sandbox_manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(50));
        loop {
            ticker.tick().await;
            if let Err(e) = sandbox_manager_tick.tick_all().await {
                tracing::error!("Sandbox tick error: {}", e);
            }
        }
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await?;

    tracing::info!("Web API listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await?;

    Ok(())
}
