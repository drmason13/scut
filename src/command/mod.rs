use clap::Subcommand;

use self::{config::ConfigCmd, download::DownloadCmd, upload::UploadCmd};

mod config;
mod download;
mod shared;
mod upload;

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    #[command(subcommand)]
    /// Read or modify the current configuration file
    ///
    /// The configuration file is used to decide what to name your saves when uploading and which saves to download
    Config(ConfigCmd),
    /// Ready a turn to be played
    ///
    /// Unzips the latest save file from your dropbox folder that isn't yours into your game saves folder
    Download(DownloadCmd),
    /// Share a turn that you've finished playing
    ///
    /// Zips your latest save from your game saves folder into your dropbox folder and names it accordingly
    Upload(UploadCmd),
}
