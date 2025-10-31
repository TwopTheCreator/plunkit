use crate::asset::AssetFetcher;
use crate::cache::CacheManager;
use crate::config::BunjaLock;
use crate::server::handlers::{cache_stats, clear_cache, health_check, serve_asset};
use crate::server::middleware::RequestLogger;
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use log::info;
use std::sync::Arc;

pub struct BunjaServer {
    lock: Arc<BunjaLock>,
    cache_manager: Arc<CacheManager>,
    fetcher: Arc<AssetFetcher>,
}

impl BunjaServer {
    pub fn new(
        lock: Arc<BunjaLock>,
        cache_manager: Arc<CacheManager>,
        fetcher: Arc<AssetFetcher>,
    ) -> Self {
        Self {
            lock,
            cache_manager,
            fetcher,
        }
    }

    pub async fn run(self) -> Result<()> {
        let port = self.lock.global_settings.server_port;
        let workers = self.lock.global_settings.worker_threads;

        let lock = Arc::clone(&self.lock);
        let cache_manager = Arc::clone(&self.cache_manager);
        let fetcher = Arc::clone(&self.fetcher);

        info!("Starting Bunja server on port {}", port);
        info!("Worker threads: {}", workers);

        HttpServer::new(move || {
            App::new()
                .wrap(RequestLogger)
                .app_data(web::Data::new(Arc::clone(&fetcher)))
                .app_data(web::Data::new(Arc::clone(&cache_manager)))
                .app_data(web::Data::new(Arc::clone(&lock)))
                .route("/health", web::get().to(health_check))
                .route("/bunja/{domain}/{path:.*}", web::get().to(serve_asset))
                .service(
                    web::scope("/api")
                        .route("/cache/stats", web::get().to(cache_stats))
                        .route("/cache/clear", web::post().to(clear_cache))
                )
        })
        .bind(("0.0.0.0", port))
        .context(format!("Failed to bind to port {}", port))?
        .workers(workers)
        .run()
        .await
        .context("Server error")?;

        Ok(())
    }
}
