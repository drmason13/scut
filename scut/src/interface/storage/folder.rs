use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::interface::FileSystem;
use crate::save::{path_to_save, Save};

/// Capable of storing saves in a Folder on the user's computer.
pub struct Folder {
    pub location: PathBuf,
    contents: HashMap<Save, PathBuf>,
    autosave: Option<PathBuf>,
    file_system: Box<dyn FileSystem>,
}

impl Folder {
    pub fn new(location: &Path, file_system: Box<dyn FileSystem>) -> anyhow::Result<Self> {
        let mut folder = Folder {
            location: location.to_owned(),
            contents: HashMap::new(),
            autosave: None,
            file_system,
        };

        folder.refresh_contents()?;

        Ok(folder)
    }
}

impl Folder {
    /// Look for the save in this Folder and return its path if it exists
    pub fn locate_save(&self, save: &Save) -> anyhow::Result<Option<&Path>> {
        Ok(self.contents.get(save).map(|p| p.as_path()))
    }

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
    pub fn refresh_contents(&mut self) -> anyhow::Result<()> {
        let full_contents = self.file_system.paths_in_folder(&self.location)?;

        let saves = full_contents
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

        self.contents = saves.collect();

        Ok(())
    }
}
