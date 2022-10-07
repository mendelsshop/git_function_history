use std::collections::HashMap;

use super::FunctionResult;
#[derive(Debug, Clone)]
pub struct CFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) parent: Vec<ParentFunction>,
    pub(crate) returns: Option<String>,
    pub(crate) lines: (usize, usize),
}

impl Function {
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

impl super::Function for CFunction {
    fn fmt_with_context(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        previous: Box<Option<&Self>>,
        next: Box<Option<&Self>>,
    ) -> std::fmt::Result {
        todo!()
    }

    fn get_metadata(&self) -> HashMap<&str, String> {
        todo!()
    }
    fn get_lines(&self) -> (usize, usize) {
        self.lines
    }

    fn matches(&self, filter: &LanguageFilter) -> bool {
        if let LanguageFilter::C(filt) = filter {
            filt.matches(self)
        } else {
            false
        }
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

pub(crate) fn find_function_in_commit<T: super::Function>(
    commit: &str,
    file_path: &str,
    name: &str,
) -> FunctionResult<T> {
    println!("Finding function {} in commit {}", name, commit);
    let file_contents = crate::find_file_in_commit(commit, file_path)?;

    todo!("find_function_in_commit")
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// when you want filter by a function that has a parent function of a specific name
    FunctionWithParent(String),
    /// when you want to filter by a function that has a has a specific return type
    FunctionWithReturnType(String),
}

impl Filter {
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
