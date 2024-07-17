use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::io;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

#[derive(Deserialize, Debug)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize, Debug)]
struct Delta {
    content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Chunk {
    choices: Vec<Choice>,
}

pub async fn send_request(
    input: &str,
) -> Result<impl Stream<Item = Result<String, io::Error>>, io::Error> {
    let client = Client::new();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let url = "https://api.openai.com/v1/chat/completions";

    let request_body = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "max_tokens": 600,
        "temperature": 0.5,
        "stream": true,
        "messages": [
            {"role": "system", "content": "You help me"},
            {"role": "user", "content": input}
        ]
    });

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let (tx, rx) = mpsc::channel(100);
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    tokio::spawn(async move {
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8(chunk.to_vec()).unwrap_or_default();
                    buffer.push_str(&chunk_str);

                    while let Some(pos) = buffer.find("\ndata: ") {
                        let json_str = buffer[..pos].trim_start_matches("data: ").to_string();
                        buffer = buffer[pos + 7..].to_string();

                        if json_str.trim() == "[DONE]" {
                            break;
                        }

                        match serde_json::from_str::<Chunk>(&json_str) {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        if tx.send(Ok(content)).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))
                                    .await;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))
                        .await;
                    break;
                }
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}
