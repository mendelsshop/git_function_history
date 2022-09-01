use git_function_history::{BlockType, CommitFunctions, File, FileType, Filter, FunctionHistory};

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
pub enum FullCommand {
    Filter(FilterType),
    List(ListType),
    Search(String, FileType, Filter),
}

#[derive(Debug, Clone)]
pub enum FilterType {
    History(HistoryFilter, FunctionHistory),
    CommitOrFile(CommitOrFileFilter, CommmitFilterValue),
}
#[derive(Debug, Clone)]
pub enum CommmitFilterValue {
    Commit(CommitFunctions),
    File(File),
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