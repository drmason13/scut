use clap::Args;

#[allow(unused_imports)]
use anyhow::Context;
use scut_core::{
    interface::{config::ConfigService, LocalStorage, RemoteStorage, UserInteraction},
    Config, Save, Side,
};

#[derive(Debug, Args)]
pub(crate) struct DownloadCmd {
    /// Turn number to download.
    ///
    /// This will override the turn set in the config.
    ///
    /// If the command is successful, your config's turn will be **replaced**
    #[arg(short, long)]
    pub(crate) turn: Option<u32>,
}

#[allow(unreachable_code, unused)]
impl DownloadCmd {
    pub(crate) fn run(
        self,
        config: &mut Config,
        config_service: Box<dyn ConfigService>,
        local_storage: Box<dyn LocalStorage>,
        remote_storage: Box<dyn RemoteStorage>,
        mut ui: Box<dyn UserInteraction>,
    ) -> anyhow::Result<()> {
        let turn = if let Some(turn_override) = self.turn {
            turn_override
        } else {
            config.turn
        };

        let start_save: Option<Save> = None;
        let team_saves = Vec::<()>::new();
        let count_of_team_saves = team_saves.len();

        // on the first turn, Axis (who go first), don't need to download a turn start save
        // but they might need to download a teammate's save!
        let is_very_first_turn = turn == 1 && config.side == Side::first();

        todo!("download");

        if ui.confirm("Is that OK?", Some(true)) {
            anyhow::bail!("I put this error here to see what would happen");

            ui.wait_for_user_before_close("Done");
        } else {
            ui.wait_for_user_before_close("User cancelled. Stopping.");
        }

        Ok(())
    }
}
