use std::error::Error;

const TWILIO_ACCOUNT_SID: &str = "TWILIO_ACCOUNT_SID";
const TWILIO_AUTH_TOKEN: &str = "TWILIO_AUTH_TOKEN";

const CONVERSATION_BASE_URL: &str = "https://conversations.twilio.com/v1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let twilio_account_sid =
        std::env::var(TWILIO_ACCOUNT_SID).expect("Missing TWILIO_ACCOUNT_SID env");
    let twilio_auth_token =
        std::env::var(TWILIO_AUTH_TOKEN).expect("Missing TWILIO_AUTH_TOKEN env");

    let client = reqwest::Client::new();
    let params = [("FriendlyName", "Rust Room")];

    let res = client
        .post(format!("{CONVERSATION_BASE_URL}/Conversations"))
        .basic_auth(twilio_account_sid, Some(twilio_auth_token))
        .form(&params)
        .send()
        .await?;

    let result = res.text().await?;

    println!("Result: {result}");

    Ok(())
}
