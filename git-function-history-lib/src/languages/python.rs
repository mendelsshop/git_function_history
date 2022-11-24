use rustpython_parser::{
    ast::{Arguments, ExprKind, Located, StmtKind},
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
    pub(crate) parameters: Params,
    pub(crate) parent: Vec<PythonParentFunction>,
    pub(crate) decorators: Vec<String>,
    pub(crate) class: Vec<PythonClass>,
    pub(crate) lines: (usize, usize),
    pub(crate) returns: Option<String>,
}

impl fmt::Display for PythonFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for class in &self.class {
            write!(f, "{}\n...\n", class.top)?;
        }
        for parent in &self.parent {
            write!(f, "{}\n...\n", parent.top)?;
        }
        write!(f, "{}", self.body)
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Params {
    pub args: Vec<Param>,
    pub kwargs: Vec<Param>,
    pub posonlyargs: Vec<Param>,
    pub varargs: Option<Param>,
    pub varkwargs: Option<Param>,
}

impl Params {
    pub fn arg_has_name(&self, name: &str) -> bool {
        self.args.iter().any(|arg| arg.name == name)
            || self.kwargs.iter().any(|arg| arg.name == name)
            || self.posonlyargs.iter().any(|arg| arg.name == name)
            || self.varargs.as_ref().map_or(false, |arg| arg.name == name)
            || self
                .varkwargs
                .as_ref()
                .map_or(false, |arg| arg.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct PythonClass {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) lines: (usize, usize),
    pub(crate) decorators: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct PythonParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) lines: (usize, usize),
    pub(crate) parameters: Params,
    pub(crate) decorators: Vec<String>,
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
    get_functions_recurisve(
        ast,
        &map,
        &mut functions,
        &mut Vec::new(),
        &mut Vec::new(),
        name,
    );
    let mut new = Vec::new();
    for func in functions {
        let start = func.0.location.row();
        let end = func
            .0
            .end_location
            .unwrap_to_error("no end location for this function")?
            .row();
        let starts = map[&(start - 1)];
        let ends = map[&(end - 1)];
        if let StmtKind::FunctionDef {
            name,
            args,
            decorator_list,
            returns,
            ..
        }
        | StmtKind::AsyncFunctionDef {
            name,
            args,
            decorator_list,
            returns,
            ..
        } = func.0.node
        {
            let start_s = func.0.location.row();
            let mut body = file_contents[*starts..*ends]
                .trim_start_matches('\n')
                .to_string();
            body = super::make_lined(&body, start_s);
            let class = func
                .1
                .iter()
                .filter_map(|class| {
                    if let StmtKind::ClassDef {
                        ref name,
                        ref decorator_list,
                        ..
                    } = class.node
                    {
                        let start = class.location.row();
                        if let Some(end) = class.end_location {
                            let end = end.row();
                            let top = match file_contents.lines().nth(start - 1) {
                                Some(l) => l.trim_end().to_string(),
                                None => return None,
                            };
                            let top = format!("{start}: {top}");
                            let decorators = get_decorator_list(decorator_list);
                            return Some(PythonClass {
                                name: name.to_string(),
                                top,
                                lines: (start, end),
                                decorators,
                            });
                        }
                    }
                    None
                })
                .collect::<Vec<PythonClass>>();
            let parent = func
                .2
                .iter()
                .filter_map(|parent| {
                    if let StmtKind::FunctionDef {
                        ref name,
                        ref args,
                        ref decorator_list,
                        ref returns,
                        ..
                    }
                    | StmtKind::AsyncFunctionDef {
                        ref name,
                        ref args,
                        ref decorator_list,
                        ref returns,
                        ..
                    } = parent.node
                    {
                        let start = parent.location.row();
                        if let Some(end) = parent.end_location {
                            let end = end.row();
                            let top = match file_contents.lines().nth(start - 1) {
                                Some(l) => l.trim_end().to_string(),
                                None => return None,
                            };
                            let top = format!("{start}: {top}");
                            let decorators = get_decorator_list(decorator_list);
                            let parameters = get_args(*args.clone());
                            let returns = get_return_type(returns.clone());
                            return Some(PythonParentFunction {
                                name: name.to_string(),
                                top,
                                lines: (start, end),
                                parameters,
                                decorators,
                                returns,
                            });
                        }
                    }
                    None
                })
                .collect::<Vec<PythonParentFunction>>();

            let new_func = PythonFunction {
                name: name.to_string(),
                parameters: get_args(*args),
                parent,
                decorators: get_decorator_list(&decorator_list),
                returns: get_return_type(returns),
                class,
                body,
                lines: (start, end),
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
    map: &HashMap<usize, &usize>,
    functions: &mut Vec<(
        Located<StmtKind>,
        Vec<Located<StmtKind>>,
        Vec<Located<StmtKind>>,
    )>,
    current_parent: &mut Vec<Located<StmtKind>>,
    current_class: &mut Vec<Located<StmtKind>>,
    lookup_name: &str,
) {
    let mut new_ast = VecDeque::from(body);
    loop {
        if new_ast.is_empty() {
            break;
        }
        let stmt = new_ast.pop_front().expect("No stmt found edge case shouldn't happen please file a bug to https://github.com/mendelsshop/git_function_history/issues");
        get_functions(
            stmt,
            map,
            functions,
            current_parent,
            current_class,
            lookup_name,
        );
    }
}

fn get_functions(
    stmt: Located<StmtKind>,
    map: &HashMap<usize, &usize>,
    functions: &mut Vec<(
        Located<StmtKind>,
        Vec<Located<StmtKind>>,
        Vec<Located<StmtKind>>,
    )>,
    current_parent: &mut Vec<Located<StmtKind>>,
    current_class: &mut Vec<Located<StmtKind>>,
    lookup_name: &str,
) {
    let stmt_clone = stmt.clone();
    match stmt.node {
        StmtKind::FunctionDef { ref name, .. } | StmtKind::AsyncFunctionDef { ref name, .. }
            if name == lookup_name =>
        {
            if stmt.end_location.is_some() {
                functions.push((stmt, current_class.clone(), current_parent.clone()));
            }
        }
        StmtKind::If { body, orelse, .. }
        | StmtKind::While { body, orelse, .. }
        | StmtKind::For { body, orelse, .. }
        | StmtKind::AsyncFor { body, orelse, .. } => {
            get_functions_recurisve(
                body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            get_functions_recurisve(
                orelse,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
        }
        StmtKind::FunctionDef { body, .. } | StmtKind::AsyncFunctionDef { body, .. } => {
            current_parent.push(stmt_clone);
            get_functions_recurisve(
                body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            current_parent.pop();
        }
        StmtKind::ClassDef { body, .. } => {
            current_class.push(stmt_clone);
            get_functions_recurisve(
                body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            current_class.pop();
        }
        StmtKind::With { body, .. } | StmtKind::AsyncWith { body, .. } => get_functions_recurisve(
            body,
            map,
            functions,
            current_parent,
            current_class,
            lookup_name,
        ),
        StmtKind::Try {
            body,
            orelse,
            finalbody,
            ..
        } => {
            get_functions_recurisve(
                body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            get_functions_recurisve(
                orelse,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            get_functions_recurisve(
                finalbody,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
        }
        _ => {}
    }
}

fn get_args(args: Arguments) -> Params {
    let mut parameters = Params {
        args: Vec::new(),
        varargs: None,
        posonlyargs: Vec::new(),
        kwargs: Vec::new(),
        varkwargs: None,
    };
    for arg in args.args {
        parameters.args.push(Param {
            name: arg.node.arg,
            r#type: arg.node.annotation.and_then(|x| {
                if let ExprKind::Name { id, .. } = x.node {
                    Some(id)
                } else {
                    None
                }
            }),
        });
    }
    for arg in args.kwonlyargs {
        parameters.kwargs.push(Param {
            name: arg.node.arg,
            r#type: arg.node.annotation.and_then(|x| {
                if let ExprKind::Name { id, .. } = x.node {
                    Some(id)
                } else {
                    None
                }
            }),
        });
    }
    if let Some(arg) = args.vararg {
        parameters.varargs = Some(Param {
            name: arg.node.arg,
            r#type: arg.node.annotation.and_then(|x| {
                if let ExprKind::Name { id, .. } = x.node {
                    Some(id)
                } else {
                    None
                }
            }),
        });
    }
    if let Some(arg) = args.kwarg {
        parameters.varkwargs = Some(Param {
            name: arg.node.arg,
            r#type: arg.node.annotation.and_then(|x| {
                if let ExprKind::Name { id, .. } = x.node {
                    Some(id)
                } else {
                    None
                }
            }),
        });
    }
    for arg in args.posonlyargs {
        parameters.posonlyargs.push(Param {
            name: arg.node.arg,
            r#type: arg.node.annotation.and_then(|x| {
                if let ExprKind::Name { id, .. } = x.node {
                    Some(id)
                } else {
                    None
                }
            }),
        });
    }
    parameters
}

fn get_return_type(retr: Option<Box<Located<ExprKind>>>) -> Option<String> {
    if let Some(retr) = retr {
        if let ExprKind::Name { ref id, .. } = retr.node {
            return Some(id.to_string());
        }
    }
    None
}

fn get_decorator_list(decorator_list: &[Located<ExprKind>]) -> Vec<String> {
    // todo add full test version ie not just name and also add line numbers so it can be displayed/retrieved with get_tops
    decorator_list
        .iter()
        .map(|x| x.node.name().to_string())
        .collect::<Vec<String>>()
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
                function.parameters.arg_has_name(parameter_name)
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
                .any(|x| x.parameters.arg_has_name(parameter_name)),
            Self::HasParentFunctionwithReturnType(return_type) => function
                .parent
                .iter()
                .any(|x| x.returns.as_ref().map_or(false, |x| x == return_type)),
        }
    }
}

impl FunctionTrait for PythonFunction {
    // TODO: return decorator list too
    fn get_tops(&self) -> Vec<(String, usize)> {
        let mut tops = Vec::new();
        for class in &self.class {
            tops.push((class.top.clone(), class.lines.0));
        }
        for parent in &self.parent {
            tops.push((parent.top.clone(), parent.lines.0));
        }
        tops
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        Vec::new()
    }

    fn get_total_lines(&self) -> (usize, usize) {
        // find the first line of the function (could be the parent or the class)
        self.class
            .iter()
            .map(|x| x.lines)
            .chain(self.parent.iter().map(|x| x.lines))
            .min()
            .unwrap_or(self.lines)
    }
    impl_function_trait!(PythonFunction);
}
