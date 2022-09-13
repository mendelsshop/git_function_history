use chrono::{DateTime, FixedOffset};
use std::{collections::HashMap, error::Error, fmt};

use crate::Filter;

pub(crate) struct InternalBlock {
    pub(crate) start: Points,
    pub(crate) full: Points,
    pub(crate) end: Points,
    pub(crate) types: BlockType,
    pub(crate) file_line: Points,
}

#[derive(Debug, Clone)]
pub(crate) struct InternalFunctions {
    pub(crate) name: String,
    pub(crate) range: Points,
    pub(crate) file_line: Points,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Points {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Points {
    pub(crate) const fn in_other(&self, other: &Self) -> bool {
        self.x > other.x && self.y < other.y
    }
}
/// This holds the information about a single function each commit will have multiple of these.
#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) name: String,
    /// The actual code of the function
    pub(crate) contents: String,
    /// is the function in a block ie `impl` `trait` etc
    pub(crate) block: Option<Block>,
    /// optional parent functions
    pub(crate) function: Option<Vec<FunctionBlock>>,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
}

impl Function {
    /// This is a formater almost like the fmt you use fro println!, but it takes a previous and next function.
    /// This is usefull for printing `CommitHistory` or a vector of functions, because if you use plain old fmt, you can get repeated lines impls, and parent function in your output.
    pub fn fmt_with_context(
        &self,
        f: &mut fmt::Formatter<'_>,
        previous: Option<&Self>,
        next: Option<&Self>,
    ) -> fmt::Result {
        match &self.block {
            None => {}
            Some(block) => match previous {
                None => write!(f, "{}\n...\n", block.top)?,
                Some(previous_function) => match &previous_function.block {
                    None => write!(f, "{}\n...\n", block.top)?,
                    Some(previous_block) => {
                        if previous_block.lines == block.lines {
                        } else {
                            write!(f, "{}\n...\n", block.top)?;
                        }
                    }
                },
            },
        };
        match &self.function {
            None => {}
            Some(function) => match previous {
                None => {
                    for i in function {
                        write!(f, "{}\n...\n", i.top)?;
                    }
                }
                Some(previous_function) => match &previous_function.function {
                    None => {
                        for i in function {
                            write!(f, "{}\n...\n", i.top)?;
                        }
                    }
                    Some(previous_function_parent) => {
                        for i in function {
                            if previous_function_parent
                                .iter()
                                .map(|parent| parent.lines)
                                .any(|x| x == i.lines)
                            {
                            } else {
                                write!(f, "{}\n...\n", i.top)?;
                            }
                        }
                    }
                },
            },
        };
        write!(f, "{}", self.contents)?;
        match &self.function {
            None => {}
            Some(function) => {
                let mut r_function = function.clone();
                r_function.reverse();
                match next {
                    None => {
                        for i in r_function {
                            write!(f, "\n...\n{}", i.bottom)?;
                        }
                    }
                    Some(next_function) => match &next_function.function {
                        None => {
                            for i in r_function {
                                write!(f, "\n...\n{}", i.bottom)?;
                            }
                        }

                        Some(next_function_parent) => {
                            for i in r_function {
                                if next_function_parent
                                    .iter()
                                    .map(|parent| parent.lines)
                                    .any(|x| x == i.lines)
                                {
                                } else {
                                    write!(f, "\n...\n{}", i.bottom)?;
                                }
                            }
                        }
                    },
                }
            }
        };
        match &self.block {
            None => {}
            Some(block) => match next {
                None => write!(f, "\n...{}", block.bottom)?,
                Some(next_function) => match &next_function.block {
                    None => write!(f, "\n...{}", block.bottom)?,
                    Some(next_block) => {
                        if next_block.lines == block.lines {
                        } else {
                            write!(f, "\n...{}", block.bottom)?;
                        }
                    }
                },
            },
        };
        Ok(())
    }

    /// get metadata like line number, number of parent function etc.
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("lines".to_string(), format!("{:?}", self.lines));
        map.insert("contents".to_string(), self.contents.clone());
        if let Some(block) = &self.block {
            map.insert("block".to_string(), format!("{}", block.block_type));
        }
        if let Some(function) = &self.function {
            map.insert(
                "number of function".to_string(),
                format!("{}", function.len()),
            );
        }
        map
    }

    /// get the parent functions
    pub fn get_parent_function(&self) -> Option<Vec<FunctionBlock>> {
        self.function.clone()
    }

