use self::actions::Actions;
use self::state::AppState;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use function_history_backend_thread::types::{CommandResult, FullCommand, Index, ListType, Status, FilterType, HistoryFilter, CommitOrFileFilter, CommmitFilterValue};
use git_function_history::{FileType, Filter, FunctionHistory, BlockType};
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
                    // check if there is enough body_height that we dont need to scroll more
                    if usize::from(ot) >= self.cmd_output().len() {
                        return AppReturn::Continue;
                    }
                    self.scroll_pos.0 += 1;
                    AppReturn::Continue
                }
                // TODO reset other things
                Action::BackCommit => {
                    if let CommandResult::History(_, Index(_, i), _) = &mut self.cmd_output {
                        if *i == 0 {
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
                        if *len - 1 <= *i {
                            *i = *len - 1;
                            return AppReturn::Continue;
                        }
                        self.scroll_pos.0 = 0;
                        *i += 1;
                    }
                    AppReturn::Continue
                }
                Action::BackFile => {
                    match &mut self.cmd_output {
                        CommandResult::History(_, _, Index(_, i)) => {
                            if *i == 0 {
                                *i = 0;
                                return AppReturn::Continue;
                            }
                            self.scroll_pos.0 = 0;

                            *i -= 1;
                        }
                        CommandResult::Commit(_, Index(_, i)) => {
                            if *i == 0 {
                                *i = 0;
                                return AppReturn::Continue;
                            }
                            self.scroll_pos.0 = 0;

                            *i -= 1;
                        }
                        _ => {}
                    }
                    AppReturn::Continue
                }
                Action::ForwardFile => {
                    match &mut self.cmd_output {
                        CommandResult::History(_, _, Index(len, i)) => {
                            if *len > *i {
                                *i = *len - 1;
                                return AppReturn::Continue;
                            }
                            self.scroll_pos.0 = 0;
                            *i += 1;
                        }
                        CommandResult::Commit(_, Index(len, i)) => {
                            if *len > *i {
                                *i = *len - 1;
                                return AppReturn::Continue;
                            }
                            self.scroll_pos.0 = 0;
                            *i += 1;
                        }
                        _ => {}
                    }
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
                "filter" => {
                    match &self.cmd_output {
                        CommandResult::History(t, _, _) => {
                        if let Some(filter) = iter.next() {
                            match filter {
                                "date" => {
                                    if let Some(date) = iter.next() {
                                        let date = date.replace('_", " ");
                                        self.channels.0.send(FullCommand::Filter(
                                            FilterType::History(HistoryFilter::Date(date), t.clone()),
                                        )).unwrap()
                                    } else {
                                        self.status = Status::Error("No date given".to_string());
                                    }
                                }
                                "commit" => {
                                    if let Some(commit) = iter.next() {
                                        self.channels.0.send(FullCommand::Filter(
                                            FilterType::History(HistoryFilter::CommitId(commit.to_owned()), t.clone()),
                                        )).unwrap()
                                    } else {
                                        self.status = Status::Error("No commit given".to_string());
                                    }
                                }
                                "parent" => {
                                    if let Some(parent) = iter.next() {
                                        self.channels.0.send(FullCommand::Filter(
                                            FilterType::History(HistoryFilter::FunctionInFunction(parent.to_owned()), t.clone()),
                                        )).unwrap()
                                    } else {
                                        self.status = Status::Error("No parent function given".to_string());
                                    }
                                }
                                "block" => {
                                    if let Some(block) = iter.next() {
                                        self.channels.0.send(FullCommand::Filter(
                                            FilterType::History(HistoryFilter::FunctionInBlock(BlockType::from_string(block)), t.clone()),
                                        )).unwrap()
                                    } else {
                                        self.status = Status::Error("No block type given".to_string());
                                    }
                                }
                                "date-range" => {
                                    if let Some(start) = iter.next() {
                                        if let Some(end) = iter.next() {
                                            // remove all - from the date
                                            let start = start.replace('_", " ");
                                            let end = end.replace('_", " ");
                                            self.channels.0.send(FullCommand::Filter(
                                                FilterType::History(HistoryFilter::DateRange(start, end), t.clone()),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No end date given".to_string());
                                        }
                                    } else {
                                        self.status = Status::Error("No start date given".to_string());
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
                                            self.channels.0.send(FullCommand::Filter(
                                                
                                                FilterType::History(HistoryFilter::FunctionInLines(start.to_owned(), end.to_owned()), t.clone()),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No end line given".to_string());
                                        }
                                    } else {
                                        self.status = Status::Error("No start line given".to_string());
                                    }
                                }
                                _ => {
                                    self.status = Status::Error("Invalid filter".to_string());
                                }
                            }
                        } else {
                            self.status = Status::Error("No filter given".to_string());
                        }
                        }
                        CommandResult::Commit(t, _)  => {
                            if let Some(filter) = iter.next() {
                                match filter {
                                    "parent" => {
                                        if let Some(parent) = iter.next() {
                                            self.channels.0.send(FullCommand::Filter(
                                                FilterType::CommitOrFile(CommitOrFileFilter::FunctionInFunction(parent.to_owned()), CommmitFilterValue::Commit(t.clone())),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No parent function given".to_string());
                                        }
                                    }
                                    "block" => {
                                        if let Some(block) = iter.next() {
                                            self.channels.0.send(FullCommand::Filter(
                                                FilterType::CommitOrFile(CommitOrFileFilter::FunctionInBlock(BlockType::from_string(block)), CommmitFilterValue::Commit(t.clone())),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No block type given".to_string());
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
                                                self.channels.0.send(FullCommand::Filter(
                                                    
                                                    FilterType::CommitOrFile(CommitOrFileFilter::FunctionInLines(start.to_owned(), end.to_owned()), CommmitFilterValue::Commit(t.clone())),
                                                )).unwrap()
                                            } else {
                                                self.status = Status::Error("No end line given".to_string());
                                            }
                                        } else {
                                            self.status = Status::Error("No start line given".to_string());
                                        }
                                    }
                                    _ => {
                                        self.status = Status::Error("Invalid filter".to_string());
                                    }
                                }
                            } else {
                                self.status = Status::Error("No filter given".to_string());
                            }
                            
                        }
                        CommandResult::File(t) => {
                            if let Some(filter) = iter.next() {
                                match filter {
                                    "parent" => {
                                        if let Some(parent) = iter.next() {
                                            self.channels.0.send(FullCommand::Filter(
                                                FilterType::CommitOrFile(CommitOrFileFilter::FunctionInFunction(parent.to_owned()), CommmitFilterValue::File(t.clone())),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No parent function given".to_string());
                                        }
                                    }
                                    "block" => {
                                        if let Some(block) = iter.next() {
                                            self.channels.0.send(FullCommand::Filter(
                                                FilterType::CommitOrFile(CommitOrFileFilter::FunctionInBlock(BlockType::from_string(block)), CommmitFilterValue::File(t.clone())),
                                            )).unwrap()
                                        } else {
                                            self.status = Status::Error("No block type given".to_string());
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
                                                self.channels.0.send(FullCommand::Filter(
                                                    
                                                    FilterType::CommitOrFile(CommitOrFileFilter::FunctionInLines(start.to_owned(), end.to_owned()), CommmitFilterValue::File(t.clone())),
                                                )).unwrap()
                                            } else {
                                                self.status = Status::Error("No end line given".to_string());
                                            }
                                        } else {
                                            self.status = Status::Error("No start line given".to_string());
                                        }
                                    }
                                    _ => {
                                        self.status = Status::Error("Invalid filter".to_string());
                                    }
                                }
                            } else {
                                self.status = Status::Error("No filter given".to_string());
                            }
                        }
                        _ => {
                            if iter.next().is_some() {
                                self.status = Status::Error("no filters available".to_string());
                            }
                        }
                    }
                }
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
                                                    Some(date) => {
                                                        let date = date.replace('_", " ");
                                                        Filter::Date(date)},
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
                                                        let start = start.replace('_", " ");
                                                        let end = end.replace('_", " ");
                                                        Filter::DateRange(
                                                        start,
                                                        end,
                                                    )}
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
