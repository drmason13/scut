use std::fmt::Write;

pub mod config;

use scut_core::{
    download_predicted_saves,
    interface::{
        predict::{AutosavePrediction, AutosavePredictionReason, Predict},
        LocalStorage, RemoteStorage, UserInteraction,
    },
    upload_predicted_autosave, upload_predicted_saves, Config,
};

/// Runs scut, which will predict what downloads and uploads are desired,
/// confirms with the user and then does them!
pub fn run(
    turn_override: Option<u32>,
    config: &mut Config,
    mut local: Box<dyn LocalStorage>,
    mut remote: Box<dyn RemoteStorage>,
    predictor: Box<dyn Predict>,
    mut ui: Box<dyn UserInteraction>,
) -> anyhow::Result<()> {
    let local = &mut *local;
    let remote = &mut *remote;

    let side = config.side;
    let player = config.player.as_str();

    let playing_solo = config.solo.unwrap_or_default();
    let prediction = predictor.predict(side, player, turn_override, playing_solo, local, remote)?;

    let mut confirmation_prompt = String::new();

    let no_downloads = prediction.downloads.is_empty();
    if !no_downloads {
        writeln!(confirmation_prompt, "Will download:")?;
        for download in prediction.downloads.iter() {
            writeln!(
                confirmation_prompt,
                "  ⬇️ {download}{}",
                if download.player.is_none() {
                    " (autosave)"
                } else {
                    ""
                }
            )?;
        }
    }

    if !prediction.uploads.is_empty() {
        writeln!(confirmation_prompt, "Will upload:")?;
        for upload in prediction.uploads.iter() {
            writeln!(confirmation_prompt, "  ↗️ {upload}")?;
        }
    }

    let mut uploads_handle = None;
    let mut downloads_handle = None;

    if !confirmation_prompt.is_empty() {
        ui.message(&confirmation_prompt);

        if !ui.confirm("Is that OK?", Some(true)) {
            ui.wait_for_user_before_close("User cancelled. Stopping.");
            return Ok(());
        }

        let thread_local = dyn_clone::clone_box(&*local);
        let mut thread_remote = dyn_clone::clone_box(&*remote);

        downloads_handle = Some(std::thread::spawn(move || {
            download_predicted_saves(&*thread_local, &mut *thread_remote, prediction.downloads)
        }));

        let mut thread_local = dyn_clone::clone_box(&*local);
        let mut thread_remote = dyn_clone::clone_box(&*remote);

        uploads_handle = Some(std::thread::spawn(move || {
            upload_predicted_saves(&mut *thread_local, &mut *thread_remote, prediction.uploads)
        }));
    }

    if let Some(autosave) = match prediction.autosave {
        AutosavePrediction::Ready(autosave) => {
            if ui.confirm(
                &format!("Do you want to upload your autosave as: {autosave}?",),
                Some(true),
            ) {
                Some(autosave)
            } else {
                None
            }
        }
        AutosavePrediction::NotReady(autosave, reason) => match reason {
            AutosavePredictionReason::AutosaveAlreadyUploaded => {
                if ui.confirm(
                    &format!(
                        "⚠️ {autosave} has already been uploaded. \
                        Do you want to overwrite it with your autosave? ⚠️",
                    ),
                    Some(false),
                ) {
                    Some(autosave)
                } else {
                    None
                }
            }
            AutosavePredictionReason::TeammateSaveNotUploaded => None,
            AutosavePredictionReason::NewTeammateSaveAvailable(_) => None,
            AutosavePredictionReason::TurnNotPlayed(save) => {
                if no_downloads
                    && ui.confirm(
                        &format!(
                            "⚠️ It looks like you haven't played {save} yet. \
                            Do you want to upload your autosave as {autosave} anyway? ⚠️",
                        ),
                        Some(false),
                    )
                {
                    Some(autosave)
                } else {
                    None
                }
            }
            AutosavePredictionReason::AutosaveNotAvailable => None,
        },
    } {
        ui.message(&format!("Uploading autosave as '{autosave}' 🚀"));
        upload_predicted_autosave(&mut *local, &mut *remote, autosave)?;
    } else if confirmation_prompt.is_empty() {
        ui.message("Your local saves folder is synced with remote.");
        ui.wait_for_user_before_close("Nothing to do 💤");
        return Ok(());
    }

    // join all threads and propogate any errors
    if let Some(handle) = downloads_handle {
        match handle.join() {
            Ok(result) => result?,
            Err(_panic) => {
                anyhow::bail!(
                    "There was an unhandled error while \
                    downloading and decompressing save files"
                )
            }
        }
    }
    if let Some(handle) = uploads_handle {
        match handle.join() {
            Ok(result) => result?,
            Err(_panic) => {
                anyhow::bail!(
                    "There was an unhandled error while \
                    compressing and uploading save files"
                )
            }
        }
    }

    ui.wait_for_user_before_close("Done ✔️");
    Ok(())
}
