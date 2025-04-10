use tracing::instrument;

#[instrument(level = "info", skip(diff, context, config))]
pub async fn generate_commit_message(
    diff: String,
    context: Option<String>,
    config: crate::config::Config,
) -> Result<String, anyhow::Error> {
    let client = crate::clients::Claude::new(config.api_key());

    let prompt = crate::prompt_generator::generate_prompt(diff.clone(), config.scopes(), context);
    let request = anthropic::types::MessagesRequestBuilder::default()
        .model(config.model())
        .max_tokens(64000_usize)
        .messages(vec![anthropic::types::Message {
            role: anthropic::types::Role::User,
            content: vec![anthropic::types::ContentBlock::Text { text: prompt }],
        }])
        .stream(false)
        .stop_sequences(vec!["\nHuman: ".to_string()])
        .build()
        .unwrap();

    let response = client.client.messages(request).await?;
    let content = response
        .content
        .iter()
        .map(|c| match c {
            anthropic::types::ContentBlock::Text { text } => text.clone(),
            _ => "".to_string(),
        })
        .collect();
    Ok(content)
}
