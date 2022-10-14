use std::{error::Error, fmt, collections::HashMap};

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
        println!("map: {:?}", map);
    let parsed_file = gosyn::parse_source(file_contents).map_err(|e| format!("{:?}", e))?.decl;
     Ok(parsed_file
        .into_iter()
        .filter_map(|decl| match decl {
            gosyn::ast::Declaration::Function(func) => {
                if func.name.name == name {
                println!("{}", func.typ.pos);

                let mut lines = (func.name.pos, func.body.as_ref().unwrap().pos.1);
                for i in &func.docs {
                    if i.pos < lines.0 {
                        lines.0 = i.pos;
                    }
                }
                let body = file_contents[lines.0..lines.1+1].to_string();
                println!("body: {}", body);
                for i in 0..func.name.pos {
                    if file_contents.chars().nth(i).unwrap() == '\n' {
                        lines.0 = i + 1;
                        break;
                    }
                }
                for i in func.body.as_ref().unwrap().pos.1..file_contents.len() {
                    if file_contents.chars().nth(i).unwrap() == '\n' {
                        lines.1 = i;
                        break;
                    }
                }

                
                lines.0 = *map.keys().find(|x| **x >= lines.0).unwrap();
                // lines.1 = *map.keys().rfind(|x| **x >= lines.1).unwrap();
                
                let parameters = func.typ.params.list.iter().map(|p| &p.tag.as_ref().unwrap().value).map(|x| x.to_string()).collect();
                let returns = Some(func.typ.result.list.iter().map(|p| p.name.iter().map(|x| &x.clone().name).map(|x| x.to_string()).collect::<String>()).collect()).filter(|x: &String| !x.is_empty());
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
//g

func mains() {
    
    fmt.Println("Hello, World!")

}    "#;
        let functions = find_function_in_file(file_contents, "mains").unwrap();
        println!("{:#?}", functions[0]);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "main");
        assert_eq!(functions[0].parameters.len(), 0);
        assert_eq!(functions[0].parent.len(), 0);
        assert_eq!(functions[0].returns, None);
        assert_eq!(functions[0].lines, (3, 5));
    }
}