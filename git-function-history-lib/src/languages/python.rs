use std::{collections::HashMap, fmt};

use rustpython_parser::{
    ast::{Located, StatementType},
    location::Location,
    parser,
};

use super::LanguageFilter;

#[derive(Debug, Clone)]
pub struct PythonFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    // parameters: Params,
    pub(crate) parameters: Vec<String>,
    pub(crate) parent: Vec<ParentFunction>,
    pub(crate) decorators: Vec<String>,
    pub(crate) class: Option<Class>,
    pub(crate) lines: (usize, usize),
    pub(crate) returns: Option<String>,
}

impl fmt::Display for PythonFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.class {
            Some(ref class) => write!(f, "{}", class.top)?,
            None => {}
        }
        for parent in &self.parent {
            write!(f, "{}", parent.top)?;
        }
        write!(f, "{}", self.body)?;
        for parent in &self.parent {
            write!(f, "{}", parent.bottom)?;
        }
        match self.class {
            Some(ref class) => write!(f, "{}", class.bottom),
            None => Ok(()),
        }
    }
}
impl PythonFunction {}

impl super::Function for PythonFunction {
    fn get_lines(&self) -> (usize, usize) {
        self.lines
    }
    fn fmt_with_context(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        previous: Box<Option<&Self>>,
        next: Box<Option<&Self>>,
    ) -> std::fmt::Result {
        match &self.class {
            Some(class) => match previous.as_ref() {
                Some(previous) => {
                    if previous.class.is_some() {
                        if previous.class.as_ref().unwrap().name == class.name {
                            write!(f, "\n...\n")?;
                        } else {
                            write!(f, "{}", class.top)?;
                        }
                    } else {
                        write!(f, "{}", class.top)?;
                    }
                }
                None => {
                    writeln!(f, "{}", class.top)?;
                }
            },
            None => {}
        };
        if !self.parent.is_empty() {
            match previous.as_ref() {
                None => {
                    for parent in &self.parent {
                        writeln!(f, "{}", parent.top)?;
                    }
                }
                Some(previous) => {
                    for parent in &self.parent {
                        if previous.parent.iter().any(|x| x.lines == parent.lines) {
                        } else {
                            write!(f, "{}\n...\n", parent.top)?;
                        }
                    }
                }
            }
        }
        write!(f, "{}", self.body)?;
        if !self.parent.is_empty() {
            match next.as_ref() {
                None => {
                    for parent in &self.parent {
                        writeln!(f, "{}", parent.bottom)?;
                    }
                }
                Some(next) => {
                    for parent in &self.parent {
                        if next.parent.iter().any(|x| x.lines == parent.lines) {
                        } else {
                            write!(f, "\n...\n{}", parent.bottom)?;
                        }
                    }
                }
            }
        }
        match &self.class {
            Some(class) => match next.as_ref() {
                Some(next) => {
                    if next.class.is_some() {
                        if next.class.as_ref().unwrap().name == class.name {
                            write!(f, "\n...\n")?;
                        } else {
                            write!(f, "{}", class.bottom)?;
                        }
                    } else {
                        write!(f, "{}", class.bottom)?;
                    }
                }
                None => {
                    writeln!(f, "{}", class.bottom)?;
                }
            },
            None => {}
        };
        Ok(())
    }

    fn get_metadata(&self) -> HashMap<&str, String> {
        todo!()
    }

    fn matches(&self, filter: &LanguageFilter) -> bool {
        if let LanguageFilter::Python(filt) = filter {
            filt.matches(self)
        } else {
            false
        }
    }
}
#[derive(Debug, Clone)]
pub struct Params {
    args: Vec<String>,
    kwargs: Vec<String>,
    varargs: Option<String>,
    varkwargs: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Class {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) lines: (usize, usize),
    pub(crate) decorators: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct ParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) lines: (usize, usize),
    pub(crate) parameters: Vec<String>,
    pub(crate) decorators: Vec<String>,
    pub(crate) class: Option<String>,
    pub(crate) returns: Option<String>,
}

pub(crate) fn find_function_in_commit(
    commit: &str,
    file_path: &str,
    name: &str,
) -> Result<Vec<PythonFunction>, Box<dyn std::error::Error>> {
    let file_contents = crate::find_file_in_commit(commit, file_path)?;

    let ast = parser::parse_program(&file_contents)?;
    let mut functions = vec![];
    let mut last = None;
    for stmt in ast.statements {
        get_functions(stmt, &mut functions, name, &mut last, &mut None);
    }
    let mut starts = file_contents
        .match_indices('\n')
        .map(|x| x.0)
        .collect::<Vec<_>>();
    starts.push(0);
    starts.sort_unstable();
    let map = starts
        .iter()
        .enumerate()
        .collect::<HashMap<usize, &usize>>();
    let mut new = Vec::new();
    for func in functions {
        // get the function body based on the location
        let start = func.1 .0.row();
        let end = func.1 .1.row();
        let start = map[&(start - 1)];

        let end = map[&(end - 1)];
        if let StatementType::FunctionDef {
            name,
            args,
            decorator_list,
            returns,
            is_async: _,
            ..
        } = func.0
        {
            let mut start_s = func.1 .0.row();
            let body = file_contents[*start..*end]
                .trim_start_matches('\n')
                .to_string()
                .lines()
                .map(|l| {
                    let t = format!("{}: {}\n", start_s, l,);
                    start_s += 1;
                    t
                })
                .collect::<String>();
            let new_func = PythonFunction {
                name: name.to_string(),
                returns: returns.as_ref().map(|x| x.name().to_string()),
                parameters: args.args.iter().map(|x| x.arg.to_string()).collect(),
                parent: vec![],
                decorators: decorator_list
                    .iter()
                    .map(|x| x.name().to_string())
                    .collect(),
                class: None,
                body,
                lines: (*start, *end),
            };
            new.push(new_func);
        }
    }
    if new.is_empty() {
        Err("No function found")?;
    }
    Ok(new)
}

