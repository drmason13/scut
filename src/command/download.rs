use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{config::Config, fs::extract};

use super::shared::find_latest_archive_file;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct Download;

impl Download {
    // TODO: Guess which side a save file is for and warn and exit if the latest save is from your side.
    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        let archive =
            find_latest_archive_file(config.dropbox()).change_context(DownloadError::Read)?;

        let filename = archive
            .file_name()
            .ok_or_else(|| Report::new(DownloadError::Read))
            .attach_printable_lazy(|| {
                format!(
                    "path {} did not have a filename component",
                    archive.display()
                )
            })?;
        println!(
            "Extracting {} to {}",
            filename.to_string_lossy(),
            config.saves().display()
        );

        extract(config.seven_zip_path(), &archive, config.saves())
            .into_report()
            .change_context(DownloadError::Extract)
            .attach_printable_lazy(|| {
                format!(
                    "while extracting {} to {}",
                    &archive.display(),
                    config.dropbox().display()
                )
            })?;

        println!("Done");

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum DownloadError {
    #[error("Could not read zipped save file")]
    Read,
    #[error("Could not extract save from zip file")]
    Extract,
}
