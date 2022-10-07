use std::{collections::HashMap, error::Error, fmt};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// The python language
    Python,
    /// The rust language
    Rust,
    #[cfg(feature = "c_lang")]
    /// c language
    C,
    /// all available languages
    All,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageFilter {
    /// python filter
    Python(python::Filter),
    /// rust filter
    Rust(rust::Filter),
    #[cfg(feature = "c_lang")]
    /// c filter
    C(c::Filter),
}

impl Language {
    pub fn from_string(s: &str) -> Result<Self, Box<dyn Error>> {
        match s {
            "python" => Ok(Self::Python),
            "rust" => Ok(Self::Rust),
            #[cfg(feature = "c_lang")]
            "c" => Ok(Self::C),
            "all" => Ok(Self::All),
            _ => Err(format!("Unknown language: {}", s))?,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => write!(f, "python"),
            Self::Rust => write!(f, "rust"),
            #[cfg(feature = "c_lang")]
            Self::C => write!(f, "c"),
            Self::All => write!(f, "all"),
        }
    }
}
#[cfg(feature = "c_lang")]
pub mod c;
pub mod python;
pub mod rust;

pub trait Function: fmt::Debug + Clone  + fmt::Display   {
    fn fmt_with_context(
        &self,
        f: &mut fmt::Formatter<'_>,
        previous: Box<Option<&Self>>,
        next: Box<Option<&Self>>,
    ) -> fmt::Result;
    fn get_metadata(&self) -> HashMap<&str, String>;

    fn get_lines(&self) -> (usize, usize);

    fn matches(&self, filter: &LanguageFilter) -> bool;
}

pub type FunctionResult<T> = Result<Vec<T>, Box<dyn Error>>;

// impl File<rust::Function> {
//     pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
//         let mut vec = Vec::new();
//         for function in &self.functions {
//             match &filter {
//                 Filter::FunctionInBlock(block_type) => {
//                     if let Some(block) = &function.block {
//                         if block.block_type == *block_type {
//                             vec.push(function.clone());
//                         }
//                     }
//                 }
//                 Filter::FunctionInLines(start, end) => {
//                     if function.lines.0 >= *start && function.lines.1 <= *end {
//                         vec.push(function.clone());
//                     }
//                 }
//                 Filter::FunctionWithParent(parent) => {
//                     for parents in &function.function {
//                         if parents.name == *parent {
//                             vec.push(function.clone());
//                         }
//                     }
//                 }
//                 Filter::None => vec.push(function.clone()),
//                 _ => return Err("Filter not available")?,
//             }
//         }
//         if vec.is_empty() {
//             return Err("No functions found for filter")?;
//         }
//         Ok(Self {
//             name: self.name.clone(),
//             functions: vec,
//             current_pos: 0,
//         })
//     }
// }

// impl File<python::Function> {
//     pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
//         let mut vec = Vec::new();
//         for function in &self.functions {
//             match &filter {
//                 Filter::FunctionInBlock(block_type) => {
//                     if let Some(block) = &function.class {
//                         if block.name == *block_type.name {
//                             vec.push(function.clone());
//                         }
//                     }
//                 }
//                 Filter::FunctionInLines(start, end) => {
//                     if function.lines.0 >= *start && function.lines.1 <= *end {
//                         vec.push(function.clone());
//                     }
//                 }
//                 Filter::FunctionWithParent(parent) => {
//                     for parents in &function.parent {
//                         if parents.name == *parent {
//                             vec.push(function.clone());
//                         }
//                     }
//                 }
//                 Filter::None => vec.push(function.clone()),
//                 _ => return Err("Filter not available")?,
//             }
//         }
//         if vec.is_empty() {
//             return Err("No functions found for filter")?;
//         }
//         Ok(Self {
//             name: self.name.clone(),
//             functions: vec,
//             current_pos: 0,
//         })
//     }
// }

// impl File<c::Function> {
//     pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
//         let mut vec = Vec::new();
//         for function in &self.functions {
//             match &filter {

//                 Filter::FunctionInLines(start, end) => {
//                     if function.lines.0 >= *start && function.lines.1 <= *end {
//                         vec.push(function.clone());
//                     }
//                 }
//                 Filter::FunctionWithParent(parent) => {
//                     for parents in &function.parent {
//                         if parents.name == *parent {
//                             vec.push(function.clone());
//                         }
//                     }
//                 }
//                 Filter::None => vec.push(function.clone()),
//                 _ => return Err("Filter not available")?,
//             }
//         }
//         if vec.is_empty() {
//             return Err("No functions found for filter")?;
//         }
//         Ok(Self {
//             name: self.name.clone(),
//             functions: vec,
//             current_pos: 0,
//         })
//     }
// }
