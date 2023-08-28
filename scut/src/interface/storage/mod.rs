//! The storage module contains interfaces for [Local](LocalStorage) and [Remote](RemoteStorage) storage.
//!
//! Implementations of these interfaces live in submodules.

pub mod dropbox_folder;
pub mod folder;
pub mod game_saves_folder;

use std::path::{Path, PathBuf};

use crate::{Save, Side};

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

    fn get_latest_friendly_turn(&mut self, side: Side) -> anyhow::Result<Option<u32>>;
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

    fn get_latest_enemy_turn(&mut self, side: Side) -> anyhow::Result<Option<u32>>;

    /// Return the location of the remote storage as a string suitable for display to the end user
    fn location(&self) -> String;
}

// TODO: can a client use this interface to perform parallel uploads and/or downloads or is some kind of extension interface required?
