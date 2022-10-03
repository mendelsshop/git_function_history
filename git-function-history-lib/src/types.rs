use chrono::{DateTime, FixedOffset};
#[cfg(feature = "parallel")]
use rayon::prelude::ParallelIterator;
use std::{collections::HashMap, error::Error, fmt};

use crate::{
    languages::{c, python, rust, Function},
    Filter,
};

/// This is used to store each individual file in a commit and the associated functions in that file.
#[derive(Debug, Clone)]
pub struct File {
    /// The name of the file
    pub(crate) name: String,
    pub(crate) functions: FileType,
    pub(crate) current_pos: usize,
}

impl File {
    /// Create a new file with the given name and functions
    pub const fn new(name: String, functions: FileType) -> Self {
        Self {
            name,
            functions,
            current_pos: 0,
        }
    }

    /// This is used to get the functions in the file
    pub const fn get_functions(&self) -> &FileType {
        &self.functions
    }

    /// This is used to get the functions in the file (mutable)
    pub fn get_functions_mut(&mut self) -> &mut FileType {
        &mut self.functions
    }

    // /// This is will get the current function in the file
    // pub fn get_current_function(&self) -> Option<&FileType> {
    //     self.functions.get(self.current_pos)
    // }

    // /// This is will get the current function in the file (mutable)
    // pub fn get_current_function_mut(&mut self) -> Option<&mut FileType> {
    //     self.functions.get_mut(self.current_pos)
    // }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{}", self.name)
        match &self.functions {
            FileType::Python(python) => {
                for (i, function) in python.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => python.get(i - 1),
                    };
                    let next = python.get(i + 1);
                    function.fmt_with_context(f, previous, next)?;
                }
            }
            FileType::Rust(rust) => {
                for (i, function) in rust.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => rust.get(i - 1),
                    };
                    let next = rust.get(i + 1);
                    function.fmt_with_context(f, previous, next)?;
                }
            }
            FileType::C(c) => {
                for (i, function) in c.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => c.get(i - 1),
                    };
                    let next = c.get(i + 1);
                    function.fmt_with_context(f, previous, next)?;
                }
            }
        };
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub enum FileType {
    /// The python language
    Python(Vec<python::Function>),
    /// The rust language
    Rust(Vec<rust::Function>),
    /// c language
    C(Vec<c::Function>),
}

/// This holds information like date and commit `commit_hash` and also the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct CommitFunctions {
    commit_hash: String,
    files: Vec<File>,
    pub(crate) date: DateTime<FixedOffset>,
    current_iter_pos: usize,
    current_pos: usize,
    author: String,
    email: String,
    message: String,
}

impl CommitFunctions {
    /// Create a new `CommitFunctions` with the given `commit_hash`, functions, and date.
    pub fn new(
        commit_hash: String,
        files: Vec<File>,
        date: &str,
        author: String,
        email: String,
        message: String,
    ) -> Self {
        Self {
            commit_hash,
            files,
            date: DateTime::parse_from_rfc2822(date).expect("Failed to parse date"),
            current_pos: 0,
            current_iter_pos: 0,
            author,
            email,
            message,
        }
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
            self.files[self.current_pos].name.clone(),
        );
        map
    }

    /// returns the current file
    pub fn get_file(&self) -> &File {
        &self.files[self.current_pos]
    }

    /// returns the current file (mutable)
    pub fn get_file_mut(&mut self) -> &mut File {
        &mut self.files[self.current_pos]
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

    /// returns a new `CommitFunctions` by filtering the current one by the filter specified (does not modify the current one).
    ///
    /// valid filters are: `Filter::FunctionInBlock`, `Filter::FunctionInLines`, `Filter::FunctionWithParent`, and `Filter::FileAbsolute`, `Filter::FileRelative`, and `Filter::Directory`.
    pub fn filter_by(&self, _filter: &Filter) -> Result<Self, Box<dyn Error>> {
        // let mut vec = Vec::new();
        // for f in &self.files {
        //     match filter {
        //         Filter::FileAbsolute(file) => {
        //             if f.name == *file {
        //                 vec.push(f.clone());
        //             }
        //         }
        //         Filter::FileRelative(file) => {
        //             if f.name.ends_with(file) {
        //                 vec.push(f.clone());
        //             }
        //         }
        //         Filter::Directory(dir) => {
        //             if f.name.contains(dir) {
        //                 vec.push(f.clone());
        //             }
        //         }
        //         Filter::FunctionInLines(..)
        //         | Filter::FunctionWithParent(_)
        //         | Filter::FunctionInBlock(_) => {
        //             if f.filter_by(filter).is_ok() {
        //                 vec.push(f.clone());
        //             }
        //         }
        //         Filter::None => vec.push(f.clone()),
        //         _ => Err("Invalid filter")?,
        //     }
        // }
        // if vec.is_empty() {
        //     return Err("No files found for filter")?;
        // }
        // Ok(Self {
        //     commit_hash: self.commit_hash.clone(),
        //     files: vec,
        //     date: self.date,
        //     current_pos: 0,
        //     current_iter_pos: 0,
        //     author: self.author.clone(),
        //     email: self.email.clone(),
        //     message: self.message.clone(),
        // })
        todo!()
    }
}

