use serde_json;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use crate::models::Message;

pub struct History {
    file_path: String,
}

impl History {
    pub fn new(file_path: &str) -> Self {
        History {
            file_path: file_path.to_string(),
        }
    }

    pub fn save_entry(&self, entry: &str) -> io::Result<()> {
        let path = Path::new(&self.file_path);
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        writeln!(file, "User: {}", entry)?;
        Ok(())
    }

    pub fn save_response(&self, response: &str) -> io::Result<()> {
        let path = Path::new(&self.file_path);
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        writeln!(file, "GPT: {}", response)?;
        Ok(())
    }

    pub async fn load_context(&self) -> io::Result<Vec<Message>> {
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let messages: Vec<Message> = serde_json::from_str(&content)?;
        Ok(messages)
    }

    pub async fn save_context(&self, context: &[Message]) -> io::Result<()> {
        let content = serde_json::to_string(context)?;
        tokio::fs::write(&self.file_path, content).await?;
        Ok(())
    }
}

