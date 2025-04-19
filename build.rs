use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;

const HF_MODEL_ID: &str = "usvsnsp/code-vs-nl";

const MODEL_FILES: &[&str] = &["config.json", "tokenizer.json", "model.safetensors"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").context("Failed to get OUT_DIR")?;
    let model_dir = Path::new(&out_dir).join("model");

    fs::create_dir_all(&model_dir).context("Failed to create model directory")?;

    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.repo(hf_hub::Repo::new(
        HF_MODEL_ID.to_string(),
        hf_hub::RepoType::Model,
    ));

    for file in MODEL_FILES {
        let src_path = repo.get(file)?;
        let dest_path = model_dir.join(file);
        fs::copy(src_path, &dest_path)
            .with_context(|| format!("Failed to copy {} to {}", file, dest_path.display()))?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