impl Iterator for CommitFunctions {
    type Item = File;
    fn next(&mut self) -> Option<Self::Item> {
        // get the current function without removing it
        let function = self.files.get(self.current_iter_pos).cloned();
        self.current_iter_pos += 1;
        function
    }
}

impl fmt::Display for CommitFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.files[self.current_pos])?;
        Ok(())
    }
}

/// This struct holds the a list of commits and the function that were looked up for each commit.
#[derive(Debug, Clone)]
pub struct FunctionHistory {
    pub(crate) name: String,
    pub(crate) commit_history: Vec<CommitFunctions>,

    current_iter_pos: usize,
    current_pos: usize,
}

impl FunctionHistory {
    // creates a new `FunctionHistory` from a list of commits
    pub fn new(name: String, commit_history: Vec<CommitFunctions>) -> Self {
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
    pub fn move_forward(&mut self) {
        if self.current_pos >= self.commit_history.len() - 1 {
            return;
        }
        self.current_pos += 1;
        self.commit_history[self.current_pos].current_iter_pos = 0;
        self.commit_history[self.current_pos].current_pos = 0;
    }

    /// this will move to the previous commit if possible
    pub fn move_back(&mut self) {
        if self.current_pos == 0 {
            return;
        }
        self.current_pos -= 1;
        self.commit_history[self.current_pos].current_iter_pos = 0;
        self.commit_history[self.current_pos].current_pos = 0;
    }

    /// this will move to the next file in the current commit if possible
    pub fn move_forward_file(&mut self) {
        self.commit_history[self.current_pos].move_forward();
    }

    /// this will move to the previous file in the current commit if possible
    pub fn move_back_file(&mut self) {
        self.commit_history[self.current_pos].move_back();
    }

    /// this returns some metadata about the current commit
    /// including the `commit hash`, `date`, and `file`
    pub fn get_metadata(&self) -> HashMap<String, String> {
        self.commit_history[self.current_pos].get_metadata()
    }

    /// returns a mutable reference to the current commit
    pub fn get_mut_commit(&mut self) -> &mut CommitFunctions {
        &mut self.commit_history[self.current_pos]
    }

    /// returns a reference to the current commit
    pub fn get_commit(&self) -> &CommitFunctions {
        &self.commit_history[self.current_pos]
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
        self.commit_history[self.current_pos].get_move_direction()
    }

    /// returns a new `FunctionHistory` by filtering the current one by the filter specified (does not modify the current one).
    /// All filter are valid
    ///
    /// # examples
    /// ```rust
    /// use git_function_history::{get_function_history, Filter, FileType};
    ///
    /// let history = get_function_history("new", FileType::None, Filter::None).unwrap();
    ///
    /// history.filter_by(Filter::Directory("app".to_string())).unwrap();
    /// ```
    pub fn filter_by(&self, _filter: &Filter) -> Result<Self, Box<dyn Error>> {
        // #[cfg(feature = "parallel")]
        // let t = self.commit_history.par_iter();
        // #[cfg(not(feature = "parallel"))]
        // let t = self.commit_history.iter();
        // let vec: Vec<CommitFunctions> = t
        //     .filter(|f| match filter {
        //         Filter::FunctionInLines(..)
        //         | Filter::FunctionWithParent(_)
        //         | Filter::FunctionInBlock(_)
        //         | Filter::Directory(_)
        //         | Filter::FileAbsolute(_)
        //         | Filter::FileRelative(_) => f.filter_by(filter).is_ok(),
        //         Filter::CommitHash(commit_hash) => &f.commit_hash == commit_hash,
        //         Filter::Date(date) => &f.date.to_rfc2822() == date,
        //         Filter::DateRange(start, end) => {
        //             let start = match DateTime::parse_from_rfc2822(start) {
        //                 Ok(date) => date,
        //                 Err(_) => return false,
        //             };
        //             let end = match DateTime::parse_from_rfc2822(end) {
        //                 Ok(date) => date,
        //                 Err(_) => return false,
        //             };
        //             f.date >= start || f.date <= end
        //         }
        //         Filter::Author(author) => &f.author == author,
        //         Filter::AuthorEmail(email) => &f.email == email,
        //         Filter::Message(message) => f.message.contains(message),
        //         Filter::None => true,
        //     })
        //     .cloned()
        //     .collect();

        // if vec.is_empty() {
        //     return Err("No history found for the filter")?;
        // }
        // Ok(Self {
        //     commit_history: vec,
        //     name: self.name.clone(),
        //     current_pos: 0,
        //     current_iter_pos: 0,
        // })
        todo!()
    }
}

impl fmt::Display for FunctionHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.commit_history[self.current_pos])?;
        Ok(())
    }
}

impl Iterator for FunctionHistory {
    type Item = CommitFunctions;
    fn next(&mut self) -> Option<Self::Item> {
        self.commit_history
            .get(self.current_iter_pos)
            .cloned()
            .map(|c| {
                self.current_iter_pos += 1;
                c
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

trait ErrorToOption<FileType> {
    fn to_option(self) -> Option<FileType>;
}

impl<FileType> ErrorToOption<FileType> for Result<FileType, Box<dyn Error>> {
    fn to_option(self) -> Option<FileType> {
        match self {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}
