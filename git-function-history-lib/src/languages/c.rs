use std::collections::HashMap;

use super::FunctionResult;
#[derive(Debug, Clone)]
pub struct Function {
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

impl super::Function for Function {
    fn fmt_with_context(
        &self,
        _f: &mut std::fmt::Formatter<'_>,
        _previous: Option<&Self>,
        _next: Option<&Self>,
    ) -> std::fmt::Result {
        todo!()
    }

    fn get_metadata(&self) -> HashMap<&str, String> {
        todo!()
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
    _commit: &str,
    _file_path: &str,
    _name: &str,
) -> FunctionResult<T> {
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
    pub fn matches(&self, function: &Function) -> bool {
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
