use std::{
    io,
    path::Path,
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
        .args(["e", from, &format!("-o{to}"), "-y"])
        .output()
}

pub(crate) fn write_string_to_file(content: String, path: &Path) -> Result<(), Report<FileError>> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .into_report()
            .change_context(FileError)
            .attach_printable_lazy(|| {
                format!("Unable to create parent directory: {}", dir.display())
            })?;
    }

    std::fs::write(path, &content)
        .into_report()
        .change_context(FileError)
        .attach_printable_lazy(|| format!("Unable to write to file: {}", path.display()))
}

pub(crate) fn read_input_from_user(prompt: &str) -> io::Result<String> {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[derive(Debug, Error)]
#[error("Error creating file")]
pub struct FileError;
