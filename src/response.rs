use bat::PrettyPrinter;
use std::io::Error;
use std::io::Write;
use std::pin::Pin;
use tokio_stream::StreamExt;

pub async fn process_response(
    stream: Pin<Box<dyn tokio_stream::Stream<Item = Result<String, Error>>>>,
    code_blocks: &mut Vec<String>,
) -> Result<String, Error> {
    // Change return type to Result<String, Error>
    let mut accumulate = false;
    let mut accumulator: Vec<String> = Vec::new();
    let mut end_delimiter_buffer = String::new();
    let mut full_response = String::new(); // Accumulate full response

    tokio::pin!(stream);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(content) => {
                let content_trimmed = content.trim();
                full_response.push_str(&content); // Accumulate full response

                if content_trimmed == "```" {
                    accumulate = !accumulate;
                    if !accumulate && !accumulator.is_empty() {
                        let mut code = accumulator.join("");
                        if let Some(pos) = code.find('\n') {
                            code = code[pos + 1..].to_string();
                        }
                        PrettyPrinter::new()
                            .input_from_bytes(code.as_bytes())
                            .language("rust")
                            .print()
                            .unwrap();
                        code_blocks.push(code.clone());
                        accumulator.clear();
                        println!(); // Ensure new line after code block
                    }
                } else if accumulate {
                    end_delimiter_buffer.push_str(content_trimmed);
                    if end_delimiter_buffer.ends_with("```") {
                        let mut code = accumulator.join("");
                        let trimmed: String = code.chars().filter(|&c| c != '`').collect();
                        if let Some(pos) = trimmed.find('\n') {
                            code = trimmed[pos + 1..].to_string();
                        }
                        PrettyPrinter::new()
                            .input_from_bytes(code.as_bytes())
                            .language("rust")
                            .print()
                            .unwrap();
                        code_blocks.push(code.clone());
                        accumulator.clear();
                        accumulate = false;
                        end_delimiter_buffer.clear();
                        println!(); // Ensure new line after code block
                    } else {
                        accumulator.push(content.clone());
                    }
                } else {
                    // Print regular content normally
                    print!("{}", content);
                    std::io::stdout().flush().unwrap(); // Make sure to flush the output
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
    Ok(full_response) // Return the full accumulated response
}
