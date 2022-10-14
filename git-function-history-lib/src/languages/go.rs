use std::{collections::HashMap, error::Error, fmt};

use crate::impl_function_trait;

use super::FunctionTrait;

#[derive(Debug, Clone)]
pub struct GoFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) parent: Vec<ParentFunction>,
    pub(crate) returns: Option<String>,
    pub(crate) lines: (usize, usize),
}

impl GoFunction {
    pub fn new(
        name: String,
        body: String,
        parameters: Vec<String>,
        parent: Vec<ParentFunction>,
        returns: Option<String>,
        lines: (usize, usize),
    ) -> Self {
        Self {
            name,
            body,
            parameters,
            parent,
            returns,
            lines,
        }
    }
}

impl fmt::Display for GoFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.parameters.is_empty() {
            write!(f, "(")?;
            for (i, param) in self.parameters.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ")")?;
        }
        if let Some(ret) = &self.returns {
            write!(f, " -> {}", ret)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ParentFunction {
    pub(crate) name: String,
    pub(crate) top: String,
    pub(crate) returns: Option<String>,
    pub(crate) parameters: Vec<String>,
    pub(crate) lines: (usize, usize),
}

impl ParentFunction {
    pub fn new(
        name: String,
        top: String,
        returns: Option<String>,
        parameters: Vec<String>,
        lines: (usize, usize),
    ) -> Self {
        Self {
            name,
            top,
            returns,
            parameters,
            lines,
        }
    }
}

impl FunctionTrait for GoFunction {
    impl_function_trait!(GoFunction);

    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        for parent in &self.parent {
            bottoms.push(parent.top.clone());
        }
        bottoms
    }

    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        for parent in &self.parent {
            tops.push(parent.top.clone());
        }
        tops
    }

    fn get_total_lines(&self) -> (usize, usize) {
        let mut start = self.lines.0;
        let mut end = self.lines.1;
        for parent in &self.parent {
            if parent.lines.0 < start {
                start = parent.lines.0;
            }
            if parent.lines.1 > end {
                end = parent.lines.1;
            }
        }
        (start, end)
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<GoFunction>, Box<dyn Error>> {
    // TODO: use expect() instead of unwrap() for better error handling
    let parsed_file = gosyn::parse_source(file_contents)
        .map_err(|e| format!("{:?}", e))?
        .decl;
    Ok(parsed_file
        .into_iter()
        .filter_map(|decl| match decl {
            gosyn::ast::Declaration::Function(func) => {
                if func.name.name == name {

                    let mut lines = (func.name.pos, func.body.as_ref().unwrap().pos.1);
                    match func.recv {
                        Some(recv) => {
                            lines.0 = recv.pos();
                        }
                        None => {}
                    }
                    lines.0 =file_contents[..lines.0].rfind("func").unwrap();
                    for i in &func.docs {
                        if i.pos < lines.0 {
                            lines.0 = i.pos;
                        }
                    }
                    let body = file_contents[lines.0..lines.1 + 1].to_string();
                    let mut start_line = 0;
                    for i in file_contents.chars().enumerate() {
                        if i.1 == '\n' {
                            if i.0 > lines.0 {
                                lines.0 = i.0;
                                break;
                            }
                            start_line += 1;
                        }

                    }
                    let mut end_line = 0;
                    for i in file_contents.chars().enumerate() {
                        if i.1 == '\n' {
                            if i.0 > lines.1 {
                                lines.1 = i.0;
                                break;
                            }
                            end_line += 1;
                        }
                    }

                    lines.0 = start_line;
                    lines.1 = end_line;

                    println!("lines: {:?}-", lines);
                    let parameters = func
                        .typ
                        .params
                        .list
                        .iter()
                        .map(|p| &p.tag.as_ref().unwrap().value)
                        .map(|x| x.to_string())
                        .collect();
                    let returns = Some(
                        func.typ
                            .result
                            .list
                            .iter()
                            .map(|p| {
                                p.name
                                    .iter()
                                    .map(|x| &x.clone().name)
                                    .map(|x| x.to_string())
                                    .collect::<String>()
                            })
                            .collect(),
                    )
                    .filter(|x: &String| !x.is_empty());
                    // TODO: get parent functions
                    let parent = vec![];
                    Some(GoFunction::new(
                        func.name.name,
                        body,
                        parameters,
                        parent,
                        returns,
                        lines,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn test_go() {
        let file_contents = r#"
package main
import "fmt"


func    (s *Selection) mains() {
    
    fmt.Println("Hello, World!")

}

func  mains() {
    fmt.Println("Hello, World!")
}


"#;
        let functions = find_function_in_file(file_contents, "mains").unwrap();
        println!("{:#?}", functions[0]);
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "mains");
        assert_eq!(functions[0].parameters.len(), 0);
        assert_eq!(functions[0].parent.len(), 0);
        assert_eq!(functions[0].returns, None);
        assert_eq!(functions[0].lines, (5, 9));
    }
}
