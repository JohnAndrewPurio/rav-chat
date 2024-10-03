use std::error::Error;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub async fn outgoing_call<T>(&self, params: &T) -> Result<Value, Box<dyn Error>>
    where
        T: Serialize + ?Sized,
    {
        let account_sid = self.username.clone();
        let url = format!("{VOICE_BASE_URL}/Accounts/{account_sid}/Calls.json");
        let res = self
            .client
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(&params)
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")] 
pub struct VoiceData {
    to: String,
    from: String,
    twiml: String,
}