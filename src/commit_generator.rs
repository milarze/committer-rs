use tracing::instrument;

#[instrument(level = "info", skip(diff, context, config))]
pub async fn generate_commit_message(
    diff: String,
    context: Option<String>,
    config: crate::config::Config,
) -> Result<String, anyhow::Error> {
    let prompt = crate::prompt_generator::generate_prompt(diff.clone(), config.scopes(), context);
    return if config.use_local() {
        generate_from_local(prompt, config)
            .map_err(|e| anyhow::anyhow!("Error generating commit message: {:?}", e))
    } else {
        crate::generators::remote::generate_commit_message(prompt, config)
            .await
            .map_err(|e| anyhow::anyhow!("Error generating commit message: {:?}", e))
    };
}

fn generate_from_local(
    prompt: String,
    config: crate::config::Config,
) -> Result<String, anyhow::Error> {
    let generator = crate::generators::embedded::CommitGenerator::new()?;
    let result = generator.generate(&prompt, config.max_tokens())?;
    Ok(result)
}
