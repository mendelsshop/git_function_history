use std::{collections::HashMap, error::Error, fmt};

use crate::impl_function_trait;

use super::FunctionTrait;

#[derive(Debug, Clone)]
pub struct GoFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) returns: Option<String>,
    pub(crate) lines: (usize, usize),
}

impl GoFunction {
    pub fn new(
        name: String,
        body: String,
        parameters: Vec<String>,
        returns: Option<String>,
        lines: (usize, usize),
    ) -> Self {
        Self {
            name,
            body,
            parameters,
            returns,
            lines,
        }
    }
}

impl fmt::Display for GoFunction {
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

impl FunctionTrait for GoFunction {
    impl_function_trait!(GoFunction);

    fn get_bottoms(&self) -> Vec<String> {
        vec![]
    }

    fn get_tops(&self) -> Vec<String> {
        vec![]
    }

    fn get_total_lines(&self) -> (usize, usize) {
        let start = self.lines.0;
        let end = self.lines.1;
        (start, end)
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<GoFunction>, Box<dyn Error>> {
    // TODO: progate errors back in clusre intead of paicking with excpet
    let parsed_file = gosyn::parse_source(file_contents)
        .map_err(|e| format!("{:?}", e))?
        .decl;
    Ok(parsed_file
        .into_iter()
        .filter_map(|decl| match decl {
            gosyn::ast::Declaration::Function(func) => {
                if func.name.name == name {
                    let mut lines = (
                        func.name.pos,
                        func.body
                            .as_ref()
                            .expect("no body found for function")
                            .pos
                            .1,
                    );
                    match func.recv {
                        Some(recv) => {
                            lines.0 = recv.pos();
                        }
                        None => {}
                    }
                    lines.0 = file_contents[..lines.0]
                        .rfind("func")
                        .expect("could not find 'func' keyword before function");
                    for i in &func.docs {
                        if i.pos < lines.0 {
                            lines.0 = i.pos;
                        }
                    }
                    let body = file_contents[lines.0..lines.1 + 1].to_string();
                    let mut start_line = 0;
                    for i in file_contents.chars().enumerate() {
                        if i.1 == '\n' {
                            if i.0 > lines.0 {
                                lines.0 = i.0;
                                break;
                            }
                            start_line += 1;
                        }
                    }
                    let mut end_line = 0;
                    for i in file_contents.chars().enumerate() {
                        if i.1 == '\n' {
                            if i.0 > lines.1 {
                                lines.1 = i.0;
                                break;
                            }
                            end_line += 1;
                        }
                    }

                    lines.0 = start_line;
                    lines.1 = end_line;
                    let parameters = func
                        .typ
                        .params
                        .list
                        .iter()
                        .map(|p| &p.tag.as_ref().expect("uknown plz report bug idk").value)
                        .map(|x| x.to_string())
                        .collect();
                    let returns = Some(
                        func.typ
                            .result
                            .list
                            .iter()
                            .map(|p| {
                                p.name
                                    .iter()
                                    .map(|x| &x.clone().name)
                                    .map(|x| x.to_string())
                                    .collect::<String>()
                            })
                            .collect(),
                    )
                    .filter(|x: &String| !x.is_empty());
                    // TODO: get parent functions
                    Some(GoFunction::new(
                        func.name.name,
                        body,
                        parameters,
                        returns,
                        lines,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    FunctionWithParameter(String),
    FunctionWithReturnType(String),
    FunctionInLines(usize, usize),
}

impl Filter {
    pub fn matches(&self, func: &GoFunction) -> bool {
        match self {
            Filter::FunctionWithParameter(param) => {
                func.parameters.iter().any(|x| x.contains(param))
            }
            Filter::FunctionWithReturnType(ret) => func
                .returns
                .as_ref()
                .map(|x| x.contains(ret))
                .unwrap_or(false),
            Filter::FunctionInLines(start, end) => {
                let (s, e) = func.get_total_lines();
                s >= *start && e <= *end
            }
        }
    }
}
