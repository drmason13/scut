use std::{
    io,
    path::{Path, PathBuf},
};

pub mod local_file_system;

pub trait FileSystem {
    fn paths_in_folder(&self, folder: &Path) -> anyhow::Result<Vec<PathBuf>>;

    fn write_string_to_file(&self, content: &str, path: &Path) -> anyhow::Result<()>;

    fn read_file_to_string(&self, path: &Path) -> anyhow::Result<String>;
}

/// This function checks if a result returned from this interface is an error caused by an [`NotFound`](std::io::ErrorKind::NotFound) error
pub fn is_not_found_err(error: &anyhow::Error) -> bool {
    match error.downcast_ref::<io::Error>() {
        Some(e) => matches!(e.kind(), io::ErrorKind::NotFound),
        None => false,
    }
}
