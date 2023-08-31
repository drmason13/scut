use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::error::testing_error::{MockError, TestError};

use super::FileSystem;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Status {
    Exists,
    Missing,
    Error,
}

/// Simulates a file
#[derive(Debug, PartialEq, Clone)]
pub struct MockFile {
    pub path: PathBuf,
    pub status: Status,
}

/// Simulates a folder
#[derive(Debug, PartialEq, Clone)]
pub struct MockFolder {
    pub status: Status,
    pub files: Vec<MockFile>,
}

#[derive(Debug)]
pub struct MockFileSystem {
    folders: HashMap<PathBuf, MockFolder>,
    files: HashMap<PathBuf, MockFile>,
    log: Vec<Event>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    /// contains: path of the file, its status, what was written
    WriteStringToFile(PathBuf, Status, String),
    /// contains: path of the file, its status, what was written
    ReadFileToString(PathBuf, Status, String),
    /// contains: path of the file, its status, whether it existed
    FileExists(PathBuf, Status, bool),
    /// contains: path of the folder, its status, len of its contents
    PathsInFolder(PathBuf, Status, usize),
    /// contains: path of the relevant thing, a message
    TestFailure(PathBuf, String),
}

impl FileSystem for MockFileSystem {
    fn file_exists(&mut self, path: &Path) -> anyhow::Result<bool> {
        match self.files.get(path).map(|f| f.status) {
            Some(status @ Status::Exists) => {
                let exists = true;
                self.log
                    .push(Event::FileExists(path.into(), status, exists));
                Ok(exists)
            }
            Some(status @ Status::Missing) => {
                let exists = false;
                self.log
                    .push(Event::FileExists(path.into(), status, exists));
                Ok(exists)
            }
            Some(status @ Status::Error) => {
                self.log.push(Event::FileExists(path.into(), status, false));
                Err(MockError::new(status))?
            }
            None => {
                self.log.push(Event::TestFailure(
                    path.into(),
                    format!(
                        "file_exists was called but the mock filesystem didn't have a mock file for that path: {self:?}"
                    ),
                ));
                Err(TestError)?
            }
        }
    }

    fn paths_in_folder(&mut self, folder: &Path) -> anyhow::Result<Vec<PathBuf>> {
        match self.folders.get(folder) {
            Some(folder) => match folder.status {
                Status::Exists => Ok(folder.files.iter().cloned().map(|f| f.path).collect()),
                Status::Missing => Err(MockError::new(Status::Missing))?,
                Status::Error => Err(MockError::new(Status::Error))?,
            },
            None => Err(TestError)?,
        }
    }

    fn write_string_to_file(&mut self, content: &str, path: &Path) -> anyhow::Result<()> {
        match self.files.get(path).map(|f| f.status) {
            Some(Status::Exists) => Ok(()),
            Some(Status::Missing) => Err(MockError::new(Status::Missing))?,
            Some(Status::Error) => Err(MockError::new(Status::Error))?,
            None => Err(TestError)?,
        }
    }

    fn read_file_to_string(&mut self, path: &Path) -> anyhow::Result<String> {
        todo!()
    }
}
