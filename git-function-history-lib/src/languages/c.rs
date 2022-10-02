use std::{collections::HashMap, error::Error};

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
        f: &mut std::fmt::Formatter<'_>,
        previous: Option<&Self>,
        next: Option<&Self>,
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
    commit: &str,
    file_path: &str,
    name: &str,
) -> FunctionResult<T> {
    todo!("find_function_in_commit")
}
