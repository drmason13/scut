use anyhow::Context;
use scut_core::{
    error::ErrorSuggestions,
    interface::{
        compression::SevenZipCompression,
        file_system::local_file_system::LocalFileSystem,
        storage::{dropbox_folder::DropboxFolder, game_saves_folder::GameSavesFolder},
        LocalStorage, RemoteStorage,
    },
    Config,
};

pub(crate) fn ready_storage(
    config: &Config,
) -> anyhow::Result<(Box<dyn LocalStorage>, Box<dyn RemoteStorage>)> {
    let compression = SevenZipCompression::new(&config.seven_zip_path);

    let remote_storage = DropboxFolder::new(
        config.dropbox.clone(),
        Box::new(LocalFileSystem::new()),
        Box::new(compression),
    )
    .with_context(|| {
        format!(
            "failed to load dropbox folder with path '{}'",
            config.dropbox.display()
        )
    })
    .suggest("Use `scut config edit` to review and update your config")?;

    let local_storage =
        GameSavesFolder::new(config.saves.clone(), Box::new(LocalFileSystem::new()))
            .with_context(|| {
                format!(
                    "failed to load game saves folder with path '{}'",
                    config.saves.display()
                )
            })
            .suggest("Use `scut config edit` to review and update your config")?;

    Ok((Box::new(local_storage), Box::new(remote_storage)))
}
