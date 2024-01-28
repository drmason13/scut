// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::str::FromStr;

use runner::ScutRunner;
use scut_core::{interface::Prediction, Save};
use tray::handle_system_tray_event;
use window::handle_window_event;

mod config;
mod message;
mod runner;
mod storage;
mod tray;
mod ui;
mod window;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tauri::command(async)]
fn predict() -> Result<Prediction, String> {
    let scut = ScutRunner::new().map_err(|e| e.to_string())?;
    scut.make_prediction().map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn upload(autosave: Option<String>, items: Vec<String>) -> Result<String, String> {
    let scut = ScutRunner::new().map_err(|e| e.to_string())?;

    let autosave = autosave
        .map(|ref s| Save::from_str(s))
        .transpose()
        .map_err(|e| e.to_string())?;

    let saves = items
        .iter()
        .map(|s| Save::from_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    scut.upload(autosave, saves).map_err(|e| e.to_string())?;

    Ok(format!("Uploaded {}", items.join(", ")))
}

#[tauri::command(async)]
fn download(items: Vec<String>) -> Result<String, String> {
    let scut = ScutRunner::new().map_err(|e| e.to_string())?;

    let saves = items
        .iter()
        .map(|s| Save::from_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    scut.download(saves).map_err(|e| e.to_string())?;

    Ok(format!("Downloaded {}", items.join(", ")))
}

#[tauri::command(async)]
fn config() -> Result<String, String> {
    let scut = ScutRunner::new().map_err(|e| e.to_string())?;
    scut.config().map_err(|e| e.to_string())?;

    Ok("Config successfully updated".to_string())
}

fn main() {
    let system_tray = tray::init_system_tray();

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .system_tray(system_tray)
        .on_window_event(handle_window_event)
        .on_system_tray_event(handle_system_tray_event)
        .invoke_handler(tauri::generate_handler![upload, download, predict, config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
