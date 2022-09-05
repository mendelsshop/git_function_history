use self::state::AppState;
use self::{actions::Actions, ui::Status};
use crate::{
    app::actions::Action,
    types::{FullCommand, Index},
};
use crate::{inputs::key::Key, types::ListType};
use git_function_history::{CommitFunctions, File, FileType, Filter, FunctionHistory};
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
            Action::BackCommit,
            Action::ForwardCommit,
            Action::BackFile,
            Action::ForwardFile,
        ]
        .into();
        let state = AppState::initialized();
        match history {
            Some(history) => {
                let hist_len = history.history.len();
                let commit_len = if hist_len > 0 {
                    history.history[0].functions.len()
                } else {
                    0
                };
                Self {
                    actions,
                    state,
                    input_buffer: String::new(),
                    cmd_output: CommandResult::History(
                        history,
                        Index(hist_len, 0),
                        Index(commit_len, 0),
                    ),
                    scroll_pos: (0, 0),
                    body_height: 0,
                    text_scroll_pos: (0, 0),
                    input_width: 0,
                    channels,
                    status: Status::Ok(None),
                }
            }
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
                Action::BackCommit => {
                    if let CommandResult::History(_, Index(_, i), _) = &mut self.cmd_output {
                        if *i <= 0 {
                            *i = 0;
                            return AppReturn::Continue;
                        }
                        self.scroll_pos.0 = 0;

                        *i -= 1;
                    }
                    AppReturn::Continue
                }
                Action::ForwardCommit => {
                    if let CommandResult::History(_, Index(len, i), _) = &mut self.cmd_output {
                        if *len - 1 >= *i {
                            *i = *len - 1;
                            return AppReturn::Continue;
                        }
                        self.scroll_pos.0 = 0;
                        *i += 1;
                    }
                    AppReturn::Continue
                }
                _ => AppReturn::Continue,
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
                "search" => {
                    // check for a function name
                    if let Some(name) = iter.next() {
                        // check if there next arg stars with file or filter
                        match iter.next() {
                            None => {
                                // if there is no next arg then we are searching for a function
                                // with the given name
                                self.status = Status::Loading;
                                self.channels
                                    .0
                                    .send(FullCommand::Search(
                                        name.to_string(),
                                        FileType::None,
                                        Filter::None,
                                    ))
                                    .unwrap();
                            }
                            Some(thing) => match thing {
                                "relative" | "absolute" => {
                                    let file_type = match iter.next() {
                                        Some(filter) => match thing {
                                            "relative" => FileType::Relative(filter.to_string()),
                                            "absolute" => FileType::Absolute(filter.to_string()),
                                            _ => FileType::None,
                                        },
                                        None => {
                                            self.status =
                                                Status::Error("No filter given".to_string());
                                            return;
                                        }
                                    };
                                    let filter = match iter.next() {
                                        Some(filter) => match filter {
                                            "date" => {
                                                let date = iter.next();
                                                match date {
                                                    Some(date) => Filter::Date(date.to_string()),
                                                    None => {
                                                        self.status = Status::Error(
                                                            "No date given".to_string(),
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                            "commit" => {
                                                let commit = iter.next();
                                                match commit {
                                                    Some(commit) => {
                                                        Filter::CommitId(commit.to_string())
                                                    }
                                                    None => {
                                                        self.status = Status::Error(
                                                            "No commit given".to_string(),
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                            "date range" => {
                                                let start = iter.next();
                                                let end = iter.next();
                                                match (start, end) {
                                                    (Some(start), Some(end)) => Filter::DateRange(
                                                        start.to_string(),
                                                        end.to_string(),
                                                    ),
                                                    _ => {
                                                        self.status = Status::Error(
                                                            "No date range given".to_string(),
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                            _ => {
                                                self.status =
                                                    Status::Error("No filter given".to_string());
                                                return;
                                            }
                                        },
                                        None => Filter::None,
                                    };

                                    self.status = Status::Loading;
                                    self.channels
                                        .0
                                        .send(FullCommand::Search(
                                            name.to_string(),
                                            file_type,
                                            filter,
                                        ))
                                        .unwrap();
                                }
                                "date" | "commit" | "date range" => {
                                    let filter = match thing {
                                        "date" => {
                                            let date = iter.next();
                                            match date {
                                                Some(date) => Filter::Date(date.to_string()),
                                                None => {
                                                    self.status =
                                                        Status::Error("No date given".to_string());
                                                    return;
                                                }
                                            }
                                        }
                                        "commit" => {
                                            let commit = iter.next();
                                            match commit {
                                                Some(commit) => {
                                                    Filter::CommitId(commit.to_string())
                                                }
                                                None => {
                                                    self.status = Status::Error(
                                                        "No commit given".to_string(),
                                                    );
                                                    return;
                                                }
                                            }
                                        }
                                        "date range" => {
                                            let start = iter.next();
                                            let end = iter.next();
                                            match (start, end) {
                                                (Some(start), Some(end)) => Filter::DateRange(
                                                    start.to_string(),
                                                    end.to_string(),
                                                ),
                                                _ => {
                                                    self.status = Status::Error(
                                                        "No date range given".to_string(),
                                                    );
                                                    return;
                                                }
                                            }
                                        }
                                        _ => Filter::None,
                                    };
                                    self.status = Status::Loading;
                                    self.channels
                                        .0
                                        .send(FullCommand::Search(
                                            name.to_string(),
                                            FileType::None,
                                            filter,
                                        ))
                                        .unwrap();
                                }
                                _ => {
                                    self.status = Status::Error("Invalid file type".to_string());
                                    return;
                                }
                            },
                        }
                    } else {
                        self.status = Status::Error("No function name given".to_string());
                    }
                }
                "list" => match iter.next() {
                    Some(arg) => match arg {
                        "dates" => {
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::List(ListType::Dates))
                                .unwrap();
                        }
                        "commits" => {
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::List(ListType::Commits))
                                .unwrap();
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

    pub fn get_result(&mut self) {
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

#[derive(Debug, Clone)]
pub enum CommandResult {
    History(FunctionHistory, Index, Index),
    Commit(CommitFunctions, Index),
    File(File),
    String(Vec<String>),
    None,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult::None
    }
}

impl CommandResult {
    pub fn len(&self) -> usize {
        match self {
            CommandResult::History(history, ..) => history.to_string().len(),
            CommandResult::Commit(commit, _) => commit.to_string().len(),
            CommandResult::File(file) => file.to_string().len(),
            CommandResult::String(str) => str.len(),
            CommandResult::None => 0,
        }
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::History(history, t1, t2) => {
                write!(f, "{}", history.history[t1.1].functions[t2.1])
            }
            CommandResult::Commit(commit, t) => write!(f, "{}", commit.functions[t.1]),
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
