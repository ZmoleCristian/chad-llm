use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

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
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "User: {}", entry)?;
        Ok(())
    }

    pub fn save_response(&self, response: &str) -> io::Result<()> {
        let path = Path::new(&self.file_path);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "GPT: {}", response)?;
        Ok(())
    }

    pub fn load_history(&self) -> io::Result<Vec<String>> {
        let content = std::fs::read_to_string(&self.file_path)?;
        Ok(content.lines().map(String::from).collect())
    }
}
