use clap::Parser;

use self::{download::Download, upload::Upload};

mod download;
mod shared;
mod upload;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) enum Command {
    Download(Download),
    Upload(Upload),
}
