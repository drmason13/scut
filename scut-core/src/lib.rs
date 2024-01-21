pub mod interface;

// types
pub mod error;
mod save;
pub use save::{Save, SaveOrAutosave, Side, Turn};
mod config;
pub use config::{Config, Key, Setting};
