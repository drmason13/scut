use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{config::Config, fs};

use super::shared::find_latest_save_file;

#[derive(Debug, Args)]
pub(crate) struct Upload {
    /// Turn number to use when naming the save
    #[arg(short, long)]
    pub(crate) turn: String,
}

impl Upload {
    pub(crate) fn run(self, config: &Config) -> Result<(), Report<UploadError>> {
        // zip latest save in the "saves" folder to the "dropbox" folder

        // find latest save
        let latest_save =
            find_latest_save_file(config.saves()).change_context(UploadError::Read)?;

        let filename = latest_save
            .file_name()
            .ok_or_else(|| Report::new(UploadError::Read))
            .attach_printable_lazy(|| {
                format!(
                    "path {} did not have a filename component",
                    latest_save.display()
                )
            })?;

        println!(
            "Compressing {} to {}",
            filename.to_string_lossy(),
            config.dropbox().display()
        );

        fs::compress(
            &config.seven_zip_path,
            &latest_save,
            format!(
                "{dir}{side} {turn}",
                dir = config.dropbox().display(),
                side = &config.side,
                turn = self.turn
            )
            .as_ref(),
        )
        .into_report()
        .change_context(UploadError::Compress)
        .attach_printable_lazy(|| {
            format!(
                "while compressing {} to {}",
                &latest_save.display(),
                config.dropbox().display()
            )
        })?;

        println!("Done");

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum UploadError {
    #[error("Could not read save file")]
    Read,
    #[error("Could not compress save file")]
    Compress,
}
