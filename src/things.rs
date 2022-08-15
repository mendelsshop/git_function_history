use std::fmt::{self};

pub(crate) struct InternalBlock {
    pub(crate) start: Points,
    pub(crate) full: Points,
    pub(crate) end: Points,
    pub(crate) types: BlockType,
}

pub(crate) struct InternalFunctions {
    pub(crate) name: String,
    pub(crate) range: Points,
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
                    write!(f, "{}\n...", i.bottom)?;
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
}

/// This holds information about when a function is in an impl/trait/extern block
#[derive(Debug, Clone)]
pub struct Block {
    pub name: Option<String>,
    pub top: String,
    pub bottom: String,
    pub block_type: BlockType,
}
/// This enum is used when filtering commit history only for let say impl and not externs or traits
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    Impl, 
    Extern,
    Trait,
    Unknown,
}

/// This holds information like date and commit id andalso the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct CommitFunctions {
    pub id: String,
    pub functions: Vec<Function>,
    pub date: String,
    current_pos: usize,
}

impl CommitFunctions {
    pub(crate) const fn new(id: String, functions: Vec<Function>, date: String) -> Self {
        Self {
            id,
            functions,
            date,
            current_pos: 0,
        }
    }

    /// returns all functions in the block type specified if ore else it returns none
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
            functions: vec,
            id: self.id.clone(),
            date: self.date.clone(),
            current_pos: 0
        })
    }

    /// Gets all functions which are betwwen the start an end lines specified.
    /// If there are non the it returns none.
    pub fn get_functin_in_lines(&self, start: usize, end: usize) -> Option<Self> {
        let vec: Vec<Function> = self
            .functions
            .iter()
            .filter(|f| {
                f.lines.0 >= start && f.lines.1 <= end
            })
            .cloned()
            .collect();
        if vec.is_empty() {
            return None;
        }
        Some(Self {
            functions: vec,
            id: self.id.clone(),
            date: self.date.clone(),
            current_pos: 0
        })
    }
}


impl Iterator for CommitFunctions {
    type Item = Function;
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
        writeln!(f, "Date: {}", self.date)?;
        for (i, function) in self.functions.iter().enumerate() {
            write!(
                f,
                "{}{}",
                match i {
                    0 => "",
                    _ => "\n...\n",
                },
                function
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
    pub(crate) current_pos: usize,
}

impl FunctionHistory {
    /// This function will return a `CommitFunctions` for the given commit id.
    pub fn get_by_commit_id(&self, id: &str) -> Option<CommitFunctions> {
        self.history.iter().find(|c| c.id == id).cloned()
    }

    /// This function will return a `CommitFunctions` for a given date (Date format not decided).
    pub fn get_by_date(&self, date: &str) -> Option<&CommitFunctions> {
        self.history.iter().find(|c| c.date == date)
    }

    /// Given a date range it will return a vector of commits in that range
    pub fn get_date_range(&self, start: &str, end: &str) -> Vec<&CommitFunctions> {
        // TODO: import chrono and use it to compare dates
        todo!(
            "get_date_range(for: {}, from: {}-{})",
            self.name,
            start,
            end
        );
    }

    pub fn list_commit_ids(&self) -> Vec<String> {
        self.history.iter().map(|c| c.id.clone()).collect()
    }

    /// This function findss all functions that have a blocktype that matches the given blocktype
    /// so you can filter out functions that are not in for example an impl block:
    /// ```rust
    /// use git_function_history::{get_function, BlockType};
    /// let in_impl = get_function("empty_test", "src/test_functions.rs").unwrap().get_all_functions_in_block(BlockType::Impl);
    /// println!("{}", in_impl);
    /// assert!(in_impl.get_by_commit_id("6cc3ab3cdd24d545d93db1c4c55873596dd0ac2a").is_some())
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
            current_pos: 0
        }
    }

    /// This function finds alll function in each commit that are between the given start and end positions.
    pub fn get_all_functions_line(&self, start: usize, end: usize) -> Self {
        let t = self
            .history
            .iter()
            .filter_map(|f| f.get_functin_in_lines(start, end))
            .collect();
        Self {
            history: t,
            name: self.name.clone(),
            current_pos: 0
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

