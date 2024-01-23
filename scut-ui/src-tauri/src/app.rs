use tauri::{App, Manager};

use crate::BoxResult;

pub fn app_setup(app: &mut App) -> BoxResult<()> {
    let main_window = app.get_window("main").unwrap();
    Ok(())
}
