use std::{
    io,
    path::PathBuf,
    process::{Command, Output},
};

use crate::config::Config;

/// Compress src and place in the storage directory
pub(crate) fn compress(config: &Config, src: PathBuf) -> io::Result<Output> {
    let src_arg = &src.to_string_lossy();
    let dst_arg = &config.storage().to_string_lossy();
    Command::new("7z")
        .env("PATH", &config.seven_zip_path)
        .args(["a", "-t7z", dst_arg, src_arg])
        .output()
}

/// extract src and place in the saves directory
pub(crate) fn extract(config: &Config, src: PathBuf) -> io::Result<Output> {
    let src_arg = &src.to_string_lossy();
    let dst_arg = &config.saves().to_string_lossy();
    Command::new("7z")
        .env("PATH", &config.seven_zip_path)
        .args(["e", "-t7z", dst_arg, src_arg])
        .output()
}

pub(crate) fn list_files_in_modified_order() -> io::Result<Vec<PathBuf>> {
    let mut entries = std::fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.
    Ok(entries)
}
