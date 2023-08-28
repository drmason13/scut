use std::fmt;

use crate::Save;

/// This type can be useful for when a Save might be an autosave (None).
///
/// Typically we deal with saves and encourage providing a separate API for autosaves.
///
/// In the implementation of those APIs, often this type is used to avoid code duplication.
///
/// > It's a bit like a Cow and an Option had a baby, just for Saves!
pub enum SaveOrAutosave<'a> {
    SaveBorrowed(&'a Save),
    SaveOwned(Save),
    Autosave,
}

impl<'a> fmt::Display for SaveOrAutosave<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveOrAutosave::SaveBorrowed(save) => save.fmt(f),
            SaveOrAutosave::SaveOwned(save) => save.fmt(f),
            SaveOrAutosave::Autosave => "autosave".fmt(f),
        }
    }
}

impl<'a> SaveOrAutosave<'a> {
    /// Get a reference to the save within, unless it is an autosave
    pub fn borrow(&'a self) -> Option<&'a Save> {
        match self {
            SaveOrAutosave::SaveBorrowed(save) => Some(save),
            SaveOrAutosave::SaveOwned(ref save) => Some(save),
            SaveOrAutosave::Autosave => None,
        }
    }

    /// Get a mutable reference to the save within, unless it is an autosave.
    ///
    /// This will clone the Save if it is currently borrowed.
    pub fn to_mut(&mut self) -> Option<&mut Save> {
        match self {
            SaveOrAutosave::SaveBorrowed(save) => {
                *self = SaveOrAutosave::SaveOwned(save.clone());
                match self {
                    SaveOrAutosave::SaveOwned(ref mut save) => Some(save),
                    _ => unreachable!(),
                }
            }
            SaveOrAutosave::SaveOwned(ref mut save) => Some(save),
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
            SaveOrAutosave::SaveBorrowed(save) => Some(save.clone()),
            SaveOrAutosave::SaveOwned(save) => Some(save),
            SaveOrAutosave::Autosave => None,
        }
    }

    /// Construct as a borrowed [`&Save`](Save)
    pub fn borrowed(save: &'a Save) -> Self {
        SaveOrAutosave::SaveBorrowed(save)
    }

    /// Construct as an owned [`Save`]
    pub fn owned(save: Save) -> Self {
        SaveOrAutosave::SaveOwned(save)
    }

    /// Construct as an autosave
    pub fn autosave() -> Self {
        SaveOrAutosave::Autosave
    }
}
