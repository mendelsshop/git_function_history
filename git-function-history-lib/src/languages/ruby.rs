use std::{error::Error, fmt};

use lib_ruby_parser::{
    nodes::{Class, Def},
    Loc, Parser, ParserOptions,
};

use crate::UnwrapToError;

use super::FunctionTrait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyFunction {
    pub name: String,
    pub lines: (usize, usize),
    pub class: Option<RubyClass>,
    pub args: RubyParams,
    pub body: String,
}

impl RubyFunction {
    pub const fn new(
        name: String,
        lines: (usize, usize),
        class: Option<RubyClass>,
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.class {
            Some(class) => write!(f, "{}\n...\n", class.top)?,
            None => {}
        }
        write!(f, "{}", self.body)?;
        match &self.class {
            Some(class) => write!(f, "\n...\n{}", class.bottom)?,
            None => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyClass {
    pub name: String,
    pub line: (usize, usize),
    pub superclass: Option<String>,
    pub top: String,
    pub bottom: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyParams {
    args: Vec<String>,
    kwargs: Vec<String>,
    kwnilarg: bool,
    forwarded_args: bool,
    /// arg name, default value
    optargs: Vec<(String, String)>,
    kwoptargs: Vec<(String, String)>,
    kwrestarg: Option<String>,
    restarg: Option<String>,
}

impl RubyParams {
    pub const fn new() -> Self {
        Self {
            args: Vec::new(),
            kwnilarg: false,
            forwarded_args: false,
            optargs: Vec::new(),
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
                .optargs
                .iter()
                .any(|(arg, default)| arg == name || default == name)
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
) -> Result<Vec<RubyFunction>, Box<dyn Error>> {
    let parser = Parser::new(file_contents, ParserOptions::default());
    let parsed = parser.do_parse();
    let ast = parsed.ast.unwrap_to_error("Failed to parse file")?;
    let fns = get_functions_from_node(&ast, &None, name);
    let index = super::turn_into_index(file_contents)?;
    fns.iter()
        .map(|(f, c)| {
            let class = match c {
                Some(c) => {
                    let start_line = super::get_from_index(&index, c.expression_l.begin)
                        .unwrap_to_error("Failed to get start line")?;
                    let end_line = super::get_from_index(&index, c.expression_l.end)
                        .unwrap_to_error("Failed to get end line")?;
                    let loc_end = c.end_l;
                    let top = Loc {
                        begin: c.expression_l.begin,
                        end: c
                            .body
                            .as_ref()
                            .map_or(c.keyword_l.end, |b| b.expression().begin),
                    };
                    let mut top = top
                        .source(&parsed.input)
                        .unwrap_to_error("Failed to get top of class from source")?;
                    top = top.trim_end().to_string();
                    top = super::make_lined(&top, start_line);
                    Some(RubyClass {
                        name: parser_class_name(c),
                        line: (start_line, end_line),
                        superclass: parse_superclass(c),
                        top,
                        bottom: super::make_lined(
                            loc_end
                                .source(&parsed.input)
                                .unwrap_to_error("Failed to get last line of class source")?
                                .trim_matches('\n'),
                            end_line,
                        ),
                    })
                }
                None => None,
            };
            let start = f.expression_l.begin;
            // get the lines from map using f.expression_l.begin and f.expression_l.end
            let start_line =
                super::get_from_index(&index, start).unwrap_to_error("Failed to get start line")?;
            let end_line = super::get_from_index(&index, f.expression_l.end)
                .unwrap_to_error("Failed to get end line")?;
            let starts = start_line + 1;
            Ok(RubyFunction {
                name: f.name.clone(),
                lines: (start_line + 1, end_line + 1),
                class,
                body: super::make_lined(
                    f.expression_l
                        .with_begin(start)
                        .source(&parsed.input)
                        .unwrap_to_error("Failed to get function body from source")?
                        .trim_matches('\n'),
                    starts,
                ),
                args: f
                    .args
                    .clone()
                    .map_or_else(RubyParams::new, |a| parse_args_from_node(&a)),
            })
        })
        .collect()
}

fn get_functions_from_node(
    node: &lib_ruby_parser::Node,
    class: &Option<Class>,
    name: &str,
) -> Vec<(Def, Option<Class>)> {
    match node {
        lib_ruby_parser::Node::Def(def) => {
            if def.name == name {
                vec![(def.clone(), class.clone())]
            } else {
                vec![]
            }
        }
        lib_ruby_parser::Node::Class(class) => {
            let mut functions = vec![];
            for child in &class.body {
                functions.extend(get_functions_from_node(child, &Some(class.clone()), name));
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

fn parse_args_from_node(node: &lib_ruby_parser::Node) -> RubyParams {
    // TODO: make a Args struct has fields {args: Vec<String>, kwargs, kwoptargs, optargs, kwrestargs, block_args, kwnilarg: bool, fowardarg: bool}
    // where I didnt annotate a type its needs to be figured out the args purpose and anoatate the type based on that
    let mut ret_args = RubyParams::new();
    if let lib_ruby_parser::Node::Args(args) = node {
        args.args.iter().for_each(|arg| match arg {
            // basic arg
            //python/ruby
            lib_ruby_parser::Node::Arg(arg) => ret_args.args.push(arg.name.clone()),
            lib_ruby_parser::Node::Kwarg(arg) => ret_args.kwargs.push(arg.name.clone()),

            // args that has a default value
            // TODO: get the default value
            lib_ruby_parser::Node::Kwoptarg(arg) => {
                ret_args.kwoptargs.push((arg.name.clone(), String::new()));
            }
            lib_ruby_parser::Node::Optarg(arg) => {
                ret_args.optargs.push((arg.name.clone(), String::new()));
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
            // Node::ForwardedArgs and Node::Kwargs are for method calls and not definitions thus we are not matching on them
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
        self.class.as_ref().map_or(self.lines, |c| c.line)
    }

    fn get_tops(&self) -> Vec<(String, usize)> {
        self.class
            .as_ref()
            .map_or_else(Vec::new, |c| vec![(c.top.clone(), c.line.0)])
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        self.class
            .as_ref()
            .map_or_else(Vec::new, |c| vec![(c.bottom.clone(), c.line.1)])
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RubyFilter {
    FunctionInClass(String),
    FunctionWithParameter(String),
    FunctionWithSuperClass(String),
}

impl RubyFilter {
    pub fn matches(&self, function: &RubyFunction) -> bool {
        match self {
            Self::FunctionInClass(name) => function
                .class
                .as_ref()
                .map_or(false, |class| *name == class.name),
            Self::FunctionWithParameter(name) => function.args.contains(name),
            Self::FunctionWithSuperClass(name) => function.class.as_ref().map_or(false, |class| {
                class
                    .superclass
                    .as_ref()
                    .map_or(false, |superclass| superclass == name)
            }),
        }
    }
}
