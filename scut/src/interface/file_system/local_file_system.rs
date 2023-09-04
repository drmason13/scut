use std::{
    fs::{self},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;

use super::FileSystem;

pub struct LocalFileSystem;

impl LocalFileSystem {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        LocalFileSystem
    }
}

impl FileSystem for LocalFileSystem {
    fn file_exists(&mut self, path: &Path) -> anyhow::Result<bool> {
        Ok(path.try_exists()?)
    }

    // TODO: only search for files in folder - rename method
    fn files_in_folder(&mut self, folder: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let files = fs::read_dir(folder)
            .with_context(|| format!("failed to list files in '{}'", folder.display(),))?;

        files
            .filter_map(entry_to_path_if_file)
            .collect::<Result<_, _>>()
            .with_context(|| format!("failed to list files in '{}'", folder.display()))
    }

    fn write_string_to_file(&mut self, content: &str, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| {
                format!("failed to create parent directory: '{}'", dir.display())
            })?;
        }

        fs::write(path, content)
            .with_context(|| format!("failed to write to file: '{}'", path.display()))
    }

    fn read_file_to_string(&mut self, path: &Path) -> anyhow::Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("failed to read from file: '{}'", path.display()))
    }
}

fn entry_to_path_if_file(
    result: Result<fs::DirEntry, io::Error>,
) -> Option<anyhow::Result<PathBuf>> {
    match result {
        Ok(entry) => match entry.metadata() {
            Ok(metadata) if metadata.is_file() => Some(Ok(entry.path())),
            Ok(_) => None,
            Err(e) => Some(Err(anyhow::Error::from(e))),
        },
        Err(err) => Some(Err(anyhow::Error::from(err))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_file_system() -> Result<(), Box<dyn std::error::Error>> {
        let tmpdir = tempfile::tempdir()?;

        let _ = fs::File::create(tmpdir.path().join("file1"))?;
        let _ = fs::File::create(tmpdir.path().join("file2"))?;

        fs::create_dir(tmpdir.path().join("folder1"))?;
        let file3_path = tmpdir.path().join("folder1").join("file3");
        let _ = fs::File::create(file3_path.clone())?;

        let mut local_file_system = LocalFileSystem;

        assert!(
            local_file_system.file_exists(tmpdir.path().join("file1").as_path())?,
            "file should exist"
        );
        assert!(
            !local_file_system.file_exists(tmpdir.path().join("file3").as_path())?,
            "file should not exist"
        );

        let mut files_in_folder = local_file_system.files_in_folder(tmpdir.path())?;
        files_in_folder.sort();

        assert_eq!(
            files_in_folder,
            vec![tmpdir.path().join("file1"), tmpdir.path().join("file2"),]
        );

        local_file_system.write_string_to_file("test content", &file3_path)?;
        assert_eq!(
            local_file_system.read_file_to_string(&file3_path)?,
            "test content"
        );

        Ok(())
    }
}
