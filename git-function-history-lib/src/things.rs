use chrono::{DateTime, FixedOffset};
use std::fmt::{self};

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

/// This is used to store each individual file in a commit and the associated functions in that file.
#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub functions: Vec<Function>,
    current_pos: usize,
}

impl File {
    pub fn new(name: String, functions: Vec<Function>) -> Self {
        Self { name, functions, current_pos: 0 }
        }
    /// Returns all functions in the block type specified if ore else it returns none.
    pub fn get_function_from_block(&self, block_type: BlockType) -> Option<Self> {
        let vec: Vec<Function> = self
            .functions
            .iter()
            .filter(|f| {
                f.block
                    .as_ref()
                    .map_or(false, |block| block.block_type == block_type)
            })
            .cloned()
            .collect();
        if vec.is_empty() {
            return None;
        }
        Some(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }

    /// Gets all functions which are betwwen the start an end lines specified.
    /// If there are none the it returns none.
    pub fn get_functin_in_lines(&self, start: usize, end: usize) -> Option<Self> {
        let vec: Vec<Function> = self
            .functions
            .iter()
            .filter(|f| f.lines.0 >= start && f.lines.1 <= end)
            .cloned()
            .collect();
        if vec.is_empty() {
            return None;
        }
        Some(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }

    /// This returns a list of functions which  have a parent function that has the same name as the one specified.
    pub fn get_function_with_parent(&self, parent: &str) -> Option<Self> {
        let vec: Vec<Function> = self
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
            .collect();
        if vec.is_empty() {
            return None;
        }
        Some(Self {
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
        writeln!(f, "File {}", self.name)?;
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
    current_pos: usize,
}

impl CommitFunctions {
    pub(crate) fn new(id: String, functions: Vec<File>, date: &str) -> Self {
        Self {
            id,
            functions,
            date: DateTime::parse_from_rfc2822(date).expect("Failed to parse date"),
            current_pos: 0,
        }
    }

    pub fn get_function_from_block(&self, block_type: BlockType) -> Option<Self> {
        let t: Vec<File> = self
        .functions
        .iter()
        .filter_map(|f| f.get_function_from_block(block_type))
        .collect();
        match t {
            t if t.is_empty() => {
                return None;
            }
            _ => {},
        }
    Some(Self {
        id: self.id.clone(),
        functions: t,
        date: self.date,
        current_pos: 0,
    })
    }
    
    pub fn get_function_in_lines(&self, start: usize, end: usize) -> Option<Self> {
        let t: Vec<File> = self
            .functions
            .iter()
            .filter_map(|f| f.get_functin_in_lines(start, end))
            .collect();
        match t {
            t if t.is_empty() => {
                return None;
            }
            _ => {},
        }
        Some(Self {
            id: self.id.clone(),
            functions: t,
            date: self.date,
            current_pos: 0,
        })
    }

    pub fn get_function_with_parent(&self, parent: &str) -> Option<Self> {
        let t: Vec<File> = self
            .functions
            .iter()
            .filter_map(|f| f.get_function_with_parent(parent))
            .collect();
        match t {
            t if t.is_empty() => {
                return None;
            }
            _ => {},
        }
        Some(Self {
            id: self.id.clone(),
            functions: t,
            date: self.date,
            current_pos: 0,
        })
    }
}

impl Iterator for CommitFunctions {
    type Item = File;
    fn next(&mut self) -> Option<Self::Item> {
        // get the current function without removing it
        let function = self.functions.get(self.current_pos).cloned();
        self.current_pos += 1;
        function
    }
}

impl fmt::Display for CommitFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Commit {}", self.id)?;
        writeln!(f, "Date: {}", self.date.format("%Y-%m-%d %H:%M:%S"))?;
        for file in &self.functions {
            write!(
                f,
                "\n{}",
                file

            )?;
        }
        Ok(())
    }
    
}


/// This struct holds the a list of commits and the function that were looked up for each commit.
#[derive(Debug)]
pub struct FunctionHistory {
    pub name: String,
    pub history: Vec<CommitFunctions>,
    current_pos: usize,
}

impl FunctionHistory {
    pub fn new(name: String, history: Vec<CommitFunctions>) -> Self {
        Self {
            name,
            history,
            current_pos: 0,
        }
    }
    /// This function will return a `CommitFunctions` for the given commit id.
    pub fn get_by_commit_id(&self, id: &str) -> Option<&CommitFunctions> {
        self.history.iter().find(|c| c.id == id)
    }

    /// This function will return a `CommitFunctions` for a given date in the rfc2822 format.
    pub fn get_by_date(&self, date: &str) -> Option<&CommitFunctions> {
        self.history.iter().find(|c| c.date.to_rfc2822() == date)
    }

    /// Given a date range in the rfc2822 format, this function will return a vector of commits in that range.
    pub fn get_date_range(&self, start: &str, end: &str) -> Self {
        let start = DateTime::parse_from_rfc2822(start).expect("Failed to parse date");
        let end = DateTime::parse_from_rfc2822(end).expect("Failed to parse date");
        assert!(start <= end, "Start date is greater than end date");
        let t = self
            .history
            .iter()
            .filter(|c| c.date >= start && c.date <= end)
            .cloned()
            .collect();
        Self {
            history: t,
            name: self.name.clone(),
            current_pos: 0,
        }
    }

    /// This will return a vector of all the commit ids in the history.
    pub fn list_commit_ids(&self) -> Vec<&str> {
        self.history.iter().map(|c| c.id.as_ref()).collect()
    }

    /// This function findss all functions that have a blocktype that matches the given blocktype
    /// so you can filter out functions that are not in for example an impl block:
    /// ```rust
    /// use git_function_history::{get_all_functions, BlockType};
    /// let in_impl = get_all_functions("empty_test").unwrap();
    /// println!("{}", in_impl);
    /// assert!(in_impl.get_by_commit_id("3c7847613cf70ce81ce0e992269911451aad61c3").is_some())
    /// ```
    pub fn get_all_functions_in_block(&self, block_type: BlockType) -> Self {
        let t = self
            .history
            .iter()
            .filter_map(|f| f.get_function_from_block(block_type))
            .collect();
        Self {
            history: t,
            name: self.name.clone(),
            current_pos: 0,
        }
    }

    /// This function finds all function in each commit that are between the given start and end positions.
    pub fn get_all_functions_line(&self, start: usize, end: usize) -> Self {
        let t = self
            .history
            .iter()
            .filter_map(|f| f.get_function_in_lines(start, end))
            .collect();
        Self {
            history: t,
            name: self.name.clone(),
            current_pos: 0,
        }
    }

    /// This function finds all functions that have a parent function that has the same name as the one specified.
    pub fn get_all_function_with_parent(&self, parent: &str) -> Self {
        let t = self
            .history
            .iter()
            .filter_map(|f| f.get_function_with_parent(parent))
            .collect();
        Self {
            history: t,
            name: self.name.clone(),
            current_pos: 0,
        }
    }
}

impl fmt::Display for FunctionHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for commit in &self.history {
            write!(f, "\n{}", commit)?;
        }
        Ok(())
    }
}

impl Iterator for FunctionHistory {
    type Item = CommitFunctions;
    fn next(&mut self) -> Option<Self::Item> {
        self.history.get(self.current_pos).cloned().map(|c| {
            self.current_pos += 1;
            c
        })
    }
}
