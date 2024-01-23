use tauri::GlobalWindowEvent;

pub fn handle_window_event(event: GlobalWindowEvent) {
    if let tauri::WindowEvent::Focused(focused) = event.event() {
        if !focused {
            event.window().hide().unwrap();
        }
    }
}
