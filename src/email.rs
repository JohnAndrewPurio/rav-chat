use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

const EMAIL_BASE_URL: &str = "https://api.sendgrid.com/v3/mail";

pub struct Email {
    client: Client,

    api_key: String,
}

impl Email {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),

            api_key,
        }
    }

    pub async fn send_mail(&self, body: MailData) -> Result<Value, Box<dyn Error>> {
        let body = serde_json::json!(body);

        println!("Body: {body}");

        let url = format!("{EMAIL_BASE_URL}/send");
        let res = self
            .client
            .post(url)
            .json(&body)
            .bearer_auth(self.api_key.clone())
            .send()
            .await?;

        let result = res.text().await?;
        let result = serde_json::from_str(&result)?;

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailData {
    pub personalizations: Vec<Personalization>,
    pub from: Endpoint,
    pub subject: String,
    pub content: Vec<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Endpoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Personalization {
    pub to: Vec<Endpoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub email: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename(serialize = "type"))]
    pub type_: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    content: String,
    filename: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<Disposition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Disposition {
    #[serde(rename(serialize = "attachment"))]
    Attachment,

    #[serde(rename(serialize = "inline"))]
    Inline,
}
