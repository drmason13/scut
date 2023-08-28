use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;

use super::FileSystem;

pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn paths_in_folder(&self, folder: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let files = fs::read_dir(folder)
            .with_context(|| format!("failed to list files in {}", folder.display()))?;

        files
            .map(entry_to_path)
            .collect::<Result<_, _>>()
            .with_context(|| format!("failed to list files in {}", folder.display()))
    }

    fn write_string_to_file(&self, content: &str, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("failed to create parent directory: {}", dir.display()))?;
        }

        std::fs::write(path, content)
            .with_context(|| format!("failed to write to file: {}", path.display()))
    }

    fn read_file_to_string(&self, path: &Path) -> anyhow::Result<String> {
        std::fs::read_to_string(path)
            .with_context(|| format!("failed to read from file: {}", path.display()))
    }
}

fn entry_to_path(result: Result<DirEntry, io::Error>) -> anyhow::Result<PathBuf> {
    match result {
        Ok(entry) => Ok(entry.path()),
        Err(err) => Err(anyhow::Error::from(err)),
    }
}
