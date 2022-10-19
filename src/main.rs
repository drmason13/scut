#![deny(missing_docs)]
//! # SCUT
//! ## Strategic Command Utility Tool
//!
//! ### What is this for?
//! This is a cli tool to facilitate sharing saves in 2v2 play by email games of Strategic Command (World at War).
//!
//! ### Installation (Windows only)
//! * Download the latest release from https://github.com/drmason13/scut/releases
//! * Extract scut.exe out of scut.zip
//! * Place scut.exe wherever you like, e.g. create the folder `C:\\Program Files\scut\` and place scut.exe inside it.
//! * You might want to add scut.exe's location to your path, otherwise use a full path to execute scut e.g. `'C:\\Program Files\scut\scut.exe' config show` instead of just `scut config show`
//!
//! ### Usage
//! Run `scut help` for a list of commands, run `scut help <subcommand>` to see help for a particular subcommand.
//!
//! ```plaintext
//! .\scut\scut.exe help
//! Usage: scut.exe [OPTIONS] <COMMAND>
//!
//! Commands:
//!   config
//!   download  Ready a turn to be played
//!   upload    Share a turn that you've finished playing
//!   help      Print this message or the help of the given subcommand(s)
//!
//! Options:
//!   -c, --config <CONFIG>  Load config from PATH instead of the default config path
//!   -h, --help             Print help information
//!   -V, --version          Print version information
//! ```
//!
//! #### `scut help config`
//!

use std::path::PathBuf;

use clap::{Parser, ValueHint};
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
    #[arg(short, long, value_name = "PATH", value_hint=ValueHint::FilePath)]
    pub(crate) config: Option<PathBuf>,
}

fn main() -> Result<(), Report<RuntimeError>> {
    let cli = Cli::parse();

    run(cli)
}

pub(crate) fn run(cli: Cli) -> Result<(), Report<RuntimeError>> {
    let mut config = Config::read_config_file(cli.config).change_context(RuntimeError)?;

    match cli.command {
        Command::Config(cmd) => cmd
            .run(config)
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
