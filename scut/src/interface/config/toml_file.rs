use std::{cell::RefCell, ops::DerefMut, path::PathBuf, time::Duration};

use anyhow::Context;

use crate::error::ErrorSuggestions;
use crate::interface::{
    user_interaction::query_and_parse, ConfigPersistence, FileSystem, UserInteraction,
};

use super::Config;

pub struct TomlFileConfig {
    location: PathBuf,
    file_system: Box<dyn FileSystem>,
    user_interaction: RefCell<Box<dyn UserInteraction>>,
    config: Option<Config>,
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
            user_interaction: RefCell::new(user_interaction),
            config: None,
        }
    }

    fn load_config_from_disk(&mut self) -> anyhow::Result<&Config> {
        let content = self.file_system.read_file_to_string(&self.location)?;
        self.config.replace(self.deserialize(&content)?);
        // unwrap: we just replaced our on the line config above
        Ok(self.config.as_ref().unwrap())
    }

    // TODO: abstract config defaults into an interface
    fn create_default_config_file(&mut self) -> anyhow::Result<()> {
        let dropbox_result = dropbox_dir::personal_dir();

        let dropbox = match dropbox_result {
            Ok(dropbox) => dropbox,
            Err(_) => {
                todo!()
                // .ok_or_else(|| anyhow::anyhow!("Dropbox folder configuration is missing"))?
            }
        }
        .into();

        let home = dirs::home_dir().context("Unable to find your documents folder")?;
        let saves = home.join(
            r#"Documents\My Games\Strategic Command WWII - World at War\Multiplayer\Hotseat"#,
        );
        let seven_zip_path = PathBuf::from(r#"C:\Program Files\7-Zip\"#);
        let config_path = dirs::config_dir()
            .context("Unable to find your documents folder")?
            .join("scut")
            .join("config.toml");

        let side = query_and_parse(
            "What side will you be playing as?",
            &mut **self.user_interaction.borrow_mut(),
        )
        .ok_or_else(|| anyhow::anyhow!("no side provided"))
        .suggest("Decide which side to play as and try again")?;

        let player = self
            .user_interaction
            .borrow_mut()
            .query("How do you want to sign your saves?");

        let turn = query_and_parse::<u32>(
            "What turn are you on?",
            &mut **self.user_interaction.borrow_mut(),
        )
        .ok_or_else(|| anyhow::anyhow!("no side provided"))
        .suggest("Find out which turn you are on and try again")?;

        let default_config = Config {
            dropbox,
            saves,
            seven_zip_path,
            side,
            player,
            turn,
        };

        self.save(default_config)?;

        self.user_interaction
            .borrow_mut()
            .message(&format!("New config written to {}", config_path.display()));
        Ok(())
    }
}

