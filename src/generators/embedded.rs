use anyhow::{Error, Result};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use std::io::Cursor;
use std::path::PathBuf;
use tokenizers::Tokenizer;
// Include model files directly in the binary
const MODEL_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/pytorch_model.bin"));
const CONFIG_JSON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/config.json"));
const TOKENIZER_JSON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/tokenizer.json"));
const VOCAB_JSON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/vocab.json"));
const MERGES_TXT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/merges.txt"));

struct CommitGenerator {
    model: GPT2Model,
    tokenizer: Tokenizer,
    device: Device,
}
