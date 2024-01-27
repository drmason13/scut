use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::runner::ScutRunner;

pub fn init_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let config = CustomMenuItem::new("config".to_string(), "Config");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(config);

    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    tauri_plugin_positioner::on_tray_event(app, &event);
    match event {
        SystemTrayEvent::LeftClick { .. } => open_window(app),
        SystemTrayEvent::RightClick { .. } => {}
        SystemTrayEvent::DoubleClick { .. } => {}
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => app.exit(0),
            "config" => on_config_menu_click(app),
            _ => {}
        },
        _ => {}
    }
}

pub fn open_window(app: &AppHandle) {
    let window = app.get_window("tray").unwrap();

    // let the window know it's being opened
    window.emit("trayOpen", ()).unwrap();

    // open the window
    window.move_window(Position::TrayCenter).unwrap();
    window.unminimize().unwrap();
    window.show().unwrap();
    window.set_focus().unwrap();
}

pub fn on_config_menu_click(app: &AppHandle) {
    let _ = app;
    let scut_runner = ScutRunner::new().expect("everything to go perfectly always of course!");
    let result = scut_runner.config();
    println!("config: {result:?}");
}
