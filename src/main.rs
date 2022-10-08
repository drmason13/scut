use std::path::PathBuf;

use clap::Parser;
use error::RuntimeError;
use error_stack::{Report, ResultExt};

mod command;
mod config;
mod error;
mod io_utils;
mod save;
mod side;
#[cfg(test)]
mod test;

use command::Command;
use config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    /// Load config from PATH instead of the default config path
    #[arg(short, long)]
    pub(crate) config: Option<PathBuf>,
}

fn main() -> Result<(), Report<RuntimeError>> {
    let cli = Cli::parse();

    run(cli)
}

pub(crate) fn run(cli: Cli) -> Result<(), Report<RuntimeError>> {
    let (mut config, config_path) =
        Config::read_config_file(cli.config).change_context(RuntimeError)?;

    match cli.command {
        Command::Config(cmd) => cmd
            .run(config, config_path)
            .change_context(RuntimeError)
            .attach_printable("Something went wrong using the config"),
        Command::Download(cmd) => cmd
            .run(&config)
            .change_context(RuntimeError)
            .attach_printable("Something went wrong downloading"),
        Command::Upload(cmd) => cmd
            .run(&mut config)
            .change_context(RuntimeError)
            .attach_printable("Something went wrong uploading"),
    }
}
