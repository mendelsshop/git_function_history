use std::{collections::HashMap, fmt};

use lib_ruby_parser::{
    nodes::{Class, Def},
    source::DecodedInput,
    Loc, Parser, ParserOptions,
};

use crate::UnwrapToError;

use super::FunctionTrait;

#[derive(Debug, Clone, PartialEq, Eq)]
// repersentation of a ruby function
pub struct RubyFunction {
    pub name: String,
    pub lines: (usize, usize),
    pub class: Vec<RubyClass>,
    pub args: RubyParams,
    pub body: String,
}

impl RubyFunction {
    /// creates a new `RubyFunction`
    pub const fn new(
        name: String,
        lines: (usize, usize),
        class: Vec<RubyClass>,
        args: RubyParams,
        body: String,
    ) -> Self {
        Self {
            name,
            lines,
            class,
            args,
            body,
        }
    }
}

impl fmt::Display for RubyFunction {
    /// don't use this for anything other than debugging the output is not guaranteed to be in the right order
    /// use `fmt::Display` for `RubyFile` instead
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for class in &self.class {
            write!(f, "{}", class.top)?;
        }
        write!(f, "{}", self.body)?;
        for class in &self.class {
            write!(f, "{}", class.bottom)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// represents a Ruby class
pub struct RubyClass {
    pub name: String,
    pub lines: (usize, usize),
    pub superclass: Option<String>,
    pub top: String,
    pub bottom: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// repersents parameters of a ruby function
pub struct RubyParams {
    /// required parameters
    args: Vec<String>,
    /// keyword parameters
    kwargs: Vec<String>,
    /// keyword parameter is nil ie `**nil` in `def foo(a, **nil)`
    kwnilarg: bool,
    /// parameters are forwarded ie `...` in `def foo(...)`
    forwarded_args: bool,
    /// parameters that have optional default values ie `a: 1` in `def foo(a: 1)`
    kwoptargs: Vec<(String, String)>,
    /// keyword rest parameter ie `**a` in `def foo(**a)`
    kwrestarg: Option<String>,
    /// rest parameter ie `*a` in `def foo(*a)`
    restarg: Option<String>,
}

impl RubyParams {
    /// creates `RubyParams` which is a repersentation of the parameters of a ruby function
    pub const fn new() -> Self {
        Self {
            args: Vec::new(),
            kwnilarg: false,
            forwarded_args: false,
            kwrestarg: None,
            restarg: None,
            kwargs: Vec::new(),
            kwoptargs: Vec::new(),
        }
    }

    fn contains(&self, name: &String) -> bool {
        self.args.contains(name)
            || self.kwargs.contains(name)
            || self
                .kwoptargs
                .iter()
                .any(|(arg, default)| arg == name || default == name)
            || Some(name.clone()) == self.kwrestarg
            || Some(name.clone()) == self.restarg
    }
}
impl Default for RubyParams {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<RubyFunction>, String> {
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
    let parser = Parser::new(file_contents, ParserOptions::default());
    let parsed = parser.do_parse();
    for d in parsed.diagnostics {
        if d.is_error() {
            return Err(d.message.render())?;
        }
    }
    let ast = parsed.ast.unwrap_to_error("Failed to parse file")?;
    let fns = get_functions_from_node(&ast, &vec![], name);
    let index = super::turn_into_index(file_contents)?;
    let fns = fns
        .iter()
        .map(|(f, c)| {
            let class = c
                .iter()
                .filter_map(|c| {
                    let start_line = super::get_from_index(&index, c.expression_l.begin)?;
                    let end_line = super::get_from_index(&index, c.expression_l.end)?;
                    let loc_end = c.end_l;
                    let top = Loc {
                        begin: **map.get(&(start_line - 1))?,
                        end: c.body.as_ref().map_or(
                            c.superclass
                                .as_ref()
                                .map_or(c.name.expression().end, |c| c.expression().end),
                            |b| b.expression().begin,
                        ) - 1,
                    };
                    let mut top = top.source(&parsed.input)?;
                    top = top.trim_matches('\n').to_string();
                    top = super::make_lined(&top, start_line);
                    Some(RubyClass {
                        name: parser_class_name(c),
                        lines: (start_line, end_line),
                        superclass: parse_superclass(c),
                        top,
                        bottom: super::make_lined(
                            loc_end
                                .with_begin(**map.get(&(end_line - 1))?)
                                .source(&parsed.input)?
                                .trim_matches('\n'),
                            end_line,
                        ),
                    })
                })
                .collect();
            let start = f.expression_l.begin;
            // get the lines from map using f.expression_l.begin and f.expression_l.end
            let start_line = super::get_from_index(&index, start)
                .unwrap_to_error("Failed to get start lines")?;
            let end_line = super::get_from_index(&index, f.expression_l.end)
                .unwrap_to_error("Failed to get end line")?;
            let starts = start_line + 1;
            Ok::<RubyFunction, String>(RubyFunction {
                name: f.name.clone(),
                lines: (start_line, end_line),
                class,
                body: super::make_lined(
                    f.expression_l
                        .with_begin(**map.get(&(start_line - 1)).unwrap_or(&&0))
                        .source(&parsed.input)
                        .unwrap_to_error("Failed to get function body from source")?
                        .trim_matches('\n'),
                    starts - 1,
                ),
                args: f
                    .args
                    .clone()
                    .map_or_else(RubyParams::new, |a| parse_args_from_node(&a, &parsed.input)),
            })
        })
        .filter_map(Result::ok)
        .collect::<Vec<RubyFunction>>();
    if fns.is_empty() {
        Err("No functions with this name was found in the this file")?;
    }
    Ok(fns)
}

fn get_functions_from_node(
    node: &lib_ruby_parser::Node,
    class: &Vec<Class>,
    name: &str,
) -> Vec<(Def, Vec<Class>)> {
    match node {
        lib_ruby_parser::Node::Def(def) => {
            if def.name == name {
                vec![(def.clone(), class.clone())]
            } else {
                vec![]
            }
        }
        lib_ruby_parser::Node::Class(new_class) => {
            let mut functions = vec![];
            let mut new_list = class.clone();
            new_list.push(new_class.clone());
            for child in &new_class.body {
                functions.extend(get_functions_from_node(child, &new_list, name));
            }
            functions
        }
        lib_ruby_parser::Node::Begin(stat) => {
            let mut functions = vec![];
            for child in &stat.statements {
                functions.extend(get_functions_from_node(child, class, name));
            }
            functions
        }
        _ => vec![],
    }
}

fn parse_args_from_node(node: &lib_ruby_parser::Node, parsed_file: &DecodedInput) -> RubyParams {
    let mut ret_args = RubyParams::new();
    if let lib_ruby_parser::Node::Args(args) = node {
        args.args.iter().for_each(|arg| match arg {
            // basic arg
            lib_ruby_parser::Node::Arg(arg) => ret_args.args.push(arg.name.clone()),
            lib_ruby_parser::Node::Kwarg(arg) => ret_args.kwargs.push(arg.name.clone()),
            // args that has a default value
            lib_ruby_parser::Node::Kwoptarg(arg) => {
                ret_args.kwoptargs.push((
                    arg.name.clone(),
                    arg.default
                        .expression()
                        .source(parsed_file)
                        .unwrap_or_else(|| "could not retrieve default value".to_string()),
                ));
            }
            lib_ruby_parser::Node::Restarg(arg) => {
                if let Some(name) = &arg.name {
                    ret_args.restarg = Some(name.clone());
                }
            }
            lib_ruby_parser::Node::Kwrestarg(arg) => {
                if let Some(name) = &arg.name {
                    ret_args.kwrestarg = Some(name.clone());
                }
            }

            // ruby specific
            lib_ruby_parser::Node::ForwardArg(_) => ret_args.forwarded_args = true,
            lib_ruby_parser::Node::Kwnilarg(_) => ret_args.kwnilarg = true,
            // Node::ForwardedArgs and Node::Kwargs, node::Optarg are for method calls and not definitions thus we are not matching on them
            // Node::Blockarg is for block methods similar to labmdas whic we do not currently support
            _ => {}
        });
    };
    ret_args
}

fn parser_class_name(class: &Class) -> String {
    match class.name.as_ref() {
        lib_ruby_parser::Node::Const(constant) => constant.name.clone(),
        _ => String::new(),
    }
}

fn parse_superclass(class: &Class) -> Option<String> {
    class
        .superclass
        .as_ref()
        .and_then(|superclass| match superclass.as_ref() {
            lib_ruby_parser::Node::Const(constant) => Some(constant.name.clone()),
            _ => None,
        })
}

impl FunctionTrait for RubyFunction {
    crate::impl_function_trait!(RubyFunction);

    fn get_total_lines(&self) -> (usize, usize) {
        self.class
            .iter()
            .map(|class| class.lines)
            .min_by(Ord::cmp)
            .unwrap_or(self.lines)
    }

    fn get_tops(&self) -> Vec<(String, usize)> {
        self.class
            .iter()
            .map(|class| (class.top.clone(), class.lines.0))
            .collect()
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        self.class
            .iter()
            .map(|class| (class.bottom.clone(), class.lines.1))
            .collect()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
/// filter for ruby functions
pub enum RubyFilter {
    /// find a Ruby functions in a specific class
    FunctionInClass(String),
    /// find a Ruby function with a specific parameter
    FunctionWithParameter(String),
    /// find a Ruby function in a class that inherits from a specific class
    FunctionWithSuperClass(String),
}

impl RubyFilter {
    /// check if a function matches the filter
    pub fn matches(&self, function: &RubyFunction) -> bool {
        match self {
            Self::FunctionInClass(name) => function.class.iter().any(|class| &class.name == name),
            Self::FunctionWithParameter(name) => function.args.contains(name),
            Self::FunctionWithSuperClass(name) => function.class.iter().any(|class| {
                class
                    .superclass
                    .as_ref()
                    .map_or(false, |superclass| superclass == name)
            }),
        }
    }
}
