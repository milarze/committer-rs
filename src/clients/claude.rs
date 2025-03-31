pub struct Claude {
    pub client: anthropic::client::Client,
}

impl Claude {
    pub fn new(api_key: String) -> Claude {
        Claude {
            client: anthropic::client::ClientBuilder::default()
                .api_key(api_key)
                .build()
                .unwrap(),
        }
    }
}
