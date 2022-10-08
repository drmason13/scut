use std::path::Path;

use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{
    command::shared::{get_confirmation, iter_turn_saves_in_dir},
    config::{Config, Key, Setting},
    io_utils,
    save::TurnSave,
};

use super::shared::iter_saves_in_dir;

#[derive(Debug, Args)]
pub(crate) struct Upload {
    /// Turn number to use when naming the save.
    ///
    /// This will override the turn set in the config.
    ///
    /// If the command is successful, your config's turn will be **replaced**
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,
}

impl Upload {
    pub(crate) fn run(self, config: &mut Config) -> Result<(), Report<UploadError>> {
        // find and upload your autosave

        if let Some(turn_override) = self.turn {
            config
                .set(Key::Turn, Setting::Turn(turn_override))
                .change_context(UploadError::UpdateConfig)
                .attach_printable_lazy(|| format!("to override the turn to {}", turn_override))?;
        }

        let next_start_save = TurnSave::from_config(config).next_turn();

        if let Some((_autosave, path)) = iter_saves_in_dir(&config.saves, "sav")
            .into_report()
            .change_context(UploadError::Read)?
            .filter_map(|result| result.ok())
            .find(|(save, _)| save.is_autosave())
        {
            if get_confirmation(&format!(
                "Found autosave file. This will be uploaded as '{}'.\nIs that OK?",
                &next_start_save
            ))
            .into_report()
            .change_context(UploadError::ConfirmationFailed)?
            {
                upload_save(&path, &next_start_save, config)?;
            } else {
                println!("User cancelled. Stopping.");
                return Ok(());
            }
        };

        // update turn in config
        config
            .set(Key::Turn, Setting::Turn(next_start_save.turn))
            .change_context(UploadError::UpdateConfig)
            .attach_printable("after successfully loading a save")?;

        // find and upload your "intermediate" save

        let mut available_saves = iter_turn_saves_in_dir(&config.saves, "sav")
            .into_report()
            .change_context(UploadError::Read)?;

        if let Some(Ok((save, path))) = available_saves.find(|save| match save {
            Err(_) => false,
            Ok((save, _)) => {
                save.turn == config.turn
                    && save.side == config.side
                    // find your save
                    && save.player.as_ref() == Some(&config.player)
            }
        }) {
            if get_confirmation(&format!(
                "Found your end of turn save file. This will be uploaded as '{}'.\nIs that OK?",
                &save
            ))
            .into_report()
            .change_context(UploadError::ConfirmationFailed)?
            {
                upload_save(&path, &save, config)?;
            } else {
                println!("User cancelled. Stopping.");
                return Ok(());
            }
        }

        println!("Done");

        Ok(())
    }
}

fn upload_save(path: &Path, save: &TurnSave, config: &Config) -> Result<(), Report<UploadError>> {
    let save_name = format!("{}.7z", save);
    let dst_path = config.dropbox.join(&save_name);

    let filename = path
        .file_name()
        .ok_or_else(|| Report::new(UploadError::Read))
        .attach_printable_lazy(|| {
            format!("path {} did not have a filename component", path.display())
        })?;

    println!(
        "Compressing {src} to {dst}",
        src = filename.to_string_lossy(),
        dst = dst_path.display()
    );

    io_utils::compress(&config.seven_zip_path, path, &dst_path)
        .into_report()
        .change_context(UploadError::Compress)
        .attach_printable_lazy(|| {
            format!(
                "while compressing {src} to {dst}",
                src = path.display(),
                dst = &dst_path.display()
            )
        })?;

    Ok(())
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum UploadError {
    #[error("Could not read save file")]
    Read,
    #[error("Could not compress save file")]
    Compress,
    #[error("Could not get confirmation from user")]
    ConfirmationFailed,
    #[error("There was a problem updating the config")]
    UpdateConfig,
}
