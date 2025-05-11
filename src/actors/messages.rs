use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
pub enum MessageType {
    INPUT,
    OUTPUT,
    EMPTY,
}

#[derive(Debug, Serialize)]
pub struct PostBody {
    pub chat_id: i32,
    pub block_data: String,
}

#[derive(Debug, Serialize)]
pub struct StateActorMessage {
    pub message_type: MessageType,
    pub chat_id: Option<i32>,
    pub single_data: Option<String>,
    pub block_data: Option<Vec<String>>,
}

impl StateActorMessage {
    pub async fn send_to_server(&self) {
        let url = match env::var("SERVER_URL") {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Read env var error: {}", e);
                return;
            }
        };

        let joined = match self.block_data.clone() {
            Some(data) => data.join("\n"),
            None => {
                eprintln!("No block_data available");
                return;
            }
        };

        let body = PostBody {
            chat_id: match self.chat_id {
                Some(id) => id,
                None => {
                    eprintln!("chat_id is None");
                    return;
                }
            },
            block_data: joined,
        };

        let client = reqwest::Client::new();
        match client.post(url).json(&body).send().await {
            Ok(res) => println!("Response: {:?}", res),
            Err(e) => eprintln!("Request error: {}", e),
        }
    }
}
