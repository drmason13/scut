//! The names of these Prediction implementations are faily arbitrary!
//!
//! This Prediction implementation implements `predict_turn`, using the friendly turns already available in RemoteStorage to determine what turn it must be.

use tracing::{debug, instrument};

use crate::{
    interface::{index::Query, LocalStorage, RemoteStorage},
    Save, Side,
};

#[cfg(test)]
mod ddt;

use super::Prediction;

#[derive(Debug, Default)]
pub struct SimplePrediction;

impl SimplePrediction {
    fn count_predicted_downloads(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<usize> {
        Ok(self
            .predict_downloads(turn, side, player, local, remote)?
            .len())
    }
}

impl Prediction for SimplePrediction {
    #[instrument(
        level = "DEBUG",
        skip(self, _local, remote),
        fields(local = "<ignored>")
    )]
    fn predict_turn(
        &self,
        side: Side,
        player: &str,
        _local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<u32>> {
        let remote_index = remote.index();

        // the latest save that we uploaded
        let query = &Query::new().side(side).player(Some(player));

        let turn = if let Some(save) = remote_index.latest(query)? {
            let predicted_turn = save.turn;

            debug!(%save, predicted_turn);

            predicted_turn
        } else {
            // we'll try to fetch the next turn, so we actually want to think it's turn 0 for the first turn
            let predicted_turn = 0;

            debug!(predicted_turn, "no friendly save found in remote");

            predicted_turn
        };

        Ok(Some(turn))
    }

    #[instrument(
        level = "DEBUG",
        skip(self, _player, local, remote),
        fields(player = "<ignored>")
    )]
    fn predict_downloads(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .side(side)
            .not_player(None)
            .turn_in_range(Some(turn), None)
            .or_side(side)
            .or_player(None)
            .or_turn(turn + 1);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let saves_to_download: Vec<_> = remote_saves
            .into_iter()
            .filter(|s| !local_saves.contains(s))
            .collect();

        Ok(saves_to_download)
    }

    #[instrument(
        level = "DEBUG",
        skip(self, _player, local, remote),
        fields(player = "<ignored>")
    )]
    fn predict_uploads(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new().side(side).turn_in_range(Some(turn), None);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let missing_remote_saves = local_saves
            .into_iter()
            .filter(|s| !remote_saves.contains(s))
            .collect();

        Ok(missing_remote_saves)
    }

    #[instrument(level = "DEBUG", skip(self, local, remote))]
    fn predict_autosave(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<(crate::Save, Option<bool>)> {
        let autosave_exists = local.locate_autosave()?.is_some();

        let enemy_side = side.other_side();
        let enemy_turn = side.next_turn(turn);

        debug!(autosave_exists, %enemy_side, enemy_turn);

        let download_count = self.count_predicted_downloads(turn, side, player, local, remote)?;

        debug!(download_count);

        let autosave_uploaded_already = remote.index().count(
            &Query::new()
                .side(enemy_side)
                .turn(enemy_turn)
                .player(None)
                .part(None),
        )? >= 1;

        let upload_autosave_as = Save {
            turn: enemy_turn,
            side: enemy_side,
            player: None,
            part: None,
        };

        let should_upload_autosave =
            Some(autosave_exists && download_count == 0 && !autosave_uploaded_already);

        debug!(autosave_uploaded_already, %upload_autosave_as, should_upload_autosave);

        Ok((upload_autosave_as, should_upload_autosave))
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::storage::mock_index_storage::MockIndexStorage;

    use super::*;

    #[test]
    fn simple_prediction_works() -> anyhow::Result<()> {
        let prediction = SimplePrediction;

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

        assert_eq!(
            prediction.predict_downloads(
                prediction
                    .predict_turn(Side::Axis, "DG", &mut local_storage, &mut remote_storage)?
                    .unwrap(),
                Side::Axis,
                "DG",
                &mut local_storage,
                &mut remote_storage
            )?,
            vec![
                Save::new(Side::Axis, 1).player("DM"),
                Save::new(Side::Axis, 2),
            ]
        );

        Ok(())
    }
}
