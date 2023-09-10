//! The names of these Predict implementations are faily arbitrary!
//!
//! This Predict implementation implements `predict_turn`, using the friendly turns already available in RemoteStorage to determine what turn it must be.

use tracing::{debug, instrument};

use crate::{
    interface::{index::Query, LocalStorage, RemoteStorage},
    Save, Side, Turn,
};

#[cfg(test)]
mod ddt;

use super::{Predict, Prediction, TurnDetail};

#[derive(Debug, Default)]
pub struct SimplePredict;

impl Predict for SimplePredict {
    #[instrument(skip(self, local, remote), ret, err)]
    fn predict(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction> {
        let turn_detail = &self.predict_turn_detail(side, player, local, remote)?;

        let uploads = self.predict_uploads(turn_detail, local, remote)?;
        let downloads = self.predict_downloads(turn_detail, local, remote)?;

        let autosave = self.predict_autosave(turn_detail, local, remote)?;

        Ok(Prediction {
            autosave,
            uploads,
            downloads,
        })
    }

    // let _ = || {
    //     let remote_index = remote.index();

    //     // the latest save that we uploaded
    //     let query = &Query::new().side(side).player(Some(player));

    //     let turn = if let Some(save) = remote_index.latest(query)? {
    //         let predicted_turn = save.turn;

    //         debug!(%save, predicted_turn);

    //         predicted_turn
    //     } else {
    //         // we'll try to fetch the next turn, so we actually want to think it's turn 0 for the first turn
    //         let predicted_turn = 0;

    //         debug!(predicted_turn, "no friendly save found in remote");

    //         predicted_turn
    //     };

    //     Ok(Some(turn))
    // };
    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_turn_detail(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<super::TurnDetail> {
        let local_index = local.index();
        let remote_index = remote.index();

        let enemy_side = side.other_side();

        let your_last_uploaded_turn = remote_index
            .latest(&Query::new().side(side).player(Some(player)))?
            .map(|s| s.turn);

        let teammate_last_uploaded_turn = remote_index
            .latest(&Query::new().side(side).not_player(Some(player)))?
            .map(|s| s.turn);

        let last_finished_friendly_turn = remote_index
            .latest(&Query::new().side(side).player(None))?
            .map(|s| s.turn);

        let last_finished_enemy_turn = remote_index
            .latest(&Query::new().side(enemy_side).player(None))?
            .map(|s| s.turn);

        todo!()
    }

    // let _ = || {
    // let autosave_exists = local.locate_autosave()?.is_some();

    // let enemy_side = side.other_side();

    // let upload_count = self.count_predicted_uploads(turn, side, player, local, remote)?;

    // let predicted_autosave_turn = if upload_count == 0 { turn + 1 } else { turn };

    // let enemy_turn = side.next_turn(predicted_autosave_turn);

    // debug!(autosave_exists, %enemy_side, enemy_turn);

    // let download_count = self.count_predicted_downloads(turn, side, player, local, remote)?;

    // debug!(download_count);

    // let autosave_uploaded_already = remote.index().count(
    //     &Query::new()
    //         .side(enemy_side)
    //         .turn(enemy_turn)
    //         .player(None)
    //         .part(None),
    // )? >= 1;

    // let upload_autosave_as = Save {
    //     turn: enemy_turn,
    //     side: enemy_side,
    //     player: None,
    //     part: None,
    // };

    // let should_upload_autosave =
    //     Some(autosave_exists && download_count == 0 && !autosave_uploaded_already);

    // debug!(autosave_uploaded_already, %upload_autosave_as, should_upload_autosave);

    // Ok((upload_autosave_as, should_upload_autosave))
    // };
    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_autosave(
        &self,
        TurnDetail {
            your_turn,
            teammate_turn,
            next_friendly_turn,
            next_enemy_turn,
        }: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<Save>> {
        let autosave_exists = local.locate_autosave()?.is_some();
        let autosave_uploaded_already = remote.index().count(
            &Query::new()
                .side(next_enemy_turn.side)
                .turn_number(next_enemy_turn.number)
                .player(None)
                .part(None),
        )? >= 1;
        todo!();
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_downloads(
        &self,
        TurnDetail {
            your_turn,
            teammate_turn,
            next_friendly_turn,
            next_enemy_turn,
        }: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new()
            .turn(*teammate_turn)
            .not_player(None)
            .or_turn(*next_friendly_turn)
            .or_player(None);

        let local_saves = local.index().search(&query)?;
        let remote_saves = remote.index().search(&query)?;

        let saves_to_download: Vec<_> = remote_saves
            .into_iter()
            .filter(|s| !local_saves.contains(s))
            .collect();

        Ok(saves_to_download)
    }

    #[instrument(skip(self, local, remote), ret, err)]
    fn predict_uploads(
        &self,
        TurnDetail {
            your_turn,
            teammate_turn,
            next_friendly_turn,
            next_enemy_turn,
        }: &TurnDetail,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<crate::Save>> {
        let query = Query::new().min_turn();

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

        let turn_detail = predict.predict_turn_detail(
            Side::Axis,
            "DM".into(),
            &mut local_storage,
            &mut remote_storage,
        )?;

        assert_eq!(
            predict.predict_downloads(turn_detail, &mut local_storage, &mut remote_storage)?,
            vec![
                Save::new(Side::Axis, 1).player("DM"),
                Save::new(Side::Axis, 2),
            ]
        );

        Ok(())
    }
}
