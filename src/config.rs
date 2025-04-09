use std::{env, fs};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    api_key: Option<String>,
    #[serde(default = "get_default_model")]
    model: String,
    scopes: Vec<String>,
}

impl Config {
    pub fn api_key(&self) -> String {
        self.api_key
            .clone()
            .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
            .expect("Unable to retrieve API key")
    }

    pub fn model(&self) -> String {
        self.model.clone()
    }

    pub fn scopes(&self) -> Vec<String> {
        self.scopes.clone()
    }

    pub fn build_default() -> Self {
        Self {
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            model: get_default_model(),
            scopes: vec![],
        }
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
