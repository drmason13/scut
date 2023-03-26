use std::path::PathBuf;

use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{
    config::Config,
    io_utils::{extract, get_confirmation, wait_for_user_before_close},
    save::{SavOrArchive::Archive, TurnSave},
    side::Side,
};

use super::shared::{self, find_turn_start_save};

#[derive(Debug, Args)]
pub(crate) struct DownloadCmd {
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
    team_saves: Vec<DownloadableSave>,
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

        let cmd_output = extract(&config.seven_zip_path, &self.src, &self.dst)
            .into_report()
            .change_context(DownloadError::Extract)
            .attach_printable_lazy(|| {
                format!(
                    "while extracting {src} to {dst}",
                    src = &self.src.display(),
                    dst = &self.dst.display()
                )
            })?;

        if cmd_output.status.success() {
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&cmd_output.stderr);
            let stdout = String::from_utf8_lossy(&cmd_output.stdout);
            Err(Report::new(DownloadError::Extract))
                .attach_printable(format!("Error running 7zip: {error}\n{stdout}"))
        }
    }
}

impl Downloader {
    fn download_saves(self, config: &Config) -> Result<(), Report<DownloadError>> {
        if let Some(save) = self.start_save {
            save.download(config)?;
        }

        for save in self.team_saves {
            save.download(config)?;
        }

        Ok(())
    }
}

impl DownloadCmd {
    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        let turn = if let Some(turn_override) = self.turn {
            turn_override
        } else {
            config.turn
        };

        let start_save = find_start_save(config, turn)?;
        let team_saves = find_team_saves(config, turn)?;
        let count_of_team_saves = team_saves.len();

        let downloader = Downloader {
            start_save,
            team_saves,
        };

        // on the first turn, Axis (who go first), don't need to download a turn start save
        // but they might need to download a teammate's save!
        let is_very_first_turn = turn == 1 && config.side == Side::first();

        match &downloader {
            Downloader {
                start_save: Some(ref start_save),
                team_saves,
            } if count_of_team_saves == 1 => {
                println!("Found turn start save: {start_save}");
                println!("Found teammate's save: {}", team_saves[0]);
            }
            Downloader {
                start_save: Some(ref start_save),
                team_saves,
            } if count_of_team_saves > 1 => {
                println!("Found turn start save: {start_save}");
                println!("Found multiple parts to teammate's save:");
                for save in team_saves {
                    println!("\t{save}");
                }
            }
            Downloader {
                start_save: Some(ref save),
                team_saves: _,
            } if count_of_team_saves == 0 => {
                println!("Found turn start save: {save}");
                println!("Did not find any teammate's save");
            }
            Downloader {
                start_save: None,
                team_saves,
            } if is_very_first_turn && count_of_team_saves == 1 => {
                println!("It's the very first turn, so there's no turn start save");
                println!("Found teammate's save: {}", team_saves[0]);
            }
            Downloader {
                start_save: None,
                team_saves,
            } if is_very_first_turn && count_of_team_saves > 1 => {
                println!("It's the very first turn, so there's no turn start save");
                println!("Found multiple parts to teammate's save:");
                for save in team_saves {
                    println!("\t{save}");
                }
            }
            Downloader {
                start_save: None,
                team_saves,
            } if count_of_team_saves == 1 => {
                println!("No save found for {} turn {}", &config.side, turn);
                println!("Found teammate's save: {}", team_saves[0]);
                println!("Maybe ask your teammate if they have a copy of the turn start save you can borrow?");
            }
            Downloader {
                start_save: None,
                team_saves,
            } if count_of_team_saves > 1 => {
                println!("No save found for {} turn {}", &config.side, turn);
                println!("Found multiple parts to teammate's save:");
                for save in team_saves {
                    println!("\t{save}");
                }
                println!("Maybe ask your teammate if they have a copy of the turn start save you can borrow?");
            }
            _ => {
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

fn find_team_saves(
    config: &Config,
    turn: u32,
) -> Result<Vec<DownloadableSave>, Report<DownloadError>> {
    let mut saves =
        shared::find_team_saves(&config.dropbox, config.side, &config.player, turn, Archive)
            .into_report()
            .change_context(DownloadError::Read)?;

    match turn.checked_sub(1) {
        None | Some(0) => {}
        Some(prev_turn) => {
            let mut team_saves_prev_turn = shared::find_team_saves(
                &config.dropbox,
                config.side,
                &config.player,
                prev_turn,
                Archive,
            )
            .into_report()
            .change_context(DownloadError::Read)?;

            saves.append(&mut team_saves_prev_turn)
        }
    };

    saves
        .into_iter()
        .map(|(save, path)| DownloadableSave::new(path, save, config))
        .collect()
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
