pub const MODEL_SAFE_TENSORS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/model/model.safetensors"));
pub const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/model/config.json"));
pub const TOKENIZER_JSON: &str =
    include_str!(concat!(env!("OUT_DIR"), "/model/tokenizer.json"));
