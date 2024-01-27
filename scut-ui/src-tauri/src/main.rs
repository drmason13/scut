// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::app_setup;
use runner::AppState;
use scut_core::{
    interface::{
        predict::{AutosavePrediction, AutosavePredictionReason},
        Prediction,
    },
    Save, Side,
};
use tray::handle_system_tray_event;
use window::handle_window_event;

mod app;
mod config;
mod message;
mod runner;
mod storage;
mod tray;
mod ui;
mod window;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tauri::command]
fn upload(items: Vec<String>) -> String {
    format!(
        "Hello, {}! You've been greeted from Rust!",
        items.join(", ")
    )
}

#[tauri::command]
fn predict() -> Prediction {
    // TODO: run scut :)
    Prediction {
        autosave: AutosavePrediction::NotReady(
            Save::new(Side::Allies, 1),
            AutosavePredictionReason::AutosaveAlreadyUploaded,
        ),
        uploads: vec![],
        downloads: vec![Save::new(Side::Axis, 1)],
    }
}

fn main() {
    let system_tray = tray::init_system_tray();

    let state = AppState::new();

    tauri::Builder::default()
        .setup(app_setup)
        .plugin(tauri_plugin_positioner::init())
        .manage(state)
        .system_tray(system_tray)
        .on_window_event(handle_window_event)
        .on_system_tray_event(handle_system_tray_event)
        .invoke_handler(tauri::generate_handler![upload, predict])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
