use clap::Parser;

use self::{config::ConfigCmd, download::Download, upload::Upload};

mod config;
mod download;
mod shared;
mod upload;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) enum Command {
    #[command(subcommand)]
    Config(ConfigCmd),
    Download(Download),
    Upload(Upload),
}
