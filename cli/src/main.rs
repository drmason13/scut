#![deny(missing_docs)]
//! # SCUT
//! ## Strategic Command Utility Tool
//!
//! ### What is this for?
//! This is a cli tool to facilitate sharing saves in 2v2 play by email games of Strategic Command (World at War).
//!
//! ### Installation (Windows only)
//! * Download the latest release from <https://github.com/drmason13/scut/releases>
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
use clap::{Parser, Subcommand, ValueHint};
use command::config::ConfigArgs;
use scut_core::{
    error::Report,
    interface::{prediction::simple_prediction::SimplePrediction, Terminal},
};
use tracing::{debug, info, instrument};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_rolling_file::{RollingConditionBase, RollingFileAppenderBase};
use tracing_subscriber::EnvFilter;

pub(crate) mod command;
mod config;
mod error;
mod storage;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) sub_cmd: Option<CliSubcommand>,

    /// Load config from PATH instead of the default config path
    #[arg(short, long, value_name = "PATH", value_hint=ValueHint::FilePath)]
    pub(crate) config: Option<PathBuf>,

    /// Override the turn number set in the config.
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliSubcommand {
    Config(ConfigArgs),
}

fn main() -> Result<(), Report> {
    let args = Cli::parse();

    let _guard = setup_tracing()?;

    debug!("starting scut");

    Ok(run(args)?).map(|_| info!("success"))
}

#[instrument(skip(args), level = "INFO")]
pub(crate) fn run(args: Cli) -> anyhow::Result<()> {
    info!(config_path = ?args.config.as_ref().map(|p| p.display()));

    let (mut config, config_service) = config::ready_config(args.config)?;
    let command_user_interaction = Box::new(Terminal::new());

    match args.sub_cmd {
        Some(CliSubcommand::Config(config_args)) => command::config::run(
            config_args,
            config,
            config_service,
            command_user_interaction,
        ),
        None => {
            let (local_storage, remote_storage) = storage::ready_storage(&config)?;
            let prediction = Box::<SimplePrediction>::default();

            command::run(
                args.turn,
                &mut config,
                local_storage,
                remote_storage,
                prediction,
                command_user_interaction,
            )
        }
    }
}

fn setup_tracing() -> anyhow::Result<WorkerGuard> {
    let log_file =
        RollingFileAppenderBase::new("./tracing-logs", RollingConditionBase::new().daily(), 7)
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| "failed to set up a rolling file appender for bug logging")?;

    let (appender, guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(appender)
        .with_env_filter(EnvFilter::from_default_env())
        .log_internal_errors(false)
        .with_target(false)
        .init();

    Ok(guard)
}
