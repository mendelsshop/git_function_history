use function_grep::filter::Filter;
use std::{collections::HashMap, fmt};

pub enum HistoryFilterType {
    Date(String),
    DateRange(String, String),
    CommitHash(String),
    FileAbsolute(String),
    FileRelative(String),
    Directory(String),
    PL(HashMap<String, String>, &'static dyn Filter),
    None,
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
            (Self::PL(_, l1), Self::PL(_, r1)) => l1.filter_info() == r1.filter_info(),
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
            Self::PL(arg0, arg1) =>f
                .debug_tuple("PL")
                .field(arg0)
                .field(&arg1.filter_info())
                .finish(),
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
            HistoryFilterType::PL(_, pl) => write!(f, "{}", pl.filter_info()),
            HistoryFilterType::None => write!(f, "none"),
        }
    }
}
