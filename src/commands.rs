use bat::PrettyPrinter;
use clipboard::{ClipboardContext, ClipboardProvider};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::io::Write;
use std::process;

pub fn is_command(input: &str) -> bool {
    input.starts_with('/') && !input.strip_prefix('/').unwrap().contains(' ')
}

pub fn handle_command(cmd: &str, code_blocks: &[String]) {
    match cmd {
        "/exit" => process::exit(0),
        "/clear" => println!("\x1B[2J\x1B[1;1H"),
        "/paste" => {
            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
            let content = clipboard.get_contents().unwrap();
            //println!("\n{}", content);
            std::io::stdout().flush().unwrap();
        }
        "/copy" => {
            if code_blocks.is_empty() {
                println!("No code blocks to copy.");
                return;
            }

            let selections: Vec<&str> = code_blocks.iter().map(|s| s.as_str()).collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select code block to copy")
                .items(&selections)
                .default(0)
                .interact()
                .unwrap();

            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
            clipboard
                .set_contents(code_blocks[selection].clone())
                .unwrap();
            println!("Code block copied to clipboard");
        }
        "/copy_all" => {
            if code_blocks.is_empty() {
                println!("No code blocks to copy.");
                return;
            }

            let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
            let all_code = code_blocks.join("\n\n");
            clipboard.set_contents(all_code.clone()).unwrap();
            println!("All code blocks copied to clipboard");
        }
        _ => println!("Unknown command: {}", cmd),
    }
}
