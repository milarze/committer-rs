use std::{io::Write as _, str::FromStr as _};

use anyhow::{Error, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::{
    generation::LogitsProcessor,
    models::t5::{Config, T5ForConditionalGeneration},
};
use tokenizers::Tokenizer;

pub struct CodeT5Generator {
    model: T5ForConditionalGeneration,
    config: Config,
    tokenizer: Tokenizer,
    device: Device,
}

impl CodeT5Generator {
    pub fn new() -> Result<Self> {
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);

        let config: Config = serde_json::from_str(crate::generators::embedded::CONFIG_JSON)?;

        let tokenizer =
            Tokenizer::from_str(crate::generators::embedded::TOKENIZER_JSON).map_err(Error::msg)?;

        let vb = VarBuilder::from_buffered_safetensors(
            crate::generators::embedded::MODEL_SAFE_TENSORS.to_vec(),
            DType::F32,
            &device,
        )
        .map_err(Error::msg)?;

        let model = T5ForConditionalGeneration::load(vb, &config)?;

        Ok(Self {
            model,
            config,
            tokenizer,
            device,
        })
    }

    pub fn generate(&mut self, prompt: &str, max_length: usize) -> Result<String> {
        let tokenizer = self
            .tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(Error::msg)?;

        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(Error::msg)?
            .get_ids()
            .to_vec();

        let input_token_ids = Tensor::new(&tokens[..], &self.device)?.unsqueeze(0)?;

        let mut output_token_ids = [self
            .config
            .decoder_start_token_id
            .unwrap_or(self.config.pad_token_id) as u32]
        .to_vec();

        let mut logits_processor = LogitsProcessor::new(299792458, None, None);
        let encoder_output = self.model.encode(&input_token_ids)?;
        let start = std::time::Instant::now();

        let mut results = Vec::new();
        for index in 0.. {
            if output_token_ids.len() > max_length {
                break;
            }
            let decoder_token_ids = if index == 0 || !self.config.use_cache {
                Tensor::new(output_token_ids.as_slice(), &self.device)?.unsqueeze(0)?
            } else {
                let last_token = *output_token_ids.last().unwrap();
                Tensor::new(&[last_token], &self.device)?.unsqueeze(0)?
            };
            let logits = self
                .model
                .decode(&decoder_token_ids, &encoder_output)?
                .squeeze(0)?;
            let logits = {
                let start_at = output_token_ids.len().saturating_sub(64);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    1.9,
                    &output_token_ids[start_at..],
                )?
            };
            let next_token_id = logits_processor.sample(&logits)?;
            if next_token_id as usize == self.config.eos_token_id {
                break;
            }
            output_token_ids.push(next_token_id);
            if let Some(text) = tokenizer.id_to_token(next_token_id) {
                let text = text.replace('▁', " ").replace("<0x0A>", "\n");
                print!("{text}");
                results.push(text);
                std::io::stdout().flush()?;
            }
        }
        let dt = start.elapsed();
        println!(
            "\n{} tokens generated ({:.2} token/s)\n",
            output_token_ids.len(),
            output_token_ids.len() as f64 / dt.as_secs_f64(),
        );

        Ok(results.join(""))
    }
}
