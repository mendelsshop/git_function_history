use std::{error::Error, fmt};

use lib_ruby_parser::{
    nodes::{Class, Def},
    Parser, ParserOptions,
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
            Some(class) => write!(f, "{}", class.top)?,
            None => {}
        }
        write!(f, "{}", self.body)?;
        match &self.class {
            Some(class) => write!(f, "{}", class.bottom)?,
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
    fns.iter()
        .map(|(f, c)| {
            let class = c.as_ref().map(|c| RubyClass {
                name: parser_class_name(c),
                line: (f.expression_l.begin, f.expression_l.end),
                superclass: None,
                // TODO: get top and bottom
                top: String::new(),
                bottom: String::new(),
            });
            Ok(RubyFunction {
                name: f.name.clone(),
                // TODO: turn the positions into lines
                lines: (f.expression_l.begin, f.expression_l.end),
                class,
                // TODO: put line numbers in here
                body: f
                    .expression_l
                    .source(&parsed.input)
                    .expect("Failed to get function body"),
                args: f
                    .args
                    .clone()
                    .map_or_else(Vec::new, |a| parse_args_from_node(&a)),
            })
        })
        .collect()
}

pub fn get_functions_from_node(
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
    // match node {
    //     lib_ruby_parser::Node::Args(args) => args
    //         .args
    //         .iter()
    //         .map(|arg| arg.name.str_type().to_string())
    //         .collect(),
    //     _ => vec![],
    // }
    vec![]
}

fn parser_class_name(class: &Class) -> String {
    String::new()
}

impl FunctionTrait for RubyFunction {
    crate::impl_function_trait!(RubyFunction);

    fn get_tops(&self) -> Vec<String> {
        self.class.as_ref().map_or_else(Vec::new, |c| vec![c.top.clone()])
    }

    fn get_bottoms(&self) -> Vec<String> {
        self.class.as_ref().map_or_else(Vec::new, |c| vec![c.bottom.clone()])
    }

    fn get_total_lines(&self) -> (usize, usize) {
        self.class.as_ref().map_or(self.lines, |c| c.line)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    FunctionInLines((usize, usize)),
}

impl Filter {
    pub const fn matches(&self, function: &RubyFunction) -> bool {
        match self {
            Self::FunctionInLines((start, end)) => {
                function.lines.0 >= *start && function.lines.1 <= *end
            }
        }
    }
}
