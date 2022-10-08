use std::path::Path;

use clap::Args;
use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

use crate::{command::shared::iter_turn_saves_in_dir, config::Config, io_utils::extract};

use super::shared::wait_for_user_before_close;

#[derive(Debug, Args)]
pub(crate) struct Download;

impl Download {
    // you only ever want to download saves for your team (they're "from" the other team, but they are labelled with *your* team)
    // you might want to download your teammate's save, but only if its for the same turn you're about to play

    // i.e. if you want to play Axis turn 8, and there's "Axis start 8" and "Axis JB 8" in the saves, you'll want both
    //      if there's "Axis start 8" and "Axis JB 7" or "Axis ME 7" but no "Axis JB 8" then you just want to download "Axis start 8"

    // it would actually be convenient if we could ask for the saves we want and download all of them that are there.
    // problem being we can't know exactly what they're called (cos other saves are being uploaded by pesky humans)
    // but we can work out if they are the right save by parsing them... we could be... lazy? and parse them all
    // (they're just local filenames after all - it might actually be faster than sorting them and taking the most recent X)

    pub(crate) fn run(self, config: &Config) -> Result<(), Report<DownloadError>> {
        let mut available_saves = iter_turn_saves_in_dir(&config.dropbox, "7z")
            .into_report()
            .change_context(DownloadError::Read)?;

        // find turn start save and teammate's save if there is one
        if let Some(Ok((save, path))) = available_saves.find(|save| match save {
            Err(_) => false,
            Ok((save, _)) => {
                save.turn == config.turn
                    && save.side == config.side
                    && save.player.as_ref() != Some(&config.player)
            }
        }) {
            if let Some(teammate) = save.player.as_ref() {
                // we found a save for the right turn belonging to someone else playing on the same side as us
                // let's assume they are a teammate!
                println!("Found turn belonging to teammate: {}", teammate);
                extract_save(&path, config)?;
            }
            println!("Found turn start save: {}", &save);
            extract_save(&path, config)?;
        } else {
            println!("No save found for {} turn {}", &config.side, &config.turn);
            return Ok(());
        }

        wait_for_user_before_close("Done");
        Ok(())
    }
}

fn extract_save(path: &Path, config: &Config) -> Result<(), Report<DownloadError>> {
    let filename = path
        .file_name()
        .ok_or_else(|| Report::new(DownloadError::Read))
        .attach_printable_lazy(|| {
            format!("path {} did not have a filename component", path.display())
        })?;

    println!(
        "Extracting {src} to {dst}",
        src = filename.to_string_lossy(),
        dst = config.saves.display()
    );

    extract(&config.seven_zip_path, path, &config.saves)
        .into_report()
        .change_context(DownloadError::Extract)
        .attach_printable_lazy(|| {
            format!(
                "while extracting {src} to {dst}",
                src = path.display(),
                dst = config.dropbox.display()
            )
        })?;
    Ok(())
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum DownloadError {
    #[error("Could not read zipped save file")]
    Read,
    #[error("Could not extract save from zip file")]
    Extract,
}
