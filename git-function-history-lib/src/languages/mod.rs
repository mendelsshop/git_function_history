use crate::Filter;
use std::{
    error::Error,
    fmt::{self, Display},
};

use self::{python::PythonFunction, rust::RustFunction};

#[cfg(feature = "c_lang")]
use self::c::CFunction;
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

pub trait FunctionTrait: fmt::Debug + fmt::Display {
    fn get_tops(&self) -> Vec<String>;
    fn get_lines(&self) -> (usize, usize);
    fn get_total_lines(&self) -> (usize, usize);
    fn get_name(&self) -> String;
    fn get_bottoms(&self) -> Vec<String>;
    fn get_body(&self) -> String;
}

// mace macro that generates get_lines, get_body,get_name
#[macro_export]
macro_rules! impl_function_trait {
    ($name:ident) => {
        fn get_lines(&self) -> (usize, usize) {
            self.lines
        }

        fn get_name(&self) -> String {
            self.name.clone()
        }
        fn get_body(&self) -> String {
            self.body.clone()
        }
    };
}

pub fn fmt_with_context<T: FunctionTrait + Display>(
    current: &T,
    prev: Option<&T>,
    next: Option<&T>,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match (prev, next) {
        (Some(prev), Some(next)) => {
            if prev.get_total_lines() == current.get_total_lines()
                && next.get_total_lines() == current.get_total_lines()
            {
                write!(f, "{}", current.get_body())?;
            } else if prev.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?;
                write!(f, "{}", current.get_bottoms().join("\n"))?;
            } else if next.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_tops().join("\n"))?;
                write!(f, "{}", current.get_body())?;
            } else {
                write!(f, "{}", current)?;
            }
        }
        (Some(prev), None) => {
            if prev.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?;
            } else {
                write!(f, "{}", current)?;
            }
        }
        (None, Some(next)) => {
            if next.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?;
            } else {
                write!(f, "{}", current)?;
            }
        }
        (None, None) => {
            // print the function
            write!(f, "{}", current)?;
        }
    }
    Ok(())
}

// functiontrait is not object safe, so we can't implement it for a trait object ie box<dyn FunctionTrait>
pub trait FileTrait: fmt::Debug + fmt::Display {
    fn get_file_name(&self) -> String;
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>>;
    fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn get_current(&self) -> Option<Box<dyn FunctionTrait>>;
}

// make a macro that generates the code for the different languages
macro_rules! make_file {
    ($name:ident, $function:ident, $filtername:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            file_name: String,
            functions: Vec<$function>,
            current_pos: usize,
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for (index, func) in self.functions.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match index {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match index {
                        0 => None,
                        _ => Some(&self.functions[index - 1]),
                    };
                    let next = self.functions.get(index + 1);
                    crate::languages::fmt_with_context(func, previous, next, f)?;
                }
                Ok(())
            }
        }

        impl FileTrait for $name {
            fn get_file_name(&self) -> String {
                self.file_name.clone()
            }
            fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>> {
                self.functions
                    .clone()
                    .iter()
                    .cloned()
                    .map(|x| Box::new(x) as Box<dyn FunctionTrait>)
                    .collect()
            }
            fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
                let mut filtered_functions = Vec::new();
                if let Filter::PLFilter(LanguageFilter::$filtername(_))
                | Filter::FunctionInLines(..) = filter
                {
                } else if let Filter::None = filter {
                    return Ok(self.clone());
                } else {
                    return Err("filter not supported for this type")?;
                }
                for function in &self.functions {
                    match filter {
                        Filter::FunctionInLines(start, end) => {
                            if function.get_lines().0 >= *start && function.get_lines().1 <= *end {
                                filtered_functions.push(function.clone());
                            }
                        }
                        Filter::PLFilter(LanguageFilter::$filtername(filter)) => {
                            if filter.matches(function) {
                                filtered_functions.push(function.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Ok($name::new(self.file_name.clone(), filtered_functions))
            }
            fn get_current(&self) -> Option<Box<dyn FunctionTrait>> {
                self.functions
                    .get(self.current_pos)
                    .map(|function| Box::new(function.clone()) as Box<dyn FunctionTrait>)
            }
        }

        impl $name {
            pub fn new(file_name: String, functions: Vec<$function>) -> Self {
                $name {
                    file_name,
                    functions,
                    current_pos: 0,
                }
            }
        }
    };
}

make_file!(PythonFile, PythonFunction, Python);
make_file!(RustFile, RustFunction, Rust);
#[cfg(feature = "c_lang")]
make_file!(CFile, CFunction, C);
