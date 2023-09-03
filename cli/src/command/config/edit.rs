use anyhow::Context;
use scut_core::{
    interface::{config::ConfigService, UserInteraction},
    Config,
};

pub fn edit(
    config: Config,
    mut config_service: Box<dyn ConfigService>,
    ui: &mut dyn UserInteraction,
) -> anyhow::Result<()> {
    let new_string = loop {
        match edit::edit(config_service.serialize(&config)?) {
            Ok(new_string) => break new_string,
            Err(io_err) if io_err.kind() == std::io::ErrorKind::InvalidData => {
                println!("The edited config was not valid UTF-8");
                println!("Your changes have not been saved.");

                if ui.confirm(
                    "Would you like to try and edit the config again?",
                    Some(true),
                ) {
                    continue;
                } else {
                    ui.wait_for_user_before_close("Config was not updated. Exiting.");
                }
            }
            Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                ui.message("Unable to find an editor to edit the config");
                ui.wait_for_user_before_close("You can edit the config from the commandline using `scut config set KEY VALUE`");
                return Ok(());
            }
            Err(e) => return Err(e).context("failed to open an editor to edit the config"),
        }
    };

    let new_config = loop {
        match config_service.deserialize(new_string.as_str()) {
            Ok(config) => break config,
            Err(e) => {
                println!("Invalid config: {e}");
                println!("Your changes have not been saved.");

                if ui.confirm("Would you like to try and edit the config again?", None) {
                    continue;
                } else {
                    ui.wait_for_user_before_close(
                        "User has abandoned editing the config. Exiting.",
                    );
                    return Ok(());
                }
            }
        }
    };

    config_service
        .save(&new_config)
        .context("failed to save changes to config")?;

    println!("Config was updated successfully");

    Ok(())
}
