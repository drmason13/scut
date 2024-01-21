pub mod toml_file;

use crate::Config;

/// Config persistence is how the [`Config`] is saved and loaded between usages of scut.
pub trait ConfigPersistence {
    /// Save the config, storing it for future retrieval
    fn save(&mut self, config: &Config) -> anyhow::Result<()>;

    /// Load the stored config and return it, if it exists
    fn load(&mut self) -> anyhow::Result<Option<Config>>;

    /// Return the string represention of the given config
    fn serialize(&self, config: &Config) -> anyhow::Result<String>;

    /// Return the Config struct represented by the given string
    fn deserialize(&self, s: &str) -> anyhow::Result<Config>;

    /// Return a string describing the location of the stored config
    fn location(&self) -> anyhow::Result<String>;
}

/// Config init is how the [`Config`] is created for the first time.
pub trait ConfigInit {
    fn init_config(&mut self) -> anyhow::Result<Config>;
}

pub trait ConfigService: ConfigPersistence + ConfigInit {}
