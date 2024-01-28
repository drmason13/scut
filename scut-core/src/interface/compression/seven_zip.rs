use std::{
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::{output_error, path::ErrorPaths};

use anyhow::Context;
use tracing::{debug, instrument};

const CREATE_NO_WINDOW: u32 = 0x08000000;

use super::Compression;

/// An implementation of Compression using 7z
#[derive(Debug, Clone)]
pub struct SevenZipCompression {
    seven_zip_path: PathBuf,
}

impl SevenZipCompression {
    pub fn new(seven_zip_path: &Path) -> Self {
        SevenZipCompression {
            seven_zip_path: seven_zip_path.to_path_buf(),
        }
    }
}

impl Compression for SevenZipCompression {
    #[instrument(skip_all, ret, err)]
    fn compress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let path = &self.seven_zip_path;

        debug!(?from, ?to, PATH = path.to_str(), "compressing");

        let mut command = Command::new("7z");
        command
            .creation_flags(CREATE_NO_WINDOW)
            .env("PATH", path.as_os_str())
            .arg("a")
            .arg(to)
            .arg(from);

        let output = command
            .output()
            // Assumption: user is running windows and should have a 7z.exe file
            .path(path.join("7z.exe"))
            .with_context(|| "failed to run 7zip")?;

        output_error(&output)
            .with_context(|| {
                format!(
                    "`{:?}` returned unsuccessful status code: '{}'",
                    command, output.status
                )
            })
            .with_context(|| {
                format!(
                    "failed to compress '{}' to '{}'",
                    from.display(),
                    to.display()
                )
            })
    }

    #[instrument(skip_all, ret, err)]
    fn decompress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let path = &self.seven_zip_path;

        debug!(?from, ?to, PATH = path.to_str(), "decompressing");

        let mut command = Command::new("7z");
        command
            .env("PATH", path.as_os_str())
            .arg("e")
            .arg(from)
            .arg(format!("-o{}", to.display()))
            .arg("-y");

        let output = command.output().with_context(|| "failed to run 7zip")?;

        output_error(&output)
            .with_context(|| {
                format!(
                    "`{:?}` returned unsuccessful status code: '{}'",
                    command, output.status
                )
            })
            .with_context(|| {
                format!(
                    "failed to decompress '{}' to '{}'",
                    from.display(),
                    to.display()
                )
            })?;

        Ok(())
    }
}
