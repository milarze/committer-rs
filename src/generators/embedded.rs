use anyhow::{Error, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::{linear, Module, VarBuilder};
use tokenizers::Tokenizer;

// Include model files directly in the binary
const MODEL_SAFE_TENSORS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/model/model.safetensors"));
const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/model/config.json"));
const TOKENIZER_JSON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/tokenizer.json"));

pub struct CommitGenerator {
    model: candle_transformers::models::distilbert::DistilBertModel,
    tokenizer: Tokenizer,
    device: Device,
    lm_head: candle_nn::Linear,
}

impl CommitGenerator {
    pub fn new() -> Result<Self, Error> {
        // Load the tokenizer
        let tokenizer = Tokenizer::from_bytes(TOKENIZER_JSON)
            .map_err(|e| Error::msg(format!("Failed to load tokenizer: {}", e)))?;

        let config: serde_json::Value = serde_json::from_str(CONFIG_JSON)
            .map_err(|e| Error::msg(format!("Failed to load config: {}", e)))?;
        let hidden_size = config["hidden_size"]
            .as_u64()
            .ok_or_else(|| Error::msg("Failed to get hidden size from config"))?
            as usize;

        let config: candle_transformers::models::distilbert::Config =
            serde_json::from_str(CONFIG_JSON)
                .map_err(|e| Error::msg(format!("Failed to load config: {}", e)))?;

        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        let vb = VarBuilder::from_buffered_safetensors(
            MODEL_SAFE_TENSORS.to_vec(),
            DType::F32,
            &device,
        )?;
        let model =
            candle_transformers::models::distilbert::DistilBertModel::load(vb.clone(), &config)?;
        let vocab_size = tokenizer.get_vocab_size(true);
        let lm_head = linear(hidden_size, vocab_size, vb.pp("lm_head"))?;
        Ok(CommitGenerator {
            model,
            tokenizer,
            device,
            lm_head,
        })
    }

    pub fn generate(&self, prompt: &str, max_new_tokens: usize) -> Result<String, Error> {
        println!("Generating response for prompt: {}", prompt);

        // Encode prompt
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| Error::msg(format!("Failed to encode prompt: {}", e)))?;
        let mut input_ids = encoding.get_ids().to_vec();
        let mut attention_mask = encoding.get_attention_mask().to_vec();

        // Store generated tokens
        let mut generated_tokens = Vec::new();

        // Generation loop
        for _ in 0..max_new_tokens {
            // Prepare input tensors
            let input_tensor = Tensor::new(&*input_ids, &self.device)?.unsqueeze(0)?;
            let mask_tensor = Tensor::new(&*attention_mask, &self.device)?.unsqueeze(0)?;

            // Run model inference
            let hidden_states = self.model.forward(&input_tensor, &mask_tensor)?;

            // Get logits
            let logits = self.lm_head.forward(&hidden_states)?;

            // Focus on the last position
            let last_position = input_ids.len() - 1;
            let next_token_logits = logits.get(0)?.get(last_position)?;

            // Simple sampling - you could implement more complex strategies
            let next_token_id = sample_token(&next_token_logits, 0.7)?;
            let token_id = next_token_id.to_scalar::<u32>()?;

            // Add to generated tokens
            generated_tokens.push(token_id);

            // Check for end of generation
            if is_end_token(&self.tokenizer, token_id as usize) {
                break;
            }

            // Extend input for next iteration
            input_ids.push(token_id);
            attention_mask.push(1);
        }

        // Decode generated tokens
        let response = self
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| Error::msg(format!("Failed to decode generated tokens: {}", e)))?;

        Ok(response)
    }
}

// Helper function to sample a token given logits
fn sample_token(logits: &Tensor, temperature: f64) -> Result<Tensor> {
    // Apply temperature to control randomness
    let scaled_logits = if temperature != 1.0 {
        (logits / temperature)?
    } else {
        logits.clone()
    };

    // Convert to probabilities
    let probs = candle_nn::ops::softmax(&scaled_logits, 0)?;

    // For simplicity, we'll use greedy decoding
    // (take the highest probability token)
    let next_token = probs.argmax(0)?;

    Ok(next_token)
}

// Helper function to check if a token is an end token
fn is_end_token(tokenizer: &Tokenizer, token_id: usize) -> bool {
    // Check common end tokens for different tokenizers
    let common_end_tokens = ["</s>", "[SEP]", "<|endoftext|>"];

    for &token in &common_end_tokens {
        if let Some(id) = tokenizer.token_to_id(token) {
            if id as usize == token_id {
                return true;
            }
        }
    }

    // Check for period or newline (simplistic approach)
    if let Ok(text) = tokenizer.decode(&[token_id as u32], false) {
        if text == "." || text == "\n" {
            return true;
        }
    }

    false
}
