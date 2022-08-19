use git_function_history::{File, FunctionHistory, CommitFunctions};

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
    current_file: Option<File>,
    current_commit: Option<CommitFunctions>,
    whole_function_history: Option<FunctionHistory>,
    /// State
    state: AppState,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(history: Option<FunctionHistory>) -> Self {
        let actions = vec![Action::Quit].into();
        let state = AppState::initialized();
        match history {Some(history) => {
            Self {
                current_file: Some(history.history[0].functions[0].clone()),
                current_commit: Some(history.history[0].clone()),
                whole_function_history: Some(history),
                actions,
                state,
                is_loading: false,
            }
        }
        None => {
            Self {
                current_file: None,
                current_commit: None,
                whole_function_history: None,
                actions,
                state,
                is_loading: false,
            }
        }
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
