use std::path::{Path, PathBuf};

use error_stack::{IntoReport, Report, ResultExt};

use crate::{error::NoSaveFileFound, fs::list_files_in_modified_order};

/// utility functions shared between commands

pub(crate) fn find_latest_save_file(dir: &Path) -> Result<PathBuf, Report<NoSaveFileFound>> {
    list_files_in_modified_order(dir, "sav")
        .into_report()
        .change_context(NoSaveFileFound)
        .attach_printable("Unable to list save files")?
        .into_iter()
        .next()
        .ok_or_else(|| Report::new(NoSaveFileFound))
        .attach_printable_lazy(|| format!("Looked inside directory: {}", dir.display()))
}

pub(crate) fn find_latest_archive_file(dir: &Path) -> Result<PathBuf, Report<NoSaveFileFound>> {
    list_files_in_modified_order(dir, "7z")
        .into_report()
        .change_context(NoSaveFileFound)
        .attach_printable("Unable to list save files")?
        .into_iter()
        .next()
        .ok_or_else(|| Report::new(NoSaveFileFound))
        .attach_printable_lazy(|| format!("Looked inside directory: {}", dir.display()))
}

#[cfg(test)]
mod test {
    use crate::test::create_test_directory;

    use super::*;
    #[test]
    fn test_find_latest_save_file() {
        let test_dir = create_test_directory();

        let latest_save = find_latest_save_file(test_dir.path()).expect("latest_save");

        assert_eq!(
            latest_save.file_name().expect("filename"),
            PathBuf::from("test2.sav")
        )
    }

    #[test]
    fn test_find_latest_archive_file() {
        let test_dir = create_test_directory();

        let latest_save = find_latest_archive_file(test_dir.path()).expect("latest_archive");

        assert_eq!(
            latest_save.file_name().expect("filename"),
            PathBuf::from("test2.7z")
        )
    }
}
