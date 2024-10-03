use std::error::Error;

use conversation::Conversation;

mod conversation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let params = [("FriendlyName", "Rust Room")];
    // let params = serde_json::json!({
    //     "FriendlyName": "Another Rust Room"
    // });

    let conversation = Conversation::new();

    // let new_conversation = conversation.create_conversation(&params).await?;

    // println!("Result: {new_conversation}");

    // let _ = conversation
    //     .delete_conversation("CHeb2ae7ba9c654c36a538f918151296f3".to_owned())
    //     .await?;

    // println!("Result: {result}");

    // let params = serde_json::json!({
    //     "Author": "some rando",
    //     "Body": "Another one"
    // });
    // let _message = conversation.create_message("CH7ab92711322841599af2dfd59f5e9631".to_owned(), &params).await?;

    // println!("Message: {message}");
    // let message_list = conversation.list_messages("CH7ab92711322841599af2dfd59f5e9631".to_owned()).await?;
    // println!("Messages: {message_list}");

    // conversation
    //     .delete_message(
    //         "CH7ab92711322841599af2dfd59f5e9631".to_owned(),
    //         "IMef8b7764ea4e4bcdb45c15d3d1438c9c".to_owned(),
    //     )
    //     .await?;
    let result = conversation
        .upload_media(
            "IS3ff2c2d08e3d42d7bce007de1e4e740c".to_owned(),
            "/home/wsl/projects/rav-chat/R.jpg".to_owned(),
            "cat.png".to_owned(),
        )
        .await?;

    println!("Result: {result}");

    Ok(())
}
