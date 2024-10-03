use std::{collections::HashMap, error::Error, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Form, Json, Router,
};
use conversation::Conversation;
use email::{Email, MailData};
use reqwest::{header, StatusCode};
use serde_json::Value;
use sms::Sms;
use voice::{Voice, VoiceData};

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

    let shared_state = Arc::new(AppState {
        conversation,
        sms,
        mail,
        voice,
    });

    let app = Router::new()
        .route("/chat", get(initiate_conversation))
        .route("/chat/message", post(incoming_message))
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
        .route("/sms", post(create_sms_message))
        .route("/sms/receive", post(inbound_sms_message))
        .route("/email", post(send_email))
        .route("/voice/call", post(outgoing_call))
        .route("/voice", post(inbound_call))
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

async fn create_sms_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let sms = &state.sms;
    let result = sms
        .create_message(&payload)
        .await
        .expect("Failed to create sms");

    Ok(Json(result))
}

async fn send_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MailData>,
) -> Result<(), AppError> {
    let email = &state.mail;

    email
        .send_mail(payload)
        .await
        .expect("Failed to send email.");

    Ok(())
}

async fn outgoing_call(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VoiceData>,
) -> Result<(), AppError> {
    let voice = &state.voice;
    voice
        .outgoing_call(payload)
        .await
        .expect("Failed to start outgoing call.");

    Ok(())
}

async fn inbound_call(Form(payload): Form<HashMap<String, String>>) -> impl IntoResponse {
    println!("Received a call from: {payload:?}\n\n");

    let mut headers = HeaderMap::new();
    let response = r#"<?xml version="1.0" encoding="UTF-8"?><Response><Say>Receiving your call!</Say></Response>"#;

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/xml").expect("To parse text."),
    );

    (headers, response)
}

async fn incoming_message(Form(payload): Form<HashMap<String, String>>) {
    println!("Received a chat message from: {payload:?}\n\n");
}

async fn inbound_sms_message(Form(payload): Form<HashMap<String, String>>) {
    println!("Received an sms message from: {payload:?}\n\n");
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
