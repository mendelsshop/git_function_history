use std::{fmt, collections::HashMap, error::Error};

use crate::{File, Filter};

pub enum Language {
    /// The python language
    Python,
    /// The rust language
    Rust,
    /// c language
    C,
}

pub mod c;
pub mod python;
pub mod rust;


// make macro that turns a language into its function struct
macro_rules! language {
    ($name:ident, $struct:ident) => {
        pub fn $name() -> Language {
            Language::$struct
        }
    };
}

language!(python, Python);

pub trait Function {
    fn fmt_with_context(
        &self,
        f: &mut fmt::Formatter<'_>,
        previous: Option<&Self>,
        next: Option<&Self>,
    ) -> fmt::Result;
    fn get_metadata(&self) -> HashMap<&str, String>;
}

impl File<rust::Function> {
    pub fn filter_by(&self, filter: &Filter<rust::BlockType>) -> Result<Self, Box<dyn Error>> {
        let mut vec = Vec::new();
        for function in &self.functions {
            match &filter {
                Filter::FunctionInBlock(block_type) => {
                    if let Some(block) = &function.block {
                        if block.block_type == *block_type {
                            vec.push(function.clone());
                        }
                    }
                }
                Filter::FunctionInLines(start, end) => {
                    if function.lines.0 >= *start && function.lines.1 <= *end {
                        vec.push(function.clone());
                    }
                }
                Filter::FunctionWithParent(parent) => {
                    for parents in &function.function {
                        if parents.name == *parent {
                            vec.push(function.clone());
                        }
                    }
                }
                Filter::None => vec.push(function.clone()),
                _ => return Err("Filter not available")?,
            }
        }
        if vec.is_empty() {
            return Err("No functions found for filter")?;
        }
        Ok(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }
}


impl File<python::Function> {
    pub fn filter_by(&self, filter: &Filter<python::Class>) -> Result<Self, Box<dyn Error>> {
        let mut vec = Vec::new();
        for function in &self.functions {
            match &filter {
                Filter::FunctionInBlock(block_type) => {
                    if let Some(block) = &function.class {
                        if block.name == *block_type.name {
                            vec.push(function.clone());
                        }
                    }
                }
                Filter::FunctionInLines(start, end) => {
                    if function.lines.0 >= *start && function.lines.1 <= *end {
                        vec.push(function.clone());
                    }
                }
                Filter::FunctionWithParent(parent) => {
                    for parents in &function.parent {
                        if parents.name == *parent {
                            vec.push(function.clone());
                        }
                    }
                }
                Filter::None => vec.push(function.clone()),
                _ => return Err("Filter not available")?,
            }
        }
        if vec.is_empty() {
            return Err("No functions found for filter")?;
        }
        Ok(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }
}

impl File<c::Function> {
    pub fn filter_by(&self, filter: &Filter<c::Function>) -> Result<Self, Box<dyn Error>> {
        let mut vec = Vec::new();
        for function in &self.functions {
            match &filter {

                Filter::FunctionInLines(start, end) => {
                    if function.lines.0 >= *start && function.lines.1 <= *end {
                        vec.push(function.clone());
                    }
                }
                Filter::FunctionWithParent(parent) => {
                    for parents in &function.parent {
                        if parents.name == *parent {
                            vec.push(function.clone());
                        }
                    }
                }
                Filter::None => vec.push(function.clone()),
                _ => return Err("Filter not available")?,
            }
        }
        if vec.is_empty() {
            return Err("No functions found for filter")?;
        }
        Ok(Self {
            name: self.name.clone(),
            functions: vec,
            current_pos: 0,
        })
    }
}