use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};

use crate::error::WriteDefaultConfigError;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) download: PathBuf,
    pub(crate) saves: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) storage: Option<PathBuf>,
    pub(crate) seven_zip_path: PathBuf,
    pub(crate) upload: bool,
}

impl Config {
    pub(crate) fn download(&self) -> &Path {
        &self.download
    }
    pub(crate) fn saves(&self) -> &Path {
        &self.saves
    }
    pub(crate) fn storage(&self) -> &Path {
        self.storage.as_ref().unwrap_or(&self.download)
    }
    pub(crate) fn seven_zip_path(&self) -> &Path {
        &self.seven_zip_path
    }

    pub(crate) fn write_default_config_file() -> Result<Config, Report<WriteDefaultConfigError>> {
        let download = dirs::download_dir()
            .ok_or_else(|| Report::new(WriteDefaultConfigError))
            .attach_printable("Unable to find your downloads folder")?;
        let home = dirs::home_dir()
            .ok_or_else(|| Report::new(WriteDefaultConfigError))
            .attach_printable("Unable to find your documents folder")?;
        let saves = home.join(
            r#"Documents\My Games\Strategic Command WWII - World at War\Multiplayer\Hotseat"#,
        );
        let seven_zip_path = PathBuf::from(r#"C:\Program Files\7-Zip\"#);

        let default_config = Config {
            storage: None,
            download,
            saves,
            upload: false,
            seven_zip_path,
        };

        Ok(default_config)
    }
}
