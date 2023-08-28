use std::io;

use super::UserInteraction;

/// User Interaction implemented by printing and reading input from the terminal.
///
/// Suitable for CLI applications.
pub struct Terminal {
    buffer: String,
}

impl Terminal {
    /// Creates a new [`Terminal`].
    pub fn new() -> Self {
        Terminal {
            buffer: String::new(),
        }
    }

    fn readline(&mut self) -> &str {
        self.buffer.clear();
        io::stdin()
            .read_line(&mut self.buffer)
            .expect("stdin should be available");

        self.buffer.trim_end()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl UserInteraction for Terminal {
    fn message(&mut self, message: &str) {
        println!("{message}");
    }

    fn confirm(&mut self, message: &str, default: Option<bool>) -> bool {
        match default {
            Some(true) => println!("{message}: [Y] / N"),
            Some(false) => println!("{message}: Y / [N]"),
            None => println!("{message}: Y / N"),
        }

        loop {
            let input = self.readline();

            if matches!(input, "Y" | "y" | "yes" | "Yes" | "YES") {
                break true;
            };
            if matches!(input, "N" | "n" | "no" | "No" | "NO") {
                break false;
            };
            if input.is_empty() {
                if let Some(default) = default {
                    break default;
                } else {
                    self.message("Please confirm using Y / N");
                    continue;
                }
            } else {
                self.message("Please confirm using Y / N");
                continue;
            };
        }
    }

    fn query(&mut self, message: &str) -> String {
        self.message(message);
        self.readline().to_string()
    }

    fn wait_for_user_before_close(&mut self, message: &str) {
        self.message(message);
        self.message("<Press Enter to exit>");
        self.readline();
    }
}
