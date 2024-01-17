use std::path::Path;

use tracing::instrument;

use super::Compression;

#[derive(Debug, Clone)]
pub struct MockCompression;

impl MockCompression {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        MockCompression
    }
}

#[allow(unused_variables)]
impl Compression for MockCompression {
    #[instrument(skip(self), err)]
    fn compress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        // let content = match self.file_system.get_file_content(from) {
        //     Ok(Some(content)) => content,
        //     Ok(None) => anyhow::bail!(
        //         "failed to mock compress '{}', file does not exist",
        //         from.display()
        //     ),
        //     Err(e) => anyhow::bail!(e),
        // };

        // self.file_system
        //     .add_file(to.to_path_buf(), Status::Exists, Some(content.clone()));
        // self.file_system.set_file_content(to, content.clone())?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    fn decompress(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        // let content = match self.file_system.get_file_content(from) {
        //     Ok(Some(content)) => content,
        //     Ok(None) => anyhow::bail!(
        //         "failed to mock decompress '{}', file does not exist",
        //         from.display()
        //     ),
        //     Err(e) => anyhow::bail!(e),
        // };

        // self.file_system
        //     .add_file(to.to_path_buf(), Status::Exists, Some(content.clone()));
        // self.file_system.set_file_content(to, content.clone())?;

        Ok(())
    }
}
