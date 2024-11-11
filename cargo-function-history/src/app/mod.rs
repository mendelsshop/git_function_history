use self::{actions::Actions, state::AppState};
use crate::{app::actions::Action, keys::Key};

use function_history_backend_thread::types::{
    CommandResult, FilterType, FullCommand, ListType, SearchType, Status,
};
use git_function_history::{FileFilterType, Filter};
use ratatui::{
    style::Modifier,
    widgets::{Block, Borders, ScrollbarState},
};
use std::{
    fs,
    io::{Read, Write},
    sync::mpsc,
    time::Duration,
};
use tui_textarea::TextArea;
// use tui_input::InputRequest;
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
pub struct App<'a> {
    actions: Actions,
    pub state: AppState,
    pub input_buffer: TextArea<'a>,
    cmd_output: CommandResult,
    pub scroll_pos: (u16, u16),
    pub scroll_state: ScrollbarState,
    pub body_height: u16,
    channels: (
        mpsc::Sender<FullCommand>,
        mpsc::Receiver<(CommandResult, Status)>,
    ),
    status: Status,
    pub history: Vec<String>,
    pub history_index: usize,
}
macro_rules! unwrap_set_error {
    ($self:ident, $expr:expr, $error:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                $self.status = Status::Error($error.to_string());
                return None;
            }
        }
    }; // does the same thing but takes a closure to call when is some
}
impl App<'_> {
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
            .truncate(false)
            .open(super::get_history_dir().expect("No history path"))
            .expect("Failed to open history file");
        let mut history = String::new();
        file.read_to_string(&mut history)
            .expect("Failed to read history file");
        let history: Vec<String> = history.split('\n').map(|s| s.to_string()).collect();
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
            input_buffer: {
                let mut area = TextArea::default();
                area.set_cursor_line_style(
                    area.cursor_line_style()
                        .remove_modifier(Modifier::UNDERLINED),
                );
                area.set_cursor_style(area.cursor_style().remove_modifier(Modifier::REVERSED));
                area.set_block(Block::default().borders(Borders::BOTTOM));
                area
            },
            cmd_output: CommandResult::None,
            scroll_pos: (0, 0),
            body_height: 0,
            channels,
            status,
            history_index: history.len() - 1,
            history,
            scroll_state: ScrollbarState::default(),
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
                    let style = self
                        .input_buffer
                        .cursor_style()
                        .add_modifier(Modifier::REVERSED);
                    self.input_buffer.set_cursor_style(style);
                    AppReturn::Continue
                }
                Action::ScrollUp => {
                    if self.scroll_pos.0 == 0 {
                        return AppReturn::Continue;
                    }
                    self.scroll_pos.0 -= 1;
                    self.scroll_state = self.scroll_state.position(self.scroll_pos.0.into());
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
                    self.scroll_state = self.scroll_state.position(self.scroll_pos.0.into());
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
        self.input_buffer.lines()[0].clone()
    }

    pub fn cmd_output(&self) -> &CommandResult {
        &self.cmd_output
    }

    pub fn run_command(&mut self) {
        let command = self.parse_command(&self.input_buffer());
        if let Some(command) = command {
            self.channels
                .0
                .send(command)
                .expect("could not send message in thread");
        }
    }

    pub fn parse_command(&mut self, command: &str) -> Option<FullCommand> {
        match command.split_once(' ') {
            Some((cmd, args)) => {
                let args = args.split(' ').collect::<Vec<_>>();
                let iter = args.as_slice();
                self.status = Status::Loading;
                match cmd {
                    "search" => Some(FullCommand::Search(self.parse_search(iter)?)),
                    "filter" => Some(FullCommand::Filter(FilterType {
                        thing: self.cmd_output.clone(),
                        filter: self.parse_filter(iter)?,
                    })),
                    "list" => Some(FullCommand::List(self.parse_list(iter)?)),
                    _ => {
                        self.status = Status::Error(format!("Invalid command: {cmd}"));
                        None
                    }
                }
            }
            None => {
                self.status = Status::Error("No command given".to_string());
                None
            }
        }
    }

    fn parse_search(&mut self, command: &[&str]) -> Option<SearchType> {
        let mut command_iter = command.iter();
        let mut file = FileFilterType::None;
        let mut filter = Filter::None;

        // TODO: allow searching with specific langauges
        let name = unwrap_set_error!(self, command_iter.next(), "No function name");
        while let Some(cmd) = command_iter.next() {
            match *cmd {
                "date" => {
                    filter = match *unwrap_set_error!(self, command_iter.next(), "No date given") {
                        "range" => Filter::DateRange(
                            unwrap_set_error!(self, command_iter.next(), "No start date given")
                                .to_string(),
                            unwrap_set_error!(self, command_iter.next(), "No end date given")
                                .to_string(),
                        ),
                        date => Filter::Date(date.to_string()),
                    };
                }
                "commit" => {
                    filter = Filter::CommitHash(
                        unwrap_set_error!(self, command_iter.next(), "No commit given").to_string(),
                    );
                }
                "author" => {
                    filter =
                        match *unwrap_set_error!(self, command_iter.next(), "No author name given")
                        {
                            "email" => Filter::AuthorEmail(
                                unwrap_set_error!(self, command_iter.next(), "No email given")
                                    .to_string(),
                            ),
                            name => Filter::Author(name.to_string()),
                        };
                }
                "file" => {
                    file = match *unwrap_set_error!(self, command_iter.next(), "Invalid file type")
                    {
                        "absolute" => FileFilterType::Absolute,
                        "relative" => FileFilterType::Relative,
                        "directory" => FileFilterType::Directory,
                        _ => {
                            self.status = Status::Error("Invalid file type".to_string());
                            return None;
                        }
                    }(
                        unwrap_set_error!(self, command_iter.next(), "No file given").to_string(),
                    );
                }
                "message" => {
                    filter = Filter::Message(
                        unwrap_set_error!(self, command_iter.next(), "No commit message given")
                            .to_string(),
                    )
                }
                _ => {
                    self.status = Status::Error(format!("Invalid search command: {cmd}"));
                    return None;
                }
            }
        }
        Some(SearchType {
            search: name.to_string(),
            file,
            filter,
        })
    }

    fn parse_filter(&mut self, command: &[&str]) -> Option<Filter> {
        let mut command_iter = command.iter();
        let mut filter = Filter::None;
        while let Some(cmd) = command_iter.next() {
            match cmd {
                &"language" => {
                    // TODO: support specifying many languages
                    filter = Filter::Language(
                        unwrap_set_error!(self, command_iter.next(), "No language given")
                            .to_string(),
                    );
                }
                &"date" => {
                    filter = match *unwrap_set_error!(self, command_iter.next(), "No date given") {
                        "range" => Filter::DateRange(
                            unwrap_set_error!(self, command_iter.next(), "No start date given")
                                .to_string(),
                            unwrap_set_error!(self, command_iter.next(), "No end date given")
                                .to_string(),
                        ),
                        date => Filter::Date(date.to_string()),
                    };
                }
                &"commit" => {
                    filter = Filter::CommitHash(
                        unwrap_set_error!(self, command_iter.next(), "No commit given").to_string(),
                    );
                }
                &"author" => {
                    filter =
                        match *unwrap_set_error!(self, command_iter.next(), "No author name given")
                        {
                            "email" => Filter::AuthorEmail(
                                unwrap_set_error!(self, command_iter.next(), "No email given")
                                    .to_string(),
                            ),
                            name => Filter::Author(name.to_string()),
                        };
                }
                &"file" => {
                    filter =
                        match *unwrap_set_error!(self, command_iter.next(), "Invalid file type") {
                            "absolute" => Filter::FileAbsolute,
                            "relative" => Filter::FileRelative,
                            "directory" => Filter::Directory,
                            _ => {
                                self.status = Status::Error("Invalid file type".to_string());
                                return None;
                            }
                        }(
                            unwrap_set_error!(self, command_iter.next(), "No file given")
                                .to_string(),
                        );
                }
                &"message" => {
                    filter = Filter::Message(
                        unwrap_set_error!(self, command_iter.next(), "No commit message given")
                            .to_string(),
                    )
                }
                filter_name => {
                    if let Some(filters) =
                        function_grep::filter::Filters::default().get_filter(*filter_name)
                    {
                        let rest = command_iter.copied().collect::<Vec<_>>().join(" ");
                        let filt = match filters.to_filter(&rest) {
                            Ok(val) => val,
                            Err(e) => {
                                self.status = Status::Error(
                                    format!(
                                        "filters of {} could not be parsed properly, reason: {e} ",
                                        filter_name
                                    )
                                    .to_string(),
                                );
                                return None;
                            }
                        };
                        log::info!("filtering by {:?}", filt);
                        filter = Filter::PLFilter(filt);
                        break;
                    } else {
                        self.status =
                            Status::Error(format!("Invalid search command filter: {cmd}"));
                        return None;
                    }
                }
            }
        }
        Some(filter)
    }

    fn parse_list(&mut self, command: &[&str]) -> Option<ListType> {
        match command {
            ["dates"] => Some(ListType::Dates),
            ["commits"] => Some(ListType::Commits),
            _ => {
                self.status = Status::Error("Invalid list type".to_string());
                None
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
        let mut input: String = self.input_buffer();
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
        self.input_buffer
            .move_cursor(tui_textarea::CursorMove::Head);
        self.input_buffer.delete_line_by_end();
    }

    pub fn scroll_up(&mut self) {
        self.history_index = self.history_index.saturating_sub(1);
        let strs = match self.history.get(self.history_index) {
            Some(string) => string.as_str(),
            None => "",
        };
        self.input_buffer.insert_str(strs);
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
        self.input_buffer.insert_str(strs);
    }
}
