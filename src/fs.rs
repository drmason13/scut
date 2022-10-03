use std::{
    fs::DirEntry,
    io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use error_stack::{IntoReport, Report, ResultExt};
use thiserror::Error;

/// Compress src and place in dest
pub(crate) fn compress(seven_zip_path: &Path, src: &Path, dest: &Path) -> io::Result<Output> {
    let from = &src.to_string_lossy();
    let to = &dest.to_string_lossy();
    Command::new("7z")
        .env("PATH", seven_zip_path)
        .args(["a", to, from])
        .output()
}

/// extract src and place in dest
pub(crate) fn extract(seven_zip_path: &Path, src: &Path, dest: &Path) -> io::Result<Output> {
    let from = &src.to_string_lossy();
    let to = &dest.to_string_lossy();
    Command::new("7z")
        .env("PATH", seven_zip_path)
        .args(["e", from, &format!("-o{to}")])
        .output()
}

pub(crate) fn list_files_in_modified_order<T>(dir: T, extension: &str) -> io::Result<Vec<PathBuf>>
where
    T: AsRef<Path>,
{
    let mut entries: Vec<DirEntry> = std::fs::read_dir(dir.as_ref())?
        .filter(|entry| match entry {
            Ok(entry) => entry.path().extension().map(|ext| ext == extension) == Some(true),
            Err(_) => true, // pass thru errors
        })
        .collect::<Result<_, _>>()?;

    entries.sort_by(|a, b| {
        b.metadata()
            .expect("metadata")
            .modified()
            .expect("modified")
            .cmp(
                &a.metadata()
                    .expect("metadata")
                    .modified()
                    .expect("modified"),
            )
    });

    // The entries have now been sorted by their path.
    Ok(entries.into_iter().map(|entry| entry.path()).collect())
}

pub(crate) fn write_string_to_file(content: String, path: &Path) -> Result<(), Report<FileError>> {
    let dir = path
        .parent()
        .ok_or_else(|| Report::new(FileError).attach_printable("Expected an absolute path"))?;

    std::fs::create_dir_all(dir)
        .into_report()
        .change_context(FileError)
        .attach_printable_lazy(|| {
            format!("Unable to create parent directory: {}", dir.display())
        })?;

    std::fs::write(path, &content)
        .into_report()
        .change_context(FileError)
        .attach_printable_lazy(|| format!("Unable to write to file: {}", path.display()))
}

#[derive(Debug, Error)]
#[error("Error creating file")]
pub struct FileError;

#[cfg(test)]
mod test {
    use crate::test::create_test_directory;

    use super::*;

    #[test]
    fn test_list_save_files_in_modified_order() {
        let test_dir = create_test_directory();
        let ordered_files = list_files_in_modified_order(test_dir, "sav")
            .expect("test_list_sav_files_in_modified_order");

        assert_eq!(ordered_files.len(), 2);
        assert!(ordered_files[0].to_str().unwrap().contains("test2.sav"));
        assert!(ordered_files[1].to_str().unwrap().contains("test.sav"));
    }
}
