use umpl::{self, parser::Thing};

use std::{fmt};

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
        for i in &self.parents {
            write!(f, "{}\n...\n", i.top)?;
        }
        write!(f, "{}", self.body)?;
        for i in &self.parents {
            write!(f, "\n...\n{}", i.bottom)?;
        }
        Ok(())
    }
}

impl FunctionTrait for UMPLFunction {
    impl_function_trait!(UMPLFunction);

    fn get_total_lines(&self) -> (usize, usize) {
        let mut start = self.lines.0;
        let mut end = self.lines.1;
        for parent in &self.parents {
            if parent.lines.0 < start {
                start = parent.lines.0;
                end = parent.lines.1;
            }
        }
        (start, end)
    }

    fn get_tops(&self) -> Vec<(String, usize)> {
        self.parents
            .iter()
            .map(|f| (f.top.clone(), f.lines.0))
            .collect()
    }

    fn get_bottoms(&self) -> Vec<(String, usize)> {
        self.parents
            .iter()
            .map(|f| (f.bottom.clone(), f.lines.1))
            .collect()
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<UMPLFunction>, String> {
    // parse the file contents
    let lexed = umpl::lexer::Lexer::new(file_contents.to_string());
    let tokens = lexed.scan_tokens();
    let mut parsed = umpl::parser::Parser::new(tokens);
    let ast = parsed.parse();
    let res = find_function_recurse(name, file_contents, ast, &vec![]);
    if !res.is_empty() {
        return Ok(res);
    }
    Err(String::from("Function not found"))
}

fn find_function_recurse(
    name: &str,
    file_contents: &str,
    ast: Vec<Thing>,
    current: &Vec<UMPLParentFunction>,
) -> Vec<UMPLFunction> {
    let mut results = Vec::new();
    for node in ast {
        match node {
            Thing::Function(fns) => {
                if fns.name.to_string() == name {
                    let lines = (fns.line as usize, fns.end_line as usize);
                    let body = file_contents
                        .lines()
                        .enumerate()
                        .filter(|line| line.0 >= lines.0 - 1 && line.0 < lines.1)
                        .map(|line| format!("{}\n", line.1))
                        .collect::<String>();
                    let new_fn = UMPLFunction {
                        lines,
                        name: fns.name.to_string(),
                        body: super::make_lined(&body, lines.0),
                        args_count: fns.num_arguments as usize,
                        parents: current.clone(),
                    };
                    results.push(new_fn);
                } else {
                    let mut new_current = current.clone();
                    // turn into a parent function
                    let lines = (fns.line as usize, fns.end_line as usize);
                    let bottom = if let Some(line) = file_contents.lines().nth(lines.1 - 1) {
                        super::make_lined(line, lines.1)
                    } else {
                        results.append(&mut find_function_recurse(
                            name,
                            file_contents,
                            fns.body,
                            &new_current,
                        ));
                        continue;
                    };
                    // FIXME: this should find the last line of the `top` by looking at its first element of is ast instead of finding the the first `⧼` (which could be commented out)
                    let top_end = if let Some(top_end) = file_contents
                        .lines()
                        .enumerate()
                        .skip(lines.0 - 1)
                        .find_map(|line| line.1.contains('⧼').then_some(line.0))
                    {
                        top_end
                    } else {
                        (if let Some(body) = fns.body.first() {
                            match body {
                                Thing::Identifier(ident) => ident.line,
                                Thing::Expression(expr) => expr.line,
                                Thing::Function(fns) => fns.line,
                                Thing::IfStatement(ifs) => ifs.line,
                                Thing::LoopStatement(loops) => loops.line,
                                Thing::Break(line)
                                | Thing::Continue(line)
                                | Thing::Return(_, line) => *line,
                            }
                        } else {
                            results.append(&mut find_function_recurse(
                                name,
                                file_contents,
                                fns.body,
                                &new_current,
                            ));
                            continue;
                        }) as usize
                    };
                    let top = file_contents
                        .lines()
                        .enumerate()
                        .filter(|line| line.0 >= lines.0 - 1 && line.0 <= top_end)
                        .map(|line| format!("{}\n", line.1))
                        .collect::<String>();
                    let pfn = UMPLParentFunction {
                        lines,
                        name: fns.name.to_string(),
                        top: super::make_lined(&top, lines.0),
                        bottom,
                        args_count: fns.num_arguments as usize,
                    };
                    new_current.push(pfn);
                    results.append(&mut find_function_recurse(
                        name,
                        file_contents,
                        fns.body,
                        &new_current,
                    ));
                }
            }
            Thing::LoopStatement(loops) => {
                results.append(&mut find_function_recurse(
                    name,
                    file_contents,
                    loops.body,
                    current,
                ));
            }
            Thing::IfStatement(ifs) => {
                results.append(&mut find_function_recurse(
                    name,
                    file_contents,
                    ifs.body_true,
                    current,
                ));
                results.append(&mut find_function_recurse(
                    name,
                    file_contents,
                    ifs.body_false,
                    current,
                ));
            }
            _ => {}
        }
    }
    results
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UMPLFilter {
    HasParameterCount(usize),
    HasParentWithParamCount(usize),
    HasParentWithName(String),
    HasParents,
}

impl UMPLFilter {
    pub fn matches(&self, function: &UMPLFunction) -> bool {
        match self {
            Self::HasParameterCount(count) => function.args_count == *count,
            Self::HasParentWithParamCount(count) => {
                function.parents.iter().any(|p| p.args_count == *count)
            }
            Self::HasParentWithName(name) => function.parents.iter().any(|p| p.name == *name),
            Self::HasParents => !function.parents.is_empty(),
        }
    }
}
