use std::error::Error;

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

const BASE_URL: &str = "https://api.twilio.com";

pub struct Sms {
    client: Client,

    username: String,
    password: String,
}

impl Sms {
    pub fn new(username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            username,
            password,
        }
    }

    pub async fn create_message<T>(&self, params: &T) -> Result<Value, Box<dyn Error>>
    where
        T: Serialize + ?Sized,
    {
        let account_sid = self.username.clone();
        let url = format!("{BASE_URL}/2010-04-01/Accounts/{account_sid}/Messages.json");
        let res = self
            .client
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(&params)
            .send()
            .await?;

        let result = res.text().await?;
        let result = serde_json::from_str(&result)?;

        Ok(result)
    }
}
