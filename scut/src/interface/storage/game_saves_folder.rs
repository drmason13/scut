use std::path::PathBuf;

use anyhow::Context;

use crate::error::ErrorSuggestions;
use crate::interface::{Folder, LocalStorage};
use crate::save::SaveOrAutosave;
use crate::Save;

/// This implementation is used to store the saves in your Strategic Command save game folder where they can be loaded by the game.
pub struct GameSavesFolder {
    folder: Folder,
}

impl GameSavesFolder {
    pub fn new(folder: Folder) -> Self {
        GameSavesFolder { folder }
    }

    fn attempt_locate_save(
        &mut self,
        attempt: usize,
        save: SaveOrAutosave,
    ) -> anyhow::Result<Option<PathBuf>> {
        if attempt >= 1 {
            return Err(anyhow::anyhow!(
                "failed to find {save} in your game saves folder {}",
                self.folder.location.display()
            ))
            .suggest("Did you remember to save the game?")?;
        }

        let save_path = match save.borrow() {
            None => self.folder.locate_autosave()?,
            Some(save) => self.folder.locate_save(save)?,
        };

        if let Some(path) = save_path {
            Ok(Some(path.to_path_buf()))
        } else {
            // Retry after loading contents from disk again
            self.folder.refresh_saves().with_context(|| {
                format!(
                    "failed to find {save} in your game saves folder {}",
                    self.folder.location.display()
                )
            })?;

            self.attempt_locate_save(attempt + 1, save)
        }
    }
}

impl LocalStorage for GameSavesFolder {
    fn locate_save(&mut self, save: &Save) -> anyhow::Result<Option<PathBuf>> {
        self.attempt_locate_save(0, SaveOrAutosave::borrowed(save))
    }

    fn locate_autosave(&mut self) -> anyhow::Result<Option<PathBuf>> {
        self.attempt_locate_save(0, SaveOrAutosave::autosave())
    }
}
