mod commands;
mod data;
mod openai;
mod response;

use clipboard::{ClipboardContext, ClipboardProvider};
use commands::{handle_command, is_command};
use data::MyCompletion;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use openai::send_request;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

type SharedContext = Arc<Mutex<Vec<serde_json::Value>>>;

fn main() {
    let context = Arc::new(Mutex::new(Vec::new()));
    let mut history = BasicHistory::new().max_entries(99).no_duplicates(false);
    let rt = Runtime::new().unwrap();
    let completion = MyCompletion::default();

    loop {
        let mut code_blocks: Vec<String> = Vec::new(); // Reset code_blocks for each interaction

        let mut input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("N1ptic") // Add newline before prompt
            .completion_with(&completion)
            .history_with(&mut history)
            .interact_text()
            .unwrap();

        // Check if the input is a command
        if is_command(&input) {
            handle_command(&input, &code_blocks);
            if input.trim() == "/paste" {
                // If the command was /paste, append the clipboard content to the input
                let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                let paste_content = clipboard.get_contents().unwrap();
                print!("\n{}", paste_content); // Print the clipboard content
                std::io::stdout().flush().unwrap();

                let additional_input = Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Add additional details")
                    .interact_text()
                    .unwrap();

                input.push_str(&paste_content);
                input.push_str(&additional_input);
            }
        }

        if !is_command(&input) {
            let response_stream = rt.block_on(send_request(&input, Arc::clone(&context)));
            match response_stream {
                Ok(stream) => {
                    rt.block_on(response::process_response(
                        Box::pin(stream),
                        &mut code_blocks,
                    ))
                    .unwrap();
                }
                Err(err) => eprintln!("Request failed: {}", err),
            }

            if !code_blocks.is_empty() {
                if let Ok(command_input) = Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter command")
                    .interact_text()
                {
                    handle_command(&command_input, &code_blocks);
                }
            }
        }
        println!(); // Ensure new line after each interaction
        std::io::stdout().flush().unwrap(); // Ensure stdout is flushed after each interaction
    }
}
