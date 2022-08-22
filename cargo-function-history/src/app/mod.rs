use std::fmt;

use git_function_history::{CommitFunctions, File, FunctionHistory};

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
    Run,
}

/// The main application, containing the state
pub struct App {
    is_loading: bool,
    actions: Actions,
    state: AppState,
    input_buffer: String,
    cmd_output: CommandResult,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(history: Option<FunctionHistory>) -> Self {
        let actions = vec![Action::Quit, Action::Run, Action::Backspace].into();
        let state = AppState::initialized();
        match history {
            Some(history) => Self {
                actions,
                state,
                is_loading: false,
                input_buffer: String::new(),
                cmd_output: CommandResult::History(history),
            },
            None => Self {
                actions,
                state,
                is_loading: false,
                input_buffer: String::new(),
                cmd_output: CommandResult::None,
            },
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
                Action::Run => {
                    let buf = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.is_loading = true;
                    self.run_command(&buf);
                    AppReturn::Continue
                }
                Action::Backspace => {
                    self.input_buffer.pop();
                    AppReturn::Continue
                }
            }
        } else {
            self.input_buffer.push_str(&key.to_string());
            AppReturn::Continue
        }
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn input_buffer(&self) -> &String {
        &self.input_buffer
    }

    pub fn cmd_output(&self) -> &CommandResult {
        &self.cmd_output
    }

    // TODO: figure outt what to name ceach commnad and something based on that
    pub fn run_command(&mut self, command: &str) {
        let mut cmd_output = CommandResult::None;
        // iterate through the tha commnad by space
        let mut iter = command.split(' ');
        let cmd = iter.next();
        match cmd {
            Some(cmd) => {
                cmd_output = CommandResult::String(format!("{} is not a valid command", cmd));
            }
            None => {
                cmd_output = CommandResult::String(format!("{} is not a valid command", "sd"));
            }
        }

        self.cmd_output = cmd_output;
    }
}

pub enum CommandResult {
    History(FunctionHistory),
    Commit(CommitFunctions),
    File(File),
    String(String),
    None,
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::History(history) => write!(f, "{}", history),
            CommandResult::Commit(commit) => write!(f, "{}", commit),
            CommandResult::File(file) => write!(f, "{}", file),
            CommandResult::String(string) => write!(f, "{}", string),
            CommandResult::None => {
                write!(f, "Please enter some commands to search for a function",)
            }
        }
    }
}
