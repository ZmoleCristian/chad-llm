use bat::PrettyPrinter;

pub fn pretty_print(content: &str) {
    PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("rust")
        .print()
        .unwrap();
}

pub fn handle_output(content: &str) {
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for line in content.lines() {
        if line.trim() == "```" {
            if in_code_block {
                pretty_print(&code_block_content);
                code_block_content.clear();
            }
            in_code_block = !in_code_block;
        } else if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
        } else {
            println!("{}", line);
        }
    }
}
