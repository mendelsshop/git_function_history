use crate::{Filter, UnwrapToError};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self},
};
// TODO: lisp/scheme js, java?(https://github.com/tanin47/javaparser.rs) php?(https://docs.rs/tagua-parser/0.1.0/tagua_parser/)
use self::{python::PythonFunction, ruby::RubyFunction, rust::RustFunction};

// #[cfg(feature = "c_lang")]
// use self::c::CFunction;

#[cfg(feature = "unstable")]
use go::GoFunction;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// The python language
    Python,
    /// The rust language
    Rust,
    // #[cfg(feature = "c_lang")]
    // /// c language
    // C,
    #[cfg(feature = "unstable")]
    /// The go language
    Go,
    /// the Ruby language
    Ruby,
    /// all available languages
    All,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageFilter {
    /// python filter
    Python(python::PythonFilter),
    /// rust filter
    Rust(rust::RustFilter),
    // #[cfg(feature = "c_lang")]
    // /// c filter
    // C(c::CFilter),
    #[cfg(feature = "unstable")]
    /// go filter
    Go(go::GoFilter),
    /// ruby filter
    Ruby(ruby::RubyFilter),
}

impl Language {
    /// takes string and returns the corresponding language
    ///
    /// # Errors
    ///
    /// `Err` will be returned if the string is not a valid language
    pub fn from_string(s: &str) -> Result<Self, Box<dyn Error>> {
        match s {
            "python" => Ok(Self::Python),
            "rust" => Ok(Self::Rust),
            // #[cfg(feature = "c_lang")]
            // "c" => Ok(Self::C),
            #[cfg(feature = "unstable")]
            "go" => Ok(Self::Go),
            "all" => Ok(Self::All),
            "ruby" => Ok(Self::Ruby),
            _ => Err(format!("Unknown language: {s}"))?,
        }
    }

    pub const fn get_names(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Rust => "rust",
            // #[cfg(feature = "c_lang")]
            // Language::C => "c",
            #[cfg(feature = "unstable")]
            Self::Go => "go",
            Self::Ruby => "ruby",
            #[cfg(feature = "unstable")]
            Self::All => "python, rust, go, or ruby",
            #[cfg(not(feature = "unstable"))]
            Self::All => "python, rust, or ruby",
        }
    }

    pub const fn get_file_endings(&self) -> &[&str] {
        match self {
            Self::Python => &["py", "pyw"],
            Self::Rust => &["rs"],
            // #[cfg(feature = "c_lang")]
            // Language::C => &["c", "h"],
            #[cfg(feature = "unstable")]
            Self::Go => &["go"],
            Self::Ruby => &["rb"],
            #[cfg(feature = "unstable")]
            Self::All => &["py", "pyw", "rs", "go", "rb"],
            #[cfg(not(feature = "unstable"))]
            Self::All => &["py", "pyw", "rs", "rb"],
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => write!(f, "python"),
            Self::Rust => write!(f, "rust"),
            // #[cfg(feature = "c_lang")]
            // Self::C => write!(f, "c"),
            #[cfg(feature = "unstable")]
            Self::Go => write!(f, "go"),
            Self::Ruby => write!(f, "ruby"),
            Self::All => write!(f, "all"),
        }
    }
}
// #[cfg(feature = "c_lang")]
// pub mod c;
#[cfg(feature = "unstable")]
pub mod go;
// #[cfg(feature = "unstable")]
// pub mod java;
pub mod python;
pub mod ruby;
pub mod rust;

pub trait FunctionTrait: fmt::Debug + fmt::Display {
    fn get_lines(&self) -> (usize, usize);
    fn get_total_lines(&self) -> (usize, usize);
    fn get_name(&self) -> String;
    fn get_body(&self) -> String;
    /// returns the tops like any the heading of classes/impls (among others) the function is part of along with the starting line of each heading
    /// for example it could return `[("impl Test {", 3)]`
    /// to get just for example the headings use the map method `function.get_tops().map(|top| top.0)`
    fn get_tops(&self) -> Vec<(String, usize)>;
    /// same as `get_tops` just retrieves the bottoms like so `[("}", 22)]`
    fn get_bottoms(&self) -> Vec<(String, usize)>;
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
            self.body.to_string()
        }
    };
}

fn make_lined(snippet: &str, mut start: usize) -> String {
    snippet
        .lines()
        .map(|line| {
            let new = format!("{start}: {line}\n");
            start += 1;
            new
        })
        .collect::<String>()
        .trim_end()
        .to_string()
}

