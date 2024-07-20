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

use super::{AutosavePrediction, AutosavePredictionReason, Predict, Prediction};

#[derive(Debug, Default)]
pub struct SimplePredict;

impl Predict for SimplePredict {
    fn predict(
        &self,
        side: Side,
        player: &str,
        turn_override: Option<u32>,
        playing_solo: bool,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction> {
        let turn = turn_override
            .map(|turn_number| Turn::new(side, turn_number))
            .unwrap_or(self.predict_turn(side, player, playing_solo, local, remote)?);

        let uploads = self.predict_uploads(turn, side, player, playing_solo, local, remote)?;
        let downloads = self.predict_downloads(turn, side, player, playing_solo, local, remote)?;

        let autosave =
            self.predict_autosave(turn, &downloads, side, player, playing_solo, local, remote)?;

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
        playing_solo: bool,
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
            // this happens when playing solo
            (Some(your_turn), None, Some(enemy_turn)) => {
                let mut turn = your_turn.max(enemy_turn);
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
            (None, None, Some(enemy_turn)) => Ok(dbg!(enemy_turn).next()),
        }
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_autosave(
        &self,
        predicted_turn: Turn,
        predicted_downloads: &[Save],
        side: Side,
        player: &str,
        playing_solo: bool,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<AutosavePrediction> {
        let autosave = Save::new(predicted_turn.next());

        let autosave_exists = local.locate_autosave()?.is_some();
        if !autosave_exists {
            return Ok(AutosavePrediction::NotReady(
                autosave,
                AutosavePredictionReason::AutosaveNotAvailable,
            ));
        }

        let query_played_save = Query::new()
            .side(side)
            .turn_number(predicted_turn.number)
            .player(Some(player));

        let have_played_your_turn = local.index().count(&query_played_save)? >= 1;
        if !have_played_your_turn {
            return Ok(AutosavePrediction::NotReady(
                autosave,
                AutosavePredictionReason::TurnNotPlayed(Save::new(predicted_turn).player(player)),
            ));
        }

        let new_teammate_save_available = predicted_downloads
            .iter()
            .filter(|save| {
                save.player.as_ref().is_some_and(|p| p != player) && save.turn.side == side
            })
            .next();

        if let Some(save) = new_teammate_save_available {
            return Ok(AutosavePrediction::NotReady(
                autosave,
                AutosavePredictionReason::NewTeammateSaveAvailable(save.clone()),
            ));
        }

        let query_autosave = Query::new()
            .side(side.other_side())
            .turn_number(predicted_turn.next().number)
            .player(None)
            .part(None);

        let autosave_uploaded_already = remote.index().count(&query_autosave)? >= 1;
        if autosave_uploaded_already {
            return Ok(AutosavePrediction::NotReady(
                autosave,
                AutosavePredictionReason::AutosaveAlreadyUploaded,
            ));
        }

        if playing_solo {
            return Ok(AutosavePrediction::Ready(autosave));
        }

        let query = Query::new()
            .not_player(Some(player))
            .side(predicted_turn.side)
            .turn_number(predicted_turn.number)
            .and(
                Query::new()
                    .not_player(None)
                    .side(predicted_turn.side)
                    .turn_number(predicted_turn.number),
            );

        let friendly_turn = remote.index().search(&query)?;

        remote.index().search(&Query::new())?;

        if friendly_turn.is_empty() {
            Ok(AutosavePrediction::NotReady(
                autosave,
                AutosavePredictionReason::TeammateSaveNotUploaded,
            ))
        } else {
            Ok(AutosavePrediction::Ready(autosave))
        }
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_downloads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        playing_solo: bool,
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
        _playing_solo: bool,
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
}

#[cfg(test)]
mod tests {
    use crate::interface::storage::mock_index_storage::MockIndexStorage;

    use super::*;

    #[test]
    fn simple_predict_works() -> anyhow::Result<()> {
        let predict = SimplePredict;

        let mut remote_storage = MockIndexStorage::new(
            true,
            vec![
                Save::from_parts(Side::Axis, 1),
                Save::from_parts(Side::Axis, 1).player("DM"),
                Save::from_parts(Side::Axis, 1).player("DG"),
                Save::from_parts(Side::Axis, 2),
                Save::from_parts(Side::Allies, 1),
                Save::from_parts(Side::Allies, 1).player("GM"),
                Save::from_parts(Side::Allies, 1).player("TG"),
            ],
        );

        let mut local_storage = MockIndexStorage::new(
            true,
            vec![
                Save::from_parts(Side::Axis, 1),
                Save::from_parts(Side::Axis, 1).player("DG"),
            ],
        );

        let turn = predict.predict_turn(
            Side::Axis,
            "DM",
            true,
            &mut local_storage,
            &mut remote_storage,
        )?;

        assert_eq!(
            predict.predict_downloads(
                turn,
                Side::Axis,
                "DM",
                true,
                &mut local_storage,
                &mut remote_storage
            )?,
            vec![Save::from_parts(Side::Axis, 2),]
        );

        Ok(())
    }
}
