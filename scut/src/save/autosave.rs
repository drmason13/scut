use std::fmt;

use crate::Save;

/// This type can be useful for when a Save might be an autosave.
pub enum SaveOrAutosave {
    Save(Save),
    Autosave,
}

impl fmt::Display for SaveOrAutosave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveOrAutosave::Save(save) => save.fmt(f),
            SaveOrAutosave::Autosave => "autosave".fmt(f),
        }
    }
}

impl SaveOrAutosave {
    /// Get a reference to the save within, unless it is an autosave
    pub fn borrow(&self) -> Option<&Save> {
        match self {
            SaveOrAutosave::Save(ref save) => Some(save),
            SaveOrAutosave::Autosave => None,
        }
    }

    /// Get a mutable reference to the save within, unless it is an autosave.
    pub fn to_mut(&mut self) -> Option<&mut Save> {
        match self {
            SaveOrAutosave::Save(ref mut save) => Some(save),
            SaveOrAutosave::Autosave => None,
        }
    }

    /// Clone the Save and return it unless it is an autosave.
    ///
    /// Consumes self, so if it was an Autosave, you'll be left with None and may have to construct a new Autosave.
    ///
    /// Note: Autosaves are all identical and creating them is very cheap.
    pub fn into_save(self) -> Option<Save> {
        match self {
            SaveOrAutosave::Save(save) => Some(save),
            SaveOrAutosave::Autosave => None,
        }
    }

    /// Construct as a [`Save`]
    pub fn save(save: Save) -> Self {
        SaveOrAutosave::Save(save)
    }

    /// Construct as an autosave
    pub fn autosave() -> Self {
        SaveOrAutosave::Autosave
    }
}
