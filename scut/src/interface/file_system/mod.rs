use std::path::{Path, PathBuf};

pub mod local_file_system;

pub trait FileSystem {
    fn paths_in_folder(&self, folder: &Path) -> anyhow::Result<Vec<PathBuf>>;

    fn write_string_to_file(&self, content: &str, path: &Path) -> anyhow::Result<()>;

    fn read_file_to_string(&self, path: &Path) -> anyhow::Result<String>;
}

pub struct FileSystemError {}
