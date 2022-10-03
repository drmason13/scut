use clap::Args;
use either::Either;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{config::Config, fs::extract, side::Side};

use super::shared::find_latest_archive_file;

#[derive(Debug, Args)]
pub(crate) struct Download;

impl Download {
    // TODO: Guess which side a save file is for and warn and exit if the latest save is from your side.
    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        let archive =
            find_latest_archive_file(config.dropbox()).change_context(DownloadError::Read)?;

        let side: Either<Side, Result<Side, Report<DownloadError>>> = match (
            &config.side,
            Side::detect_side_in_string(&archive.to_string_lossy()),
        ) {
            (Side::Allies, Ok(side @ Side::Axis)) => Either::Left(side),
            (Side::Axis, Ok(side @ Side::Allies)) => Either::Left(side),
            (_, Err(e)) => Either::Right(Err(Report::new(e)
                .change_context(DownloadError::IndeterminateAction)
                .attach_printable("Could not determine which side the latest saved belongs to"))),
            (_, Ok(side)) => Either::Right(Ok(side)),
        };

        if let Either::Left(wrong_side) = side {
            println!("The latest save is an {} save, scut will stop since you're configured to be playing for the {}", wrong_side, wrong_side.other_side());
            return Ok(());
        }

        let side = side.unwrap_right()?;
        println!("Found an {} save", side);

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
    #[error("Could not work out what to do with save file")]
    IndeterminateAction,
}
