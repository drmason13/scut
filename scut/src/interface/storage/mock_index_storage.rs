use std::path::{Path, PathBuf};

use crate::{
    interface::{
        index::{mock_index::MockIndex, IterIndex},
        Index,
    },
    Save,
};

use super::{LocalStorage, RemoteStorage};

#[derive(Debug, Clone)]
pub struct MockIndexStorage {
    index: MockIndex,
    path: PathBuf,
    autosave: bool,
}

impl MockIndexStorage {
    pub fn new(autosave: bool, saves: Vec<Save>) -> Self {
        MockIndexStorage {
            autosave,
            index: MockIndex::new(saves.iter()),
            path: PathBuf::from("wherever"),
        }
    }
}

impl<'a> IterIndex<'a> for MockIndexStorage {
    type Iter = <MockIndex as IterIndex<'a>>::Iter;

    fn iter(&'a self) -> Self::Iter {
        self.index.iter()
    }
}

impl LocalStorage for MockIndexStorage {
    fn locate_save(&mut self, _save: &Save) -> anyhow::Result<Option<PathBuf>> {
        Ok(None)
    }

    fn locate_autosave(&mut self) -> anyhow::Result<Option<PathBuf>> {
        Ok(if self.autosave {
            Some(PathBuf::new())
        } else {
            None
        })
    }

    fn location(&self) -> &Path {
        self.path.as_path()
    }

    fn index(&self) -> &dyn Index {
        self
    }
}

impl RemoteStorage for MockIndexStorage {
    fn download(&mut self, _save: &Save, _local_path: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn upload(&mut self, _save: &Save, _local_path: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn index(&self) -> &dyn Index {
        self
    }
}
