use anyhow::{Error, Result};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use std::io::Cursor;
use std::path::PathBuf;
use tokenizers::Tokenizer;
// Include model files directly in the binary
const MODEL_SAFE_TENSORS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/model/model.safetensors"));
const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/model/config.json"));
const TOKENIZER_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/model/tokenizer.json"));

struct CommitGenerator {
    model: candle_transformers::models::bert::BertModel,
    tokenizer: Tokenizer,
    device: Device,
}
