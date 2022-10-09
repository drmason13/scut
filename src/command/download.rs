use std::path::PathBuf;

use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{config::Config, io_utils::extract, save::TurnSave, side::Side};

use super::shared::{self, find_turn_start_save, get_confirmation, wait_for_user_before_close};
use crate::save::SavOrArchive::Archive;

#[derive(Debug, Args)]
pub(crate) struct Download {
    /// Turn number to download.
    ///
    /// This will override the turn set in the config.
    ///
    /// If the command is successful, your config's turn will be **replaced**
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,
}

struct Downloader {
    start_save: Option<DownloadableSave>,
    team_save: Option<DownloadableSave>,
}

struct DownloadableSave {
    /// path to download from
    src: PathBuf,
    /// path to download to
    dst: PathBuf,
    /// the turn/save being downloaded
    save: TurnSave,
}

impl DownloadableSave {
    fn new(path: PathBuf, save: TurnSave, config: &Config) -> Result<Self, Report<DownloadError>> {
        let dst = config.saves.clone();

        Ok(DownloadableSave {
            src: path,
            dst,
            save,
        })
    }

    fn download(&self, config: &Config) -> Result<(), Report<DownloadError>> {
        println!(
            "Extracting {src} to {dst}",
            src = self.src.display(),
            dst = self.dst.display(),
        );

        extract(&config.seven_zip_path, &self.src, &self.dst)
            .into_report()
            .change_context(DownloadError::Extract)
            .attach_printable_lazy(|| {
                format!(
                    "while extracting {src} to {dst}",
                    src = &self.src.display(),
                    dst = &self.dst.display()
                )
            })?;

        Ok(())
    }
}

impl Downloader {
    fn download_saves(self, config: &Config) -> Result<(), Report<DownloadError>> {
        if let Some(save) = self.start_save {
            save.download(config)?;
        }

        if let Some(save) = self.team_save {
            save.download(config)?;
        }

        Ok(())
    }
}

impl Download {
    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        // TODO: download teammate's save if you haven't got it already?
        // They might have uploaded their previous turn after you uploaded yours, so you'll want to watch that before the other side's end of turn!

        let turn = if let Some(turn_override) = self.turn {
            turn_override
        } else {
            config.turn
        };

        let start_save = find_start_save(config, turn)?;
        let team_save = find_team_save(config, turn)?;

        let downloader = Downloader {
            start_save,
            team_save,
        };

        // on the first turn, Axis (who go first), don't need to download a turn start save
        // but they might need to download a teammate's save!
        let is_very_first_turn = turn == 1 && config.side == Side::first();

        match downloader {
            Downloader {
                start_save: Some(ref start_save),
                team_save: Some(ref team_save),
            } => {
                println!("Found turn start save: {}", start_save);
                println!("Found teammate's save: {}", team_save);
            }
            Downloader {
                start_save: Some(ref save),
                team_save: None,
            } => {
                println!("Found turn start save: {}", save);
                println!("Did not find any teammate's save");
            }
            Downloader {
                start_save: None,
                team_save: Some(ref save),
            } if is_very_first_turn => {
                println!("It's the very first turn, so there's no turn start save");
                println!("Found teammate's save: {}", save);
            }
            Downloader {
                start_save: None,
                team_save: Some(ref save),
            } => {
                println!("No save found for {} turn {}", &config.side, turn);
                println!("Found teammate's save: {}", save);
                println!("Maybe ask your teammate if they have a copy of the turn start save you can borrow?");
            }
            Downloader {
                start_save: None,
                team_save: None,
            } => {
                if is_very_first_turn {
                    println!("It's the very first turn, so there's no turn start save");
                    println!("Did not find any teammate's save");
                } else {
                    println!("No save found for {} turn {}", &config.side, turn);
                }
                wait_for_user_before_close("Nothing to do. Stopping.");
                return Ok(());
            }
        }

        if get_confirmation("Is that OK?")
            .into_report()
            .change_context(DownloadError::ConfirmationFailed)?
        {
            downloader.download_saves(config)?;

            wait_for_user_before_close("Done");
        } else {
            wait_for_user_before_close("User cancelled. Stopping.");
        }

        Ok(())
    }
}

fn find_start_save(
    config: &Config,
    turn: u32,
) -> Result<Option<DownloadableSave>, Report<DownloadError>> {
    let saves = find_turn_start_save(&config.dropbox, config.side, turn)
        .into_report()
        .change_context(DownloadError::Read)?;

    saves
        .map(|(save, path)| DownloadableSave::new(path, save, config))
        .transpose()
}

fn find_team_save(
    config: &Config,
    turn: u32,
) -> Result<Option<DownloadableSave>, Report<DownloadError>> {
    let saves = shared::find_team_save(&config.dropbox, config.side, &config.player, turn, Archive)
        .into_report()
        .change_context(DownloadError::Read)?;

    saves
        .map(|(save, path)| DownloadableSave::new(path, save, config))
        .transpose()
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum DownloadError {
    #[error("Could not read zipped save file")]
    Read,
    #[error("Could not extract save from zip file")]
    Extract,
    #[error("Could not get confirmation from user")]
    ConfirmationFailed,
}

impl std::fmt::Display for DownloadableSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.save)
    }
}
