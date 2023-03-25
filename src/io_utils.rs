use std::{
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

    std::fs::write(path, content)
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

pub(crate) fn iter_files_with_extension<'a, 'b>(
    dir: &'a Path,
    extension: &'b str,
) -> io::Result<impl Iterator<Item = io::Result<PathBuf>> + 'a>
where
    'b: 'a,
{
    Ok(std::fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .filter(move |path| match path {
            Ok(path) => matches!(path.extension(), Some(ext) if ext == extension),
            Err(_) => true, // pass thru io errors
        }))
}

pub(crate) fn get_confirmation(prompt: &str) -> std::io::Result<bool> {
    loop {
        let response = read_input_from_user(&format!("{prompt}: [Y] / N"))?;
        let response = response.trim();

        if response.is_empty() {
            // user pressed enter
            return Ok(true);
        }
        match response {
            "Y" | "y" => break Ok(true),
            "N" | "n" => break Ok(false),
            _ => {
                println!("Please confirm Y or N");
                continue;
            }
        }
    }
}

pub(crate) fn wait_for_user_before_close(text: &str) {
    println!("{text}");
    println!("<Press Enter to exit>");

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).ok(); // ignore the result
}

#[derive(Debug, Error)]
#[error("Error creating file")]
pub struct FileError;
