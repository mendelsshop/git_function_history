use chrono::{DateTime, FixedOffset};
use function_grep::ParsedFile;
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::Filter;

#[derive(Debug, Clone)]
pub enum ErrorReason {
    NoHistory,
    Other(String),
}

impl fmt::Display for ErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHistory => write!(f, "nothing found"),
            Self::Other(other) => write!(f, "{other}"),
        }
    }
}

/// This holds information like date and commit `commit_hash` and also the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct Commit {
    commit_hash: String,
    pub(crate) files: Vec<ParsedFile>,
    pub(crate) date: DateTime<FixedOffset>,
    current_iter_pos: usize,
    current_pos: usize,
    author: String,
    email: String,
    message: String,
}

impl Commit {
    /// Create a new `Commit` with the given `commit_hash`, functions, and date.
    ///
    /// # Errors
    ///
    /// will return `Err` if it cannot parse the date provided.
    pub fn new(
        commit_hash: &str,
        files: Vec<ParsedFile>,
        date: &str,
        author: &str,
        email: &str,
        message: &str,
    ) -> Result<Self, String> {
        Ok(Self {
            commit_hash: commit_hash.to_string(),
            files,
            date: DateTime::parse_from_rfc2822(date).map_err(|e| e.to_string())?,
            current_pos: 0,
            current_iter_pos: 0,
            author: author.to_string(),
            email: email.to_string(),
            message: message.to_string(),
        })
    }

    /// sets the current file to the next file if possible
    pub fn move_forward(&mut self) {
        if self.current_pos >= self.files.len() - 1 {
            return;
        }
        self.current_pos += 1;
    }

    /// sets the current file to the previous file if possible
    pub fn move_back(&mut self) {
        if self.current_pos == 0 {
            return;
        }
        self.current_pos -= 1;
    }

