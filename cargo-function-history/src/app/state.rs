#[derive(Clone)]
pub enum AppState {
    Loading,
    Looking,
    Editing,
}

impl AppState {
    pub fn initialized() -> Self {
        AppState::Looking
    }
}
