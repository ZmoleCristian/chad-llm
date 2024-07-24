use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;

type SharedContext = Arc<Mutex<Vec<serde_json::Value>>>;

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

    let mut messages = vec![json!({"role": "system", "content": "You help me"})];
    {
        let ctx = context.lock().await;
        messages.append(&mut ctx.clone());
    }
    messages.push(json!({"role": "user", "content": input}));

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "max_tokens": 2048,
        "temperature": 0.5,
        "stream": true,
        "messages": messages
    });

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
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8(chunk.to_vec()).unwrap_or_default();
                    let lines: Vec<&str> = chunk_str.split("\n").collect();
                    for line in lines {
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
                            if let Ok(chunk) = serde_json::from_str::<Chunk>(json_str) {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        if tx.send(Ok(content.clone())).await.is_err() {
                                            return;
                                        }
                                        let mut ctx = context_clone.lock().await;
                                        ctx.push(json!({"role": "assistant", "content": content}));
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
    });

    Ok(ReceiverStream::new(rx))
}
