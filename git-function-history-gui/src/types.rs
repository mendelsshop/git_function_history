use function_grep::filter::FilterType;
use std::{collections::HashMap, fmt};

pub enum HistoryFilterType {
    Date(String),
    DateRange(String, String),
    CommitHash(String),
    FileAbsolute(String),
    FileRelative(String),
    Directory(String),
    // if filter type is a many and it has more than on filter
    // 1. if you can pick which language to use
    // 2. or you can add or remove a field to apply to the filter
    PL(PLFilter),
    None,
}
pub enum PLFilter {
    Single(HashMap<String, String>, FilterType<'static>),
    Many(
        // if the field should be deleted
        HashMap<String, (bool, String)>,
        FilterType<'static>,
        Vec<String>,
        // choose a sepcific language that supports this filter
        Option<String>,
        // next field
        String,
    ),
}
impl PLFilter {
    fn filter_name(&self) -> String {
        match self {
            PLFilter::Single(_, single_or_many) => single_or_many.filter_name(),
            PLFilter::Many(_, filter, _, _, _) => filter.filter_name(),
        }
    }
}

impl PartialEq for HistoryFilterType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Date(l0), Self::Date(r0)) => l0 == r0,
            (Self::DateRange(l0, l1), Self::DateRange(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::CommitHash(l0), Self::CommitHash(r0)) => l0 == r0,
            (Self::FileAbsolute(l0), Self::FileAbsolute(r0)) => l0 == r0,
            (Self::FileRelative(l0), Self::FileRelative(r0)) => l0 == r0,
            (Self::Directory(l0), Self::Directory(r0)) => l0 == r0,
            (Self::PL(l0), Self::PL(r0)) => l0.filter_name() == r0.filter_name(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for HistoryFilterType {}

impl fmt::Debug for HistoryFilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date(arg0) => f.debug_tuple("Date").field(arg0).finish(),
            Self::DateRange(arg0, arg1) => {
                f.debug_tuple("DateRange").field(arg0).field(arg1).finish()
            }
            Self::CommitHash(arg0) => f.debug_tuple("CommitHash").field(arg0).finish(),
            Self::FileAbsolute(arg0) => f.debug_tuple("FileAbsolute").field(arg0).finish(),
            Self::FileRelative(arg0) => f.debug_tuple("FileRelative").field(arg0).finish(),
            Self::Directory(arg0) => f.debug_tuple("Directory").field(arg0).finish(),
            Self::PL(filter) => f.debug_tuple("PL").field(&filter.filter_name()).finish(),
            Self::None => write!(f, "None"),
        }
    }
}

impl fmt::Display for HistoryFilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryFilterType::Date(_) => write!(f, "date"),
            HistoryFilterType::DateRange(_, _) => write!(f, "date range"),
            HistoryFilterType::CommitHash(_) => write!(f, "commit hash"),
            HistoryFilterType::FileAbsolute(_) => write!(f, "file absolute"),
            HistoryFilterType::FileRelative(_) => write!(f, "file relative"),
            HistoryFilterType::Directory(_) => write!(f, "directory"),
            HistoryFilterType::PL(pl) => write!(f, "{}", pl.filter_name()),
            HistoryFilterType::None => write!(f, "none"),
        }
    }
}