    /// returns a hashmap containing the commits metadata
    /// inlcuding the `commit hash`, `date`, and `file`
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("commit hash".to_string(), self.commit_hash.clone());
        map.insert("date".to_string(), self.date.to_rfc2822());
        map.insert(
            "file".to_string(),
            self.files.get(self.current_pos).map_or("error occured, could not get filename, no file found\nfile a bug to https://github.com/mendelsshop/git_function_history/issues".to_string(), |file|file.file_name().expect("error ocurred, could not get filename, no filename for current file\nfile a bug to https://github.com/mendelsshop/git_function_history/issues").to_string()),
        );
        map
    }

    /// returns the current file
    pub fn get_file(&self) -> Option<&ParsedFile> {
        self.files.get(self.current_pos)
    }

    /// returns the current file (mutable)
    pub fn get_file_mut(&mut self) -> Option<&mut ParsedFile> {
        self.files.get_mut(self.current_pos)
    }

    /// tells you in which directions you can move through the files in the commit
    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.files.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.files.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    /// returns a new `Commit` by filtering the current one by the filter specified (does not modify the current one).
    ///
    /// valid filters are: `Filter::Language`, `Filter::PLFilter`,  `Filter::FileAbsolute`, `Filter::FileRelative`, `Filter::None`, and `Filter::Directory`.
    ///
    /// # Errors
    ///
    /// Will result in an `Err` if a non-valid filter is given, or if no results are found for the given filter
    pub fn filter_by(&self, filter: &Filter) -> Result<Self, ErrorReason> {
        match filter {
            Filter::FileAbsolute(_)
            | Filter::FileRelative(_)
            | Filter::Directory(_)
            | Filter::PLFilter(_)
            | Filter::Language(_)
            | Filter::None => {}
            _ => return Err(ErrorReason::Other(format!("Invalid filter {filter:?}"))),
        }
        #[cfg(feature = "parallel")]
        let t = self.files.iter();
        #[cfg(not(feature = "parallel"))]
        let t = self.files.iter();
        let vec: Vec<_> = t
            .filter_map(|f| match filter {
                Filter::FileAbsolute(file) => {
                    if f.file_name()? == *file {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::FileRelative(file) => {
                    if f.file_name()?.ends_with(file) {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::Directory(dir) => {
                    if f.file_name()?.contains(dir) {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::Language(lang) => {
                    if f.language() == *lang {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::PLFilter(filter) => f.filter(filter).ok(),
                _ => None,
            })
            .collect();

        if vec.is_empty() {
            return Err(ErrorReason::Other("No files found for filter".to_string()))?;
        }
        Ok(Self {
            commit_hash: self.commit_hash.clone(),
            files: vec,
            date: self.date,
            current_pos: 0,
            current_iter_pos: 0,
            author: self.author.clone(),
            email: self.email.clone(),
            message: self.message.clone(),
        })
    }
}

impl Iterator for Commit {
    type Item = ParsedFile;
    fn next(&mut self) -> Option<Self::Item> {
        // get the current function without removing it
        let function = self.files.get(self.current_iter_pos).cloned();
        self.current_iter_pos += 1;
        function
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            match self.files.get(self.current_pos) {
                Some(file) => file,
                None => return Err(fmt::Error),
            }
        )?;
        Ok(())
    }
}

/// This struct holds the a list of commits and the function that were looked up for each commit.
#[derive(Debug, Clone)]
pub struct FunctionHistory {
    pub(crate) name: String,
    pub(crate) commit_history: Vec<Commit>,
    current_iter_pos: usize,
    current_pos: usize,
}

impl FunctionHistory {
    // creates a new `FunctionHistory` from a list of commits
    pub const fn new(name: String, commit_history: Vec<Commit>) -> Self {
        Self {
            name,
            commit_history,
            current_iter_pos: 0,
            current_pos: 0,
        }
    }
    /// This will return a vector of all the commit hashess in the history.
    pub fn list_commit_hashes(&self) -> Vec<&str> {
        self.commit_history
            .iter()
            .map(|c| c.commit_hash.as_ref())
            .collect()
    }

    /// this will move to the next commit if possible
    pub fn move_forward(&mut self) -> Option<()> {
        if self.current_pos >= self.commit_history.len() - 1 {
            return None;
        }
        self.current_pos += 1;
        self.commit_history
            .get_mut(self.current_pos)?
            .current_iter_pos = 0;
        self.commit_history.get_mut(self.current_pos)?.current_pos = 0;
        Some(())
    }

    /// this will move to the previous commit if possible
    pub fn move_back(&mut self) -> Option<()> {
        if self.current_pos == 0 {
            return None;
        }
        self.current_pos -= 1;
        self.commit_history
            .get_mut(self.current_pos)?
            .current_iter_pos = 0;
        self.commit_history.get_mut(self.current_pos)?.current_pos = 0;
        Some(())
    }

    /// this will move to the next file in the current commit if possible
    pub fn move_forward_file(&mut self) {
        self.commit_history
            .get_mut(self.current_pos)
            .map(Commit::move_forward);
    }

    /// this will move to the previous file in the current commit if possible
    pub fn move_back_file(&mut self) {
        self.commit_history
            .get_mut(self.current_pos)
            .map(Commit::move_back);
    }

    /// this returns some metadata about the current commit
    /// including the `commit hash`, `date`, and `file`
    pub fn get_metadata(&self) -> HashMap<String, String> {
        self.commit_history
            .get(self.current_pos)
            .map_or_else(HashMap::new, Commit::get_metadata)
    }

    /// returns a mutable reference to the current commit
    pub fn get_mut_commit(&mut self) -> Option<&mut Commit> {
        self.commit_history.get_mut(self.current_pos)
    }

    /// returns a reference to the current commit
    pub fn get_commit(&self) -> Option<&Commit> {
        self.commit_history.get(self.current_pos)
    }

    /// returns the directions in which ways you can move through the commit history
    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.commit_history.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.commit_history.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    /// tells you in which directions you can move through the files in the current commit
    pub fn get_commit_move_direction(&self) -> Directions {
        self.commit_history
            .get(self.current_pos)
            .map_or(Directions::None, Commit::get_move_direction)
    }
    /// returns a new `FunctionHistory` by filtering the current one by the filter specified (does not modify the current one).
    /// All filter are valid
    ///
    /// # examples
    /// ```rust
    /// use git_function_history::{get_function_history, Filter, FileFilterType, Language};
    ///
    /// let history = get_function_history("new", &FileFilterType::None, &Filter::None, &Language::Rust).unwrap();
    ///
    /// history.filter_by(&Filter::Directory("app".to_string())).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// returns `Err` if no files or commits are match the filter specified
    pub fn filter_by(&self, filter: &Filter) -> Result<Self, ErrorReason> {
        #[cfg(feature = "parallel")]
        let t = self.commit_history.par_iter();
        #[cfg(not(feature = "parallel"))]
        let t = self.commit_history.iter();
        let vec: Vec<Commit> = t
            .filter_map(|f| match filter {
                Filter::PLFilter(_)
                | Filter::Directory(_)
                | Filter::FileAbsolute(_)
                | Filter::FileRelative(_)
                | Filter::Language(_) => f.filter_by(filter).ok(),
                Filter::CommitHash(commit_hash) => {
                    if &f.commit_hash == commit_hash {
                        Some(f.clone())
                    } else {
                        None
                    }
                }

                Filter::Date(date) => {
                    if &f.date.to_rfc2822() == date {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::DateRange(start, end) => {
                    let Ok(start) = DateTime::parse_from_rfc2822(start) else {
                        return None;
                    };
                    let Ok(end) = DateTime::parse_from_rfc2822(end) else {
                        return None;
                    };
                    if f.date >= start || f.date <= end {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::Author(author) => {
                    if &f.author == author {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::AuthorEmail(email) => {
                    if &f.email == email {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::Message(message) => {
                    if f.message.contains(message) {
                        Some(f.clone())
                    } else {
                        None
                    }
                }
                Filter::None => None,
            })
            .collect();

        if vec.is_empty() {
            return Err(ErrorReason::NoHistory);
        }
        Ok(Self {
            commit_history: vec,
            name: self.name.clone(),
            current_pos: 0,
            current_iter_pos: 0,
        })
    }
}

// TODO: fix this documentaton (and maybe the whole macro)

/// Macro to filter a the whole git history, a singe commit, or a file.
///
/// All variants take the thing to be filtered as the first argument.
///
/// If you just want to pass in a filter of type `Filter` pass in as the second argument the filter.
///
/// if you just want to pass in a `LanguageFilter` pass in as the second argument the filter and the final argument literal such as 5 or 'a' or "a".
/// This is just to differentiate between the first two variants of the macro.
///
/// Finally, if you just want to pass in a specific `LanguageFilter` like `RustFilter` pass in as the second argument the filter
/// and the 3rd argument should the variant of `LanguageFilter` such as `Rust`
#[macro_export]
macro_rules! filter_by {
    // option 1: takes a filter
    ($self:expr, $filter:expr) => {
        $self.filter_by(&$filter)
    };
    // option 2: takes a PLFilter variant
    ($self:expr, $pl_filter:expr, $cfg:literal) => {
        $self.filter_by(&Filter::PLFilter($pl_filter))
    };
    // option 3: takes a language specific filter ie RustFilter and a language ie Rust
    ($self:expr, $lang_filter:expr, $language:ident) => {{
        use $crate::languages::LanguageFilter;
        $self.filter_by(&Filter::PLFilter(LanguageFilter::$language($lang_filter)))
    }};
}

impl Display for FunctionHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            self.commit_history.get(self.current_pos).map_or(
                "could not retrieve commit please file a bug".to_string(),
                ToString::to_string
            )
        )?;
        Ok(())
    }
}

impl Iterator for FunctionHistory {
    type Item = Commit;
    fn next(&mut self) -> Option<Self::Item> {
        self.commit_history
            .get(self.current_iter_pos)
            .cloned()
            .inspect(|c| {
                self.current_iter_pos += 1;
            })
    }
}

/// Options returned when you use `get_move_direction`
/// It tells you which way you could move through the commits or files
pub enum Directions {
    /// You can only move forward
    Forward,
    /// You can only move back
    Back,
    /// You can't move in any direction
    None,
    /// You can move in both directions
    Both,
}
