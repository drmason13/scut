use std::path::PathBuf;

use clap::Args;
use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Args)]
pub(crate) struct Upload {
    /// File to upload
    #[arg(short, long)]
    pub(crate) file: PathBuf,
}

impl Upload {
    pub(crate) fn run(self) -> Result<(), Report<UploadError>> {
        let content =
            std::fs::read_to_string(self.file).map_err(|_e| Report::new(UploadError::Read))?;
        println!("content to be uploaded:\n{}", content);
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum UploadError {
    #[error("Could not read file")]
    Read,
}
