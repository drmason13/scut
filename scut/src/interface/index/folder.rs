use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::save::{path_to_save, Save};
use crate::{interface::FileSystem, Side};

/// Capable of storing and retrieving saves in a Folder on the user's computer.
pub struct Folder {
    pub location: PathBuf,
    saves: HashMap<Save, PathBuf>,
    autosave: Option<PathBuf>,
    file_system: Box<dyn FileSystem>,
}

impl Folder {
    pub fn new(location: &Path, file_system: Box<dyn FileSystem>) -> anyhow::Result<Self> {
        let mut folder = Folder {
            location: location.to_owned(),
            saves: HashMap::new(),
            autosave: None,
            file_system,
        };

        folder.refresh_saves()?;

        Ok(folder)
    }
}

impl Folder {
    /// Look for a save in this Folder and return its path if it exists
    pub fn locate_save(&self, save: &Save) -> anyhow::Result<Option<&Path>> {
        Ok(self.saves.get(save).map(|p| p.as_path()))
    }

    /// Look for the autosave in this Folder and return its path if it exists
    pub fn locate_autosave(&self) -> anyhow::Result<Option<&Path>> {
        Ok(self.autosave.as_deref())
    }

    /// Construct the path describing where a save should be stored in this Folder
    pub fn choose_location_for_save(&self, save: &Save) -> PathBuf {
        self.location.join(save.to_string())
    }

    /// Reloads from disk what saves are in this Folder
    ///
    /// TODO: fix for potential to have multiple files that parse to the same Save
    /// currently, one of the paths will "win" but I don't think it's deterministic which one will win,
    /// so it seems preferable to detect such duplicates and error early
    pub fn refresh_saves(&mut self) -> anyhow::Result<()> {
        let all_files = self.file_system.paths_in_folder(&self.location)?;

        let saves = all_files
            .into_iter()
            .filter_map(|path| path_to_save(&path).map(|save| (save, path)));

        let autosave_path = match self.autosave.take() {
            None => self.location.join("autosave.sav"),
            Some(path) => path,
        };

        if self
            .file_system
            .file_exists(&autosave_path)
            .context("failed to read autosave on disk")?
        {
            self.autosave = Some(autosave_path);
        } else {
            self.autosave = None;
        }

        self.saves = saves.collect();

        Ok(())
    }

    pub fn get_latest_friendly_turn(&mut self, side: Side) -> anyhow::Result<Option<u32>> {
        self.refresh_saves()?;

        Ok(self
            .saves
            .keys()
            .filter(|save| save.side == side)
            .map(|save| save.turn)
            .max())
    }
}
