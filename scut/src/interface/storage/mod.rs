//! The storage module contains interfaces for [Local](LocalStorage) and [Remote](RemoteStorage) storage.
//!
//! Implementations of these interfaces live in submodules.

pub mod dropbox_folder;
pub mod game_saves_folder;

use std::path::{Path, PathBuf};

use crate::{interface::Index, Save};

/// Local storage is where the saved Games are ready to be loaded by Strategic Command and played.
///
/// The Local Storage interface defines where Saves should be located within the saved Games folder.
pub trait LocalStorage {
    /// Returns the filepath containing the given save if it exists.
    fn locate_save(&mut self, save: &Save) -> anyhow::Result<Option<PathBuf>>;

    /// Returns the location of the autosave if it exists.
    ///
    /// Autosaves are created by Strategic Command when ending the turn.
    /// They are uploaded by scut as the start of turn save for the next team.
    fn locate_autosave(&mut self) -> anyhow::Result<Option<PathBuf>>;

    /// The location of this storage, i.e. the folder where saves should be extracted to
    fn location(&self) -> &Path;

    /// Return a reference to an implementation of Index that provides the [`search`] method used to find certain saves within this storage.
    ///
    /// Note that the result of a [`search`] only contains the saves that matched, and not their path within local storage.
    ///
    /// [`search`]: Index::search
    fn index(&self) -> &dyn Index;
}

/// Remote storage is where the saved Games are sent to be shared with other players.
///
/// The Remote Storage interface defines how Saves are moved to and from an external location.
pub trait RemoteStorage {
    /// Move a game save file from remote storage to local storage.
    ///
    /// The game save file must be uncompressed when saved in local storage.
    fn download(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()>;

    /// Move a game save file from local storage to remote storage.
    ///
    /// The game save file could be compressed when moved to remote storage.
    fn upload(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()>;

    /// Return a reference to an implementation of [`Index`] that provides the [`search`] method used to find certain saves within this storage.
    ///
    /// Note that the result of a [`search`] only contains the saves that matched, and not their location within remote storage.
    ///
    /// [`search`]: Index::search
    fn index(&self) -> &dyn Index;
}

// TODO: can a client use this interface to perform parallel uploads and/or downloads or is some kind of extension interface required?
