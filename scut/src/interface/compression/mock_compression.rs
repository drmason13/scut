use std::path::Path;

use super::Compression;

pub struct MockCompression;

impl MockCompression {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        MockCompression
    }
}

#[allow(unused_variables)]
impl Compression for MockCompression {
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