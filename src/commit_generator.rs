pub async fn generate_commit(
    diff: String,
    context: Option<String>,
    config: crate::config::Config,
) -> Result<String, anyhow::Error> {
    let client = crate::clients::Claude::new(config.api_key());

    let request = anthropic::types::MessagesRequestBuilder::default()
        .model(config.model())
        .max_tokens(64000 as usize)
        .messages(vec![anthropic::types::Message {
            role: anthropic::types::Role::User,
            content: vec![anthropic::types::ContentBlock::Text {
                text: crate::prompt_generator::generate_prompt(diff, config.scopes(), context),
            }],
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
    println!("Completion: {}", content);
    Ok(content)
} 
