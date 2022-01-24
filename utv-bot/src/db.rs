use std::collections::HashMap;

use aws_sdk_dynamodb::{model::AttributeValue, Client};
use serde::Deserialize;
use serenity::model::id::{GuildId, RoleId};

#[derive(Deserialize, Debug)]
pub struct Claims {
    pub major: Vec<String>,
    pub school: Vec<String>,
    pub affiliation: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub discord_id: String,
    pub claims: Claims,
}

pub struct DynamoDB {
    client: Client,
    users_table_name: String,
    guilds_table_name: String,
}

impl DynamoDB {
    pub async fn new(table_name: &str) -> Self {
        let shared_config = aws_config::load_from_env().await;
        let client = Client::new(&shared_config);
        Self {
            client,
            users_table_name: table_name.to_string(),
            guilds_table_name: "guilds".to_string(),
        }
    }

    /// Gets claims data under the condition that DB access succeeds and it exists
    pub async fn get_user(&self, discord_id: u64) -> Option<Claims> {
        self.client
            .get_item()
            .table_name(self.users_table_name.as_str())
            .key("discord_id", AttributeValue::S(discord_id.to_string()))
            .send()
            .await
            .ok()
            .map(|o| o.item().cloned())
            .flatten()
            .map(|m| m.clone())
            .map(|m| m.get("claims").cloned())
            .map(|v| match v {
                Some(AttributeValue::S(s)) => serde_json::from_str(s.as_str()).ok(),
                _ => {
                    eprintln!("Failed to Get Claims Data on Discord ID: {}", discord_id);
                    None
                }
            })
            .flatten()
    }

    /// Maps majors/affiliation to a role
    pub async fn get_role_config(&self, guild_id: GuildId) -> HashMap<String, u64> {
        self.client
            .get_item()
            .table_name(self.guilds_table_name.as_str())
            .key("guild_id", AttributeValue::S(guild_id.0.to_string()))
            .send()
            .await
            .ok()
            .map(|o| o.item().cloned())
            .flatten()
            .map(|m| m.clone())
            .map(|m| {
                let mut output: HashMap<String, u64> = HashMap::new();
                let keys = ["affiliation_roles", "school_roles", "major_roles"];
                for key in keys {
                    let temp: HashMap<String, u64> = match m.get(key) {
                        Some(AttributeValue::S(data)) => {
                            serde_json::from_str(data).unwrap_or(HashMap::new())
                        }
                        _ => {
                            eprintln!("{} does not exist in guild {}'s attributes", key, guild_id);
                            HashMap::new()
                        }
                    };
                    output.extend(temp);
                }
                output
            })
            .unwrap_or(HashMap::new())
    }
}

// Guild Data:
// guild_id (primary key): u64
// affiliation_roles: JSON {"student": 2322324243, "member": 4089904238094}
// major_roles: JSON {"Computer Science, Entry-Level": 32094209878097, "Computer Science":
// 348023984093}
// school_roles: JSON {"College of Natural Science": 340580932480}
