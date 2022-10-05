use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{config::Config, save::Save, utils::extract};

use super::shared::find_latest_archive_file;

#[derive(Debug, Args)]
pub(crate) struct Download;

impl Download {
    // TODO: download the latest save from your Side or the latest save from the other side according to the turn numbers of both

    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        let archive =
            find_latest_archive_file(&config.dropbox).change_context(DownloadError::Read)?;

        let save: Save = archive
            .as_path()
            .try_into()
            .into_report()
            .attach_printable("The latest save file was named in an unusual way. scut does not know how to proceed. Stopping.")
            .attach_printable(format!("save name: {}", archive.display()))
            .change_context(DownloadError::IndeterminateAction)?;

        if let Save::Autosave = save {
            return Err(Report::new(DownloadError::AutosaveArchived)
                .attach_printable("Found a zipped up autosave, the save should be renamed so we know what turn it is"));
        }

        if let Save::Turn(save) = save {
            if save.side == config.side
                && matches!(save.player, Some(ref player) if player == &config.player)
            {
                println!("Latest save ({}) is belongs to you, nothing to do.", save);
                return Ok(());
            }
        }

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
            config.saves.display()
        );

        extract(&config.seven_zip_path, &archive, &config.saves)
            .into_report()
            .change_context(DownloadError::Extract)
            .attach_printable_lazy(|| {
                format!(
                    "while extracting {} to {}",
                    &archive.display(),
                    config.dropbox.display()
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
    #[error("Unexpected 'autosave' zip file")]
    AutosaveArchived,
}
