use aws_sdk_dynamodb::{model::AttributeValue, Client};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Claims {
    pub major: Vec<String>,
    pub school: Vec<String>,
    pub affiliation: Vec<String>,
}

#[derive(Deserialize)]
pub struct UserData {
    pub discord_id: String,
    pub token_requested_at: u64,
    pub encrypted_eid: Vec<u8>,
    pub claims: Claims,
}

pub struct UserDB {
    client: Client,
    table_name: String,
}

impl UserDB {
    pub async fn new(table_name: &str) -> Self {
        let shared_config = aws_config::load_from_env().await;
        let client = Client::new(&shared_config);
        Self { client, table_name: table_name.to_string() }
    }

    // getting user data is a tad more complex since there's no deserialiazation lib that supports
    // aws-sdk just yet: https://github.com/zenlist/serde_dynamo/pull/18 (we need support for v0.4
    pub async fn user_exists(&self, discord_id: u64) -> bool {
        self.client
            .get_item()
            .table_name(self.table_name.as_str())
            .key("discord_id", AttributeValue::S(discord_id.to_string()))
            .send()
            .await
            .ok()
            .and_then(|o| o.item().map(|m| m.len()))
            .is_some()
    }
}
