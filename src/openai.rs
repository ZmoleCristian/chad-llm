// src/openai.rs
use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;

// Import the Message struct
use crate::models::Message;

type SharedContext = Arc<Mutex<Vec<Message>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: i64,
    pub temperature: f64,
    pub stream: bool,
}

#[derive(Deserialize)]
struct Chunk {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

pub async fn send_request(
    input: &str,
    context: SharedContext,
) -> Result<impl Stream<Item = Result<String, std::io::Error>>, std::io::Error> {
    let client = Client::new();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let url = "https://api.openai.com/v1/chat/completions";

    // Lock the context to access the stored messages and prepare the new message
    let messages = {
        let mut ctx = context.lock().await;
        ctx.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });
        ctx.clone()
    };

    let request_body = ChatRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: messages.clone(),
        max_tokens: 2048,
        temperature: 0.5,
        stream: true,
    };

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let (tx, rx) = mpsc::channel(100);
    let mut stream = response.bytes_stream();
    let context_clone = Arc::clone(&context);

    tokio::spawn(async move {
        let mut assistant_reply = String::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    let lines: Vec<&str> = chunk_str.split("\n").collect();
                    for line in lines {
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
                            if json_str != "[DONE]" {
                                if let Ok(chunk) = serde_json::from_str::<Chunk>(json_str) {
                                    for choice in chunk.choices {
                                        if let Some(content) = choice.delta.content {
                                            assistant_reply.push_str(&content);
                                            if tx.send(Ok(content.clone())).await.is_err() {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        )))
                        .await;
                    break;
                }
            }
        }

        // Update the shared context with the assistant's full reply
        if !assistant_reply.is_empty() {
            let mut ctx = context_clone.lock().await;
            ctx.push(Message {
                role: "assistant".to_string(),
                content: assistant_reply,
            });
        }
    });

    Ok(ReceiverStream::new(rx))
}
