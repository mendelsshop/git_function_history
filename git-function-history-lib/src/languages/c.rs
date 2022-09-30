use std::collections::HashMap;

pub struct Function {
    name: String,
    body: String,
    // parameters: Params,
    parameters: Vec<String>,
    parent: Vec<ParentFunction>,
    returns: Option<String>,
}

impl Function {
    pub fn new(name: String, body: String, parameters: Vec<String>, parent: Vec<ParentFunction>, returns: Option<String>) -> Self {
        Self {
            name,
            body,
            parameters,
            parent,
            returns,
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

pub struct ParentFunction {
    name: String,
    top: String,
    bottom: String,
    lines: (usize, usize),
    parameters: Vec<String>,
    returns: Option<String>,
}