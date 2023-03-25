use std::path::PathBuf;

use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{
    config::{Config, Key, Setting},
    io_utils::{compress, get_confirmation, wait_for_user_before_close},
    save::{SavOrArchive::*, TurnSave},
};

use super::shared::{check_for_team_save, find_autosave, find_save};

/// Contains the arguments of the upload command.
///
/// [`Upload::run`] will run the upload command.
///
/// See [`crate::command::Command`] for all commands.
#[derive(Debug, Args)]
pub(crate) struct UploadCmd {
    /// Turn number to use when naming the save.
    ///
    /// This will override the turn set in the config.
    ///
    /// If the command is successful, your config's turn will be **replaced**
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,

    /// Force uploading your autosave regardless of whether your teammate's save has been uploaded
    #[arg(short, long)]
    pub(self) force: bool,
}

/// Private helper struct for tracking what to upload, and uploading it
struct Uploader {
    /// Your save from before ending turn, for your teammate.
    ///
    /// This is required: if there isn't an intermediate save then we shouldn't upload anything and issue a warning.
    /// The user probably forgot to make this save.
    your_save: UploadableSave,

    /// This is the Save for the start of the next team's turn, and the path of your autosave file from the end of your turn.
    ///
    /// This should only be uploaded if both team members have uploaded their saves.
    /// We'll check for this by looking for the downloaded teammate's save for this turn in the saves folder.
    next_save: Option<UploadableSave>,
}

/// Private helper struct for storing the information needed to upload a save
struct UploadableSave {
    /// path to upload from
    src: PathBuf,
    /// path to upload to
    dst: PathBuf,
    /// the turn/save being uploaded
    save: TurnSave,
}

impl UploadableSave {
    fn new(path: PathBuf, save: TurnSave, config: &Config) -> Result<Self, Report<UploadError>> {
        let dst = config.dropbox.join(format!("{save}.7z"));

        Ok(UploadableSave {
            src: path,
            dst,
            save,
        })
    }

    fn upload(&self, config: &Config) -> Result<(), Report<UploadError>> {
        println!(
            "Compressing {src} to {dst}",
            src = self.src.display(),
            dst = self.dst.display(),
        );

        let cmd_output = compress(&config.seven_zip_path, &self.src, &self.dst)
            .into_report()
            .change_context(UploadError::Compress)
            .attach_printable_lazy(|| {
                format!(
                    "while compressing {src} to {dst}",
                    src = &self.src.display(),
                    dst = &self.dst.display()
                )
            })?;

        if cmd_output.status.success() {
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&cmd_output.stderr);
            let stdout = String::from_utf8_lossy(&cmd_output.stdout);
            Err(Report::new(UploadError::Compress))
                .attach_printable(format!("Error running 7zip: {error}\n{stdout}"))
        }
    }
}

impl Uploader {
    fn upload_saves(self, config: &Config) -> Result<(), Report<UploadError>> {
        self.your_save.upload(config)?;

        if let Some(uploadable_save) = self.next_save {
            uploadable_save.upload(config)?;
        }

        Ok(())
    }
}

impl UploadCmd {
    pub(crate) fn run(self, config: &mut Config) -> Result<(), Report<UploadError>> {
        // TODO: Check that teammate save is unzipped in saves folder
        // if it isn't, then the assumption is that you are playing the turn first and shouldn't upload a next_turn_start save yet!

        let turn = if let Some(turn_override) = self.turn {
            turn_override
        } else {
            config.turn
        };

        let your_save = find_your_save(config, turn)?;

        let mut uploader = if let Some(save) = your_save {
            println!(
                "Found your save for this turn. This will be uploaded as '{}'",
                &save
            );
            Uploader {
                your_save: save,
                next_save: None,
            }
        } else {
            println!("Did not find your save for this turn.");
            println!("Create a save before clicking end turn so your teammate can see what you did during your turn.");
            wait_for_user_before_close("Save missing. Nothing has been uploaded. Stopping.");
            return Ok(());
        };

        if check_for_team_save(config, turn)
            .into_report()
            .change_context(UploadError::Read)?
            || self.force
        {
            uploader.next_save = find_next_save(config, turn)?;
            if let Some(ref save) = uploader.next_save {
                if self.force {
                    println!("Did not find a save from your teammate for this turn.");
                    println!("[forced] Your autosave will be uploaded as '{save}'");
                } else {
                    println!("Your autosave will be uploaded as '{save}'");
                }
            }
        } else {
            println!("Did not find a save from your teammate for this turn. Your autosave will not be uploaded.");
        }

        if get_confirmation("Is that OK?")
            .into_report()
            .change_context(UploadError::ConfirmationFailed)?
        {
            uploader.upload_saves(config)?;
            // increment turn in config to the next turn ready for the next download.
            config
                .set(Key::Turn, Setting::Turn(turn + 1))
                .change_context(UploadError::UpdateConfig)
                .attach_printable("after successfully uploading a save")?;

            wait_for_user_before_close("Done");
        } else {
            wait_for_user_before_close("User cancelled. Stopping.");
        }

        Ok(())
    }
}

fn find_your_save(
    config: &Config,
    turn: u32,
) -> Result<Option<UploadableSave>, Report<UploadError>> {
    let saves = find_save(&config.saves, config.side, &config.player, turn, Sav)
        .into_report()
        .change_context(UploadError::Read)?;

    saves
        .map(|(save, path)| UploadableSave::new(path, save, config))
        .transpose()
}

fn find_next_save(
    config: &Config,
    turn: u32,
) -> Result<Option<UploadableSave>, Report<UploadError>> {
    let next_start_save = {
        let mut config_save = TurnSave::from_config(config);
        config_save.turn = turn;
        config_save.next_turn()
    };

    let saves = find_autosave(&config.saves)
        .into_report()
        .change_context(UploadError::Read)?;

    saves
        .map(|(_save, path)| UploadableSave::new(path, next_start_save, config))
        .transpose()
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

impl std::fmt::Display for UploadableSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.save)
    }
}
