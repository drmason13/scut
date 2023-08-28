pub mod toml_file;
pub use toml_file::TomlFileConfig;

use std::path::PathBuf;

use crate::Config;

pub trait ConfigPersistence {
    /// Config persistence is how the [`Config`] is saved and loaded between usages of scut.
    fn save(&mut self, config: &Config) -> anyhow::Result<()>;

    fn load(&mut self) -> anyhow::Result<Option<Config>>;

    fn default_location(&self) -> anyhow::Result<PathBuf>;
}

/// Config init is how the [`Config`] is created for the first time.
pub trait ConfigInit {
    fn init_config(&mut self) -> anyhow::Result<Config>;
}

pub trait ConfigService: ConfigPersistence + ConfigInit {}
