pub mod toml_file;
pub use toml_file::TomlFileConfig;

use std::fmt;

use crate::Config;

use super::UserInteraction;

/// Config persistence is how the [`Config`] is saved and loaded between usages of scut.
pub trait ConfigPersistence {
    fn save(&mut self, config: Config) -> anyhow::Result<()>;

    fn load(&mut self) -> anyhow::Result<Config>;

    fn deserialize(&self, s: &str) -> anyhow::Result<Config>;

    fn display(&self) -> &dyn fmt::Display;

    fn display_location(&self) -> &dyn fmt::Display;
}

/// Config init is how the [`Config`] is created for the first time.
pub trait ConfigInit {
    fn create_default(&mut self) -> anyhow::Result<Config>;

    fn create_init(&mut self, user_interaction: &mut dyn UserInteraction)
        -> anyhow::Result<Config>;
}
