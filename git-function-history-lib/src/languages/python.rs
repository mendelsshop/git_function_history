use rustpython_parser::{
    ast::{Located, Location, StmtKind},
    parser,
};
use std::collections::VecDeque;
use std::{collections::HashMap, fmt};

use crate::{impl_function_trait, UnwrapToError};

use super::FunctionTrait;
// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Range {
//     pub location: Location,
//     pub end_location: Location,
// }

// impl Range {
//     pub fn from_located<T>(located: &Located<T>) -> Self {
//         Range {
//             location: located.location,
//             end_location: located
//                 .end_location
//                 .expect("AST nodes should have end_location."),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct PythonFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    /// parameters: Params,
    pub(crate) parameters: Vec<String>,
    pub(crate) parent: Vec<PythonParentFunction>,
    pub(crate) decorators: Vec<String>,
    pub(crate) class: Vec<PythonClass>,
    pub(crate) lines: (usize, usize),
    pub(crate) returns: Option<String>,
}

impl fmt::Display for PythonFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for class in &self.class {
            write!(f, "{}", class.top)?;
        }
        for parent in &self.parent {
            write!(f, "{}", parent.top)?;
        }
        write!(f, "{}", self.body)?;
        for parent in &self.parent {
            write!(f, "{}", parent.bottom)?;
        }
        for class in &self.class {
            write!(f, "{}", class.bottom)?;
        }
        Ok(())
    }
}

