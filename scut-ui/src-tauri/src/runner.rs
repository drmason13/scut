use std::sync::{self, Mutex};

pub struct ScutRunner {}

impl ScutRunner {
    pub fn new() -> ScutRunner {
        ScutRunner {}
    }
}

pub struct AppState {
    pub scut: sync::Mutex<ScutRunner>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            scut: Mutex::new(ScutRunner::new()),
        }
    }
}
