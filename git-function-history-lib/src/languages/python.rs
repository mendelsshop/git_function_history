use std::collections::HashMap;

use rustpython_parser::{
    ast::{Located, StatementType},
    location::Location,
    parser,
};

#[derive(Debug, Clone)]
pub struct Function {
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

impl Function {}

impl super::Function for Function {
    fn fmt_with_context(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        previous: Option<&Self>,
        next: Option<&Self>,
    ) -> std::fmt::Result {
        match &self.class {
            Some(class) => match previous {
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
            match previous {
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
            match next {
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
            Some(class) => match next {
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
) -> Result<Vec<Function>, Box<dyn std::error::Error>> {
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
            let new_func = Function {
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
