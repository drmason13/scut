use std::fmt::Write;

pub mod config;
use anyhow::Context;
pub use config::ConfigSubcommand;
use scut_core::{
    error::ErrorSuggestions,
    interface::{predict::Predict, LocalStorage, RemoteStorage, UserInteraction},
    Config, Turn,
};

/// Runs scut, which will predict what downloads and uploads are desired, confirms with the user and then does them!
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
    let turn = if let Some(turn_number) = turn_override {
        Turn::new(config.side, turn_number)
    } else {
        predictor.predict_turn(side, player, local, remote)?
    };

    let downloads = predictor.predict_downloads(turn, side, player, local, remote)?;
    let uploads = predictor.predict_uploads(turn, side, player, local, remote)?;
    let mut predicted_autosave = predictor.predict_autosave(turn, side, player, local, remote)?;
    let should_uploads_autosave =
        predictor.should_upload_autosave(&predicted_autosave, side, player, &downloads, &uploads);

    if !should_uploads_autosave {
        predicted_autosave = None;
    }

    if let Some(ref autosave) = predicted_autosave {
        if !ui.confirm(
            &format!("Do you want to upload your autosave as: {autosave}?",),
            Some(true),
        ) {
            predicted_autosave = None;
        }
    }

    let mut confirmation_prompt = String::new();

    if !downloads.is_empty() {
        writeln!(confirmation_prompt, "Will download:")?;
        for download in downloads.iter() {
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

    if !uploads.is_empty() {
        writeln!(confirmation_prompt, "Will upload:")?;
        for upload in uploads.iter() {
            writeln!(confirmation_prompt, "  ↗️ {upload}")?;
        }
    }

    if let Some(ref autosave) = predicted_autosave {
        write!(
            confirmation_prompt,
            "Will upload autosave as '{}' 🚀",
            autosave
        )?;
    }

    if confirmation_prompt.is_empty() {
        ui.message("Your local saves folder is synced with remote.");
        ui.wait_for_user_before_close("Nothing to do 💤");
        return Ok(());
    }

    ui.message(&confirmation_prompt);

    if !ui.confirm("Is that OK?", Some(true)) {
        ui.wait_for_user_before_close("User cancelled. Stopping.");
        return Ok(());
    }

    for save in downloads.iter() {
        let download_path = local.location();
        remote.download(save, download_path)?;
    }

    for save in uploads.iter() {
        let local_path = local.locate_save(save)
            .with_context(|| format!("No save file for '{}' exists in your local saved games folder!", &save))?
            .ok_or_else(|| anyhow::anyhow!("scut predicted the need to upload your save '{}', but the corresponding file was not found!", &save))
            .suggest("This may be a bug in scut! You can report issue to github: <https://github.com/drmason13/scut/issues/new>")?;

        remote.upload(save, local_path.as_path())?;
    }

    if let Some(autosave) = predicted_autosave {
        let local_path = local.locate_autosave()
        .context("No autosave file exists in your local saved games folder!")?
        .ok_or_else(|| anyhow::anyhow!("scut predicted the need to upload your autosave, but that file was not found!"))
        .suggest("This may be a bug in scut! You can report the issue to github: <https://github.com/drmason13/scut/issues/new>")?;

        remote.upload(&autosave, local_path.as_path())?;
    }

    ui.wait_for_user_before_close("Done ✔️");

    Ok(())
}
