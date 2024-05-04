use anyhow::Context;

// types
pub mod error;
mod save;
use error::ErrorSuggestions;
pub use save::{Save, SaveOrAutosave, Side, Turn};
mod config;
pub use config::{Config, Key, Setting};

pub mod interface;
use interface::{LocalStorage, RemoteStorage};

/// Uploads a list of saves
pub fn upload_predicted_saves(
    local: &mut dyn LocalStorage,
    remote: &mut dyn RemoteStorage,
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

/// Downloads a list of saves
pub fn download_predicted_saves(
    local: &dyn LocalStorage,
    remote: &mut dyn RemoteStorage,
    saves: Vec<Save>,
) -> anyhow::Result<()> {
    for save in saves {
        let download_path = local.location();
        remote.download(&save, download_path)?;
    }
    Ok(())
}

/// Upload autosave
pub fn upload_predicted_autosave(
    local: &mut dyn LocalStorage,
    remote: &mut dyn RemoteStorage,
    autosave: Save,
) -> anyhow::Result<()> {
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
    remote.upload(&autosave, local_path.as_path())?;
    Ok(())
}
