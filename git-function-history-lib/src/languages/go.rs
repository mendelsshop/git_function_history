use crate::impl_function_trait;
use std::{collections::HashMap, error::Error, fmt};

use super::FunctionTrait;

#[derive(Debug, Clone)]
pub struct GoFunction {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) parameters: GoParameter,
    pub(crate) returns: Option<String>,
    pub(crate) lines: (usize, usize),
}
#[derive(Debug, Clone)]
pub enum GoParameter {
    /// type
    Type(Vec<String>),
    /// (name, type)
    Named(HashMap<String, String>),
}

impl GoParameter {
    pub fn extend(&mut self, other: &Self) {
        match (self, other) {
            (Self::Type(a), Self::Type(b)) => a.extend(b.clone()),
            (Self::Named(a), Self::Named(b)) => a.extend(b.clone()),
            _ => {}
        }
    }
}

impl GoFunction {
    pub const fn new(
        name: String,
        body: String,
        parameters: GoParameter,
        returns: Option<String>,
        lines: (usize, usize),
    ) -> Self {
        Self {
            name,
            body,
            parameters,
            returns,
            lines,
        }
    }
}

impl fmt::Display for GoFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.body)?;
        Ok(())
    }
}

impl FunctionTrait for GoFunction {
    impl_function_trait!(GoFunction);

    fn get_bottoms(&self) -> Vec<String> {
        vec![]
    }

    fn get_tops(&self) -> Vec<String> {
        vec![]
    }

    fn get_total_lines(&self) -> (usize, usize) {
        let start = self.lines.0;
        let end = self.lines.1;
        (start, end)
    }
}

pub(crate) fn find_function_in_file(
    file_contents: &str,
    name: &str,
) -> Result<Vec<GoFunction>, Box<dyn Error>> {
    let parsed_file = gosyn::parse_source(file_contents)
        .map_err(|e| format!("{:?}", e))?
        .decl;
    let parsed = parsed_file
        .into_iter()
        .filter_map(|decl| match decl {
            gosyn::ast::Declaration::Function(func) => {
                if func.name.name == name {
                    let mut lines = match func.body.as_ref() {
                        Some(body) => (func.name.pos, body.pos.1),
                        None => return None,
                    };
                    if let Some(recv) = func.recv {
                        lines.0 = recv.pos();
                    }
                    lines.0 = file_contents[..lines.0].rfind("func")?;

                    for i in &func.docs {
                        if i.pos < lines.0 {
                            lines.0 = i.pos;
                        }
                    }
                    let body = file_contents[lines.0..=lines.1].to_string();
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
                    // see if the first parameter has a name:
                    let mut parameters = func.typ.params.list.get(0).map_or_else(
                        || GoParameter::Type(vec![]),
                        |param| {
                            if param.name.is_empty() {
                                GoParameter::Type(match &param.typ {
                                    gosyn::ast::Expression::Type(gosyn::ast::Type::Ident(
                                        ident,
                                    )) => {
                                        vec![ident.name.name.clone()]
                                    }
                                    _ => {
                                        vec![]
                                    }
                                })
                            } else {
                                let typ = match &param.typ {
                                    gosyn::ast::Expression::Type(gosyn::ast::Type::Ident(
                                        ident,
                                    )) => ident.name.name.clone(),

                                    _ => String::new(),
                                };
                                let names = param.name.iter().map(|n| n.name.clone());
                                GoParameter::Named(
                                    names.into_iter().map(|name| (name, typ.clone())).collect(),
                                )
                            }
                        },
                    );

                    func.typ.params.list.iter().skip(1).for_each(|param| {
                        if param.name.is_empty() {
                            if let gosyn::ast::Expression::Type(gosyn::ast::Type::Ident(ident)) =
                                &param.typ
                            {
                                if let GoParameter::Type(types) = &mut parameters {
                                    types.push(ident.name.name.clone());
                                }
                            }
                        } else {
                            let typ = match &param.typ {
                                gosyn::ast::Expression::Type(gosyn::ast::Type::Ident(ident)) => {
                                    ident.name.name.clone()
                                }

                                _ => String::new(),
                            };
                            let names = param.name.iter().map(|n| n.name.clone());

                            if let GoParameter::Named(named) = &mut parameters {
                                for name in names {
                                    named.insert(name, typ.clone());
                                }
                            }
                        }
                    });
                    let returns = Some(
                        func.typ
                            .result
                            .list
                            .iter()
                            .map(|p| {
                                p.name
                                    .iter()
                                    .map(|x| &x.name)
                                    .map(std::string::ToString::to_string)
                                    .collect::<String>()
                            })
                            .collect(),
                    )
                    .filter(|x: &String| !x.is_empty());
                    Some(GoFunction::new(
                        func.name.name,
                        body,
                        parameters,
                        returns,
                        lines,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        return Err(format!("could not find function {} in file", name).into());
    }
    Ok(parsed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoFilter {
    // refers to the type of a parameter
    FunctionWithParameter(String),
    // refers to the name of a parameter
    FunctionWithParameterName(String),
    FunctionWithReturnType(String),
}

impl GoFilter {
    pub fn matches(&self, func: &GoFunction) -> bool {
        match self {
            Self::FunctionWithParameter(param) => match &func.parameters {
                GoParameter::Type(types) => types.iter().any(|t| t == param),
                GoParameter::Named(named) => named.values().any(|t| t == param),
            },
            Self::FunctionWithParameterName(param) => {
                if let GoParameter::Named(named) = &func.parameters {
                    named.iter().any(|(name, _)| name == param)
                } else {
                    false
                }
            }
            Self::FunctionWithReturnType(ret) => {
                func.returns.as_ref().map_or(false, |x| x.contains(ret))
            }
        }
    }
}
