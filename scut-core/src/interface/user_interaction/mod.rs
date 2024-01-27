pub mod terminal;

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::Save;

/// This interface defines ways to send/receive input to/from the user
pub trait UserInteraction: Send {
    /// Send a message to the user to notify them of an event.
    ///
    /// No response from the user is expected, the message is informational.
    ///
    /// To ask the user a question see [`confirm`](UserInteraction::confirm)
    fn message(&mut self, message: &str);

    /// Prompt the user for a simple yes / no response to a question
    ///
    /// default is used to indicate a default response the user can select more easily,
    /// unless None in which case both yes and no are equal
    fn confirm(&mut self, message: &str, default: Option<bool>) -> bool;

    /// Prompt the user for a free form text response
    fn query(&mut self, message: &str) -> String;

    fn wait_for_user_before_close(&mut self, message: &str) {
        let _ = self.query(message);
    }
}

/// This function provides a generic method outside of the [`UserInteraction`] trait so that it can remain Object Safe
pub fn query_and_parse<T>(message: &str, ui: &mut dyn UserInteraction) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    loop {
        let input = ui.query(message);

        match input.parse() {
            Ok(value) => break Some(value),
            Err(e) => {
                ui.message(&format!("Invalid input: {e}"));
                if ui.confirm("Would you like to try entering input again?", Some(true)) {
                    continue;
                } else {
                    break None;
                }
            }
        }
    }
}

/// A user's selection of saves to upload/download
///
/// Typically this is selected by the user
/// after having seen the prediction offered by scut.
///
/// The following data exchange takes place:
///
/// scut -> [`Prediction`] -> user
/// scut <- [`Selection`]  <- user
///
/// This doesn't have to happen in one entire block like this,
/// parts of the prediction can be *Confirmed* individually
/// using the methods on the [`UserInteraction`] trait.
///
/// [`Prediction`]: crate::interface::Prediction
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// If none, do not upload the autosave
    pub autosave: Option<Save>,
    pub uploads: Vec<Save>,
    pub downloads: Vec<Save>,
}
