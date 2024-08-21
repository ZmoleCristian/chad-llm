mod commands;
mod data;
mod history;
mod models;
mod openai;
mod response;

use clipboard::{ClipboardContext, ClipboardProvider};
use commands::{handle_command, is_command};
use data::MyCompletion;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use history::History;
use models::Message;
use openai::send_request;
use std::io::Write;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use whoami;

fn print_nicely_formatted_history(history: &[Message]) {
    for message in history {
        match message.role.as_str() {
            "user" => println!("👤 User: {}", message.content),
            "assistant" => println!("🤖 Assistant: {}", message.content),
            _ => println!("Unknown role: {}", message.content),
        }
        println!(); // Add a blank line between each message for better readability
    }
}

fn main() {
    let rt = Runtime::new().unwrap();
    let history_file = "session_history.json";
    let session_history = History::new(history_file);

    // Load previous history entries
    let context = Arc::new(Mutex::new(rt.block_on(async {
        session_history
            .load_context()
            .await
            .unwrap_or_else(|_| Vec::new())
    })));

    let mut history = BasicHistory::new().max_entries(99).no_duplicates(false);
    let completion = MyCompletion::default();

    match rt.block_on(async { session_history.load_context().await }) {
        Ok(entries) => {
            println!("Previous conversation:");
            print_nicely_formatted_history(&entries);
        }
        Err(e) => eprintln!("Failed to load history: {}", e),
    }

    loop {
        let mut code_blocks: Vec<String> = Vec::new();
        let username = whoami::username();
        let mut input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{}", username))
            .completion_with(&completion)
            .history_with(&mut history)
            .interact_text()
            .unwrap();

        // Save the input to history
        if let Err(e) = session_history.save_entry(&input) {
            eprintln!("Failed to save entry: {}", e);
        }

        // Handle commands
        if is_command(&input) {
            if input.trim() == "/paste" {
                let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                match clipboard.get_contents() {
                    Ok(paste_content) => {
                        print!("\n{}", paste_content);
                        std::io::stdout().flush().unwrap();

                        let additional_input =
                            Input::<String>::with_theme(&ColorfulTheme::default())
                                .with_prompt("Add additional details")
                                .interact_text()
                                .unwrap();

                        input.push_str(&paste_content);
                        input.push_str(&additional_input);
                    }
                    Err(err) => eprintln!("Failed to read clipboard: {}", err),
                }
            } else {
                handle_command(&input, &code_blocks, history_file);
                continue; // Skip the rest of the loop iteration
            }
        }

        // Send request and handle response
        let response_stream = rt.block_on(send_request(&input, Arc::clone(&context)));
        match response_stream {
            Ok(stream) => {
                let response = rt.block_on(response::process_response(
                    Box::pin(stream),
                    &mut code_blocks,
                ));

                match response {
                    Ok(resp) => {
                        // Save the GPT response to history
                        if let Err(e) = session_history.save_response(&resp) {
                            eprintln!("Failed to save response: {}", e);
                        }

                        // Save the updated context
                        if let Err(e) = rt.block_on(async {
                            session_history.save_context(&*context.lock().await).await
                        }) {
                            eprintln!("Failed to save context: {}", e);
                        }
                    }
                    Err(err) => eprintln!("Failed to process response: {}", err),
                }
            }
            Err(err) => eprintln!("Request failed: {}", err),
        }

        // Handle code blocks if any
        if !code_blocks.is_empty() {
            if let Ok(command_input) = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter command")
                .interact_text()
            {
                handle_command(&command_input, &code_blocks, history_file);
            }
        }
        println!();
        std::io::stdout().flush().unwrap(); // Ensure stdout is flushed after each interaction
    }
}

