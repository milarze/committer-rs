use std::{env, fs};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "Option::None")]
    api_key: Option<String>,
    #[serde(default = "get_default_model")]
    model: String,
    #[serde(default = "Vec::new")]
    scopes: Vec<String>,
    #[serde(default = "bool::default")]
    use_local: bool,
    #[serde(default = "max_tokens_default")]
    max_tokens: usize,
}

impl Config {
    pub fn api_key(&self) -> Option<String> {
        self.api_key.clone()
    }

    pub fn model(&self) -> String {
        self.model.clone()
    }

    pub fn scopes(&self) -> Vec<String> {
        self.scopes.clone()
    }

    pub fn use_local(&self) -> bool {
        self.use_local
    }

    pub fn build_default() -> Self {
        Self {
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            model: get_default_model(),
            scopes: vec![],
            use_local: false,
            max_tokens: max_tokens_default(),
        }
    }

    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }
}

use tracing::instrument;

#[instrument(level = "info")]
pub fn read_config() -> Config {
    let home_dir = match home::home_dir() {
        Some(dir) => dir,
        None => panic!("Unable to retrieve home directory"),
    };
    if let Ok(content) = fs::read_to_string(format!(
        "{}/.committer-rs/config.yml",
        home_dir.to_string_lossy()
    )) {
        serde_yaml::from_str(&content)
            .with_context(|| format!("Error parsing config: {}", content))
            .unwrap()
    } else {
        Config::build_default()
    }
}

fn get_default_model() -> String {
    "claude-3-7-sonnet-20250219".to_string()
}

fn max_tokens_default() -> usize {
    1000
}
