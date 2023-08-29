pub mod classic_decision;

use crate::Save;

/// This trait is for the logic behind choosing which saves to download, which saves to upload and what turn the autosave should be uploaded as.
///
/// The turn parameter for [`predict_downloads`], [`predict_uploads`] and [`upload_autosave_as`] is trusted absolutely.
///
/// Some implementations make their own predictions about what turn it is instead of reading the turn from the config, so
/// [`predict_turn`] should always be called first and its result fed in (if it isn't `None`).
///
/// An overriden turn (for example the user deliberately choose to download or upload a previous turn)
/// should be used first to overrule any result of [`predict_turn`]
///
/// [`predict_downloads`]: Decision::predict_downloads
/// [`predict_uploads`]: Decision::predict_uploads
/// [`predict_turn`]: Decision::predict_turn
/// [`upload_autosave_as`]: Decision::upload_autosave_as
pub trait Decision {
    /// Ask the Decision implementation what turn they think it is, implementations may choose not to implement this.
    fn predict_turn(&self) -> Option<u32> {
        None
    }

    /// Return all the [`Save`]s that should be downloaded.
    fn predict_downloads(&self, turn: u32) -> Vec<Save>;

    /// Return all the [`Save`]s that should be uploaded - disregarding the autosave, which is hanlded via [`upload_autosave_as`](Decision::upload_autosave_as)
    fn predict_uploads(&self, turn: u32) -> Vec<Save>;

    /// Return a [`Save`] the autosave should be uploaded as, and a boolean indicating whether it should be uploaded now.
    fn upload_autosave_as(&self, turn: u32) -> (Save, bool);
}
