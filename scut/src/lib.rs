pub mod interface;

// types
pub mod error;
mod save;
pub use save::{Save, SaveOrAutosave, Side};
mod config;
pub use config::{Config, Key, Setting};
