//! The names of these Prediction implementations are faily arbitrary!
//!
//! This Prediction implementation implements `predict_turn`, using the friendly turns already available in RemoteStorage to determine what turn it must be.

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
    fn predict_turn(
        &self,
        friendly_side: Side,
        _player: &str,
        _local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<u32>> {
        let remote_index = remote.index();
        let turn = if let Some(save) = remote_index.latest_save(friendly_side)? {
            save.turn
        } else {
            1
        };

        Ok(Some(turn))
    }

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
            .turn_in_range(Some(turn.saturating_sub(1)), None);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let missing_local_saves = remote_saves
            .into_iter()
            .filter(|s| !local_saves.contains(s))
            .collect();

        Ok(missing_local_saves)
    }

    fn predict_uploads(
        &self,
        turn: u32,
        side: Side,
        _player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .side(side)
            .turn_in_range(Some(turn.saturating_sub(1)), None);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let missing_remote_saves = local_saves
            .into_iter()
            .filter(|s| !remote_saves.contains(s))
            .collect();

        Ok(missing_remote_saves)
    }

    fn predict_autosave(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<(crate::Save, Option<bool>)> {
        let enemy_side = side.other_side();
        let enemy_turn = side.next_turn(turn);

        let download_count = self.count_predicted_downloads(turn, side, player, local, remote)?;

        let autosave_exists = local.locate_autosave()?.is_some();

        let autosave_uploaded_already = remote.index().count(
            &Query::new()
                .side(enemy_side)
                .turn(enemy_turn)
                .player(None)
                .part(None),
        )? >= 1;

        Ok((
            Save {
                turn: enemy_turn,
                side: enemy_side,
                player: None,
                part: None,
            },
            Some(autosave_exists && download_count == 0 && !autosave_uploaded_already),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::storage::mock_index_storage::MockIndexStorage;

    use super::*;

    #[test]
    fn simple_prediction_works() -> anyhow::Result<()> {
        let prediction = SimplePrediction;

        let mut remote_storage = MockIndexStorage::new(vec![
            Save::new(Side::Axis, 1),
            Save::new(Side::Axis, 1).player("DM"),
            Save::new(Side::Axis, 1).player("DG"),
            Save::new(Side::Axis, 2),
            Save::new(Side::Allies, 1),
            Save::new(Side::Allies, 1).player("GM"),
            Save::new(Side::Allies, 1).player("TG"),
        ]);

        let mut local_storage = MockIndexStorage::new(vec![
            Save::new(Side::Axis, 1),
            Save::new(Side::Axis, 1).player("DM"),
        ]);

        assert_eq!(
            prediction.predict_downloads(
                1,
                Side::Axis,
                "DG",
                &mut local_storage,
                &mut remote_storage
            )?,
            vec![
                Save::new(Side::Axis, 1).player("DG"),
                Save::new(Side::Axis, 2),
            ]
        );

        Ok(())
    }
}
