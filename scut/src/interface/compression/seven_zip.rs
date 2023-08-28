use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::output_error;

use anyhow::Context;

use super::Compression;

/// An implementation of Compression using 7z
pub struct SevenZipCompression {
    seven_zip_path: PathBuf,
}

impl SevenZipCompression {
    pub fn new(seven_zip_path: PathBuf) -> Self {
        SevenZipCompression { seven_zip_path }
    }
}

impl Compression for SevenZipCompression {
    fn compress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let mut command = Command::new("7z");
        command
            .env("PATH", self.seven_zip_path.as_os_str())
            .arg("a")
            .arg(to)
            .arg(from);

        let output = command.output().with_context(|| "failed to run 7zip")?;

        output_error(&output)
            .with_context(|| {
                format!(
                    "{:?} returned unsuccessful status code: {}",
                    command, output.status
                )
            })
            .with_context(|| format!("failed to compress {} to {}", from.display(), to.display()))
    }

    fn decompress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let mut command = Command::new("7z");
        command
            .env("PATH", self.seven_zip_path.as_os_str())
            .arg("e")
            .arg(from)
            .arg(format!("-o{}", to.display()))
            .arg("-y");

        let output = command.output().with_context(|| "failed to run 7zip")?;

        output_error(&output)
            .with_context(|| {
                format!(
                    "{:?} returned unsuccessful status code: {}",
                    command, output.status
                )
            })
            .with_context(|| {
                format!(
                    "failed to decompress {} to {}",
                    from.display(),
                    to.display()
                )
            })?;

        Ok(())
    }
}
