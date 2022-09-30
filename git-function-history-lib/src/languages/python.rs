use std::{error::Error, collections::HashMap};

use rustpython_parser::{location::Location, ast::{StatementType, Located}, parser};

use crate::File;

pub struct Function {
    name: String,
    body: String,
    // parameters: Params,
    parameters: Vec<String>,
    parent: Vec<ParentFunction>,
    decorators: Vec<String>,
    class: Option<Class>,
    returns: Option<String>,
}

impl Function {

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

pub struct Params {
    args: Vec<String>,
    kwargs: Vec<String>,
    varargs: Option<String>,
    varkwargs: Option<String>,
}

pub struct Class {
    name: String,
    top: String,
    bottom: String,
    lines: (usize, usize),
    decorators: Vec<String>,
}

pub struct ParentFunction {
    name: String,
    top: String,
    bottom: String,
    lines: (usize, usize),
    parameters: Vec<String>,
    decorators: Vec<String>,
    class: Option<String>,
    returns: Option<String>,
}

pub fn get_file_in_commit(
    commit: &str,
    file_path: &str,
    name: &str,
) -> Result<File<Function>, Box<dyn Error>> {
    let file_contents = crate::find_file_in_commit(commit, file_path)?;
    let ast = parser::parse_program(&file_contents)?;
    let mut functions = vec![];
    let mut last = None;
    for stmt in ast.statements {
        get_functions(stmt, &mut functions, "baz", &mut last, &mut None);
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
            let body = file_contents[*start..*end].to_string();
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
            };
            new.push(new_func);
        }
    }
    Ok(File::new(name.to_string(), new))
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
                *last_found_fn = Some((stmt.node, stmt.location));
            }
        }
        StatementType::FunctionDef { body, .. } => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
            fun_name1(
                body,
                functions,
                lookup_name,
                last_found_fn,
                other_last_found_fn,
            );
        }
        StatementType::If { body, orelse, .. } => {
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
        StatementType::While { body, orelse, .. } => {
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
        StatementType::For { body, orelse, .. } => {
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
        StatementType::With { body, .. } => {
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
        StatementType::ClassDef { body, .. } => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
            fun_name1(
                body,
                functions,
                lookup_name,
                last_found_fn,
                other_last_found_fn,
            );
        }
        _ => {
            fun_name(other_last_found_fn, last_found_fn, functions, stmt.location);
        }
    }
}
