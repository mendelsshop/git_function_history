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

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub contents: String,
    pub block: Option<Block>,
    pub function: Option<Vec<FunctionBlock>>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.block {
            None => {}
            Some(block) => write!(f, "{}\n...\n", block.top)?,
        };
        match &self.function {
            None => {}
            Some(_) => {}
        };
        write!(f, "{}", self.contents)?;
        match &self.function {
            None => {}
            Some(_) => {}
        };
        match &self.block {
            None => {}
            Some(block) => write!(f, "\n...{}", block.bottom)?,
        };
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct FunctionBlock {
    pub name: String,
    pub top: String,
    pub bottom: String,
}
#[derive(Debug, Clone)]
pub struct Block {
    pub name: Option<String>,
    pub top: String,
    pub bottom: String,
    pub block_type: BlockType,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    Impl,
    Extern,
    Trait,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommitFunctions {
    pub id: String,
    pub functions: Vec<Function>,
    pub date: String,
}

impl CommitFunctions {
    pub(crate) const fn new(id: String, functions: Vec<Function>, date: String) -> Self {
        Self {
            id,
            functions,
            date,
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
        })
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

#[derive(Debug)]
pub struct FunctionHistory {
    pub name: String,
    pub history: Vec<CommitFunctions>,
}

impl FunctionHistory {
    /// This function will return a `CommitFunctions` for the given commit id
    pub fn get_by_commit_id(&self, id: &str) -> Option<&CommitFunctions> {
        self.history.iter().find(|c| c.id == id)
    }

    /// This function will return a `CommitFunctions` for a given date (format not decided).
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

    /// This function findes all functions that have a blocktype that matches the given blocktype
    /// so you can filter out functions that are not in for example an impl block:
    /// ```rust
    /// use git_function_history::{get_function, BlockType};
    /// let in_impl = get_function("empty_test", "src/test_functions.rs").unwrap().get_all_functions_in_block(BlockType::Impl);
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
