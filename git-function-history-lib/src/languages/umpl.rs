use umpl::{self, parser::Thing};

use std::{error::Error, fmt};

use crate::impl_function_trait;

use super::FunctionTrait;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UMPLFunction {
    pub(crate) lines: (usize, usize),
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) args_count: usize,
    pub(crate) parents: Vec<UMPLParentFunction>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UMPLParentFunction {
    pub(crate) lines: (usize, usize),
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) args_count: usize,
}

impl fmt::Display for UMPLFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FunctionTrait for UMPLFunction {
    impl_function_trait!(UMPLFunction);

    fn get_total_lines(&self) -> (usize, usize) {
        todo!()
    }

    fn get_tops(&self) -> Vec<(String, usize)> {
        todo!()
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        todo!()
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<UMPLFunction>, Box<dyn Error>> {
    // parse the file contents
    let lexed = umpl::lexer::Lexer::new(file_contents.to_string());
    let tokens = lexed.scan_tokens();
    let mut parsed = umpl::parser::Parser::new(tokens);
    let ast = parsed.parse();
    let res = find_function_recurse(name, ast, &vec![]);
    if res.len() > 0 {
        return Ok(res);
    }
    Err("no function found")?
}

fn find_function_recurse(name: &str, ast: Vec<Thing>, current: &Vec<UMPLParentFunction>) -> Vec<(UMPLFunction)> {
    let mut results = Vec::new();
    for node in ast {
        match node {
            Thing::Function(fns) => {
                if fns.name.to_string() == name {
                    let new_fn = UMPLFunction {
                        lines: (fns.line as usize, fns.end_line as usize),
                        name: fns.name.to_string(),
                        // TODO: get the function body
                        body: String::new(),
                        args_count: fns.num_arguments as usize,
                        parents: current.clone()
                    };
                    results.push(new_fn);
                } else {
                    let mut new_current = current.clone();
                    // turn into a parent function
                    let pfn  = UMPLParentFunction {
                        lines: (fns.line as usize, fns.end_line as usize),
                        name: fns.name.to_string(),
                        // TODO: get the top and bottom lines
                        top: String::new(),
                        bottom: String::new(),
                        args_count: fns.num_arguments as usize
                    };
                    new_current.push(pfn);
                    results.append(&mut find_function_recurse(name, fns.body, &new_current));
                }
            }
            Thing::LoopStatement(loops) => {
                results.append(&mut find_function_recurse(name, loops.body, current));
            }
            Thing::IfStatement(ifs) => {
                results.append(&mut find_function_recurse(name, ifs.body_true, current));
                results.append(&mut find_function_recurse(name, ifs.body_false, current));
            }
            _ => {}
        }
    }
    results

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UMPLFilter {}

impl UMPLFilter {
    pub fn matches(&self, function: &UMPLFunction) -> bool {
        false
    }
}