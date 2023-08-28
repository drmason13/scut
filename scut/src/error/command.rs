use std::{error::Error, fmt, process::Output};

/// Use this error type for constructing errors from the output of spawned processes
#[derive(fmt::Debug)]
pub struct CommandError {
    stderr: String,
}

pub fn output_error(output: &Output) -> Result<(), CommandError> {
    if output.status.success() {
        Ok(())
    } else {
        Err(CommandError {
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.stderr.fmt(f)
    }
}

impl Error for CommandError {}
