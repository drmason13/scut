//! The compression module contains the [`Compression`] interface for compressing and decompressing files.
//!
//! Implementations of Compression live in submodules.
//!
//! Compression is used to save space in remote storage and increase the speed of uploads and downloads.

mod seven_zip;
pub use seven_zip::SevenZipCompression;

use dyn_clone::DynClone;

#[cfg(test)]
pub mod mock_compression;

use std::path::Path;

pub trait Compression: DynClone + Send + Sync {
    /// Compress the file located at `from` and save the resulting compressed file at `to`.
    ///
    /// * `from` must be a full path including filename and extension.
    /// * `to` must include the filename and must not include the extension.
    ///
    /// The implementation will add the specific extension if needed.
    fn compress(&self, from: &Path, to: &Path) -> anyhow::Result<()>;

    /// Decompress the file located at `from` and save the resulting decompressed file at `to`.
    ///
    /// * `to` must include the filename and must not include the extension.
    /// * `from` must be a full path including filename and extension.
    ///
    /// The implementation will add the specific extension if needed.
    fn decompress(&self, from: &Path, to: &Path) -> anyhow::Result<()>;
}

impl Clone for Box<dyn Compression> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
