use self::state::AppState;
use self::{actions::Actions, ui::Status};
use crate::{inputs::key::Key, types::ListType};
use crate::{app::actions::Action, types::FullCommand};
use git_function_history::{CommitFunctions, File, FunctionHistory};
use std::{fmt, sync::mpsc, time::Duration};

pub mod actions;
pub mod state;
pub mod ui;
#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
    TextEdit,
}

/// The main application, containing the state
pub struct App {
    actions: Actions,
    state: AppState,
    input_buffer: String,
    cmd_output: CommandResult,
    pub scroll_pos: (u16, u16),
    pub body_height: u16,
    pub text_scroll_pos: (u16, u16),
    pub input_width: u16,
    channels: (
        mpsc::Sender<FullCommand>,
        mpsc::Receiver<(CommandResult, Status)>,
    ),
    status: Status,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(
        history: Option<FunctionHistory>,
        channels: (
            mpsc::Sender<FullCommand>,
            mpsc::Receiver<(CommandResult, Status)>,
        ),
    ) -> Self {
        let actions = vec![
            Action::Quit,
            Action::TextEdit,
            Action::ScrollDown,
            Action::ScrollUp,
        ]
        .into();
        let state = AppState::initialized();
        match history {
            Some(history) => Self {
                actions,
                state,
                input_buffer: String::new(),
                cmd_output: CommandResult::History(history),
                scroll_pos: (0, 0),
                body_height: 0,
                text_scroll_pos: (0, 0),
                input_width: 0,
                channels,
                status: Status::Ok(None),
            },
            None => Self {
                actions,
                state,
                input_buffer: String::new(),
                cmd_output: CommandResult::None,
                scroll_pos: (0, 0),
                body_height: 0,
                text_scroll_pos: (0, 0),
                input_width: 0,
                channels,
                status: Status::Ok(None),
            },
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }
    /// Handle a user action
    pub fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            match action {
                Action::Quit => AppReturn::Exit,
                Action::TextEdit => {
                    self.state = AppState::Editing;
                    AppReturn::Continue
                }
                Action::ScrollUp => {
                    if self.scroll_pos.0 == 0 {
                        return AppReturn::Continue;
                    }
                    self.scroll_pos.0 -= 1;
                    AppReturn::Continue
                }
                Action::ScrollDown => {
                    let ot = self.scroll_pos.0 + self.body_height;
                    // check if there is enough body_height that we dont need to scroll more
                    if usize::from(ot) >= self.cmd_output().len() {
                        return AppReturn::Continue;
                    }
                    self.scroll_pos.0 += 1;
                    AppReturn::Continue
                }
            }
        } else {
            AppReturn::Continue
        }
    }

    pub fn do_edit_action(&mut self, key: Key) {
        // TODO: figyure ohow to handle the one extra character that doesnt get sohwn
        match key {
            Key::Esc => {
                self.state = AppState::Looking;
            }
            Key::Enter => {
                self.run_command();
                self.input_buffer.clear();
            }

            Key::Tab => {
                self.input_buffer.push_str("    ");
                if self.input_width < self.input_buffer.len() as u16 {
                    self.text_scroll_pos.1 = self.input_buffer.len() as u16 - self.input_width;
                }
            }
            Key::Char(c) => {
                self.input_buffer.push(c);
                if self.input_width < self.input_buffer.len() as u16 {
                    self.text_scroll_pos.1 = self.input_buffer.len() as u16 - self.input_width;
                }
            }
            Key::Shift(c) => {
                self.input_buffer.push(c.to_ascii_uppercase());
                if self.input_width < self.input_buffer.len() as u16 {
                    self.text_scroll_pos.1 = self.input_buffer.len() as u16 - self.input_width;
                }
            }
            Key::Backspace => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                }
                // check if we need to scroll back
                if self.input_width > self.input_buffer.len() as u16 && self.text_scroll_pos.1 > 0 {
                    self.text_scroll_pos.1 = self.input_buffer.len() as u16 - self.input_width;
                }
            }
            Key::Left => {
                if usize::from(self.text_scroll_pos.1) < self.input_buffer.len() {
                    self.text_scroll_pos.1 += 1;
                }
            }
            Key::Right => {
                if self.text_scroll_pos.1 > 0 {
                    self.text_scroll_pos.1 -= 1;
                }
            }
            Key::Delete => {
                self.input_buffer.clear();
                self.text_scroll_pos.1 = 0;
            }
            _ => {}
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
    pub fn run_command(&mut self) {
        let mut cmd_output = CommandResult::None;
        // iterate through the tha commnad by space
        let mut iter = self.input_buffer.trim().split(' ');
        let cmd = iter.next();
        match cmd {
            Some(cmd) => match cmd {
                "filter" => {}
                "search" => {}
                "list" => match iter.next() {
                    Some(arg) => match arg {
                        "dates" => {
                            self.status = Status::Loading;
                            self.channels.0.send(FullCommand::List(ListType::Dates)).unwrap();
                        }
                        "commits" => {
                            self.status = Status::Loading;
                            self.channels.0.send(FullCommand::List(ListType::Commits)).unwrap();
                        }
                        _ => {}
                    },
                    None => {}
                },
                other => {
                    cmd_output = CommandResult::String(vec![format!("Unknown command: {}", other)]);
                }
            },
            None => {
                cmd_output =
                    CommandResult::String(vec![format!("{} is not a valid command", "sd")]);
            }
        }

        self.cmd_output = cmd_output;
    }

    pub fn get_result(&mut self)  {
        match self.channels.1.recv_timeout(Duration::from_millis(100)) {
            Ok(timeout) => match timeout {
                (_, Status::Error(e)) => {
                    self.status = Status::Error(e);
                }
                (t, Status::Ok(msg)) => {
                    // TODO: clear all the old positioning/scrolling data
                    self.status = Status::Ok(msg);
                    self.cmd_output = t;
                }
                _ => {}
            },
            Err(e) => match e {
                mpsc::RecvTimeoutError::Timeout => {}
                mpsc::RecvTimeoutError::Disconnected => {
                    panic!("Thread Channel Disconnected");
                }
            },
        }
    }
}

pub enum CommandResult {
    History(FunctionHistory),
    Commit(CommitFunctions),
    File(File),
    String(Vec<String>),
    None,
}

impl CommandResult {
    pub fn len(&self) -> usize {
        match self {
            CommandResult::History(history) => history.to_string().len(),
            CommandResult::Commit(commit) => commit.to_string().len(),
            CommandResult::File(file) => file.to_string().len(),
            CommandResult::String(str) => str.len(),
            CommandResult::None => 0,
        }
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::History(history) => write!(f, "{}", history),
            CommandResult::Commit(commit) => write!(f, "{}", commit),
            CommandResult::File(file) => write!(f, "{}", file),
            CommandResult::String(string) => {
                for line in string {
                    writeln!(f, "{}", line)?;
                }
                Ok(())
            }
            CommandResult::None => {
                write!(f, "Please enter some commands to search for a function",)
            }
        }
    }
}
