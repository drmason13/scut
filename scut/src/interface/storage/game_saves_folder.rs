use std::path::Path;

use anyhow::Context;

use crate::interface::{Folder, LocalStorage};
use crate::Save;

/// This implementation is used to store the saves in your Strategic Command save game folder where they can be loaded by the game.
pub struct GameSavesFolder {
    folder: Folder,
}

impl GameSavesFolder {
    pub fn new(folder: Folder) -> Self {
        GameSavesFolder { folder }
    }
}

impl LocalStorage for GameSavesFolder {
    fn locate_save(&mut self, save: &Save) -> anyhow::Result<Option<&Path>> {
        self.folder
            .locate_save(save)
            .with_context(|| "failed to find save {save} in your game saves folder")
    }

    fn locate_autosave(&mut self) -> anyhow::Result<Option<&Path>> {
        self.folder
            .locate_autosave()
            .with_context(|| "failed to find autosave in your game saves folder")
    }
}