fn fun_name1(
    body: Vec<Located<StatementType>>,
    functions: &mut Vec<(StatementType, (Location, Location))>,
    lookup_name: &str,
    last_found_fn: &mut Option<(StatementType, Location)>,
    other_last_found_fn: &mut Option<(StatementType, Location)>,
) {
    for stmt in body {
        get_functions(
            stmt,
            functions,
            lookup_name,
            last_found_fn,
            other_last_found_fn,
        );
    }
}

fn fun_name(
    other_last_found_fn: &mut Option<(StatementType, Location)>,
    last_found_fn: &mut Option<(StatementType, Location)>,
    functions: &mut Vec<(StatementType, (Location, Location))>,
    stmt: Location,
) {
    std::mem::swap(other_last_found_fn, last_found_fn);
    let mut other = None;
    std::mem::swap(&mut other, other_last_found_fn);
    if let Some(body) = other {
        functions.push((body.0, (body.1, stmt)));
    }
}

fn get_functions<'a>(
    stmt: Located<StatementType>,
    functions: &mut Vec<(StatementType, (Location, Location))>,
    lookup_name: &str,
    last_found_fn: &'a mut Option<(StatementType, Location)>,
    other_last_found_fn: &'a mut Option<(StatementType, Location)>,
) {
    match stmt.node {
        StatementType::FunctionDef { ref name, .. } if name == lookup_name => {
            if let Some(ref mut last) = last_found_fn {
                let mut new = (stmt.node, stmt.location);
                std::mem::swap(last, &mut new);
                functions.push((new.0, (new.1, stmt.location)));
            } else {
                // TODO: figure out if its the last node if so then we can push it here otherwise we need to wait for the next node
                *last_found_fn = Some((stmt.node, stmt.location));
            }
        }

        StatementType::If { body, orelse, .. }
        | StatementType::While { body, orelse, .. }
        | StatementType::For { body, orelse, .. } => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
            fun_name1(
                body,
                functions,
                lookup_name,
                last_found_fn,
                other_last_found_fn,
            );
            if let Some(stmts) = orelse {
                fun_name1(
                    stmts,
                    functions,
                    lookup_name,
                    last_found_fn,
                    other_last_found_fn,
                );
            }
        }
        StatementType::FunctionDef { body, .. }
        | StatementType::ClassDef { body, .. }
        | StatementType::With { body, .. } => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
            fun_name1(
                body,
                functions,
                lookup_name,
                last_found_fn,
                other_last_found_fn,
            );
        }
        StatementType::Try {
            body,
            handlers,
            orelse,
            finalbody,
        } => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
            fun_name1(
                body,
                functions,
                lookup_name,
                last_found_fn,
                other_last_found_fn,
            );
            for handler in handlers {
                fun_name1(
                    handler.body,
                    functions,
                    lookup_name,
                    last_found_fn,
                    other_last_found_fn,
                );
            }

            if let Some(stmts) = orelse {
                fun_name1(
                    stmts,
                    functions,
                    lookup_name,
                    last_found_fn,
                    other_last_found_fn,
                );
            }
            if let Some(stmts) = finalbody {
                fun_name1(
                    stmts,
                    functions,
                    lookup_name,
                    last_found_fn,
                    other_last_found_fn,
                );
            }
        }
        _ => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// when you want to filter by function that are in a specific class
    FunctionInClass(String),
    /// when you want filter by a function that has a parent function of a specific name
    FunctionWithParent(String),
    /// when you want to filter by a function that has a has a specific return type
    FunctionWithReturnType(String),
    /// when you want to filter by a function that has a specific parameter name
    FunctionWithParameterName(String),
    /// when you want to filter by a function that has a specific decorator
    FunctionWithDecorator(String),
}

impl Filter {
    pub fn matches(&self, function: &PythonFunction) -> bool {
        match self {
            Self::FunctionInClass(class) => {
                function.class.as_ref().map_or(false, |x| x.name == *class)
            }
            Self::FunctionWithParent(parent) => function.parent.iter().any(|x| x.name == *parent),
            Self::FunctionWithReturnType(return_type) => function
                .returns
                .as_ref()
                .map_or(false, |x| x == return_type),
            Self::FunctionWithParameterName(parameter_name) => {
                function.parameters.iter().any(|x| x == parameter_name)
            }
            Self::FunctionWithDecorator(decorator) => {
                function.decorators.iter().any(|x| x == decorator)
            }
        }
    }
}