impl ConfigPersistence for TomlFileConfig {
    fn save(&mut self, config: Config) -> anyhow::Result<()> {
        let config_toml = toml::to_string_pretty(&config).context("failed to save config file")?;

        self.config.replace(config);

        let mut attempt = 0;
        loop {
            match self
                .file_system
                .write_string_to_file(&config_toml, &self.location)
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

    fn load(&mut self) -> anyhow::Result<&Config> {
        match self.config {
            None => self
                .load_config_from_disk()
                .context("failed to load config file"),
            Some(_) => Ok(self.config.as_ref().unwrap()),
        }
    }

    fn deserialize(&self, toml_string: &str) -> anyhow::Result<Config> {
        toml::from_str(toml_string).context("failed to parse config file")
    }

    fn display(&self) -> &dyn std::fmt::Display {
        todo!()
    }

    fn display_location(&self) -> &dyn std::fmt::Display {
        todo!()
    }

    fn create_default(&mut self) -> anyhow::Result<&Config> {
        self.create_default_config_file()?;
        self.load()
    }
}
/*
fn ask_player_for_a_side() -> Side {
    loop {
        let side = read_input_from_user("What side will you be playing as?");

        let side: Result<Side, _> = side.parse();

        match side {
            Ok(side) => break side,
            Err(_) => {
                println!("The valid sides are 'Axis' and 'Allies', please enter one of those");
                continue;
            }
        };
    }
}

fn ask_player_for_a_name() -> String {
    loop {
        let player = read_input_from_user();
        let player = player.trim();

        if player.is_empty() {
            println!("A player signature is needed so people know which saves are yours");
            continue;
        }
        break player.to_string();
    }
}

fn ask_player_for_a_turn() -> u32 {
    loop {
        let turn = read_input_from_user("What turn are you on?");
        let turn = turn.trim();

        if turn.is_empty() {
            println!("A turn is needed to know which save to download/upload next");
            continue;
        }
        match turn.parse() {
            Ok(turn) => break turn,
            Err(_) => {
                println!("That's not a valid turn number, please enter a positive integer");
                continue;
            }
        };
    }
}

fn ask_player_for_dropbox_folder() -> Option<String> {
    println!("Unable to find your dropbox folder");
    println!("You may not have the dropbox client installed. This is required to use scut.");
    println!("If you have installed the dropbox client, then you can enter your dropbox folder to continue.");
    if !get_confirmation("Would you like to enter your dropbox folder?") {
        return None;
    }
    loop {
        let dropbox = read_input_from_user("Please enter the absolute path to your dropbox folder");
        let dropbox = dropbox.trim();

        if dropbox.is_empty() {
            println!("That's not a valid path");
            continue;
        }
        match PathBuf::from_str(dropbox) {
            // TODO: check path exists, is absolute and valid and can be read before returning it!
            Ok(dropbox) => match std::fs::read_dir(&dropbox) {
                Ok(_) => break Some(dropbox.to_string_lossy().to_string()),
                Err(_) => {
                    println!("scut wasn't able to list the contents of that folder, which means scut is unlikely to work properly.");
                    if get_confirmation("Would you still like to use that folder?") {
                        break Some(dropbox.to_string_lossy().to_string());
                    } else {
                        continue;
                    }
                }
            },
            Err(_) => {
                println!("That path doesn't appear to be a folder that scut is able to read, which means scut is unlikely to work properly.");
                if get_confirmation("Would you still like to use that path?") {
                    break Some(dropbox.to_string());
                } else {
                    continue;
                }
            }
        };
    }
}

impl TomlFileConfig {
    /*
        pub fn save(&self) -> anyhow::Result<()> {
            let config_toml = toml::to_string_pretty(&self).context("failed to save config file")?;

            write_string_to_file(config_toml, &self.path).context("failed to save config file")?;

            Ok(())
        }

        pub fn from_toml(toml_string: &str, config_path: &Path) -> anyhow::Result<Config> {
            let mut config: Config =
                toml::from_str(toml_string).context("failed to parse config file")?;
            config.path = config_path.into();
            Ok(config)
        }

        pub(crate) fn file_path() -> anyhow::Result<PathBuf> {
            Ok(dirs::config_dir()
                .context("failed to locate system config directory")?
                .join("scut")
                .join("config.toml"))
        }

        pub(crate) fn read_config_file(config_path: Option<PathBuf>) -> anyhow::Result<Config> {
            let config_path = match config_path {
                Some(config_path) => config_path,
                None => Config::file_path()?,
            };

            let file_result = std::fs::read_to_string(&config_path);

            match file_result {
                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "No config file found.\nWriting default config file to {}",
                        config_path.display()
                    );
                    let default_config = Config::write_default_config_file(&config_path).context(
                        "Attempted to create a default config for you but there was a problem",
                    )?;
                    Ok(default_config)
                }
                Err(e) => Err(e).context("Unexpected error while reading config file"),
                Ok(ref config_content) => Config::from_toml(config_content, &config_path),
            }
        }
    }

    impl Display for Config {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                toml::to_string_pretty(self).unwrap_or_else(|_| format!("{self:?}"))
            )
        }
    }
    */
}
 */
