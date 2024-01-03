use std::str::FromStr;
use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use tracing::instrument;

use crate::error::ErrorSuggestions;
use crate::interface::file_system::is_not_found_err;
use crate::interface::{
    user_interaction::query_and_parse, ConfigPersistence, FileSystem, UserInteraction,
};

use super::{Config, ConfigInit, ConfigService};

pub struct TomlFileConfig {
    location: PathBuf,
    file_system: Box<dyn FileSystem>,
    user_interaction: Box<dyn UserInteraction>,
}

impl TomlFileConfig {
    pub fn new(
        location: PathBuf,
        file_system: Box<dyn FileSystem>,
        user_interaction: Box<dyn UserInteraction>,
    ) -> Self {
        TomlFileConfig {
            location,
            file_system,
            user_interaction,
        }
    }

    #[instrument(skip_all, ret, err)]
    pub fn default_location() -> anyhow::Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("scut").join("config.toml"))
            .context("failed to find your system config folder")
    }

    #[instrument(skip_all, ret, err)]
    fn load_config_from_disk(&mut self) -> anyhow::Result<Option<Config>> {
        let result = self.file_system.read_file_to_string(&self.location);
        let toml_string = match result {
            Err(e) if is_not_found_err(&e) => return Ok(None),
            Ok(ok) => ok,
            Err(e) => return Err(e),
        };

        let config = toml::from_str(&toml_string)
            .suggest("Your config file may be corrupted, move the config file and try again to create a new config file")
            .context("failed to parse config file")?;

        Ok(Some(config))
    }

    #[instrument(skip_all, ret, err)]
    fn save_config_to_disk(&mut self, config: &Config) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(config).context("failed to save config file")?;

        let mut attempt = 0;
        loop {
            match self
                .file_system
                .write_string_to_file(&content, &self.location)
            {
                Ok(ok) => return Ok(ok),
                Err(e) if attempt > 1 => return Err(e).context("failed to save config file"),
                _ => {
                    attempt += 1;
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }
            }
        }
    }

    // TODO: abstract config defaults into an interface
    #[instrument(skip_all, ret, err)]
    fn init_config(&mut self) -> anyhow::Result<Config> {
        let ui = &mut *self.user_interaction;

        let dropbox_result = dropbox_dir::personal_dir();

        let dropbox = match dropbox_result {
            Ok(dropbox) => dropbox,
            Err(_) => ask_player_for_dropbox_folder(ui)
                .context("Dropbox folder configuration is missing")?,
        }
        .into();

        let home = dirs::home_dir().context("Unable to find your documents folder")?;
        let saves = home
            .join(r"Documents\My Games\Strategic Command WWII - World at War\Multiplayer\Hotseat");
        let seven_zip_path = PathBuf::from(r"C:\Program Files\7-Zip\");

        let side = query_and_parse("What side will you be playing as?", ui)
            .ok_or_else(|| anyhow::anyhow!("no side provided"))
            .suggest("Decide which side to play as and try again")?;

        let player = ui.query("How do you want to sign your saves?");

        let turn = None;

        Ok(Config {
            dropbox,
            saves,
            seven_zip_path,
            side,
            player,
            turn,
        })
    }
}

impl ConfigPersistence for TomlFileConfig {
    #[instrument(skip_all, ret, err)]
    fn save(&mut self, config: &Config) -> anyhow::Result<()> {
        self.save_config_to_disk(config)
    }

    #[instrument(skip_all, ret, err)]
    fn load(&mut self) -> anyhow::Result<Option<Config>> {
        self.load_config_from_disk()
            .context("failed to load config file")
    }

    #[instrument(skip_all, ret, err)]
    fn serialize(&self, config: &Config) -> anyhow::Result<String> {
        toml::to_string_pretty(config).context("failed to save config file")
    }

    #[instrument(skip_all, ret, err)]
    fn deserialize(&self, s: &str) -> anyhow::Result<Config> {
        toml::from_str(s).context("failed to parse config file")
    }

    #[instrument(level = "TRACE", skip_all, ret, err)]
    fn location(&self) -> anyhow::Result<String> {
        Ok(self
            .location
            .as_path()
            .as_os_str()
            .to_string_lossy()
            .to_string())
    }
}

impl ConfigInit for TomlFileConfig {
    #[instrument(skip_all, ret, err)]
    fn init_config(&mut self) -> anyhow::Result<Config> {
        let config = self.init_config()?;
        self.save(&config)?;

        self.user_interaction.message(&format!(
            "New config written to {}",
            self.location.display()
        ));

        Ok(config)
    }
}

impl ConfigService for TomlFileConfig {}

#[instrument(skip_all, ret)]
fn ask_player_for_dropbox_folder(ui: &mut dyn UserInteraction) -> Option<String> {
    ui.message("Unable to find your dropbox folder");
    ui.message("You may not have the dropbox client installed. This is required to use scut.");
    ui.message("If you have installed the dropbox client, then you can enter your dropbox folder to continue.");
    if !ui.confirm("Would you like to enter your dropbox folder?", Some(true)) {
        return None;
    }
    loop {
        let dropbox = ui.query("Please enter the absolute path to your dropbox folder");

        if dropbox.is_empty() {
            ui.message("That's not a valid path");
            continue;
        }
        match PathBuf::from_str(&dropbox) {
            // TODO: check path exists, is absolute and valid and can be read before returning it!
            Ok(dropbox) => match std::fs::read_dir(&dropbox) {
                Ok(_) => break Some(dropbox.to_string_lossy().to_string()),
                Err(_) => {
                    ui.message("scut wasn't able to list the contents of that folder, which means scut is unlikely to work properly.");
                    if ui.confirm("Would you still like to use that folder?", None) {
                        break Some(dropbox.to_string_lossy().to_string());
                    } else {
                        continue;
                    }
                }
            },
            Err(_) => {
                ui.message("That path doesn't appear to be a folder that scut is able to read, which means scut is unlikely to work properly.");
                if ui.confirm("Would you still like to use that path?", Some(false)) {
                    break Some(dropbox.to_string());
                } else {
                    continue;
                }
            }
        };
    }
}
