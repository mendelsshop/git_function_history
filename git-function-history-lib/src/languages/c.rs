use std::{error::Error, fmt};

use crate::impl_function_trait;

use super::FunctionTrait;

#[derive(Debug, Clone)]
pub struct CFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) parent: Vec<ParentFunction>,
    pub(crate) returns: Option<String>,
    pub(crate) lines: (usize, usize),
}

impl CFunction {
    pub fn new(
        name: String,
        body: String,
        parameters: Vec<String>,
        parent: Vec<ParentFunction>,
        returns: Option<String>,
        lines: (usize, usize),
    ) -> Self {
        Self {
            name,
            body,
            parameters,
            parent,
            returns,
            lines,
        }
    }
}

impl fmt::Display for CFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.parameters.is_empty() {
            write!(f, "(")?;
            for (i, param) in self.parameters.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ")")?;
        }
        if let Some(ret) = &self.returns {
            write!(f, " -> {}", ret)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) lines: (usize, usize),
    pub(crate) parameters: Vec<String>,
    pub(crate) returns: Option<String>,
}
#[inline]
/*
use cached::proc_macro::cached;
#[cached(result = true)]
*/
pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<CFunction>, Box<dyn Error>> {
    println!("Finding function {} in commit {}", name, file_contents);

    todo!("find_function_in_commit")
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CFilter {
    /// when you want filter by a function that has a parent function of a specific name
    FunctionWithParent(String),
    /// when you want to filter by a function that has a has a specific return type
    FunctionWithReturnType(String),
}

impl CFilter {
    pub fn matches(&self, function: &CFunction) -> bool {
        match self {
            Self::FunctionWithParent(parent) => function
                .parent
                .iter()
                .any(|parent_function| parent_function.name == *parent),
            Self::FunctionWithReturnType(return_type) => function
                .returns
                .as_ref()
                .map_or(false, |r| r == return_type),
        }
    }
}

impl FunctionTrait for CFunction {
    fn get_total_lines(&self) -> (usize, usize) {
        let mut start = self.lines.0;
        let mut end = self.lines.1;
        for parent in &self.parent {
            if parent.lines.0 < start {
                start = parent.lines.0;
                end = parent.lines.1;
            }
        }
        (start, end)
    }

    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        for parent in &self.parent {
            tops.push(parent.top.clone());
        }
        tops
    }

    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        for parent in &self.parent {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }

    impl_function_trait!(CFunction);
}
