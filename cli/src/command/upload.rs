use anyhow::Context;
use clap::Args;

use scut_core::interface::config::ConfigService;
use scut_core::interface::{LocalStorage, RemoteStorage, UserInteraction};
use scut_core::{Config, Key, Setting};

/// Contains the arguments of the upload command.
///
/// [`Upload::run`] will run the upload command.
///
/// See [`crate::command::Command`] for all commands.
#[derive(Debug, Args)]
pub(crate) struct UploadCmd {
    /// Turn number to use when naming the save.
    ///
    /// This will override the turn set in the config.
    ///
    /// If the command is successful, your config's turn will be **replaced**
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,

    /// Force uploading your autosave regardless of whether your teammate's save has been uploaded
    #[arg(short, long)]
    pub(self) force: bool,
}

#[allow(unreachable_code, unused)]
impl UploadCmd {
    pub(crate) fn run(
        self,
        config: &Config,
        config_service: Box<dyn ConfigService>,
        local_storage: Box<dyn LocalStorage>,
        remote_storage: Box<dyn RemoteStorage>,
        mut ui: Box<dyn UserInteraction>,
    ) -> anyhow::Result<()> {
        // TODO: Check that teammate save is unzipped in saves folder
        // if it isn't, then the assumption is that you are playing the turn first and shouldn't upload a next_turn_start save yet!

        let turn = if let Some(turn_override) = self.turn {
            turn_override
        } else {
            config.turn
        };

        let your_saves: Vec<()> = Vec::new();

        if your_saves.is_empty() {
            ui.message("Did not find your save for this turn.");
            ui.message("Create a save before clicking end turn so your teammate can see what you did during your turn.");
            ui.wait_for_user_before_close("Save missing. Nothing has been uploaded. Stopping.");
            return Ok(());
        };

        let found_team_save = false;

        todo!("");

        if !found_team_save {
            ui.message("Did not find a save from your teammate for this turn.");
        }

        // upload saves
        let uploaded_next_save = false;
        todo!();

        let prompt = if uploaded_next_save {
            // increment turn in config to the next turn ready for the next download.
            config
                .set(Key::Turn, Setting::Turn(turn + 1))
                .context("failed to update the turn after successfully uploading a save")?;
            format!("Done. It will be turn {} next", turn + 1)
        } else {
            format!("Ok. It is still turn {turn}")
        };

        ui.wait_for_user_before_close(prompt.as_str());

        Ok(())
    }
}