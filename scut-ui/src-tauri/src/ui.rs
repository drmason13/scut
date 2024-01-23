use scut_core::interface::UserInteraction;

pub struct TauriWindow;

impl UserInteraction for TauriWindow {
    fn message(&mut self, message: &str) {
        println!("{message}");
    }

    fn confirm(&mut self, message: &str, default: Option<bool>) -> bool {
        println!("{message}");
        default.unwrap_or(false)
    }

    fn query(&mut self, message: &str) -> String {
        panic!(
            "Unable to respond to query: `{message}` \
            with this particular implementation of 'UserInteration'"
        )
    }
}
