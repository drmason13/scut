use clap::Args;
use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct Download {
    /// Whether to simulate an error (for development purposes only!)
    error: bool,
}

impl Download {
    pub(crate) fn run(self) -> Result<(), Report<DownloadError>> {
        if self.error {
            return Err(Report::new(DownloadError::Connection));
        }
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum DownloadError {
    #[error("Could not reach file server")]
    Connection,
}
