use crate::asset::AssetFetcher;
use crate::cache::CacheManager;
use actix_web::{web, HttpResponse, Result};
use log::{error, info};
use mime_guess::from_path;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AssetPath {
    domain: String,
    path: String,
}

pub async fn serve_asset(
    params: web::Path<AssetPath>,
    fetcher: web::Data<Arc<AssetFetcher>>,
) -> Result<HttpResponse> {
    let domain = &params.domain;
    let asset_path = &params.path;

    info!("Serving asset: {}/{}", domain, asset_path);

    match fetcher.fetch(domain, asset_path).await {
        Ok(data) => {
            let mime_type = from_path(asset_path).first_or_octet_stream();

            Ok(HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(data))
        }
        Err(e) => {
            error!("Failed to fetch asset {}/{}: {}", domain, asset_path, e);

            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Asset not found",
                "domain": domain,
                "path": asset_path,
                "details": e.to_string(),
            })))
        }
    }
}

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "bunja",
    })))
}

pub async fn cache_stats(
    cache_manager: web::Data<Arc<CacheManager>>,
) -> Result<HttpResponse> {
    let stats = cache_manager.get_stats().await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_entries": stats.total_entries,
        "total_size_bytes": stats.total_size_bytes,
        "total_size_mb": stats.total_size_bytes / (1024 * 1024),
        "max_size_bytes": stats.max_size_bytes,
        "max_size_mb": stats.max_size_bytes / (1024 * 1024),
        "usage_percent": (stats.total_size_bytes as f64 / stats.max_size_bytes as f64 * 100.0),
    })))
}

pub async fn clear_cache(
    cache_manager: web::Data<Arc<CacheManager>>,
) -> Result<HttpResponse> {
    match cache_manager.clear().await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Cache cleared successfully",
        }))),
        Err(e) => {
            error!("Failed to clear cache: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to clear cache",
                "details": e.to_string(),
            })))
        }
    }
}
