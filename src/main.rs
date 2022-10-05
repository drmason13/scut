use clap::Parser;
use error::RuntimeError;
use error_stack::{Report, ResultExt};

mod command;
mod config;
mod error;
mod save;
mod side;
#[cfg(test)]
mod test;
mod utils;

use command::Command;
use config::Config;

fn main() -> Result<(), Report<RuntimeError>> {
    let cmd = Command::parse();

    run(cmd)
}

pub(crate) fn run(command: Command) -> Result<(), Report<RuntimeError>> {
    let config = Config::read_config_file().change_context(RuntimeError)?;

    match command {
        Command::Config(cmd) => cmd
            .run()
            .change_context(RuntimeError)
            .attach_printable("Something went wrong using the config"),
        Command::Download(cmd) => cmd
            .run(&config)
            .change_context(RuntimeError)
            .attach_printable("Something went wrong downloading"),
        Command::Upload(cmd) => cmd
            .run(&config)
            .change_context(RuntimeError)
            .attach_printable("Something went wrong uploading"),
    }
}
