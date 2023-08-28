//! The names of these Predict implementations are faily arbitrary!
//!
//! This Predict implementation implements `predict_turn`, using the friendly turns already available in RemoteStorage to determine what turn it must be.

use compose::{And, Or};
use tracing::instrument;

use crate::{
    interface::{index::Query, LocalStorage, RemoteStorage},
    Save, Side, Turn,
};

#[cfg(test)]
mod ddt;

use super::{Predict, Prediction};

#[derive(Debug, Default)]
pub struct SimplePredict {
    /// Used to help predict if autosave needs uploading
    predicted_download_count: usize,
}

impl Predict for SimplePredict {
    fn predict(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction> {
        let turn = self.predict_turn(side, player, local, remote)?;

        let uploads = self.predict_uploads(turn, side, player, local, remote)?;
        let downloads = self.predict_downloads(turn, side, player, local, remote)?;

        let autosave = self.predict_autosave(turn, side, player, local, remote)?;

        Ok(Prediction {
            autosave,
            uploads,
            downloads,
        })
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_turn(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Turn> {
        let _local_index = local.index();
        let remote_index = remote.index();

        let enemy_side = side.other_side();

        let your_last_uploaded_turn = remote_index
            .latest(&Query::new().side(side).player(Some(player)))?
            .map(|s| s.turn);

        let last_friendly_start_turn = remote_index
            .latest(&Query::new().side(side).player(None))?
            .map(|s| s.turn);

        let last_enemy_start_turn = remote_index
            .latest(&Query::new().side(enemy_side).player(None))?
            .map(|s| s.turn);

        match (
            your_last_uploaded_turn,
            last_friendly_start_turn,
            last_enemy_start_turn,
        ) {
            // this happens on turn 1 Axis, before uploading anything
            (None, None, None) => Ok(Turn::new(Side::Axis, 1)),
            // this happens on turn 1 Axis, after you've uploaded your turn
            (Some(your_turn), None, None) => Ok(your_turn),

            // this happens on turn 1 Allies, before uploading anything
            (None, Some(friendly_turn), None) => Ok(friendly_turn),
            // this happens on turn 1 Allies, after you've uploaded your turn
            (Some(your_turn), Some(friendly_turn), None) => Ok(your_turn.max(friendly_turn)),

            // this happens most turns - we need to look at the turns to work out which is the latest turn
            (Some(your_turn), Some(friendly_turn), Some(enemy_turn)) => {
                let mut turn = your_turn.max(friendly_turn).max(enemy_turn);
                if turn.side == enemy_side {
                    turn.side = side;
                }
                Ok(turn)
            }

            // this is unlikely in real scenarios
            (None, Some(friendly_turn), Some(enemy_turn)) => {
                let mut turn = friendly_turn.max(enemy_turn);
                if turn.side == enemy_side {
                    turn.side = side;
                }
                Ok(turn)
            }
            // this is unlikely in real scenarios
            (Some(your_turn), None, Some(enemy_turn)) => {
                let mut turn = your_turn.max(enemy_turn);
                if turn.side == enemy_side {
                    turn.side = side;
                }
                Ok(turn)
            }
            // this is highly unlikely in real scenarios
            (None, None, Some(enemy_turn)) => Ok(enemy_turn.next()),
        }
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_autosave(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<Save>> {
        if self.predicted_download_count > 0 {
            return Ok(None);
        }

        let autosave_exists = local.locate_autosave()?.is_some();
        if !autosave_exists {
            return Ok(None);
        }

        let query = Query::new()
            .side(side.other_side())
            .turn_number(predicted_turn.next().number)
            .player(None)
            .part(None);

        remote.index().search(&query)?;

        let autosave_uploaded_already = remote.index().count(&query)? >= 1;
        if autosave_uploaded_already {
            return Ok(None);
        }

        Ok(Some(Save::new(
            predicted_turn.side.other_side(),
            predicted_turn.next().number,
        )))
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_downloads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .side(side)
            .turn_number_in_range(Some(predicted_turn.number - 1), Some(predicted_turn.number));

        let our_save = Query::new().player(Some(player));
        let turn_start_save = Query::new().player(None).turn(predicted_turn);
        let teammate_save = query
            .clone()
            .not_player(Some(player))
            .and(query.not_player(None));

        let query = teammate_save.or(turn_start_save);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let mut our_played_saves = remote.index().search(&our_save)?;
        our_played_saves.extend(local.index().search(&our_save)?);

        let saves_to_download: Vec<_> = remote_saves
            .into_iter()
            // don't download saves you already have
            .filter(|s| !local_saves.contains(s))
            // don't download turn start saves for turns you've already played
            .filter(|s| {
                if s.player.is_none() && s.turn.side == side {
                    // check if we've played this turn
                    !our_played_saves
                        .iter()
                        .any(|our_save| our_save.turn == s.turn)
                } else {
                    true
                }
            })
            .collect();

        Ok(saves_to_download)
    }

    fn predict_uploads(
        &self,
        predicted_turn: Turn,
        _side: Side,
        _player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new().min_turn(predicted_turn);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let missing_remote_saves = local_saves
            .into_iter()
            .filter(|s| !remote_saves.contains(s))
            .collect();

        Ok(missing_remote_saves)
    }

    fn should_upload_autosave(
        &self,
        predicted_autosave: &Option<Save>,
        _side: Side,
        _player: &str,
        predicted_downloads: &[Save],
        predicted_uploads: &[Save],
    ) -> bool {
        predicted_autosave.is_some() && (predicted_downloads.len() + predicted_uploads.len() > 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::storage::mock_index_storage::MockIndexStorage;

    use super::*;

    #[test]
    fn simple_predict_works() -> anyhow::Result<()> {
        let predict = SimplePredict::default();

        let mut remote_storage = MockIndexStorage::new(
            true,
            vec![
                Save::new(Side::Axis, 1),
                Save::new(Side::Axis, 1).player("DM"),
                Save::new(Side::Axis, 1).player("DG"),
                Save::new(Side::Axis, 2),
                Save::new(Side::Allies, 1),
                Save::new(Side::Allies, 1).player("GM"),
                Save::new(Side::Allies, 1).player("TG"),
            ],
        );

        let mut local_storage = MockIndexStorage::new(
            true,
            vec![
                Save::new(Side::Axis, 1),
                Save::new(Side::Axis, 1).player("DG"),
            ],
        );

        let turn =
            predict.predict_turn(Side::Axis, "DM", &mut local_storage, &mut remote_storage)?;

        assert_eq!(
            predict.predict_downloads(
                turn,
                Side::Axis,
                "DM",
                &mut local_storage,
                &mut remote_storage
            )?,
            vec![
                // Save::new(Side::Axis, 1).player("DM"),  // we don't predict downloading our own previous turns anymore
                Save::new(Side::Axis, 2),
            ]
        );

        Ok(())
    }
}
