use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCall {
    pub domain: String,
    pub path: String,
    pub original: String,
    pub call_type: AssetCallType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetCallType {
    BunjaProtocol,
    HtmlTag,
    CssUrl,
    JsImport,
}

lazy_static! {
    pub static ref BUNJA_PROTOCOL: Regex =
        Regex::new(r#"bunja://([a-zA-Z0-9_-]+)/(.+?)(?:\s|"|'|$)"#).unwrap();

    pub static ref HTML_IMG_TAG: Regex =
        Regex::new(r#"<img[^>]+src=["']bunja://([a-zA-Z0-9_-]+)/([^"']+)["'][^>]*>"#).unwrap();

    pub static ref CSS_URL: Regex =
        Regex::new(r#"url\(['"]?bunja://([a-zA-Z0-9_-]+)/([^'")\s]+)['"]?\)"#).unwrap();

    pub static ref JS_IMPORT: Regex =
        Regex::new(r#"(?:import|require)\s*\(\s*['"]bunja://([a-zA-Z0-9_-]+)/([^'"]+)['"]\s*\)"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct AssetPattern {
    patterns: Vec<(Regex, AssetCallType)>,
}

impl AssetPattern {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (BUNJA_PROTOCOL.clone(), AssetCallType::BunjaProtocol),
                (HTML_IMG_TAG.clone(), AssetCallType::HtmlTag),
                (CSS_URL.clone(), AssetCallType::CssUrl),
                (JS_IMPORT.clone(), AssetCallType::JsImport),
            ],
        }
    }

    pub fn find_all(&self, content: &str) -> Vec<AssetCall> {
        let mut calls = vec![];

        for (pattern, call_type) in &self.patterns {
            for cap in pattern.captures_iter(content) {
                if cap.len() >= 3 {
                    calls.push(AssetCall {
                        domain: cap[1].to_string(),
                        path: cap[2].to_string(),
                        original: cap[0].to_string(),
                        call_type: call_type.clone(),
                    });
                }
            }
        }

        calls
    }

    pub fn replace_all(&self, content: &str, replacer: impl Fn(&AssetCall) -> String) -> String {
        let calls = self.find_all(content);
        let mut result = content.to_string();

        for call in calls {
            let replacement = replacer(&call);
            result = result.replace(&call.original, &replacement);
        }

        result
    }
}

impl Default for AssetPattern {
    fn default() -> Self {
        Self::new()
    }
}
