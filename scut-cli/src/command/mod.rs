use std::fmt::Write;

pub mod config;
use anyhow::Context;
use scut_core::{
    error::ErrorSuggestions,
    interface::{
        predict::{AutosavePrediction, AutosavePredictionReason, Predict},
        LocalStorage, RemoteStorage, UserInteraction,
    },
    Config, Save,
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

    let prediction = predictor.predict(side, player, turn_override, local, remote)?;

    let mut confirmation_prompt = String::new();

    if !prediction.downloads.is_empty() {
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
        let thread_remote = dyn_clone::clone_box(&*remote);

        downloads_handle = Some(std::thread::spawn(|| {
            download_predicted_saves(thread_local, thread_remote, prediction.downloads)
        }));

        let thread_local = dyn_clone::clone_box(&*local);
        let thread_remote = dyn_clone::clone_box(&*remote);

        uploads_handle = Some(std::thread::spawn(|| {
            upload_predicted_saves(thread_local, thread_remote, prediction.uploads)
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
            AutosavePredictionReason::NewTurnAvailable => None,
            AutosavePredictionReason::AutosaveNotAvailable => None,
        },
    } {
        let local_path = local
            .locate_autosave()
            .context("No autosave file exists in your local saved games folder!")?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "scut predicted the need to upload your autosave, but that file was not found!"
                )
            })
            .suggest(
                "This may be a bug in scut! You can report the issue to github: \
                <https://github.com/drmason13/scut/issues/new>",
            )?;

        ui.message(&format!("Uploading autosave as '{autosave}' 🚀"));
        remote.upload(&autosave, local_path.as_path())?;
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

fn download_predicted_saves(
    local: Box<dyn LocalStorage>,
    mut remote: Box<dyn RemoteStorage>,
    saves: Vec<Save>,
) -> anyhow::Result<()> {
    for save in saves {
        let download_path = local.location();
        remote.download(&save, download_path)?;
    }
    Ok(())
}

fn upload_predicted_saves(
    mut local: Box<dyn LocalStorage>,
    mut remote: Box<dyn RemoteStorage>,
    saves: Vec<Save>,
) -> anyhow::Result<()> {
    for save in saves {
        let local_path = local
            .locate_save(&save)
            .with_context(|| {
                format!(
                    "No save file for '{}' exists in your \
                    local saved games folder!",
                    &save
                )
            })?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "scut predicted the need to upload \
                    your save '{}', but the corresponding file was not found!",
                    &save
                )
            })
            .suggest(
                "This may be a bug in scut! You can report issue to github: \
                <https://github.com/drmason13/scut/issues/new>",
            )?;

        remote.upload(&save, local_path.as_path())?;
    }
    Ok(())
}
