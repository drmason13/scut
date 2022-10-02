use clap::Parser;
use error::RuntimeError;
use error_stack::{Report, ResultExt};

mod command;
mod config;
mod error;
mod fs;

use command::Command;

fn main() -> Result<(), Report<RuntimeError>> {
    let cmd = Command::parse();

    run(cmd)
}

pub(crate) fn run(command: Command) -> Result<(), Report<RuntimeError>> {
    match command {
        Command::Greet(cmd) => cmd
            .run()
            .change_context(RuntimeError)
            .attach_printable("Something went wrong greeting"),
        Command::Download(cmd) => cmd
            .run()
            .change_context(RuntimeError)
            .attach_printable("Something went wrong downloading"),
        Command::Upload(cmd) => cmd
            .run()
            .change_context(RuntimeError)
            .attach_printable("Something went wrong uploading"),
    }
}
