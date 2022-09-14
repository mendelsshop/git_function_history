use self::actions::Actions;
use self::state::AppState;
use crate::{app::actions::Action, keys::Key};

use function_history_backend_thread::types::{
    CommandResult, FilterType, FullCommand, ListType, Status,
};
use git_function_history::{BlockType, FileType, Filter};
use std::{sync::mpsc, time::Duration};
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
    pub state: AppState,
    pub input_buffer: tui_input::Input,
    cmd_output: CommandResult,
    pub scroll_pos: (u16, u16),
    pub body_height: u16,
    channels: (
        mpsc::Sender<FullCommand>,
        mpsc::Receiver<(CommandResult, Status)>,
    ),
    status: Status,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(
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
        Self {
            actions,
            state,
            input_buffer: tui_input::Input::default(),
            cmd_output: CommandResult::None,
            scroll_pos: (0, 0),
            body_height: 0,
            channels,
            status: Status::Ok(None),
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
                    log::info!("change to edit mode");
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
                    log::trace!("scroll down: ot:{} output:{}", ot, self.cmd_output.len());
                    // check if there is enough body_height that we dont need to scroll more
                    if usize::from(ot) >= self.cmd_output().len() {
                        return AppReturn::Continue;
                    }
                    self.scroll_pos.0 += 1;
                    AppReturn::Continue
                }
                // TODO reset other things
                Action::BackCommit => {
                    if let CommandResult::History(t) = &mut self.cmd_output {
                        t.move_back();
                    }
                    AppReturn::Continue
                }
                Action::ForwardCommit => {
                    if let CommandResult::History(t) = &mut self.cmd_output {
                        t.move_forward();
                    }
                    AppReturn::Continue
                }
                Action::BackFile => {
                    if let CommandResult::History(t) = &mut self.cmd_output {
                        t.move_back_file();
                    }
                    AppReturn::Continue
                }
                Action::ForwardFile => {
                    if let CommandResult::History(t) = &mut self.cmd_output {
                        t.move_forward_file();
                    }
                    AppReturn::Continue
                }
            }
        } else {
            AppReturn::Continue
        }
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn input_buffer(&self) -> String {
        self.input_buffer.to_string()
    }

    pub fn cmd_output(&self) -> &CommandResult {
        &self.cmd_output
    }

    // TODO: figure outt what to name ceach commnad and something based on that
    pub fn run_command(&mut self) {
        // iterate through the tha commnad by space
        let iter = self.input_buffer.to_string();
        let mut iter = iter.trim().split(' ');
        match iter.next() {
            Some(cmd) => match cmd {
                "filter" => {
                    if let CommandResult::History(_) = &self.cmd_output {
                        self.status = Status::Loading;
                        if let Some(filter) = iter.next() {
                            match filter {
                                "date" => {
                                    if let Some(date) = iter.next() {
                                        let date = date.replace('_', " ");
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::Date(date),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status = Status::Error("No date given".to_string());
                                    }
                                }
                                "commit" => {
                                    if let Some(commit) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::CommitId(commit.to_string()),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status = Status::Error("No commit given".to_string());
                                    }
                                }
                                "parent" => {
                                    if let Some(parent) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::FunctionWithParent(
                                                    parent.to_string(),
                                                ),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status =
                                            Status::Error("No parent function given".to_string());
                                    }
                                }
                                "block" => {
                                    if let Some(block) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::FunctionInBlock(
                                                    BlockType::from_string(block),
                                                ),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status =
                                            Status::Error("No block type given".to_string());
                                    }
                                }
                                "date-range" => {
                                    if let Some(start) = iter.next() {
                                        if let Some(end) = iter.next() {
                                            // remove all - from the date
                                            let start = start.replace('_', " ");
                                            let end = end.replace('_', " ");
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType {
                                                    thing: self.cmd_output.clone(),
                                                    filter: Filter::DateRange(start, end),
                                                }))
                                                .unwrap()
                                        } else {
                                            self.status =
                                                Status::Error("No end date given".to_string());
                                        }
                                    } else {
                                        self.status =
                                            Status::Error("No start date given".to_string());
                                    }
                                }
                                "line-range" => {
                                    if let Some(start) = iter.next() {
                                        if let Some(end) = iter.next() {
                                            let start = match start.parse::<usize>() {
                                                Ok(x) => x,
                                                Err(e) => {
                                                    self.status = Status::Error(format!("{}", e));
                                                    return;
                                                }
                                            };
                                            let end = match end.parse::<usize>() {
                                                Ok(x) => x,
                                                Err(e) => {
                                                    self.status = Status::Error(format!("{}", e));
                                                    return;
                                                }
                                            };
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType {
                                                    thing: self.cmd_output.clone(),
                                                    filter: Filter::FunctionInLines(start, end),
                                                }))
                                                .unwrap()
                                        } else {
                                            self.status =
                                                Status::Error("No end line given".to_string());
                                        }
                                    } else {
                                        self.status =
                                            Status::Error("No start line given".to_string());
                                    }
                                }
                                "file-absolute" => {
                                    if let Some(file) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::FileAbsolute(file.to_string()),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status = Status::Error("No file given".to_string());
                                    }
                                }
                                "file-relative" => {
                                    if let Some(file) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::FileRelative(file.to_string()),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status = Status::Error("No file given".to_string());
                                    }
                                }
                                "directory" => {
                                    if let Some(dir) = iter.next() {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter: Filter::Directory(dir.to_string()),
                                            }))
                                            .unwrap()
                                    } else {
                                        self.status =
                                            Status::Error("No directory given".to_string());
                                    }
                                }
                                _ => {
                                    self.status = Status::Error("Invalid filter".to_string());
                                }
                            }
                        } else {
                            self.status = Status::Error("No filter given".to_string());
                        }
                    } else if iter.next().is_some() {
                        self.status = Status::Error("no filters available".to_string());
                    }
                }
                "search" => {
                    // check for a function name
                    if let Some(name) = iter.next() {
                        // check if there next arg stars with file or filter
                        self.status = Status::Loading;
                        match iter.next() {
                            None => {
                                // if there is no next arg then we are searching for a function
                                // with the given name
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
                                                    Some(date) => {
                                                        let date = date.replace('_', " ");
                                                        Filter::Date(date)
                                                    }
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
                                                    (Some(start), Some(end)) => {
                                                        let start = start.replace('_', " ");
                                                        let end = end.replace('_', " ");
                                                        Filter::DateRange(start, end)
                                                    }
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
                                }
                            },
                        }
                    } else {
                        self.status = Status::Error("No function name given".to_string());
                    }
                }
                "list" => {
                    self.status = Status::Loading;
                    match iter.next() {
                        Some(arg) => match arg {
                            "dates" => {
                                self.channels
                                    .0
                                    .send(FullCommand::List(ListType::Dates))
                                    .unwrap();
                            }
                            "commits" => {
                                self.channels
                                    .0
                                    .send(FullCommand::List(ListType::Commits))
                                    .unwrap();
                            }
                            _ => {
                                self.status = Status::Error("Invalid list type".to_string());
                            }
                        },
                        None => {
                            self.status = Status::Error("No list type given".to_string());
                        }
                    }
                }
                other => {
                    self.status = Status::Error(format!("Invalid command: {}", other));
                }
            },
            None => {
                self.status = Status::Error("No command given".to_string());
            }
        }
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
