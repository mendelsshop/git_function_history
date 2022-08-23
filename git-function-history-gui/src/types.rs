use std::fmt;

use git_function_history::{CommitFunctions, File, FileType, Filter, FunctionHistory};

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
#[derive(Debug, Clone)]
pub enum Status {
    Ok(Option<String>),
    Error(String),
    Loading,
}

impl Default for Status {
    fn default() -> Self {
        Status::Ok(None)
    }
}
#[derive(Debug, Clone)]
pub enum FullCommand {
    Filter(),
    List(ListType),
    Search(String, FileType, Filter),
}

#[derive(Debug, Clone)]
pub struct Index(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTypeS {
    None,
    Absolute,
    Relative,
}

impl fmt::Display for FileTypeS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTypeS::None => write!(f, "none"),
            FileTypeS::Absolute => write!(f, "absolute"),
            FileTypeS::Relative => write!(f, "relative"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterS {

        CommitId,

        Date,

        DateRange,

        None,
}

impl fmt::Display for FilterS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterS::CommitId => write!(f, "commit hash"),
            FilterS::Date => write!(f, "date"),
            FilterS::DateRange => write!(f, "date range"),
            FilterS::None => write!(f, "none"),
        }
    }
}