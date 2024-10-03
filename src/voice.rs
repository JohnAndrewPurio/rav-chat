use std::error::Error;

use reqwest::Client;
use serde::{Deserialize, Serialize};

const VOICE_BASE_URL: &str = "https://api.singapore.us1.twilio.com/2010-04-01";

pub struct Voice {
    client: Client,

    username: String,
    password: String,
}

impl Voice {
    pub fn new(username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            username,
            password,
        }
    }

    pub async fn outgoing_call(&self, params: VoiceData) -> Result<(), Box<dyn Error>> {
        let account_sid = self.username.clone();
        let url = format!("{VOICE_BASE_URL}/Accounts/{account_sid}/Calls.json");
        self.client
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(&params)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VoiceData {
    to: String,
    from: String,
    twiml: String,
}
