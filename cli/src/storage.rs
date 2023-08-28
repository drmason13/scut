use anyhow::Context;
use scut_core::{
    error::ErrorSuggestions,
    interface::{
        compression::SevenZipCompression,
        file_system::local_file_system::LocalFileSystem,
        storage::{dropbox_folder::DropboxFolder, game_saves_folder::GameSavesFolder},
        Folder, LocalStorage, RemoteStorage,
    },
    Config,
};

pub(crate) fn ready_storage(
    config: &Config,
) -> anyhow::Result<(Box<dyn LocalStorage>, Box<dyn RemoteStorage>)> {
    let compression = SevenZipCompression::new(&config.seven_zip_path);

    let remote_storage = DropboxFolder::new(
        Folder::new(&config.dropbox, Box::new(LocalFileSystem::new()))
            .suggest("Use `scut config edit` to review and update your config")
            .with_context(|| {
                format!(
                    "failed to load dropbox folder with path '{}'",
                    config.dropbox.display()
                )
            })?,
        Box::new(compression),
    );
    let local_storage = GameSavesFolder::new(Folder::new(
        &config.saves,
        Box::new(LocalFileSystem::new()),
    )?);

    Ok((Box::new(local_storage), Box::new(remote_storage)))
}
