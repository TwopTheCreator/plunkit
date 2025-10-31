use crate::translator::patterns::{AssetCall, AssetPattern};
use anyhow::Result;

pub struct AssetCallParser {
    pattern: AssetPattern,
}

impl AssetCallParser {
    pub fn new() -> Self {
        Self {
            pattern: AssetPattern::new(),
        }
    }

    pub fn parse(&self, content: &str) -> Vec<AssetCall> {
        self.pattern.find_all(content)
    }

    pub fn has_asset_calls(&self, content: &str) -> bool {
        !self.parse(content).is_empty()
    }

    pub fn replace_with<F>(&self, content: &str, replacer: F) -> String
    where
        F: Fn(&AssetCall) -> String,
    {
        self.pattern.replace_all(content, replacer)
    }

    pub async fn translate_content(
        &self,
        content: &str,
        resolver: impl Fn(&str, &str) -> Result<String>,
    ) -> Result<String> {
        let calls = self.parse(content);
        let mut result = content.to_string();

        for call in calls {
            let resolved_url = resolver(&call.domain, &call.path)?;

            let replacement = match call.call_type {
                crate::translator::patterns::AssetCallType::BunjaProtocol => {
                    resolved_url
                }
                crate::translator::patterns::AssetCallType::HtmlTag => {
                    call.original.replace(
                        &format!("bunja://{}/{}", call.domain, call.path),
                        &resolved_url,
                    )
                }
                crate::translator::patterns::AssetCallType::CssUrl => {
                    format!("url('{}')", resolved_url)
                }
                crate::translator::patterns::AssetCallType::JsImport => {
                    call.original.replace(
                        &format!("bunja://{}/{}", call.domain, call.path),
                        &resolved_url,
                    )
                }
            };

            result = result.replace(&call.original, &replacement);
        }

        Ok(result)
    }
}

impl Default for AssetCallParser {
    fn default() -> Self {
        Self::new()
    }
}
