// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::str::FromStr;

use runner::{AppState, ScutRunner};
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

#[tauri::command]
fn predict() -> Prediction {
    let scut = ScutRunner::new().expect("everything to go perfectly always of course!");
    scut.make_prediction()
        .expect("everything to go perfectly always of course!")
}

#[tauri::command]
fn upload(items: Vec<String>) -> String {
    let scut = ScutRunner::new().expect("everything to go perfectly always of course!");
    scut.upload(items.iter().map(|s| Save::from_str(s).unwrap()).collect())
        .expect("everything to go perfectly always of course!");

    format!("Uploaded {:?}", items.join(", "))
}

#[tauri::command]
fn download(items: Vec<String>) -> String {
    let scut = ScutRunner::new().expect("everything to go perfectly always of course!");
    scut.download(items.iter().map(|s| Save::from_str(s).unwrap()).collect())
        .expect("everything to go perfectly always of course!");

    format!("Downloaded {:?}", items.join(", "))
}

fn main() {
    let system_tray = tray::init_system_tray();

    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .manage(state)
        .system_tray(system_tray)
        .on_window_event(handle_window_event)
        .on_system_tray_event(handle_system_tray_event)
        .invoke_handler(tauri::generate_handler![upload, download, predict])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
