use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::{
    db::Database,
    lua::PluginManager,
    sandbox::SandboxManager,
    world::WorldManager,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub world_manager: Arc<WorldManager>,
    pub sandbox_manager: Arc<SandboxManager>,
    pub plugin_manager: Arc<PluginManager>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/worlds", get(list_worlds).post(create_world))
        .route("/api/worlds/:id", get(get_world).delete(delete_world))
        .route("/api/worlds/:id/start", post(start_world))
        .route("/api/worlds/:id/stop", post(stop_world))
        .route("/api/worlds/:id/players", get(list_world_players))
        .route("/api/worlds/:id/chunks", get(list_world_chunks))
        .route("/api/plugins", get(list_plugins).post(create_plugin))
        .route("/api/plugins/:id", get(get_plugin).delete(delete_plugin))
        .route("/api/plugins/:id/enable", post(enable_plugin))
        .route("/api/plugins/:id/disable", post(disable_plugin))
        .route("/api/stats", get(get_stats))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorldRequest {
    pub name: String,
    pub max_players: Option<i32>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub port: i32,
    pub max_players: i32,
    pub player_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub script: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
}

async fn list_worlds(State(state): State<AppState>) -> Json<Vec<WorldResponse>> {
    let world_ids = state.world_manager.list_worlds().await;

    let mut worlds = Vec::new();
    for id in world_ids {
        if let Some(world) = state.world_manager.get_world(&id).await {
            worlds.push(WorldResponse {
                id: world.id.clone(),
                name: world.name.clone(),
                status: "running".to_string(),
                port: 25565,
                max_players: 20,
                player_count: world.get_all_players().await.len(),
            });
        }
    }

    Json(worlds)
}

async fn create_world(
    State(state): State<AppState>,
    Json(req): Json<CreateWorldRequest>,
) -> Result<Json<WorldResponse>, StatusCode> {
    let world_id = uuid::Uuid::new_v4().to_string();

    let world = state
        .world_manager
        .create_world(world_id.clone(), req.name.clone())
        .await;

    Ok(Json(WorldResponse {
        id: world.id.clone(),
        name: world.name.clone(),
        status: "stopped".to_string(),
        port: 25565,
        max_players: req.max_players.unwrap_or(20),
        player_count: 0,
    }))
}

async fn get_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorldResponse>, StatusCode> {
    if let Some(world) = state.world_manager.get_world(&id).await {
        Ok(Json(WorldResponse {
            id: world.id.clone(),
            name: world.name.clone(),
            status: "running".to_string(),
            port: 25565,
            max_players: 20,
            player_count: world.get_all_players().await.len(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.world_manager.remove_world(&id).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn start_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorldResponse>, StatusCode> {
    if let Some(world) = state.world_manager.get_world(&id).await {
        Ok(Json(WorldResponse {
            id: world.id.clone(),
            name: world.name.clone(),
            status: "running".to_string(),
            port: 25565,
            max_players: 20,
            player_count: 0,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn stop_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorldResponse>, StatusCode> {
    if let Some(world) = state.world_manager.get_world(&id).await {
        Ok(Json(WorldResponse {
            id: world.id.clone(),
            name: world.name.clone(),
            status: "stopped".to_string(),
            port: 25565,
            max_players: 20,
            player_count: 0,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_world_players(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PlayerInfo>>, StatusCode> {
    if let Some(world) = state.world_manager.get_world(&id).await {
        let players = world.get_all_players().await;

        let player_infos: Vec<PlayerInfo> = players
            .into_iter()
            .map(|(_, username, position)| PlayerInfo {
                username,
                x: position.x,
                y: position.y,
                z: position.z,
            })
            .collect();

        Ok(Json(player_infos))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Debug, Serialize)]
struct PlayerInfo {
    username: String,
    x: f64,
    y: f64,
    z: f64,
}

async fn list_world_chunks(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ChunkInfo>>, StatusCode> {
    Ok(Json(Vec::new()))
}

#[derive(Debug, Serialize)]
struct ChunkInfo {
    x: i32,
    z: i32,
}

async fn list_plugins(State(state): State<AppState>) -> Json<Vec<PluginResponse>> {
    let plugins = state.plugin_manager.list_plugins().await;

    let plugin_responses: Vec<PluginResponse> = plugins
        .into_iter()
        .map(|p| PluginResponse {
            id: uuid::Uuid::new_v4().to_string(),
            name: p.name,
            version: p.version,
            author: p.author,
            description: p.description,
            enabled: true,
        })
        .collect();

    Json(plugin_responses)
}

async fn create_plugin(
    State(state): State<AppState>,
    Json(req): Json<CreatePluginRequest>,
) -> Result<Json<PluginResponse>, StatusCode> {
    let metadata = crate::lua::PluginMetadata {
        name: req.name.clone(),
        version: req.version.clone(),
        author: req.author.clone(),
        description: req.description.clone(),
        dependencies: Vec::new(),
    };

    if let Err(e) = state.plugin_manager.load_plugin(metadata, &req.script).await {
        tracing::error!("Failed to load plugin: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(PluginResponse {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        version: req.version,
        author: req.author,
        description: req.description,
        enabled: true,
    }))
}

async fn get_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PluginResponse>, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}

async fn delete_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

async fn enable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}

async fn disable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    active_worlds: usize,
    total_players: usize,
    cpu_usage: f32,
    memory_usage: u64,
}

async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let world_ids = state.world_manager.list_worlds().await;

    let mut total_players = 0;
    for id in &world_ids {
        if let Some(world) = state.world_manager.get_world(id).await {
            total_players += world.get_all_players().await.len();
        }
    }

    Json(StatsResponse {
        active_worlds: world_ids.len(),
        total_players,
        cpu_usage: 0.0,
        memory_usage: 0,
    })
}
