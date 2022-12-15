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
    pub args: Vec<String>,
    pub body: String,
}

impl RubyFunction {
    pub fn new(
        name: String,
        lines: (usize, usize),
        class: Option<RubyClass>,
        args: Vec<String>,
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
                    .map_or_else(Vec::new, |a| parse_args_from_node(&a)),
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

fn parse_args_from_node(node: &lib_ruby_parser::Node) -> Vec<String> {
    match node {
        lib_ruby_parser::Node::Args(args) => args
            .args
            .iter()
            .map(|arg| match arg {
                lib_ruby_parser::Node::Arg(arg) => arg.name.clone(),
                lib_ruby_parser::Node::Kwarg(arg) => arg.name.clone(),
                lib_ruby_parser::Node::Kwoptarg(arg) => arg.name.clone(),
                lib_ruby_parser::Node::Optarg(arg) => arg.name.clone(),
                lib_ruby_parser::Node::Restarg(arg) => arg
                    .name
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                lib_ruby_parser::Node::Kwrestarg(arg) => arg
                    .name
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                _ => String::new(),
            })
            .filter(|f| !f.is_empty())
            .collect(),
        _ => vec![],
    };
    vec![]
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
