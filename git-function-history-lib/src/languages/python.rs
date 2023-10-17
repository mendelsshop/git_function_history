use git_function_history_proc_macro::enumstuff;
use rustpython_parser::{
    ast::{
        self,
        located::{
            Arguments, Expr, Located, Stmt, StmtAsyncFunctionDef, StmtClassDef, StmtFunctionDef,
        },
        Fold,
    },
    source_code::LinearLocator,
    Parse,
};
use std::collections::VecDeque;
use std::{collections::HashMap, fmt};

use crate::{impl_function_trait, UnwrapToError};

// TODO: cleanup adapting from rp2 to rp3

use super::FunctionTrait;

#[derive(Debug, Clone)]
/// A python function
pub struct PythonFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: PythonParams,
    pub(crate) parent: Vec<PythonParentFunction>,
    pub(crate) decorators: Vec<(usize, String)>,
    pub(crate) class: Vec<PythonClass>,
    pub(crate) lines: (usize, usize),
    pub(crate) returns: Option<String>,
}

impl fmt::Display for PythonFunction {
    /// don't use this for anything other than debugging the output is not guaranteed to be in the right order
    /// use `fmt::Displa`y for `PythonFile` instead
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for class in &self.class {
            for decorator in &class.decorators {
                write!(f, "{}\n...\n", decorator.1)?;
            }
            write!(f, "{}\n...\n", class.top)?;
        }
        for parent in &self.parent {
            for decorator in &parent.decorators {
                write!(f, "{}\n...\n", decorator.1)?;
            }
            write!(f, "{}\n...\n", parent.top)?;
        }
        for decorator in &self.decorators {
            write!(f, "{}\n...\n", decorator.1)?;
        }
        write!(f, "{}", self.body)
    }
}

#[derive(Debug, Clone)]
/// A single parameter of a python function
pub struct Param {
    /// The name of the parameter
    pub name: String,
    /// The optional type of the parameter
    pub r#type: Option<String>,
}

#[derive(Debug, Clone)]
/// The parameters of a python function
/// refer to python docs for more info
/// note: currently we don't save default values
pub struct PythonParams {
    pub args: Vec<Param>,
    pub kwargs: Vec<Param>,
    pub posonlyargs: Vec<Param>,
    pub varargs: Option<Param>,
    pub varkwargs: Option<Param>,
}

