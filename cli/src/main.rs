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

use anyhow::Context;
use clap::{Parser, ValueHint};
use scut_core::{error::Report, interface::Terminal};

mod command;
pub mod config;
mod error;

use command::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    /// Load config from PATH instead of the default config path
    #[arg(short, long, value_name = "PATH", value_hint=ValueHint::FilePath)]
    pub(crate) config: Option<PathBuf>,
}

fn main() -> Result<(), Report> {
    let cli = Cli::parse();

    Ok(run(cli)?)
}

pub(crate) fn run(cli: Cli) -> anyhow::Result<()> {
    let (mut config, config_service) = config::ready_config(cli.config)?;
    let command_user_interaction = Box::new(Terminal::new());

    match cli.command {
        Command::Config(cmd) => cmd
            .run(config, config_service, command_user_interaction)
            .context("Something went wrong using the config"),
        Command::Download(cmd) => cmd
            .run(&mut config, config_service, command_user_interaction)
            .context("Something went wrong downloading"),
        Command::Upload(cmd) => cmd
            .run(&config, config_service, command_user_interaction)
            .context("Something went wrong uploading"),
    }
}
