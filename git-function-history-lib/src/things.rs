use chrono::{DateTime, FixedOffset};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self},
};

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
    pub name: String,
    pub contents: String,
    pub block: Option<Block>,
    pub function: Option<Vec<FunctionBlock>>,
    pub lines: (usize, usize),
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
    pub name: String,
    pub top: String,
    pub bottom: String,
    pub lines: (usize, usize),
}

/// This holds information about when a function is in an impl/trait/extern block
#[derive(Debug, Clone)]
pub struct Block {
    pub name: Option<String>,
    pub top: String,
    pub bottom: String,
    pub block_type: BlockType,
    pub lines: (usize, usize),
}
/// This enum is used when filtering commit history only for let say impl and not externs or traits
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    Impl,
    Extern,
    Trait,
    Unknown,
}

impl BlockType {
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
    pub name: String,
    pub functions: Vec<Function>,
    current_pos: usize,
}

impl File {
    pub fn new(name: String, functions: Vec<Function>) -> Self {
        Self {
            name,
            functions,
            current_pos: 0,
        }
    }

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

/// This holds information like date and commit id and also the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct CommitFunctions {
    pub id: String,
    pub functions: Vec<File>,
    pub date: DateTime<FixedOffset>,
    current_iter_pos: usize,
    current_pos: usize,
}

impl CommitFunctions {
    // TODO: add a function to filter by filename
    pub(crate) fn new(id: String, functions: Vec<File>, date: &str) -> Self {
        Self {
            id,
            functions,
            date: DateTime::parse_from_rfc2822(date).expect("Failed to parse date"),
            current_pos: 0,
            current_iter_pos: 0,
        }
    }

    pub fn move_forward(&mut self) -> bool {
        if self.current_pos >= self.functions.len() - 1 {
            return false;
        }
        self.current_pos += 1;
        true
    }

    pub fn move_back(&mut self) -> bool {
        if self.current_pos == 0 {
            return false;
        }
        self.current_pos -= 1;
        true
    }

    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("commit hash".to_string(), self.id.clone());
        map.insert("date".to_string(), self.date.to_rfc2822());
        map.insert(
            "file".to_string(),
            self.functions[self.current_pos].name.clone(),
        );
        map
    }

    pub fn get_file(&self) -> File {
        self.functions[self.current_pos].clone()
    }
    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.functions.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.functions.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    pub fn filter_by(&self, filter: Filter) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<File> = match filter {
            Filter::FileAbsolute(file) => self
                .functions
                .iter()
                .filter(|f| f.name == file)
                .cloned()
                .collect(),
            Filter::FileRelative(file) => self
                .functions
                .iter()
                .filter(|f| f.name.ends_with(&file))
                .cloned()
                .collect(),
            Filter::Directory(dir) => self
                .functions
                .iter()
                .filter(|f| f.name.contains(&dir))
                .cloned()
                .collect(),
            Filter::FunctionInLines(..)
            | Filter::FunctionWithParent(_)
            | Filter::FunctionInBlock(_) => self
                .functions
                .iter()
                .filter_map(|f| f.filter_by(filter.clone()).to_option())
                .collect(),

            _ => return Err("Invalid filter")?,
        };
        if vec.is_empty() {
            return Err("No files found for filter")?;
        }
        Ok(Self {
            id: self.id.clone(),
            functions: vec,
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
        let function = self.functions.get(self.current_iter_pos).cloned();
        self.current_iter_pos += 1;
        function
    }
}

impl fmt::Display for CommitFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.functions[self.current_pos])?;
        Ok(())
    }
}

/// This struct holds the a list of commits and the function that were looked up for each commit.
#[derive(Debug, Clone)]
pub struct FunctionHistory {
    pub name: String,
    pub history: Vec<CommitFunctions>,
    current_iter_pos: usize,
    current_pos: usize,
}

impl FunctionHistory {
    // TODO: add a function to filter by filename
    pub fn new(name: String, history: Vec<CommitFunctions>) -> Self {
        Self {
            name,
            history,
            current_iter_pos: 0,
            current_pos: 0,
        }
    }
    /// This will return a vector of all the commit ids in the history.
    pub fn list_commit_ids(&self) -> Vec<&str> {
        self.history.iter().map(|c| c.id.as_ref()).collect()
    }

    pub fn move_forward(&mut self) -> bool {
        if self.current_pos >= self.history.len() - 1 {
            return false;
        }
        self.current_pos += 1;
        true
    }

    pub fn move_back(&mut self) -> bool {
        if self.current_pos == 0 {
            return false;
        }
        self.current_pos -= 1;
        true
    }

    pub fn move_forward_file(&mut self) -> bool {
        self.history[self.current_pos].move_forward()
    }

    pub fn move_back_file(&mut self) -> bool {
        self.history[self.current_pos].move_back()
    }

    pub fn get_metadata(&self) -> HashMap<String, String> {
        self.history[self.current_pos].get_metadata()
    }

    pub fn get_mut_commit(&mut self) -> &mut CommitFunctions {
        &mut self.history[self.current_pos]
    }

    pub fn get_commit(&self) -> &CommitFunctions {
        &self.history[self.current_pos]
    }

    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.history.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.history.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    pub fn filter_by(&self, filter: Filter) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<CommitFunctions> = match filter {
            Filter::FunctionInLines(..)
            | Filter::FunctionWithParent(_)
            | Filter::FunctionInBlock(_)
            | Filter::Directory(_)
            | Filter::FileAbsolute(_)
            | Filter::FileRelative(_) => self
                .history
                .iter()
                .filter_map(|f| f.filter_by(filter.clone()).to_option())
                .collect(),
            Filter::CommitId(id) => self
                .history
                .iter()
                .filter(|f| f.id == id)
                .cloned()
                .collect(),
            Filter::Date(date) => self
                .history
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
                self.history
                    .iter()
                    .filter(|c| c.date >= start || c.date <= end)
                    .cloned()
                    .collect()
            }
            _ => return Err("Invalid filter")?,
        };
        if vec.is_empty() {
            return Err("No history found for the filter")?;
        }
        Ok(Self {
            history: vec,
            name: self.name.clone(),
            current_pos: 0,
            current_iter_pos: 0,
        })
    }
}

impl fmt::Display for FunctionHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.history[self.current_pos])?;
        Ok(())
    }
}

impl Iterator for FunctionHistory {
    type Item = CommitFunctions;
    fn next(&mut self) -> Option<Self::Item> {
        self.history.get(self.current_iter_pos).cloned().map(|c| {
            self.current_iter_pos += 1;
            c
        })
    }
}

pub enum Directions {
    Forward,
    Back,
    None,
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