pub trait FileTrait: fmt::Debug + fmt::Display {
    fn get_file_name(&self) -> String;
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>>;

    /// # Errors
    ///
    /// returns `Err` if the wrong filter is given, only `PLFilter` and `FunctionInLines` variants of `Filter` are valid.
    /// with `PLFilter` it will return `Err` if you mismatch the file type with the filter Ie: using `RustFile` and `PythonFilter` will return `Err`.
    fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn get_current(&self) -> Option<Box<dyn FunctionTrait>>;
}

fn turn_into_index(snippet: &str) -> Result<HashMap<usize, Vec<usize>>, Box<dyn Error>> {
    // turn snippet into a hashmap of line number to char index
    // so line 1 is 0 to 10, line 2 is 11 to 20, etc
    let mut index = HashMap::new();
    index.insert(1, vec![]);
    let mut line: usize = 1;
    let mut char_index: usize = 0;
    for c in snippet.chars() {
        if c == '\n' {
            line += 1;
            index.insert(line, vec![char_index]);
        } else {
            index
                .get_mut(&line)
                .unwrap_to_error("line not found")?
                .push(char_index);
        }
        char_index += c.len_utf8();
    }
    Ok(index)
}

fn get_from_index(index: &HashMap<usize, Vec<usize>>, char: usize) -> Option<usize> {
    // gets the line number from the index
    index
        .iter()
        .find(|(_, v)| v.contains(&char))
        .map(|(k, _)| *k)
}

#[test]
fn test_turn_into_index() {
    let snippet = "hello world
Python is cool
Rust is cool
Go is cool
Ruby😂 is cool";
    let index = turn_into_index(snippet).unwrap();
    // assert_eq!(index.get(&0], vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10));
    // assert_eq!(index.get(&0], vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
    println!("done {index:?}");

    // assert_eq!(get_from_index(&index, 0), 0);
    let emoji_index = snippet.find('😂').unwrap();
    println!("{:?}", get_from_index(&index, emoji_index));
}

// macro that generates the code for the different languages
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
                let mut file: Vec<(String, usize)> = Vec::new();
                for function in &self.functions {
                    // get the tops and their starting line number ie: parentfn.lines.0
                    file.extend(function.get_tops());
                    file.push((function.body.to_string(), function.get_lines().0));
                    // get the bottoms and their end line number ie: parentfn.lines.1
                    file.extend(function.get_bottoms());
                }
                file.sort_by(|a, b| a.1.cmp(&b.1));
                file.dedup();
                // order the file by line number
                file.sort_by(|a, b| a.1.cmp(&b.1));
                // print the file each element sperated by a \n...\n
                for (i, (body, _)) in file.iter().enumerate() {
                    write!(f, "{}", body)?;
                    if i != file.len() - 1 {
                        write!(f, "\n...\n")?;
                    }
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
                } else if matches!(filter, Filter::None) {
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
// #[cfg(feature = "c_lang")]
// make_file!(CFile, CFunction, C);
#[cfg(feature = "unstable")]
make_file!(GoFile, GoFunction, Go);
make_file!(RubyFile, RubyFunction, Ruby);

#[cfg(test)]
mod lang_tests {
    // macro that auto genertes the test parse_<lang>_file_time
    macro_rules! make_file_time_test {
        ($name:ident, $extname:ident, $function:ident, $filetype:ident) => {
            #[test]
            fn $name() {
                let mut file = std::env::current_dir().unwrap();
                file.push("src");
                file.push("test_functions.".to_string() + stringify!($extname));
                let files = std::fs::read_to_string(file.clone())
                    .expect(format!("could not read file {:?}", file).as_str());
                let start = std::time::Instant::now();
                let ok = $function::find_function_in_file(&files, "empty_test");
                let end = std::time::Instant::now();
                match &ok {
                    Ok(hist) => {
                        // turn the hist into a file
                        let file = $filetype::new(file.display().to_string(), hist.clone());
                        println!("{}", file);
                        println!("-------------------");
                        for i in hist {
                            println!("{}", i);
                            println!("{:?}", i);
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
                println!("{} took {:?}", stringify!($name), end - start);
                assert!(ok.is_ok());
            }
        };
    }

    use super::*;
    make_file_time_test!(python_parses, py, python, PythonFile);
    make_file_time_test!(rust_parses, rs, rust, RustFile);
    // #[cfg(feature = "c_lang")]
    // make_file_time_test!(c_parses, c, c, CFile);
    #[cfg(feature = "unstable")]
    make_file_time_test!(go_parses, go, go, GoFile);
    make_file_time_test!(ruby_parses, rb, ruby, RubyFile);
}
