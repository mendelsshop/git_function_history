
use self::actions::Actions;
use self::state::AppState;
use crate::app::actions::Action;
use crate::inputs::key::Key;

pub mod actions;
pub mod state;
pub mod ui;
#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    is_loading: bool,
    actions: Actions,
    /// State
    state: AppState,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let actions = vec![Action::Quit].into();
        let state = AppState::initialized();

        Self {
            actions,
            state,
            is_loading: false,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

        /// Handle a user action
        pub fn do_action(&mut self, key: Key) -> AppReturn {
            if let Some(action) = self.actions.find(key) {
                match action {
                    Action::Quit => AppReturn::Exit,
                }
            } else {
                AppReturn::Continue
            }
        }
    
        /// We could update the app or dispatch event on tick
        pub fn update_on_tick(&mut self) -> AppReturn {
            // here we just increment a counter
            self.state.incr_tick();
            AppReturn::Continue
        }

        pub fn actions(&self) -> &Actions {
            &self.actions
        }

    pub fn state(&self) -> &AppState {
        &self.state
    }
}