use crate::asset::{AssetFetcher, AssetResolver};
use crate::cache::CacheManager;
use crate::config::BunjaLock;
use crate::translator::parser::AssetCallParser;
use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

pub struct TranslationEngine {
    parser: AssetCallParser,
    fetcher: AssetFetcher,
    resolver: AssetResolver,
}

impl TranslationEngine {
    pub fn new(
        lock: Arc<BunjaLock>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        let parser = AssetCallParser::new();
        let fetcher = AssetFetcher::new(Arc::clone(&lock), cache_manager);
        let resolver = AssetResolver::new(lock);

        Self {
            parser,
            fetcher,
            resolver,
        }
    }

    pub async fn translate_file<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let content = fs::read_to_string(input_path.as_ref()).await
            .context("Failed to read input file")?;

        let translated = self.translate_content(&content).await?;

        fs::write(output_path.as_ref(), translated).await
            .context("Failed to write output file")?;

        info!(
            "Translated {} -> {}",
            input_path.as_ref().display(),
            output_path.as_ref().display()
        );

        Ok(())
    }

    pub async fn translate_content(&self, content: &str) -> Result<String> {
        if !self.parser.has_asset_calls(content) {
            return Ok(content.to_string());
        }

        let calls = self.parser.parse(content);

        info!("Found {} asset calls to translate", calls.len());

        for call in &calls {
            let _ = self.fetcher.fetch(&call.domain, &call.path).await;
        }

        let result = self.parser.replace_with(content, |call| {
            format!("/bunja/{}/{}", call.domain, call.path)
        });

        Ok(result)
    }

    pub async fn resolve_urls(&self, content: &str) -> Result<String> {
        let resolver = &self.resolver;

        self.parser.translate_content(content, |domain, path| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    resolver.resolve_url(domain, path).await
                })
            })
        }).await
    }

    pub async fn prefetch_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let mut entries = fs::read_dir(dir.as_ref()).await
            .context("Failed to read directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if matches!(ext.to_str(), Some("html") | Some("css") | Some("js")) {
                        let content = fs::read_to_string(&path).await?;
                        let calls = self.parser.parse(&content);

                        for call in calls {
                            info!("Prefetching: {}/{}", call.domain, call.path);
                            let _ = self.fetcher.fetch(&call.domain, &call.path).await;
                        }
                    }
                }
            } else if path.is_dir() {
                self.prefetch_directory(&path).await?;
            }
        }

        Ok(())
    }

    pub fn get_fetcher(&self) -> &AssetFetcher {
        &self.fetcher
    }

    pub fn get_resolver(&self) -> &AssetResolver {
        &self.resolver
    }
}
