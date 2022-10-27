use rustpython_parser::{
    ast::{Located, StatementType},
    location::Location,
    parser,
};
use std::collections::VecDeque;
use std::{collections::HashMap, fmt};

use crate::{impl_function_trait, UnwrapToError};

use super::FunctionTrait;

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
        if let Some(ref class) = self.class {
            write!(f, "{}", class.top)?;
        }
        for parent in &self.parent {
            write!(f, "{}", parent.top)?;
        }
        write!(f, "{}", self.body)?;
        for parent in &self.parent {
            write!(f, "{}", parent.bottom)?;
        }
        self.class
            .as_ref()
            .map_or(Ok(()), |class| write!(f, "{}", class.bottom))
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

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<PythonFunction>, Box<dyn std::error::Error>> {
    let ast = parser::parse_program(file_contents)?;
    let mut functions = vec![];
    let mut last = None;

    if ast.statements.is_empty() {
        return Err("No code found")?;
    }
    let mut new_ast = VecDeque::from(ast.statements);
    loop {
        if new_ast.is_empty() {
            break;
        }
        let stmt = new_ast
            .pop_front()
            .unwrap_to_error("could not get statement")?;
        let next = new_ast.front();
        get_functions(stmt, next, &mut functions, name, &mut last, &mut None);
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
    for mut func in functions {
        if func.1 .0 == func.1 .1 {
            // get the last line of the file
            let last_line = file_contents.lines().last().unwrap_to_error("could not get last line")?;
            let row = file_contents.lines().count();
            let column = last_line.len();
            let end = Location::new(row, column);
            func.1 .1 = end;
        }
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
#[inline]
fn fun_name1(
    body: Vec<Located<StatementType>>,
    functions: &mut Vec<(StatementType, (Location, Location))>,
    lookup_name: &str,
    last_found_fn: &mut Option<(StatementType, Location)>,
    other_last_found_fn: &mut Option<(StatementType, Location)>,
) {
    let mut new_ast = VecDeque::from(body);
    loop {
        if new_ast.is_empty() {
            break;
        }
        let stmt = new_ast.pop_front().expect("could not get statement");
        let next = new_ast.front();
        get_functions(
            stmt,
            next,
            functions,
            lookup_name,
            last_found_fn,
            other_last_found_fn,
        );
    }
}
#[inline]
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
    next_stmt: Option<&Located<StatementType>>,
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
                if next_stmt.is_none() {
                    functions.push((stmt.node, (stmt.location, stmt.location)));
                } else {
                    *last_found_fn = Some((stmt.node, stmt.location));
                }
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
pub enum PythonFilter {
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

impl PythonFilter {
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

impl FunctionTrait for PythonFunction {
    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        self.class.as_ref().map_or((), |block| {
            tops.push(block.top.clone());
        });
        for parent in &self.parent {
            tops.push(parent.top.clone());
        }
        tops
    }

    fn get_total_lines(&self) -> (usize, usize) {
        self.class.as_ref().map_or_else(
            || {
                let mut start = self.lines.0;
                let mut end = self.lines.1;
                for parent in &self.parent {
                    if parent.lines.0 < start {
                        start = parent.lines.0;
                        end = parent.lines.1;
                    }
                }
                (start, end)
            },
            |block| block.lines,
        )
    }

    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        self.class.as_ref().map_or((), |block| {
            bottoms.push(block.bottom.clone());
        });
        for parent in &self.parent {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }
    impl_function_trait!(PythonFunction);
}
