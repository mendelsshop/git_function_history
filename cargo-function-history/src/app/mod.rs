use self::{actions::Actions, state::AppState};
use crate::{app::actions::Action, keys::Key};

use function_history_backend_thread::types::{
    CommandResult, FilterType, FullCommand, ListType, Status,
};
use git_function_history::{languages::Language, FileFilterType, Filter};
use std::{
    fs,
    io::{Read, Write},
    sync::mpsc,
    time::Duration,
};
use tui_input::InputRequest;
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
    pub history: Vec<String>,
    pub history_index: usize,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(
        channels: (
            mpsc::Sender<FullCommand>,
            mpsc::Receiver<(CommandResult, Status)>,
        ),
        status: Status,
    ) -> Self {
        // read history from file
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(super::get_history_dir().expect("No history path"))
            .expect("Failed to open history file");
        let mut history = String::new();
        file.read_to_string(&mut history)
            .expect("Failed to read history file");
        let history: Vec<String> = history.split('\n').map(|s| s.to_string()).collect();
        // let history_index = history.len();
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
            status,
            history_index: history.len() - 1,
            history,
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

    pub fn run_command(&mut self) {
        // iterate through the tha commnad by space
        let iter = self.input_buffer.to_string();
        let mut iter = iter.trim().split(' ');
        match iter.next() {
            Some(cmd) => match cmd {
                "search" => {
                    // check for a function name
                    if let Some(name) = iter.next() {
                        // check if there next arg stars with file or filter
                        self.status = Status::Loading;
                        let mut file = FileFilterType::None;
                        let mut filter = Filter::None;
                        let mut lang = Language::All;
                        let new_vec = iter.collect::<Vec<_>>();
                        let mut new_iter = new_vec.windows(2);
                        log::debug!("searching for {:?}", new_iter);

                        if new_vec.len() % 2 != 0 {
                            self.status = Status::Error(format!("uncomplete search, command: {} doesnt have its parameters",new_vec.last().expect("oops look like theres nothing in this vec don't how this happened")));
                            return;
                        }
                        for i in &mut new_iter {
                            log::info!("i: {:?}", i);
                            match i {
                                ["relative", filepath] => {
                                    log::trace!("relative file: {}", filepath);
                                    file = FileFilterType::Relative(filepath.to_string());
                                }
                                ["absolute", filepath] => {
                                    log::trace!("absolute file: {}", filepath);
                                    file = FileFilterType::Absolute(filepath.to_string());
                                }
                                ["date", date] => {
                                    log::trace!("date: {}", date);
                                    filter = Filter::Date(date.to_string());
                                }
                                ["commit", commit] => {
                                    log::trace!("commit: {}", commit);
                                    filter = Filter::CommitHash(commit.to_string());
                                }
                                ["directory", dir] => {
                                    log::trace!("directory: {}", dir);
                                    file = FileFilterType::Directory(dir.to_string());
                                }
                                ["date-range", pos] => {
                                    log::trace!("date range: {}", pos);
                                    let (start, end) = match pos.split_once("..") {
                                        Some((start, end)) => (start, end),
                                        None => {
                                            self.status = Status::Error(
                                                "Invalid date range, expected start..end"
                                                    .to_string(),
                                            );
                                            return;
                                        }
                                    };
                                    filter = Filter::DateRange(start.to_string(), end.to_string());
                                }
                                ["language", language] => {
                                    log::trace!("language: {}", language);
                                    lang = match language {
                                        &"rust" => Language::Rust,
                                        &"python" => Language::Python,
                                        // #[cfg(feature = "c_lang")]
                                        // &"c" => Language::C,
                                        #[cfg(feature = "unstable")]
                                        &"go" => Language::Go,
                                        &"ruby" => Language::Ruby,
                                        _ => {
                                            self.status =
                                                Status::Error("Invalid language".to_string());
                                            return;
                                        }
                                    };
                                }
                                ["author", author] => {
                                    log::trace!("author: {}", author);
                                    filter = Filter::Author(author.to_string());
                                }
                                ["author-email", author_email] => {
                                    log::trace!("author-email: {}", author_email);
                                    filter = Filter::AuthorEmail(author_email.to_string());
                                }
                                ["message", message] => {
                                    log::trace!("message: {}", message);
                                    filter = Filter::Message(message.to_string());
                                }

                                _ => {
                                    log::debug!("invalid arg: {:?}", i);
                                    self.status = Status::Error(format!("Invalid search {:?}", i));
                                    return;
                                }
                            }
                        }
                        self.channels
                            .0
                            .send(FullCommand::Search(name.to_string(), file, filter, lang))
                            .unwrap();
                    } else {
                        self.status = Status::Error("No function name given".to_string());
                    }
                }
                "filter" => {
                    self.status = Status::Loading;
                    let mut filter = Filter::None;
                    for i in &mut iter.clone().collect::<Vec<_>>().windows(2) {
                        match i {
                            ["date", date] => {
                                filter = Filter::Date(date.to_string());
                            }
                            ["commit", commit] => {
                                filter = Filter::CommitHash(commit.to_string());
                            }
                            ["date-range", pos] => {
                                let (start, end) = match pos.split_once("..") {
                                    Some((start, end)) => (start, end),
                                    None => {
                                        self.status = Status::Error(
                                            "Invalid date range, expected start..end".to_string(),
                                        );
                                        return;
                                    }
                                };
                                filter = Filter::DateRange(start.to_string(), end.to_string());
                            }
                            ["author", author] => {
                                log::trace!("author: {}", author);
                                filter = Filter::Author(author.to_string());
                            }
                            ["author-email", author_email] => {
                                log::trace!("author-email: {}", author_email);
                                filter = Filter::AuthorEmail(author_email.to_string());
                            }
                            ["message", message] => {
                                log::trace!("message: {}", message);
                                filter = Filter::Message(message.to_string());
                            }
                            ["line-range", pos] => {
                                // get the start and end by splitting the pos by: ..
                                let (start, end) = match pos.split_once("..") {
                                    Some((start, end)) => (start, end),
                                    None => {
                                        self.status = Status::Error(
                                            "Invalid line range, expected start..end".to_string(),
                                        );
                                        return;
                                    }
                                };
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
                                filter = Filter::FunctionInLines(start, end);
                            }
                            ["file-absolute", file] => {
                                filter = Filter::FileAbsolute(file.to_string());
                            }
                            ["file-relative", file] => {
                                filter = Filter::FileRelative(file.to_string());
                            }
                            ["directory", dir] => {
                                filter = Filter::Directory(dir.to_string());
                            }
                            _ => {
                                self.status = Status::Error(format!("Invalid filter {}", i[0]));
                                return;
                            }
                        }
                    }
                    if iter.clone().count() > 0 {
                        self.status = Status::Error(format!(
                            "Invalid filter, command: {:?} missing args",
                            iter.collect::<Vec<_>>()
                        ));
                        return;
                    }

                    self.channels
                        .0
                        .send(FullCommand::Filter(FilterType {
                            thing: self.cmd_output.clone(),
                            filter,
                        }))
                        .unwrap();
                }
                "list" => {
                    self.status = Status::Loading;
                    let list = match iter.next() {
                        Some(arg) => match arg {
                            "dates" => Some(FullCommand::List(ListType::Dates)),
                            "commits" => Some(FullCommand::List(ListType::Commits)),
                            _ => {
                                self.status = Status::Error("Invalid list type".to_string());
                                None
                            }
                        },
                        None => {
                            self.status = Status::Error("No list type given".to_string());
                            None
                        }
                    };
                    if let Some(list) = list {
                        self.channels.0.send(list).unwrap();
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
                    let e = e.split_once("why").unwrap_or((&e, ""));
                    let e = format!(
                        " error recieved last command didn't work; {}{}",
                        e.0,
                        e.1.split_once("why").unwrap_or(("", "")).0,
                    );
                    log::warn!("{}", e);
                    self.status = Status::Error(e);
                }
                (t, Status::Ok(msg)) => {
                    log::info!("got results of last command");
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
    pub fn reset_and_save(&mut self) {
        let mut input = self.input_buffer.to_string();
        if !input.is_empty() {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(super::get_history_dir().expect("No history path"))
                .expect("Failed to open history file");
            // check if the last command was the same as the current one
            if let Some(last) = self.history.last() {
                if last != &input {
                    input.push('\n');
                    file.write_all(input.as_bytes())
                        .expect("Failed to write to history file");
                    self.history.push(input.trim().to_string());
                }
            } else {
                input.push('\n');
                file.write_all(input.as_bytes())
                    .expect("Failed to write to history file");
                self.history.push(input.trim().to_string());
            }
        }
        self.input_buffer.reset();
    }

    pub fn scroll_up(&mut self) {
        self.history_index = self.history_index.saturating_sub(1);
        let strs = match self.history.get(self.history_index) {
            Some(string) => string.as_str(),
            None => "",
        };
        for character in strs.chars() {
            let req = InputRequest::InsertChar(character);
            self.input_buffer.handle(req);
        }
    }

    pub fn scroll_down(&mut self) {
        self.history_index = match self.history_index.saturating_add(1) {
            i if i >= self.history.len() - 1 => self.history.len() - 1,
            i => i,
        };
        let strs = match self.history.get(self.history_index) {
            Some(string) => string.as_str(),
            None => "",
        };
        for character in strs.chars() {
            let req = InputRequest::InsertChar(character);
            self.input_buffer.handle(req);
        }
    }
}