/// #[derive(Debug, Clone)]
/// pub struct Params {
///     args: Vec<String>,
///     kwargs: Vec<String>,
///     varargs: Option<String>,
///     varkwargs: Option<String>,
/// }
#[derive(Debug, Clone)]
pub struct PythonClass {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) lines: (usize, usize),
    pub(crate) decorators: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct PythonParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) lines: (usize, usize),
    pub(crate) parameters: Vec<String>,
    pub(crate) decorators: Vec<String>,
    /// pub(crate) class: Option<String>,
    pub(crate) returns: Option<String>,
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<PythonFunction>, Box<dyn std::error::Error>> {
    let ast = parser::parse_program(file_contents, "<stdin>")?;
    let mut functions = vec![];

    if ast.is_empty() {
        return Err("No code found")?;
    }
    get_functions_recurisve(ast, &mut functions, &mut Vec::new(), &mut Vec::new(), name)?;
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
        let start = func.1 .0.row();
        let end = func.1 .1.row();
        let start = map[&(start - 1)];
        let end = map[&(end - 1)];
        if let StmtKind::FunctionDef {
            name,
            args,
            decorator_list,
            returns,
            ..
        } = func.0
        {
            let mut start_s = func.1 .0.row();
            let body = file_contents[*start..*end]
                .trim_start_matches('\n')
                .to_string()
                .lines()
                .map(|l| {
                    let t = format!("{start_s}: {l}\n",);
                    start_s += 1;
                    t
                })
                .collect::<String>();
            let new_func = PythonFunction {
                name: name.to_string(),
                returns: returns.as_ref().map(|x| x.node.name().to_string()),
                parameters: args.args.iter().map(|x| x.node.arg.to_string()).collect(),
                parent: func.3,
                decorators: decorator_list
                    .iter()
                    .map(|x| x.node.name().to_string())
                    .collect(),
                class: func.2,
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
fn get_functions_recurisve(
    body: Vec<Located<StmtKind>>,
    functions: &mut Vec<(
        StmtKind,
        (Location, Location),
        Vec<PythonClass>,
        Vec<PythonParentFunction>,
    )>,
    current_parent: &mut Vec<PythonParentFunction>,
    current_class: &mut Vec<PythonClass>,
    lookup_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut new_ast = VecDeque::from(body);
    loop {
        if new_ast.is_empty() {
            break;
        }
        let stmt = new_ast.pop_front().unwrap_to_error("No stmt")?;
        get_functions(stmt, functions, current_parent, current_class, lookup_name);
    }
    Ok(())
}

fn get_functions(
    stmt: Located<StmtKind>,
    functions: &mut Vec<(
        StmtKind,
        (Location, Location),
        Vec<PythonClass>,
        Vec<PythonParentFunction>,
    )>,
    current_parent: &mut Vec<PythonParentFunction>,
    current_class: &mut Vec<PythonClass>,
    lookup_name: &str,
) {
    match stmt.node {
        StmtKind::FunctionDef { ref name, .. } | StmtKind::AsyncFunctionDef { ref name, .. }
            if name == lookup_name =>
        {
            if let Some(end) = stmt.end_location {
                functions.push((
                    stmt.node,
                    (stmt.location, end),
                    current_class.clone(),
                    current_parent.clone(),
                ));
            }
        }
        StmtKind::If { body, orelse, .. }
        | StmtKind::While { body, orelse, .. }
        | StmtKind::For { body, orelse, .. }
        | StmtKind::AsyncFor { body, orelse, .. } => {
            get_functions_recurisve(body, functions, current_parent, current_class, lookup_name)
                .unwrap();
            get_functions_recurisve(
                orelse,
                functions,
                current_parent,
                current_class,
                lookup_name,
            )
            .unwrap();
        }
        StmtKind::FunctionDef { name, body, .. }
        | StmtKind::AsyncFunctionDef { name, body, .. } => {
            // turn the function into a parent function
            let parent = PythonParentFunction {
                name,
                top: String::new(),
                bottom: String::new(),
                lines: (0, 0),
                parameters: vec![],
                decorators: vec![],
                returns: None,
            };
            // and add it to the current parent
            current_parent.push(parent);
            // and then recurse with the get_functions_recurisve
            get_functions_recurisve(body, functions, current_parent, current_class, lookup_name)
                .unwrap();
            // and then remove the parent function
            current_parent.pop();
        }
        StmtKind::ClassDef { name, body, .. } => {
            // turn the class into a python class
            let class = PythonClass {
                name,
                top: String::new(),
                bottom: String::new(),
                lines: (0, 0),
                decorators: vec![],
            };

            // and add it to the current class
            current_class.push(class);
            // and then recurse with the get_functions_recurisve
            get_functions_recurisve(body, functions, current_parent, current_class, lookup_name)
                .unwrap();
            // and then remove the class
            current_class.pop();
        }
        StmtKind::With { body, .. } | StmtKind::AsyncWith { body, .. } => {
            get_functions_recurisve(body, functions, current_parent, current_class, lookup_name)
                .unwrap();
        }
        // TODO: add handles.body
        StmtKind::Try {
            body,
            orelse,
            finalbody,
            ..
        } => {
            get_functions_recurisve(body, functions, current_parent, current_class, lookup_name)
                .unwrap();
            get_functions_recurisve(
                orelse,
                functions,
                current_parent,
                current_class,
                lookup_name,
            )
            .unwrap();
            get_functions_recurisve(
                finalbody,
                functions,
                current_parent,
                current_class,
                lookup_name,
            )
            .unwrap();
        }
        // TODO: add match.body
        _ => {}
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonFilter {
    /// when you want to filter by function that are in a specific class
    InClass(String),
    /// when you want filter by a function that has a parent function of a specific name
    HasParentFunction(String),
    /// when you want to filter by a function that has a has a specific return type
    HasReturnType(String),
    /// when you want to filter by a function that has a specific parameter name
    HasParameterName(String),
    /// when you want to filter by a function that has a specific decorator
    HasDecorator(String),
    /// when you want to filter by a function thats class has a specific decorator
    HasClasswithDecorator(String),
    /// when you want to filter by a function that's parent function has a specific decorator
    HasParentFunctionwithDecorator(String),
    /// when you want to filter by a function that's parent function has a specific parameter name
    HasParentFunctionwithParameterName(String),
    /// when you want to filter by a function that's parent function has a specific return type
    HasParentFunctionwithReturnType(String),
}

impl PythonFilter {
    pub fn matches(&self, function: &PythonFunction) -> bool {
        match self {
            Self::InClass(class) => function.class.iter().any(|c| c.name == *class),
            Self::HasParentFunction(parent) => function.parent.iter().any(|x| x.name == *parent),
            Self::HasReturnType(return_type) => function
                .returns
                .as_ref()
                .map_or(false, |x| x == return_type),
            Self::HasParameterName(parameter_name) => {
                function.parameters.iter().any(|x| x == parameter_name)
            }
            Self::HasDecorator(decorator) => function.decorators.iter().any(|x| x == decorator),
            Self::HasClasswithDecorator(decorator) => function
                .class
                .iter()
                .any(|x| x.decorators.iter().any(|y| y == decorator)),

            Self::HasParentFunctionwithDecorator(decorator) => function
                .parent
                .iter()
                .any(|x| x.decorators.iter().any(|x| x == decorator)),
            Self::HasParentFunctionwithParameterName(parameter_name) => function
                .parent
                .iter()
                .any(|x| x.parameters.iter().any(|x| x == parameter_name)),
            Self::HasParentFunctionwithReturnType(return_type) => function
                .parent
                .iter()
                .any(|x| x.returns.as_ref().map_or(false, |x| x == return_type)),
        }
    }
}

impl FunctionTrait for PythonFunction {
    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        for class in &self.class {
            tops.push(class.top.clone());
        }
        for parent in &self.parent {
            tops.push(parent.top.clone());
        }
        tops
    }

    fn get_total_lines(&self) -> (usize, usize) {
        let mut lines = (0, 0);
        for class in &self.class {
            if class.lines.0 < lines.0 {
                lines = class.lines;
            }
        }
        for parent in &self.parent {
            if parent.lines.0 < lines.0 {
                lines = parent.lines;
            }
        }
        if self.lines.0 < lines.0 {
            lines = self.lines;
        }
        lines
    }

    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        for class in &self.class {
            bottoms.push(class.bottom.clone());
        }
        for parent in &self.parent {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }
    impl_function_trait!(PythonFunction);
}
