use std::fmt;

use git_function_history::{FileType, Filter, FunctionHistory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Filter,
    List,
    Search,
}

impl Default for Command {
    fn default() -> Self {
        Command::Search
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Filter => write!(f, "filter"),
            Command::List => write!(f, "list"),
            Command::Search => write!(f, "search"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Dates,
    Commits,
}

impl std::fmt::Display for ListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListType::Dates => write!(f, "dates"),
            ListType::Commits => write!(f, "commits"),
        }
    }
}

impl Default for ListType {
    fn default() -> Self {
        ListType::Dates
    }
}

#[derive(Debug, Clone)]
pub enum CommandResult {
    History(FunctionHistory),
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
            CommandResult::History(history) => history.to_string().split('\n').count(),
            CommandResult::String(str) => str.len(),
            CommandResult::None => 0,
        }
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::History(history) => {
                write!(f, "{}", history)
            }
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
#[derive(Debug, Clone)]
pub enum Status {
    Ok(Option<String>),
    Error(String),
    Warning(String),
    Loading,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok(s) => match s {
                Some(s) => write!(f, "Ok: {}", s),
                None => write!(f, "Ok"),
            },
            Status::Error(s) => write!(f, "Err {}", s),
            Status::Warning(s) => write!(f, "Warn {}", s),
            Status::Loading => write!(f, "Loading..."),
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Ok(None)
    }
}
#[derive(Debug, Clone)]
pub enum FullCommand {
    Filter(FilterType),
    List(ListType),
    Search(String, FileType, Filter),
}

#[derive(Debug, Clone)]
pub struct FilterType {
    pub thing: CommandResult,
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryFilterType {
    Date(String),
    DateRange(String, String),
    FunctionInBlock(String),
    FunctionInLines(String, String),
    FunctionInFunction(String),
    CommitId(String),
    FileAbsolute(String),
    FileRelative(String),
    Directory(String),
    None,
}

impl fmt::Display for HistoryFilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryFilterType::Date(_) => write!(f, "date"),
            HistoryFilterType::DateRange(_, _) => write!(f, "date range"),
            HistoryFilterType::FunctionInBlock(_) => write!(f, "function in block"),
            HistoryFilterType::FunctionInLines(_, _) => write!(f, "function in lines"),
            HistoryFilterType::FunctionInFunction(_) => write!(f, "function in function"),
            HistoryFilterType::CommitId(_) => write!(f, "commit hash"),
            HistoryFilterType::FileAbsolute(_) => write!(f, "file absolute"),
            HistoryFilterType::FileRelative(_) => write!(f, "file relative"),
            HistoryFilterType::Directory(_) => write!(f, "directory"),
            HistoryFilterType::None => write!(f, "none"),
        }
    }
}
