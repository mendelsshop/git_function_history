use self::actions::Actions;
use self::state::AppState;
use crate::{app::actions::Action, keys::Key};

use function_history_backend_thread::types::{
    CommandResult, FilterType, FullCommand, ListType, Status,
};
use git_function_history::{
    languages::Language,
    // BlockType,
    FileFilterType,
    Filter,
};
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
        let history = history.split('\n').map(|s| s.to_string()).collect();

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
            history,
            history_index: 0,
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
                "filter" => {
                    if let CommandResult::History(_) = &self.cmd_output {
                        self.status = Status::Loading;
                        if let Some(filter) = iter.next() {
                            let filter = match filter {
                                "date" => {
                                    if let Some(date) = iter.next() {
                                        let date = date.replace('_', " ");
                                        Some(Filter::Date(date))
                                    } else {
                                        self.status = Status::Error("No date given".to_string());
                                        None
                                    }
                                }
                                "commit" => {
                                    if let Some(commit) = iter.next() {
                                        Some(Filter::CommitHash(commit.to_string()))
                                    } else {
                                        self.status = Status::Error("No commit given".to_string());
                                        None
                                    }
                                }
                                "parent" => {
                                    if let Some(_parent) = iter.next() {
                                        // Some(Filter::FunctionWithParent(parent.to_string()))
                                        None
                                    } else {
                                        self.status =
                                            Status::Error("No parent function given".to_string());
                                        None
                                    }
                                }
                                // "block" => {
                                //     if let Some(block) = iter.next() {
                                //         Some(Filter::FunctionInBlock(BlockType::from_string(block)))
                                //     } else {
                                //         self.status =
                                //             Status::Error("No block type given".to_string());
                                //         None
                                //     }
                                // }
                                "date-range" => {
                                    if let Some(start) = iter.next() {
                                        if let Some(end) = iter.next() {
                                            // remove all - from the date
                                            let start = start.replace('_', " ");
                                            let end = end.replace('_', " ");
                                            Some(Filter::DateRange(start, end))
                                        } else {
                                            self.status =
                                                Status::Error("No end date given".to_string());
                                            None
                                        }
                                    } else {
                                        self.status =
                                            Status::Error("No start date given".to_string());
                                        None
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
                                            Some(Filter::FunctionInLines(start, end))
                                        } else {
                                            self.status =
                                                Status::Error("No end line given".to_string());
                                            None
                                        }
                                    } else {
                                        self.status =
                                            Status::Error("No start line given".to_string());
                                        None
                                    }
                                }
                                "file-absolute" => {
                                    if let Some(file) = iter.next() {
                                        Some(Filter::FileAbsolute(file.to_string()))
                                    } else {
                                        self.status = Status::Error("No file given".to_string());
                                        None
                                    }
                                }
                                "file-relative" => {
                                    if let Some(file) = iter.next() {
                                        Some(Filter::FileRelative(file.to_string()))
                                    } else {
                                        self.status = Status::Error("No file given".to_string());
                                        None
                                    }
                                }
                                "directory" => {
                                    if let Some(dir) = iter.next() {
                                        Some(Filter::Directory(dir.to_string()))
                                    } else {
                                        self.status =
                                            Status::Error("No directory given".to_string());
                                        None
                                    }
                                }
                                _ => {
                                    self.status = Status::Error("Invalid filter".to_string());
                                    None
                                }
                            };
                            if let Some(filter) = filter {
                                self.channels
                                    .0
                                    .send(FullCommand::Filter(FilterType {
                                        thing: self.cmd_output.clone(),
                                        filter,
                                    }))
                                    .unwrap();
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
                        let mut file = FileFilterType::None;
                        let mut filter = Filter::None;
                        let mut lang = Language::All;
                        // let search = match iter.next() {
                        //     None => {
                        //         // if there is no next arg then we are searching for a function
                        //         // with the given name
                        //         Some(FullCommand::Search(
                        //             name.to_string(),
                        //             FileFilterType::None,
                        //             Filter::None,
                        //             Language::Rust,
                        //         ))
                        //     }
                        //     Some(thing) => match thing {
                        //         "relative" | "absolute" => {
                        //             let file_type = match iter.next() {
                        //                 Some(filter) => match thing {
                        //                     "relative" => {
                        //                         FileFilterType::Relative(filter.to_string())
                        //                     }
                        //                     "absolute" => {
                        //                         FileFilterType::Absolute(filter.to_string())
                        //                     }
                        //                     _ => FileFilterType::None,
                        //                 },
                        //                 None => {
                        //                     self.status =
                        //                         Status::Error("No filter given".to_string());
                        //                     return;
                        //                 }
                        //             };
                        //             let filter = match iter.next() {
                        //                 Some(filter) => match filter {
                        //                     "date" => {
                        //                         let date = iter.next();
                        //                         match date {
                        //                             Some(date) => {
                        //                                 let date = date.replace('_', " ");
                        //                                 Filter::Date(date)
                        //                             }
                        //                             None => {
                        //                                 self.status = Status::Error(
                        //                                     "No date given".to_string(),
                        //                                 );
                        //                                 return;
                        //                             }
                        //                         }
                        //                     }
                        //                     "commit" => {
                        //                         let commit = iter.next();
                        //                         match commit {
                        //                             Some(commit) => {
                        //                                 Filter::CommitHash(commit.to_string())
                        //                             }
                        //                             None => {
                        //                                 self.status = Status::Error(
                        //                                     "No commit given".to_string(),
                        //                                 );
                        //                                 return;
                        //                             }
                        //                         }
                        //                     }
                        //                     "date range" => {
                        //                         let start = iter.next();
                        //                         let end = iter.next();
                        //                         match (start, end) {
                        //                             (Some(start), Some(end)) => {
                        //                                 let start = start.replace('_', " ");
                        //                                 let end = end.replace('_', " ");
                        //                                 Filter::DateRange(start, end)
                        //                             }
                        //                             _ => {
                        //                                 self.status = Status::Error(
                        //                                     "No date range given".to_string(),
                        //                                 );
                        //                                 return;
                        //                             }
                        //                         }
                        //                     }
                        //                     _ => {
                        //                         self.status =
                        //                             Status::Error("No filter given".to_string());
                        //                         return;
                        //                     }
                        //                 },
                        //                 None => Filter::None,
                        //             };
                        //             Some(FullCommand::Search(
                        //                 name.to_string(),
                        //                 file_type,
                        //                 filter,
                        //                 Language::Rust,
                        //             ))
                        //         }
                        //         "date" | "commit" | "date range" => {
                        //             let filter = match thing {
                        //                 "date" => {
                        //                     let date = iter.next();
                        //                     match date {
                        //                         Some(date) => Filter::Date(date.to_string()),
                        //                         None => {
                        //                             self.status =
                        //                                 Status::Error("No date given".to_string());
                        //                             return;
                        //                         }
                        //                     }
                        //                 }
                        //                 "commit" => {
                        //                     let commit = iter.next();
                        //                     match commit {
                        //                         Some(commit) => {
                        //                             Filter::CommitHash(commit.to_string())
                        //                         }
                        //                         None => {
                        //                             self.status = Status::Error(
                        //                                 "No commit given".to_string(),
                        //                             );
                        //                             return;
                        //                         }
                        //                     }
                        //                 }
                        //                 "date range" => {
                        //                     let start = iter.next();
                        //                     let end = iter.next();
                        //                     match (start, end) {
                        //                         (Some(start), Some(end)) => Filter::DateRange(
                        //                             start.to_string(),
                        //                             end.to_string(),
                        //                         ),
                        //                         _ => {
                        //                             self.status = Status::Error(
                        //                                 "No date range given".to_string(),
                        //                             );
                        //                             return;
                        //                         }
                        //                     }
                        //                 }
                        //                 _ => Filter::None,
                        //             };
                        //             Some(FullCommand::Search(
                        //                 name.to_string(),
                        //                 FileFilterType::None,
                        //                 filter,
                        //                 Language::Rust,
                        //             ))
                        //         }
                        //         "language" => {
                        //             let language = iter.next();
                        //             match language {
                        //                 Some(language) => {
                        //                     let language = match language {
                        //                         "rust" => Language::Rust,
                        //                         "python" => Language::Python,
                        //                         "c" => Language::C,
                        //                         _ => {
                        //                             self.status = Status::Error(
                        //                                 "Invalid language".to_string(),
                        //                             );
                        //                             return;
                        //                         }
                        //                     };
                        //                     Some(FullCommand::Search(
                        //                         name.to_string(),
                        //                         FileFilterType::None,
                        //                         Filter::None,
                        //                         language,
                        //                     ))
                        //                 }
                        //                 None => {
                        //                     self.status =
                        //                         Status::Error("No language given".to_string());
                        //                     return;
                        //                 }
                        //             }
                        //         }
                        //         _ => {
                        //             self.status = Status::Error("Invalid file type".to_string());
                        //             None
                        //         }
                        //     },
                        // };
                        for i in iter.collect::<Vec<_>>().windows(2) {
                            match i {
                                ["relative", filepath] => {
                                    file = FileFilterType::Relative(filepath.to_string());
                                }
                                ["absolute", filepath] => {
                                    file = FileFilterType::Absolute(filepath.to_string());
                                }
                                ["date", date] => {
                                    filter = Filter::Date(date.to_string());
                                }
                                ["commit", commit] => {
                                    filter = Filter::CommitHash(commit.to_string());
                                }
                                ["date range", start, end] => {
                                    filter = Filter::DateRange(start.to_string(), end.to_string());
                                }
                                ["language", language] => {
                                    lang = match language {
                                        &"rust" => Language::Rust,
                                        &"python" => Language::Python,
                                        #[cfg(feature = "c_lang")]
                                        &"c" => Language::C,
                                        _ => {
                                            self.status =
                                                Status::Error("Invalid language".to_string());
                                            return;
                                        }
                                    };
                                }

                                _ => {
                                    self.status = Status::Error(format!("Invalid search {}", i[0]));
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