impl PythonParams {
    /// Check if a parameter with the given name exists
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
/// A python class
pub struct PythonClass {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) lines: (usize, usize),
    pub(crate) decorators: Vec<(usize, String)>,
}
#[derive(Debug, Clone)]
/// A python function that is a parent of another python function, we don't keep the body of the function
pub struct PythonParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) lines: (usize, usize),
    pub(crate) parameters: PythonParams,
    pub(crate) decorators: Vec<(usize, String)>,
    pub(crate) returns: Option<String>,
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<PythonFunction>, String> {
    let ast = match rustpython_parser::ast::Suite::parse(file_contents, name) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("error parsing file: {e}")),
    };
    let mut locator = LinearLocator::new(file_contents);
    let ast = match locator.fold(ast) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("error parsing file: {e}")),
    };
    let mut functions = vec![];
    if ast.is_empty() {
        return Err("no code found".to_string());
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
        let start = func.0.location().row.to_usize();
        let end = func
            .0
            .end_location()
            .unwrap_to_error("no end location for this function")?
            .row
            .to_usize();
        let (Some(starts), Some(ends)) = (map.get(&(start - 1)), map.get(&(end))) else {
            continue;
        };

        {
            let start_line = func.0.location().row.to_usize();
            let body = match file_contents.get(**starts..**ends) {
                Some(str) => str,
                None => continue,
            }
            .trim_start_matches('\n');
            let body = super::make_lined(body, start_line);
            let class = func
                .1
                .iter()
                .filter_map(|class| {
                    let start = class.location().row.to_usize();
                    if let Some(end) = class.end_location() {
                        let end = end.row.to_usize();
                        let top = match file_contents.lines().nth(start - 1) {
                            Some(l) => l.trim_end().to_string(),
                            None => return None,
                        };
                        let top = format!("{start}: {top}");
                        let decorators =
                            get_decorator_list_new(&class.decorator_list, file_contents);
                        return Some(PythonClass {
                            name: name.to_string(),
                            top,
                            lines: (start, end),
                            decorators,
                        });
                    }
                    None
                })
                .collect::<Vec<PythonClass>>();
            let parent = func
                .2
                .iter()
                .filter_map(|parent| {
                    let start = parent.location().row.to_usize();
                    if let Some(end) = parent.end_location() {
                        let end = end.row.to_usize();
                        let top = match file_contents.lines().nth(start - 1) {
                            Some(l) => l.trim_end().to_string(),
                            None => return None,
                        };
                        let top = format!("{start}: {top}");
                        let decorators =
                            get_decorator_list_new(parent.decorator_list(), file_contents);
                        let parameters = get_args(parent.args().clone());
                        let returns = get_return_type(parent.returns().clone());
                        return Some(PythonParentFunction {
                            name: name.to_string(),
                            top,
                            lines: (start, end),
                            parameters,
                            decorators,
                            returns,
                        });
                    }

                    None
                })
                .collect::<Vec<PythonParentFunction>>();

            let new_func = PythonFunction {
                name: name.to_string(),
                parameters: get_args(func.0.args().clone()),
                parent,
                decorators: get_decorator_list_new(func.0.decorator_list(), file_contents),
                returns: get_return_type(func.0.returns().clone()),
                class,
                body,
                lines: (start, end),
            };
            new.push(new_func);
        }
    }
    if new.is_empty() {
        return Err("no functions found".to_string());
    }
    Ok(new)
}
#[inline]
fn get_functions_recurisve(
    body: Vec<Stmt>,
    map: &HashMap<usize, &usize>,
    functions: &mut FnState,
    current_parent: &mut Vec<FunctionDef>,
    current_class: &mut Vec<StmtClassDef>,
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

#[derive(Clone)]
enum FunctionDef {
    Normal(StmtFunctionDef),
    Async(StmtAsyncFunctionDef),
}

impl From<StmtAsyncFunctionDef> for FunctionDef {
    fn from(v: StmtAsyncFunctionDef) -> Self {
        Self::Async(v)
    }
}

impl From<StmtFunctionDef> for FunctionDef {
    fn from(v: StmtFunctionDef) -> Self {
        Self::Normal(v)
    }
}

impl FunctionDef {
    fn location(&self) -> rustpython_parser::source_code::SourceLocation {
        match self {
            Self::Normal(n) => n.location(),
            Self::Async(a) => a.location(),
        }
    }

    fn end_location(&self) -> Option<rustpython_parser::source_code::SourceLocation> {
        match self {
            Self::Normal(n) => n.end_location(),
            Self::Async(a) => a.end_location(),
        }
    }

    fn decorator_list(&self) -> &[ast::Expr<rustpython_parser::source_code::SourceRange>] {
        match self {
            Self::Normal(n) => &n.decorator_list,
            Self::Async(a) => &a.decorator_list,
        }
    }

    const fn args(&self) -> &ast::located::Arguments {
        match self {
            Self::Normal(n) => &n.args,
            Self::Async(a) => &a.args,
        }
    }

    const fn returns(&self) -> &Option<Box<Expr>> {
        match self {
            Self::Normal(n) => &n.returns,
            Self::Async(a) => &a.returns,
        }
    }
}
type FnState = Vec<(FunctionDef, Vec<StmtClassDef>, Vec<FunctionDef>)>;
fn get_functions(
    stmt: Stmt,
    map: &HashMap<usize, &usize>,
    functions: &mut FnState,
    current_parent: &mut Vec<FunctionDef>,
    current_class: &mut Vec<StmtClassDef>,
    lookup_name: &str,
) {
    // we create a list of blocks to be processed
    let mut blocks = Vec::new();
    match stmt {
        Stmt::FunctionDef(function) if function.name.to_string() == lookup_name => {
            if function.end_location().is_some() {
                functions.push((
                    function.into(),
                    current_class.clone(),
                    current_parent.clone(),
                ));
            }
        }
        Stmt::AsyncFunctionDef(function) if function.name.to_string() == lookup_name => {
            if function.end_location().is_some() {
                functions.push((
                    function.into(),
                    current_class.clone(),
                    current_parent.clone(),
                ));
            }
        }
        Stmt::If(r#if) => {
            blocks.extend([r#if.body, r#if.orelse]);
        }
        Stmt::While(r#while) => {
            blocks.extend([r#while.body, r#while.orelse]);
        }
        Stmt::For(r#for) => {
            blocks.extend([r#for.body, r#for.orelse]);
        }
        Stmt::AsyncFor(r#for) => {
            blocks.extend([r#for.body, r#for.orelse]);
        }
        // we do functions/classes not through blocks as they tell us if a function is nested in a class/function
        Stmt::FunctionDef(function) => {
            current_parent.push(function.clone().into());
            get_functions_recurisve(
                function.body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            current_parent.pop();
        }
        Stmt::AsyncFunctionDef(function) => {
            current_parent.push(function.clone().into());
            get_functions_recurisve(
                function.body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            current_parent.pop();
        }
        Stmt::ClassDef(class) => {
            current_class.push(class.clone());
            get_functions_recurisve(
                class.body,
                map,
                functions,
                current_parent,
                current_class,
                lookup_name,
            );
            current_class.pop();
        }
        Stmt::With(with) => {
            blocks.push(with.body);
        }
        Stmt::AsyncWith(with) => {
            blocks.push(with.body);
        }
        Stmt::Try(r#try) => {
            blocks.extend([r#try.body, r#try.orelse, r#try.finalbody]);
        }
        _ => {}
    };
    for block in blocks {
        get_functions_recurisve(
            block,
            map,
            functions,
            current_parent,
            current_class,
            lookup_name,
        );
    }
}
// TODO save arg.defaults & arg.kwdefaults and attempt to map them to the write parameters
fn get_args(args: Arguments) -> PythonParams {
    let mut parameters = PythonParams {
        args: Vec::new(),
        varargs: None,
        posonlyargs: Vec::new(),
        kwargs: Vec::new(),
        varkwargs: None,
    };
    for arg in args.args {
        let arg = arg.as_arg();
        parameters.args.push(Param {
            name: arg.arg.to_string(),
            r#type: arg.type_comment.clone(),
        });
    }
    for arg in args.kwonlyargs {
        let arg = arg.as_arg();
        parameters.kwargs.push(Param {
            name: arg.arg.to_string(),
            r#type: arg.type_comment.clone(),
        });
    }
    if let Some(arg) = args.vararg {
        parameters.varargs = Some(Param {
            name: arg.arg.to_string(),
            r#type: arg.type_comment.clone(),
        });
    }
    if let Some(arg) = args.kwarg {
        parameters.varkwargs = Some(Param {
            name: arg.arg.to_string(),
            r#type: arg.type_comment.clone(),
        });
    }
    for arg in args.posonlyargs {
        let arg = arg.as_arg();
        parameters.posonlyargs.push(Param {
            name: arg.arg.to_string(),
            r#type: arg.type_comment.clone(),
        });
    }

    parameters
}

fn get_return_type(retr: Option<Box<Expr>>) -> Option<String> {
    if let Some(retr) = retr {
        if let Expr::Name(id) = *retr {
            return Some(id.id.to_string());
        }
    }
    None
}
#[allow(dead_code)]
// keeping this here just in case
fn get_decorator_list(decorator_list: &[Expr]) -> Vec<(usize, String)> {
    decorator_list
        .iter()
        .map(located_expr_to_decorator)
        .collect::<Vec<(usize, String)>>()
}

fn located_expr_to_decorator(expr: &Expr) -> (usize, String) {
    (
        expr.location().row.to_usize(),
        format!(
            "{}:{}decorator with {}",
            expr.location().row.to_usize(),
            vec![" "; expr.location().column.to_usize()].join(""),
            expr.python_name()
        ),
    )
}

fn get_located_expr_line(expr: &Expr, file_contents: &str) -> Option<String> {
    // does not add line numbers to string
    file_contents
        .lines()
        .nth(expr.location().row.to_usize() - 1)
        .map(ToString::to_string)
}

fn get_decorator_list_new(decorator_list: &[Expr], file_contents: &str) -> Vec<(usize, String)> {
    decorator_list
        .iter()
        .map(|x| {
            get_located_expr_line(x, file_contents).map_or_else(
                || located_expr_to_decorator(x),
                |dec| {
                    (
                        x.location().row.to_usize(),
                        super::make_lined(&dec, x.location().row.to_usize()),
                    )
                },
            )
        })
        .collect::<Vec<(usize, String)>>()
}

#[derive(Debug, Clone, PartialEq, Eq, enumstuff)]
/// filters for python functions
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
    /// checks if a function matches the filter
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
            Self::HasDecorator(decorator) => {
                function.decorators.iter().any(|x| x.1.contains(decorator))
            }
            Self::HasClasswithDecorator(decorator) => function
                .class
                .iter()
                .any(|x| x.decorators.iter().any(|y| y.1.contains(decorator))),
            Self::HasParentFunctionwithDecorator(decorator) => function
                .parent
                .iter()
                .any(|x| x.decorators.iter().any(|x| x.1.contains(decorator))),
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
    fn get_tops(&self) -> Vec<(String, usize)> {
        let mut tops = Vec::new();
        for class in &self.class {
            tops.push((class.top.clone(), class.lines.0));
            for decorator in &class.decorators {
                tops.push((decorator.1.clone(), decorator.0));
            }
        }
        for decorator in &self.decorators {
            tops.push((decorator.1.clone(), decorator.0));
        }
        for parent in &self.parent {
            tops.push((parent.top.clone(), parent.lines.0));
            for decorator in &parent.decorators {
                tops.push((decorator.1.clone(), decorator.0));
            }
        }
        tops.sort_by(|top1, top2| top1.1.cmp(&top2.1));
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