    /// get the block of the function
    pub fn get_block(&self) -> Option<Block> {
        self.block.clone()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.block {
            None => {}
            Some(block) => write!(f, "{}\n...\n", block.top)?,
        };
        match &self.function {
            None => {}
            Some(function) => {
                for i in function {
                    write!(f, "{}\n...\n", i.top)?;
                }
            }
        };
        write!(f, "{}", self.contents)?;
        match &self.function {
            None => {}
            Some(function) => {
                for i in function {
                    write!(f, "\n...\n{}", i.bottom)?;
                }
            }
        };
        match &self.block {
            None => {}
            Some(block) => write!(f, "\n...{}", block.bottom)?,
        };
        Ok(())
    }
}

/// This is used for the functions that are being looked up themeselves but store an outer function that may aontains a function that is being looked up.
#[derive(Debug, Clone)]
pub struct FunctionBlock {
    /// The name of the function (parent function)
    pub(crate) name: String,
    /// what the signature of the function is
    pub(crate) top: String,
    /// what the last line of the function is
    pub(crate) bottom: String,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
}

impl FunctionBlock {
    /// get the metadata for this block ie the name of the block, the type of block, the line number the block starts and ends
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("lines".to_string(), format!("{:?}", self.lines));
        map.insert("signature".to_string(), self.top.clone());
        map.insert("bottom".to_string(), self.bottom.clone());
        map
    }
}

/// This holds information about when a function is in an impl/trait/extern block
#[derive(Debug, Clone)]
pub struct Block {
    /// The name of the block ie for `impl` it would be the type were impling for
    pub(crate) name: Option<String>,
    /// The signature of the block
    pub(crate) top: String,
    /// The last line of the block
    pub(crate) bottom: String,
    /// the type of block ie `impl` `trait` `extern`
    pub(crate) block_type: BlockType,
    /// The line number the function starts and ends on
    pub(crate) lines: (usize, usize),
}

impl Block {
    /// get the metadata for this block ie the name of the block, the type of block, the line number the block starts and ends
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(name) = &self.name {
            map.insert("name".to_string(), name.to_string());
        }
        map.insert("block".to_string(), format!("{}", self.block_type));
        map.insert("lines".to_string(), format!("{:?}", self.lines));
        map.insert("signature".to_string(), self.top.clone());
        map.insert("bottom".to_string(), self.bottom.clone());
        map
    }
}

/// This enum is used when filtering commit history only for let say impl and not externs or traits
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    /// This is for `impl` blocks
    Impl,
    /// This is for `trait` blocks
    Extern,
    /// This is for `extern` blocks
    Trait,
    /// This is for code that gets labeled as a block but `get_function_history` can't find a block type
    Unknown,
}

