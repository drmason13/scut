use std::path::Path;

use anyhow::Context;

use crate::interface::{Compression, Folder, RemoteStorage};
use crate::Side;
use crate::{error::ErrorSuggestions, Save};

/// This implementation is used to store the saves in your dropbox folder where they can be shared with other players by Dropbox.
/// Dropbox handles the syncing between your local filesytem and their servers.
pub struct DropboxFolder {
    folder: Folder,
    compression: Box<dyn Compression>,
}

impl DropboxFolder {
    pub fn new(folder: Folder, compression: Box<dyn Compression>) -> Self {
        DropboxFolder {
            folder,
            compression,
        }
    }
}

impl DropboxFolder {
    fn attempt_download(
        &mut self,
        attempt: usize,
        save: &Save,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        if attempt >= 1 {
            return Err(anyhow::anyhow!(
                "{save} not found in your dropbox folder '{}'",
                self.folder.location.display()
            ))
            .suggest("Is your Dropbox client synced?")
            .suggest("Have your friends uploaded their turn to Dropbox?")?;
        }

        if let Some(src) = self.folder.locate_save(save)? {
            self.compression
                .decompress(src, local_path)
                .with_context(|| format!("failed to download {save}"))
        } else {
            // Retry after loading contents from disk again
            self.folder
                .refresh_saves()
                .with_context(|| format!("failed to download {save}"))?;
            self.attempt_download(attempt + 1, save, local_path)
        }
    }
}

impl RemoteStorage for DropboxFolder {
    fn download(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()> {
        self.attempt_download(0, save, local_path)
    }

    fn upload(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()> {
        let path = self.folder.choose_location_for_save(save);
        self.compression
            .compress(local_path, &path)
            .with_context(|| format!("failed to upload {save}"))
    }

    fn location(&self) -> String {
        self.folder.location.display().to_string()
    }

    fn get_latest_enemy_turn(&mut self, side: Side) -> anyhow::Result<Option<u32>> {
        self.folder.get_latest_friendly_turn(side)
    }
}
