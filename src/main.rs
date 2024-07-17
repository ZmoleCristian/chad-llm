mod data;
mod openai;

use bat::PrettyPrinter;
use clipboard::{ClipboardContext, ClipboardProvider};
use data::MyCompletion;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use openai::send_request;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

fn is_command(input: &str) -> bool {
    input.starts_with('/') && !input.strip_prefix('/').unwrap().contains(' ')
}

fn command_handler(cmd: String) {
    match cmd.split('/').collect::<Vec<&str>>()[1] {
        "exit" => {
            std::process::exit(0);
        }
        "clear" => {
            println!("\x1B[2J\x1B[1;1H");
        }
        "paste" => {
            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
            let content = clipboard.get_contents().unwrap();
            PrettyPrinter::new()
                .input_from_bytes(content.as_bytes())
                .language("rust")
                .print()
                .unwrap();
        }
        _ => {
            println!("Unknown command: {}", cmd);
        }
    }
}

fn main() {
    //Mite-ma frumos ca vei reformata acest spaghetti main
    let mut history = BasicHistory::new().max_entries(99).no_duplicates(false);
    let rt = Runtime::new().unwrap();
    let completion = MyCompletion::default();
    loop {
        if let Ok(input) = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("TragDate")
            .completion_with(&completion)
            .history_with(&mut history)
            .interact_text()
        {
            if input.starts_with('/') && is_command(&input) {
                command_handler(input);
            } else {
                let mut accumulate = false;
                let mut accumulator: Vec<String> = Vec::new();
                let mut end_delimiter_buffer = String::new();
                let response_stream = rt.block_on(send_request(&input));
                match response_stream {
                    Ok(mut stream) => {
                        rt.block_on(async {
                            while let Some(chunk) = stream.next().await {
                                match chunk {
                                    Ok(content) => {
                                        if content.trim() == "```" {
                                            accumulate = !accumulate;
                                            if !accumulate {
                                                // End of code block, print accumulated code
                                                let code = accumulator.join("");
                                                PrettyPrinter::new()
                                                    .input_from_bytes(code.as_bytes())
                                                    .language("rust")
                                                    .print()
                                                    .unwrap();
                                                accumulator.clear();
                                                end_delimiter_buffer.clear();
                                                append_to_chat(&code);
                                            }
                                        } else if accumulate {
                                            end_delimiter_buffer.push_str(&content.trim());
                                            if end_delimiter_buffer.ends_with("```") {
                                                // End of code block, print accumulated code
                                                let code = accumulator.join("");
                                                let trimmed: String =
                                                    code.chars().filter(|&c| c != '`').collect();

                                                PrettyPrinter::new()
                                                    .input_from_bytes(trimmed.as_bytes())
                                                    .language("rust")
                                                    .print()
                                                    .unwrap();
                                                accumulator.clear();
                                                accumulate = false;
                                                end_delimiter_buffer.clear();
                                                append_to_chat(&trimmed);
                                            } else {
                                                accumulator.push(content.clone());
                                            }
                                        } else {
                                            let trimmed: String =
                                                content.chars().filter(|&c| c != '`').collect();
                                            print!("{}", trimmed);
                                            append_to_chat(&trimmed);
                                        }
                                    }
                                    Err(err) => eprintln!("Error: {}", err),
                                }
                            }
                        });
                    }
                    Err(err) => eprintln!("Error: {}", err),
                }
                println!();
            }
        }
    }
}

fn append_to_chat(content: &str) {
    let mut file = OpenOptions::new().append(true).open("chat.txt").unwrap();
    writeln!(file, "{}", content).unwrap();
}

//padding
//padding
//padding
//padding
//padding
//padding
//paddinggggg
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
//padding
