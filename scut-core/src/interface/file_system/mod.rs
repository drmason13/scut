use std::{
    io,
    path::{Path, PathBuf},
};

use dyn_clone::DynClone;

pub mod local_file_system;

#[cfg(test)]
pub mod mock_file_system;

pub trait FileSystem: DynClone + Send + Sync {
    fn file_exists(&mut self, path: &Path) -> anyhow::Result<bool>;

    fn files_in_folder(&mut self, folder: &Path) -> anyhow::Result<Vec<PathBuf>>;

    fn write_string_to_file(&mut self, content: &str, path: &Path) -> anyhow::Result<()>;

    fn read_file_to_string(&mut self, path: &Path) -> anyhow::Result<String>;
}

impl Clone for Box<dyn FileSystem> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

/// This function checks if a result returned from this interface is an error caused by an [`NotFound`](std::io::ErrorKind::NotFound) error
pub fn is_not_found_err(error: &anyhow::Error) -> bool {
    match error.downcast_ref::<io::Error>() {
        Some(e) => matches!(e.kind(), io::ErrorKind::NotFound),
        None => false,
    }
}