impl BlockType {
    /// This is used to get the name of the block type from a string
    pub fn from_string(s: &str) -> Self {
        match s {
            "impl" => Self::Impl,
            "extern" => Self::Extern,
            "trait" => Self::Trait,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Impl => write!(f, "impl"),
            Self::Extern => write!(f, "extern"),
            Self::Trait => write!(f, "trait"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// This is used to store each individual file in a commit and the associated functions in that file.
#[derive(Debug, Clone)]
pub struct File {
    /// The name of the file
    pub(crate) name: String,
    functions: Vec<Function>,
    current_pos: usize,
}

impl File {
    /// Create a new file with the given name and functions
    pub fn new(name: String, functions: Vec<Function>) -> Self {
        Self {
            name,
            functions,
            current_pos: 0,
        }
    }

    /// returns a new `File` by filtering the current one by the filter specified (does not modify the current one).
    ///
    /// valid filters are: `Filter::FunctionInBlock`, `Filter::FunctionInLines`, and `Filter::FunctionWithParent`.
    pub fn filter_by(&self, filter: Filter) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<Function> = match filter {
            Filter::FunctionInBlock(block_type) => self
                .functions
                .iter()
                .filter(|f| {
                    f.block
                        .as_ref()
                        .map_or(false, |block| block.block_type == block_type)
                })
                .cloned()
                .collect(),
            Filter::FunctionInLines(start, end) => self
                .functions
                .iter()
                .filter(|f| f.lines.0 >= start && f.lines.1 <= end)
                .cloned()
                .collect(),
            Filter::FunctionWithParent(parent) => self
                .functions
                .iter()
                .filter(|f| {
                    if let Some(parent_function) = &f.function {
                        for parents in parent_function {
                            if parents.name == parent {
                                return true;
                            }
                        }
                    }
                    false
                })
                .cloned()
                .collect(),
            Filter::None => self.functions.clone(),
            _ => return Err("Filter not available")?,
        };
        if vec.is_empty() {
            return Err("No functions found for filter")?;
        }
        Ok(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }

    /// This is used to get the functions in the file
    pub const fn get_functions(&self) -> &Vec<Function> {
        &self.functions
    }

    /// This is used to get the functions in the file (mutable)
    pub fn get_functions_mut(&mut self) -> &mut Vec<Function> {
        &mut self.functions
    }
}

impl Iterator for File {
    type Item = Function;
    fn next(&mut self) -> Option<Self::Item> {
        // get the current function without removing it
        let function = self.functions.get(self.current_pos).cloned();
        self.current_pos += 1;
        function
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, function) in self.functions.iter().enumerate() {
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
                _ => self.functions.get(i - 1),
            };
            let next = self.functions.get(i + 1);
            function.fmt_with_context(f, previous, next)?;
        }
        Ok(())
    }
}

/// This holds information like date and commit `commit_hash` and also the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct CommitFunctions {
    commit_hash: String,
    files: Vec<File>,
    date: DateTime<FixedOffset>,
    current_iter_pos: usize,
    current_pos: usize,
}

impl CommitFunctions {
    /// Create a new `CommitFunctions` with the given `commit_hash`, functions, and date.
    pub fn new(commit_hash: String, files: Vec<File>, date: &str) -> Self {
        Self {
            commit_hash,
            files,
            date: DateTime::parse_from_rfc2822(date).expect("Failed to parse date"),
            current_pos: 0,
            current_iter_pos: 0,
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
    pub fn filter_by(&self, filter: Filter) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<File> = match filter {
            Filter::FileAbsolute(file) => self
                .files
                .iter()
                .filter(|f| f.name == file)
                .cloned()
                .collect(),
            Filter::FileRelative(file) => self
                .files
                .iter()
                .filter(|f| f.name.ends_with(&file))
                .cloned()
                .collect(),
            Filter::Directory(dir) => self
                .files
                .iter()
                .filter(|f| f.name.contains(&dir))
                .cloned()
                .collect(),
            Filter::FunctionInLines(..)
            | Filter::FunctionWithParent(_)
            | Filter::FunctionInBlock(_) => self
                .files
                .iter()
                .filter_map(|f| f.filter_by(filter.clone()).to_option())
                .collect(),
            Filter::None => self.files.clone(),
            _ => return Err("Invalid filter")?,
        };
        if vec.is_empty() {
            return Err("No files found for filter")?;
        }
        Ok(Self {
            commit_hash: self.commit_hash.clone(),
            files: vec,
            date: self.date,
            current_pos: 0,
            current_iter_pos: 0,
        })
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
    /// This will return a vector of all the commit ids in the history.
    pub fn list_commit_ids(&self) -> Vec<&str> {
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
    }

    /// this will move to the previous commit if possible
    pub fn move_back(&mut self) {
        if self.current_pos == 0 {
            return;
        }
        self.current_pos -= 1;
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
    pub fn filter_by(&self, filter: Filter) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<CommitFunctions> = match filter {
            Filter::FunctionInLines(..)
            | Filter::FunctionWithParent(_)
            | Filter::FunctionInBlock(_)
            | Filter::Directory(_)
            | Filter::FileAbsolute(_)
            | Filter::FileRelative(_) => self
                .commit_history
                .iter()
                .filter_map(|f| f.filter_by(filter.clone()).to_option())
                .collect(),
            Filter::CommitId(commit_hash) => self
                .commit_history
                .iter()
                .filter(|f| f.commit_hash == commit_hash)
                .cloned()
                .collect(),
            Filter::Date(date) => self
                .commit_history
                .iter()
                .filter(|f| f.date.to_rfc2822() == date)
                .cloned()
                .collect(),
            Filter::DateRange(start, end) => {
                let start = DateTime::parse_from_rfc2822(&start).expect("Failed to parse date");
                let end = DateTime::parse_from_rfc2822(&end).expect("Failed to parse date");
                if start >= end {
                    return Err("Start date is after end date")?;
                }
                self.commit_history
                    .iter()
                    .filter(|c| c.date >= start || c.date <= end)
                    .cloned()
                    .collect()
            }
            Filter::None => self.commit_history.clone(),
        };
        if vec.is_empty() {
            return Err("No history found for the filter")?;
        }
        Ok(Self {
            commit_history: vec,
            name: self.name.clone(),
            current_pos: 0,
            current_iter_pos: 0,
        })
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
    // You can only move forward
    Forward,
    // You can only move back
    Back,
    // You can't move in any direction
    None,
    // You can move in both directions
    Both,
}

trait ErrorToOption<T> {
    fn to_option(self) -> Option<T>;
}

impl<T> ErrorToOption<T> for Result<T, Box<dyn Error>> {
    fn to_option(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}
