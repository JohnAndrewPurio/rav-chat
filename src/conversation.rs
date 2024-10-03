use std::error::Error;

use reqwest::{multipart, Body, Client};
use serde::Serialize;
use serde_json::Value;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

const CONVERSATION_BASE_URL: &str = "https://conversations.twilio.com/v1";
const CONVERSATION_MEDIA_URL: &str = "https://mcs.us1.twilio.com/v1";

pub struct Conversation {
    client: Client,

    username: String,
    password: String,
}

impl Conversation {
    pub fn new(username: String, password: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            username,
            password,
        }
    }

    pub async fn create_conversation<T>(&self, params: &T) -> Result<Value, Box<dyn Error>>
    where
        T: Serialize + ?Sized,
    {
        let res = self
            .client
            .post(format!("{CONVERSATION_BASE_URL}/Conversations"))
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(&params)
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }

    pub async fn delete_conversation(&self, sid: String) -> Result<(), Box<dyn Error>> {
        self.client
            .delete(format!("{CONVERSATION_BASE_URL}/Conversations/{sid}"))
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn create_message<T>(
        &self,
        conversation_sid: String,
        params: &T,
    ) -> Result<Value, Box<dyn Error>>
    where
        T: Serialize + ?Sized,
    {
        let url = format!("{CONVERSATION_BASE_URL}/Conversations/{conversation_sid}/Messages");
        let res = self
            .client
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(&params)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }

    pub async fn delete_message(
        &self,
        conversation_sid: String,
        message_sid: String,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{CONVERSATION_BASE_URL}/Conversations/{conversation_sid}/Messages/{message_sid}"
        );

        self.client
            .delete(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn list_messages(&self, conversation_sid: String) -> Result<Value, Box<dyn Error>> {
        let url = format!("{CONVERSATION_BASE_URL}/Conversations/{conversation_sid}/Messages");
        let res = self
            .client
            .get(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }

    pub async fn upload_media(
        &self,
        service_sid: String,
        file_path: String,
        file_name: String,
    ) -> Result<Value, Box<dyn Error>> {
        let url = format!("{CONVERSATION_MEDIA_URL}/Services/{service_sid}/Media");
        let file = File::open(file_path).await?;

        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);

        let file_part = multipart::Part::stream(file_body).file_name(file_name);

        let form = multipart::Form::new().part("file", file_part);

        let res = self
            .client
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .multipart(form)
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }

    pub async fn retrieve_media(
        &self,
        service_sid: String,
        media_sid: String,
    ) -> Result<Value, Box<dyn Error>> {
        let url = format!("{CONVERSATION_MEDIA_URL}/Services/{service_sid}/Media/{media_sid}");
        let res = self
            .client
            .get(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await?;

        let result = res.text().await?;
        let result: Value = serde_json::from_str(&result)?;

        Ok(result)
    }
}
