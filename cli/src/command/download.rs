use clap::Args;

use crate::io_utils::{get_confirmation, wait_for_user_before_close};

#[allow(unused_imports)]
use anyhow::Context;
use scut_core::interface::ConfigPersistence;
use scut_core::{Save, Side};

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
    pub(crate) fn run(self, config: &mut dyn ConfigPersistence) -> anyhow::Result<()> {
        let config = config.load()?;

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

        if get_confirmation("Is that OK?") {
            todo!("download");

            wait_for_user_before_close("Done");
        } else {
            wait_for_user_before_close("User cancelled. Stopping.");
        }

        Ok(())
    }
}
