pub mod classic_predict;
pub mod simple_predict;

use crate::{Save, Side, Turn};

use super::{LocalStorage, RemoteStorage};

#[derive(Debug)]
pub struct Prediction {
    pub autosave: Option<Save>,
    pub uploads: Vec<Save>,
    pub downloads: Vec<Save>,
}

/// This trait is for the logic behind choosing which saves to download, which saves to upload and what turn the autosave should be uploaded as.
///
/// [`predict_downloads`]: Predict::predict_downloads
/// [`predict_uploads`]: Predict::predict_uploads
/// [`predict_turn`]: Predict::predict_turn
/// [`predict_autosave`]: Predict::predict_autosave
pub trait Predict {
    /// Predict what saves should be uploaded and downloaded
    fn predict(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Prediction>;

    /// Predict what turn it is.
    fn predict_turn(
        &self,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Turn>;

    /// Return all the [`Save`]s that should be downloaded.
    fn predict_downloads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Return all the [`Save`]s that should be uploaded - disregarding the autosave, which is handled via [`predict_autosave`](Predict::predict_autosave)
    fn predict_uploads(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Predict what to upload the autosave as, if at all
    fn predict_autosave(
        &self,
        predicted_turn: Turn,
        side: Side,
        player: &str,
        local: &mut dyn LocalStorage,
        remote: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<Save>>;

    /// Predict whether to upload the autosave as predicted, based on the predicted downloads and uploads
    fn should_upload_autosave(
        &self,
        predicted_autosave: &Option<Save>,
        side: Side,
        player: &str,
        predicted_downloads: &[Save],
        predicted_uploads: &[Save],
    ) -> bool;
}
