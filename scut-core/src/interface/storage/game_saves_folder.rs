use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::interface::index::IterIndex;
use crate::interface::{FileSystem, LocalStorage};
use crate::save::{path_to_save, SaveOrAutosave};
use crate::Save;

/// This implementation is used to store the saves in your Strategic Command save game folder where they can be loaded by the game.
#[derive(Clone)]
pub struct GameSavesFolder {
    pub location: PathBuf,
    saves: BTreeMap<Save, PathBuf>,
    autosave: Option<PathBuf>,
    file_system: Box<dyn FileSystem>,
}

impl GameSavesFolder {
    pub fn new(location: PathBuf, file_system: Box<dyn FileSystem>) -> anyhow::Result<Self> {
        let mut folder = GameSavesFolder {
            location,
            saves: BTreeMap::new(),
            autosave: None,
            file_system,
        };

        folder.refresh_saves()?;

        Ok(folder)
    }

    fn locate_save_with_retry(&mut self, save: SaveOrAutosave) -> anyhow::Result<Option<PathBuf>> {
        let mut attempt = 0;

        let save_path = loop {
            let save_path = match save.borrow() {
                None => self.get_autosave(),
                Some(save) => self.get_save(save),
            };

            if attempt > 1 {
                break save_path;
            }
            attempt += 1;

            if let Some(path) = save_path {
                return Ok(Some(path.to_path_buf()));
            } else {
                self.refresh_saves().with_context(|| {
                    format!(
                        "failed to find {save} in your game saves folder {}",
                        self.location.display()
                    )
                })?;
            }
        };

        match save_path {
            Some(path) => Ok(Some(path.to_path_buf())),
            None => Ok(None),
        }
    }

    /// Look for a save in this Folder and return its path if it exists
    pub fn get_save(&self, save: &Save) -> Option<&Path> {
        self.saves.get(save).map(|p| p.as_path())
    }

    /// Look for the autosave in this Folder and return its path if it exists
    pub fn get_autosave(&self) -> Option<&Path> {
        self.autosave.as_deref()
    }

    /// Reloads from disk what saves are in this Folder
    ///
    /// TODO: fix for potential to have multiple files that parse to the same Save
    /// currently, one of the paths will "win" but I don't think it's deterministic which one will win,
    /// so it seems preferable to detect such duplicates and error early
    pub fn refresh_saves(&mut self) -> anyhow::Result<()> {
        let all_files = self.file_system.files_in_folder(&self.location)?;

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
}

impl LocalStorage for GameSavesFolder {
    fn locate_save(&mut self, save: &Save) -> anyhow::Result<Option<PathBuf>> {
        // previously we avoided this clone with the notion of a BorrowedSave - like Cow
        // but that made autosaves very difficult to parse due to the lifetime parameter
        self.locate_save_with_retry(SaveOrAutosave::save(save.clone()))
    }

    fn locate_autosave(&mut self) -> anyhow::Result<Option<PathBuf>> {
        self.locate_save_with_retry(SaveOrAutosave::autosave())
    }

    fn location(&self) -> &Path {
        self.location.as_path()
    }

    fn index(&self) -> &dyn crate::interface::Index {
        self
    }
}

/// Folders are able to return an iterator of saves, so they fulfil the blanket implementation of [`Index`](crate::interface::Index) for iterators of saves...
/// and get a free implementation of Index - hooray!
impl<'a> IterIndex<'a> for GameSavesFolder {
    type Iter = std::collections::btree_map::Keys<'a, Save, PathBuf>;

    fn iter(&'a self) -> Self::Iter {
        self.saves.keys()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{interface::file_system::mock_file_system::MockFileSystem, Side};

    use super::*;

    #[test]
    fn folder_works() {
        let mock_file_system = MockFileSystem::from_str(indoc::indoc! {r"
        saves/
            autosave.sav
            Axis DM 1.sav
            Allies 1.sav
            ?Allies NO 99.sav
    "})
        .unwrap();

        let mut folder =
            GameSavesFolder::new(PathBuf::from("saves"), Box::new(mock_file_system)).unwrap();
        assert_eq!(
            folder
                .locate_save(&Save::from_parts(Side::Axis, 1).player("DM"))
                .expect("save should exist"),
            Some(PathBuf::from("saves/Axis DM 1.sav"))
        );
    }

    #[test]
    fn folder_works_with_missing_files() {
        let mock_file_system = MockFileSystem::from_str(indoc::indoc! {r"
        saves/
            ?autosave.sav
            ?Axis DM 1.sav
            Allies 1.sav
            ?Allies NO 99.sav
    "})
        .unwrap();

        let mut folder =
            GameSavesFolder::new(PathBuf::from("saves"), Box::new(mock_file_system)).unwrap();
        let actual = folder
            .locate_save(&Save::from_parts(Side::Axis, 1).player("DM"))
            .expect("save should exist");

        assert_eq!(actual, None);
    }

    #[test]
    fn folder_works_with_error_files() {
        let mock_file_system = MockFileSystem::from_str(indoc::indoc! {r"
        saves/
            ?autosave.sav
            !Axis DM 1.sav
            Allies 1.sav
            ?Allies NO 99.sav
    "})
        .unwrap();

        let folder = GameSavesFolder::new(PathBuf::from("saves"), Box::new(mock_file_system));

        assert!(folder.is_err());
    }
}
