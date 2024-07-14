use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::interface::index::IterIndex;
use crate::interface::{Compression, FileSystem, RemoteStorage};
use crate::save::path_to_save;
use crate::{error::ErrorSuggestions, Save};

/// This implementation is used to store the saves in your dropbox folder where they can be shared with other players by Dropbox.
/// Dropbox handles the syncing between your local filesytem and their servers.
#[derive(Clone)]
pub struct DropboxFolder {
    pub location: PathBuf,
    saves: BTreeMap<Save, PathBuf>,
    file_system: Box<dyn FileSystem>,
    compression: Box<dyn Compression>,
}

impl DropboxFolder {
    pub fn new(
        location: PathBuf,
        file_system: Box<dyn FileSystem>,
        compression: Box<dyn Compression>,
    ) -> anyhow::Result<Self> {
        let mut folder = DropboxFolder {
            location,
            saves: BTreeMap::new(),
            compression,
            file_system,
        };

        folder.refresh_saves()?;

        Ok(folder)
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
                self.location.display()
            ))
            .suggest("Is your Dropbox client synced?")
            .suggest("Have your friends uploaded their turn to Dropbox?")?;
        }

        if let Some(src) = self.locate_save(save)? {
            self.compression
                .decompress(src, local_path)
                .with_context(|| format!("failed to download {save}"))
        } else {
            // Retry after loading contents from disk again
            self.refresh_saves()
                .with_context(|| format!("failed to download {save}"))?;
            self.attempt_download(attempt + 1, save, local_path)
        }
    }

    /// Look for a save in this Folder and return its path if it exists
    pub fn locate_save(&self, save: &Save) -> anyhow::Result<Option<&Path>> {
        Ok(self.saves.get(save).map(|p| p.as_path()))
    }

    /// Construct the path describing where a save should be stored in this Folder
    pub fn choose_location_for_save(&self, save: &Save) -> PathBuf {
        self.location.join(save.to_string())
    }

    /// Reloads from disk what saves are in this Folder
    pub fn refresh_saves(&mut self) -> anyhow::Result<()> {
        let all_files = self.file_system.files_in_folder(&self.location)?;

        let mut saves = all_files
            .into_iter()
            .filter_map(|path| path_to_save(&path).map(|save| (save, path)));

        while let Some((save, path)) = saves.next() {
            match self.saves.entry(save) {
                Entry::Occupied(occupied) => {
                    let error =
                        anyhow::anyhow!("duplicate saves detected in {}", &self.location.display());
                    return Err(error).suggest(format!(
                        "You have multiple save files that look like '{}' to SCUT:\n{}.\nDetermine the correct save file and rename or delete the others.",
                        occupied.key(),
                        format!("Including {} and {}", occupied.get().display(), path.display()),
                    ))?;
                }
                Entry::Vacant(vacancy) => vacancy.insert(path),
            };
        }

        Ok(())
    }
}

impl RemoteStorage for DropboxFolder {
    fn download(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()> {
        self.attempt_download(0, save, local_path)
    }

    fn upload(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()> {
        let path = self.choose_location_for_save(save);
        self.compression
            .compress(local_path, &path)
            .with_context(|| format!("failed to upload {save}"))
    }

    fn index(&self) -> &dyn crate::interface::Index {
        self
    }
}

/// Folders are able to return an iterator of saves, so they fulfil the blanket implementation of [`Index`](crate::interface::Index) for iterators of saves...
/// and get a free implementation of Index - hooray!
impl<'a> IterIndex<'a> for DropboxFolder {
    type Iter = std::collections::btree_map::Keys<'a, Save, PathBuf>;

    fn iter(&'a self) -> Self::Iter {
        self.saves.keys()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use indoc::indoc;

    use crate::{
        interface::{
            compression::mock_compression::MockCompression,
            file_system::mock_file_system::MockFileSystem, RemoteStorage,
        },
        Side,
    };

    use super::*;

    #[test]
    fn dropbox_folder_works() -> Result<(), Box<dyn std::error::Error>> {
        let mock_file_system = MockFileSystem::from_str(indoc! {r"
            /local/
                ?autosave.sav

            /remote/
                Axis DM 1.sav
                Axis DG 1.sav
                Allies 1.sav
                bystander.txt
                nonsense.sav
        "})?;

        let mut dropbox = DropboxFolder::new(
            PathBuf::from("/remote"),
            Box::new(mock_file_system),
            Box::new(MockCompression::new()),
        )?;

        dropbox.download(
            &Save::new(Side::Allies, 1),
            PathBuf::from("/local/").as_path(),
        )?;

        Ok(())
    }
}
