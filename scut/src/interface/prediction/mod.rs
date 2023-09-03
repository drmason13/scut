pub mod classic_prediction;
pub mod simple_prediction;

use crate::{Save, Side};

use super::{LocalStorage, RemoteStorage};

/// This trait is for the logic behind choosing which saves to download, which saves to upload and what turn the autosave should be uploaded as.
///
/// The turn parameter for [`predict_downloads`], [`predict_uploads`] and [`predict_autosave`] is trusted absolutely.
///
/// Some implementations make their own predictions about what turn it is instead of reading the turn from the config, so
/// [`predict_turn`] should always be called first and its result fed in (if it isn't `None`).
///
/// An overriden turn (for example the user deliberately choose to download or upload a previous turn)
/// should be used first to overrule any result of [`predict_turn`]
///
/// [`predict_downloads`]: Prediction::predict_downloads
/// [`predict_uploads`]: Prediction::predict_uploads
/// [`predict_turn`]: Prediction::predict_turn
/// [`predict_autosave`]: Prediction::predict_autosave
pub trait Prediction {
    /// Ask the Prediction implementation what turn they think it is for the given [`Side`], implementations may choose not to implement this.
    ///
    /// The returned turn that it "is" is the turn that should be downloaded or uploaded,
    /// i.e. the "current turn" that scut needs to act on (often the "next turn" in another sense)
    #[allow(unused_variables)]
    fn predict_turn(
        &self,
        side: Side,
        player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Option<u32>> {
        Ok(None)
    }

    /// Return all the [`Save`]s that should be downloaded.
    fn predict_downloads(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Return all the [`Save`]s that should be uploaded - disregarding the autosave, which is handled via [`predict_autosave`](Prediction::predict_autosave)
    fn predict_uploads(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<Vec<Save>>;

    /// Return a [`Save`] the autosave should be uploaded as, and optionally an indication if it should be uploaded now or not.
    fn predict_autosave(
        &self,
        turn: u32,
        side: Side,
        player: &str,
        local_storage: &mut dyn LocalStorage,
        remote_storage: &mut dyn RemoteStorage,
    ) -> anyhow::Result<(Save, Option<bool>)>;
}
