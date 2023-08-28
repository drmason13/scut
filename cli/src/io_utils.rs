use std::{io, path::Path};

use anyhow::Context;
use thiserror::Error;

pub(crate) fn write_string_to_file(content: String, path: &Path) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Unable to create parent directory: {}", dir.display()))?;
    }

    std::fs::write(path, content)
        .with_context(|| format!("Unable to write to file: {}", path.display()))
}

pub(crate) fn read_input_from_user(prompt: &str) -> String {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("stdin should be available");
    input.trim().to_string()
}

pub(crate) fn get_confirmation(prompt: &str) -> bool {
    loop {
        let response = read_input_from_user(&format!("{prompt}: [Y] / N"));
        let response = response.trim();

        if response.is_empty() {
            // user pressed enter
            return true;
        }
        match response {
            "Y" | "y" => break true,
            "N" | "n" => break false,
            _ => {
                println!("Please confirm Y or N");
                continue;
            }
        }
    }
}

pub(crate) fn wait_for_user_before_close(text: &str) {
    println!("{text}");
    println!("<Press Enter to exit>");

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).ok(); // ignore the result
}

#[derive(Debug, Error)]
#[error("Error creating file")]
pub struct FileError;
