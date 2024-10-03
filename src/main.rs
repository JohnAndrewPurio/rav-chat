use std::{collections::HashMap, error::Error, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use conversation::Conversation;
use email::{Content, Email, Endpoint, MailData, Personalization};
use reqwest::StatusCode;
use serde_json::Value;
use sms::Sms;
use voice::Voice;

mod conversation;
mod email;
mod sms;
mod voice;

const TWILIO_ACCOUNT_SID: &str = "TWILIO_ACCOUNT_SID";
const TWILIO_AUTH_TOKEN: &str = "TWILIO_AUTH_TOKEN";

const SENDGRID_API_KEY: &str = "SENDGRID_API_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let twilio_account_sid =
        std::env::var(TWILIO_ACCOUNT_SID).expect("Missing TWILIO_ACCOUNT_SID env");
    let twilio_auth_token =
        std::env::var(TWILIO_AUTH_TOKEN).expect("Missing TWILIO_AUTH_TOKEN env");

    let send_grid_api_key = std::env::var(SENDGRID_API_KEY).expect("Missing SendGrid API key");

    let sms = Sms::new(twilio_account_sid.clone(), twilio_auth_token.clone());
    let conversation = Conversation::new(twilio_account_sid.clone(), twilio_auth_token.clone());
    let mail = Email::new(send_grid_api_key);
    let voice = Voice::new(twilio_account_sid, twilio_auth_token);

    // let params = serde_json::json!({
    //     "From": "+17402008913",
    //     "To": "+639280844918",
    //     "Body": "Hmmm!!!",
    // });
    // let result = sms.create_message(&params).await?;

    // println!("Result: {result}");

    // let personalization = Personalization {
    //     to: vec![Endpoint {
    //         email: "lyerdestroyer@gmail.com".to_owned(),
    //         name: None,
    //     }],
    //     cc: None,
    //     bcc: None,
    // };
    // let from = Endpoint {
    //     email: "purioandrew@gmail.com".to_owned(),
    //     name: Some("Andrew Purio".to_owned()),
    // };
    // let content = Content {
    //     type_: "text/plain".to_owned(),
    //     value: "This is a message from me Rust service, mate.".to_owned(),
    // };

    // let body = MailData {
    //     personalizations: vec![personalization],
    //     from,
    //     subject: "An email from my Rust service".to_owned(),
    //     content: vec![content],

    //     reply_to: None,
    //     attachments: None,
    // };

    // let result = mail.send_mail(body).await?;

    // println!("Result: {result}");

    // let params = serde_json::json!({
    //     "To": "+639280844918",
    //     "From": "+17402008913",
    //     "Twiml": "<Response><Say>Ahoy from Ireland</Say></Response>"
    // });
    // let result = voice.outgoing_call(&params).await?;

    // println!("Result: {result}");
    let shared_state = Arc::new(AppState {
        conversation,
        sms,
        mail,
        voice,
    });

    let app = Router::new()
        .route("/chat", get(initiate_conversation))
        .route("/chat/list/:conversation_sid", get(list_messages))
        .route(
            "/chat/message/delete/:conversation_sid/:message_sid",
            delete(delete_message),
        )
        .route("/chat/message/:conversation_sid", post(create_message))
        .route(
            "/chat/delete/:conversation_sid",
            delete(delete_conversation),
        )
        .route(
            "/media/retrieve/:service_sid/:media_sid",
            get(retrieve_media),
        )
        .route("/media/upload/:service_sid", post(upload_media))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

struct AppState {
    conversation: Conversation,
    sms: Sms,
    mail: Email,
    voice: Voice,
}

async fn initiate_conversation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let conversation = &state.conversation;
    let new_conversation = conversation
        .create_conversation(&payload)
        .await
        .expect("Error creating new conversation");

    Ok(axum::Json(new_conversation))
}

async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Path(conversation_sid): Path<String>,
) -> Result<(), AppError> {
    let conversation = &state.conversation;
    conversation
        .delete_conversation(conversation_sid.to_owned())
        .await
        .expect("Error deleting conversation");

    Ok(())
}

async fn create_message(
    State(state): State<Arc<AppState>>,
    Path(conversation_sid): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let conversation = &state.conversation;
    let message = conversation
        .create_message(conversation_sid.to_owned(), &payload)
        .await
        .expect("Failed to create message");

    Ok(axum::Json(message))
}

async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(conversation_sid): Path<String>,
) -> Result<Json<Value>, AppError> {
    let conversation = &state.conversation;
    let message_list = conversation
        .list_messages(conversation_sid.to_owned())
        .await
        .expect("Failed to list all messages.");

    Ok(axum::Json(message_list))
}

async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path((conversation_sid, message_sid)): Path<(String, String)>,
) -> Result<(), AppError> {
    let conversation = &state.conversation;

    conversation
        .delete_message(conversation_sid.to_owned(), message_sid.to_owned())
        .await
        .expect("Failed to delete message.");

    Ok(())
}

async fn upload_media(
    State(state): State<Arc<AppState>>,
    Path(service_sid): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let conversation = &state.conversation;

    let file_path = match params.get("file_path") {
        Some(p) => p,
        None => "",
    };
    let file_name = match params.get("file_name") {
        Some(n) => n,
        None => "",
    };

    let result = conversation
        .upload_media(
            service_sid.to_owned(),
            file_path.to_owned(),
            file_name.to_owned(),
        )
        .await
        .expect("Failed to upload media.");

    Ok(axum::Json(result))
}

async fn retrieve_media(
    State(state): State<Arc<AppState>>,
    Path((service_sid, media_sid)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let conversation = &state.conversation;
    let result = conversation
        .retrieve_media(service_sid.to_owned(), media_sid.to_owned())
        .await
        .expect("Failed to retrieve media.");

    Ok(axum::Json(result))
}

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
