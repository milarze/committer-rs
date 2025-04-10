use anyhow::{Context, Result};
use curl::easy::Easy;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const HF_FILES_URL: &str =
    "https://huggingface.co/JosineyJr/generate-conventional-commit-messages/resolve/main";

const MODEL_FILES: &[&str] = &[
    "config.json",
    "pytorch_model.bin",
    "tokenizer.json",
    "tokenizer_config.json",
    "vocab.json",
    "merges.txt",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").context("Failed to get OUT_DIR")?;
    let model_dir = Path::new(&out_dir).join("model");

    fs::create_dir_all(&model_dir).context("Failed to create model directory")?;

    for &file in MODEL_FILES {
        let target_path = model_dir.join(file);

        // Skip if file already exists (for faster rebuilds)
        if target_path.exists() {
            println!("File already exists: {}", file);
            continue;
        }

        println!("cargo:warning=Downloading: {}", file);
        download_file(&format!("{}/{}", HF_FILES_URL, file), &target_path)
            .with_context(|| format!("Failed to download {}", file))?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn download_file(url: &str, dest: &PathBuf) -> Result<()> {
    // Create the output file
    let mut file = fs::File::create(dest)
        .with_context(|| format!("Failed to create file: {}", dest.display()))?;

    // Initialize a curl handler
    let mut easy = Easy::new();
    easy.url(url).context("Failed to set URL")?;
    easy.follow_location(true)
        .context("Failed to enable redirect following")?;

    // Download the file
    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        file.write_all(data)
            .map_err(|_| curl::easy::WriteError::Pause)?;
        Ok(data.len())
    })?;

    transfer.perform().context("Failed to perform transfer")?;

    Ok(())
}
