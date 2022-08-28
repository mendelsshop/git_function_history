use std::fmt;

use git_function_history::{BlockType, CommitFunctions, File, FileType, Filter, FunctionHistory};

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
    Filter(FilterType),
    List(ListType),
    Search(String, FileType, Filter),
}

#[derive(Debug, Clone)]
pub struct Index(pub usize, pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTypeS {
    None,
    Absolute(String),
    Relative(String)
}

impl fmt::Display for FileTypeS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTypeS::None => write!(f, "none"),
            FileTypeS::Absolute(_) => write!(f, "absolute"),
            FileTypeS::Relative(_) => write!(f, "relative"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchFilter {
    CommitId(String),
    Date(String),
    DateRange(String, String),
    None,
}

impl fmt::Display for SearchFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchFilter::CommitId(_) => write!(f, "commit hash"),
            SearchFilter::Date(_) => write!(f, "date"),
            SearchFilter::DateRange(_, _) => write!(f, "date range"),
            SearchFilter::None => write!(f, "none"),
        }
    }
}
#[derive(Debug, Clone)]
pub enum FilterType {
    History(HistoryFilter, FunctionHistory),
    CommitOrFile(CommitOrFileFilter, CommitFunctions),
}
#[derive(Debug, Clone)]
pub enum HistoryFilter {
    Date(String),
    CommitId(String),
    DateRange(String, String),
    FunctionInBlock(BlockType),
    FunctionInLines(usize, usize),
    FunctionInFunction(String),
}
#[derive(Debug, Clone)]
pub enum CommitOrFileFilter {
    FunctionInBlock(BlockType),
    FunctionInLines(usize, usize),
    FunctionInFunction(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryFilterType {
    Date(String),
    DateRange(String, String),
    FunctionInBlock(String),
    FunctionInLines(String, String),
    FunctionInFunction(String),
    CommitId(String),
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
            HistoryFilterType::CommitId(_) => write!(f, "commit id"),
            HistoryFilterType::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Clone,  PartialEq, Eq)]
pub enum CommitFilterType {
    FunctionInBlock(String),
    FunctionInLines(String, String),
    FunctionInFunction(String),
    None,
}

impl fmt::Display for CommitFilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitFilterType::FunctionInBlock(_) => write!(f, "function in block"),
            CommitFilterType::FunctionInLines(_, _) => write!(f, "function in lines"),
            CommitFilterType::FunctionInFunction(_) => write!(f, "function in function"),
            CommitFilterType::None => write!(f, "none"),
        }
    }
}