use chrono::{DateTime, FixedOffset};
use rayon::prelude::IntoParallelRefIterator;
#[cfg(feature = "parallel")]
use rayon::prelude::ParallelIterator;
use std::{collections::HashMap, error::Error, fmt::{self, Formatter, Display}};

use crate::{
    languages::{python::{self, PythonFunction}, rust::{self, RustFunction}, Function, LanguageFilter},
    Filter,
};

#[cfg(feature = "c_lang")]
use crate::languages::c;
pub mod new_ideas {

    use std::{fmt::{self, Formatter, Display, Debug}, error::Error, collections::HashMap};

    use enum_dispatch::enum_dispatch;

    use crate::{languages::{Language, Function, rust::{self, RustFunction}, python::{self, PythonFunction} , LanguageFilter}, Filter};

    #[cfg(feature = "c_lang")]
    use crate::languages::c;
    #[derive(Debug, Clone)]
    pub struct File<T: Function +  Display + Debug > {
        pub(crate) path: String,
        pub(crate) functions: Vec<T>,
        pub(crate) current_position: usize,
    }

    impl<T: Function + Display + Debug> File<T> {
        pub fn new(path: String, functions: Vec<T>) -> File<T> {
            File { path, functions, current_position: 0 }
        }
    }

    impl <T: Function +  Display + Debug>Display for File<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            for (i, function) in self.functions.iter().enumerate() {
                write!(
                    f,
                    "{}",
                    match i {
                        0 => "",
                        _ => "\n...\n",
                    },
                )?;
                let previous = match i {
                    0 => None,
                    _ => self.functions.get(i - 1),
                };
                let next = self.functions.get(i + 1);
                function.fmt_with_context(f, previous, next)?;
            }
            Ok(())
        }
    }
    #[derive(Debug, Clone)]
    pub enum FunctionType {
        Rust(RustFunction),
        Python(PythonFunction),
        #[cfg(feature = "c_lang")]
        C(CFunction),
    }

    impl Display for FunctionType {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::Rust(rust) => std::fmt::Display::fmt(&rust, f),
                Self::Python(python) => std::fmt::Display::fmt(&python, f),
                #[cfg(feature = "c_lang")]
                Self::C(c) => c.fmt(f),
            }
        }
    }
    // impl Into<FunctionType> for &RustFunction {
    //     fn into(self) -> FunctionType {
    //         FunctionType::Rust(self.clone())
    //     }
    // }

    impl std::convert::From<&FunctionType  > for &RustFunction {
        fn from(function: &FunctionType) -> Self {
            match function {
                FunctionType::Rust(rust) => rust,
                _ => panic!("Cannot convert to RustFunction"),
            }
        }
    }
    impl std::convert::From<&FunctionType  > for &PythonFunction {
        fn from(function: &FunctionType) -> Self {
            match function {
                FunctionType::Python(python) => python,
                _ => panic!("Cannot convert to PythonFunction"),
            }
        }
    }
    impl Function for FunctionType {
        fn fmt_with_context(
            &self,
            f: &mut Formatter<'_>,
            previous: Option<&Self>,
            next: Option<&Self>,
        ) -> fmt::Result {
            match self {
                Self::Rust(rust) => rust.fmt_with_context(f, previous.map(|p| p.into()), next.map(|n| n.into())),
                Self::Python(python) => python.fmt_with_context(f, previous.map(|x| x.into()), next.map(|x| x.into())),
                #[cfg(feature = "c_lang")]
                Self::C(c) => c.fmt_with_context(f, previous, next),
            }
        }

 

        fn get_metadata(&self) -> HashMap<&str, String> {
            match self {
                Self::Rust(rust) => rust.get_metadata(),
                Self::Python(python) => python.get_metadata(),
                #[cfg(feature = "c_lang")]
                Self::C(c) => c.get_metadata(),
            }
        }

        fn get_lines(&self) -> (usize, usize) {
            match self {
                Self::Rust(rust) => rust.get_lines(),
                Self::Python(python) => python.get_lines(),
                #[cfg(feature = "c_lang")]
                Self::C(c) => c.get_lines(),
            }
        }

        fn matches(&self, filter: &LanguageFilter) -> bool {
            match self {
                Self::Rust(rust) => rust.matches(filter),
                Self::Python(python) => python.matches(filter),
                #[cfg(feature = "c_lang")]
                Self::C(c) => c.matches(filter),
            }
        }
    }


    #[enum_dispatch]
    pub trait FileTrait {
        // type Item:;
        fn get_path(&self) -> &str;
        fn get_functions(&self) -> &Vec<FunctionType>;
        fn filter_by(&self, filter: Filter) -> Result<File<FunctionType>, Box<dyn Error>> {
            let mut functions: Vec<Self::Item> = self.get_functions().clone();
            match filter {
                Filter::PLFilter(pl_filter) => {
                    functions = functions.into_iter().filter(|function| {function.matches(&pl_filter)}).collect();
                }
                Filter::FunctionInLines(start, end) => {
                    functions = functions
                        .into_iter()
                        .filter(|function| {
                            let (start_line, end_line) = function.get_lines();
                            start_line >= start && end_line <= end
                        })
                        .collect();
                }
                _ => {
                    return Err(format!("Filter {:?} not implemented", filter).into());
                }

            }

            Ok(File { path: self.get_path().to_string(), functions, current_position: 0 })
        }
    }
    #[enum_dispatch(FileTrait)]
    #[derive(Debug, Clone)]
    
    pub enum FileType {
        Python(PythonFile),
        Rust(RustFile),
        #[cfg(feature = "c_lang")]
        C(CFile),
    }
    impl FileType {
        pub fn filter_by(&self, filter: Filter) -> Result<FileType, Box<dyn Error>> {
            match self {
                FileType::Python(file) => {
                    let filtered_file = file.filter_by(filter)?;
                    Ok(filtered_file)
                }
                FileType::Rust(file) => {
                    let filtered_file = file.filter_by(filter)?;
                    Ok(FileType::Rust(filtered_file))
                }
                #[cfg(feature = "c_lang")]
                FileType::C(file) => {
                    let filtered_file = file.filter_by(filter)?;
                    Ok(FileType::C(filtered_file))
                }
            }
        }
    }
    type PythonFile = File<PythonFunction>;
    impl FileTrait for PythonFile {
        // type Item = PythonFunction;
 
        fn get_path(&self) -> &str {
            &self.path
        }
        fn get_functions(&self) -> &Vec<PythonFunction> {
            &self.functions
        }
    }
        

    type RustFile = File<RustFunction>;

    impl FileTrait for RustFile {
        // type Item = RustFunction;
        fn get_path(&self) -> &str {
            &self.path
        }
        fn get_functions(&self) -> &Vec<RustFunction> {
            &self.functions
        }
    }

    #[cfg(feature = "c_lang")]
    pub type CFile = File<c::Function>;

    #[cfg(feature = "c_lang")]
    impl FileTrait<c::Function> for CFile {
        fn get_path(&self) -> &str {
            &self.path
        }
        fn get_functions(&self) -> &Vec<c::Function> {
            &self.functions
        }
    }
    impl <T: Function +  Display + Debug + Clone>Iterator for File<T> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            self.functions.get(self.current_position).map(|function| {
                self.current_position += 1;
                function.clone()
            })
        }

    }

    // impl Into <FileType> for PythonFile {
    //     fn into(self) -> FileType {
    //         FileType::Python(self)
    //     }
    // }

    

    // impl FileTrait for FileType {
    //     type Item = 
    //     fn get_path(&self) -> &str {
    //         match self {
    //             FileType::Python(file) => file.get_path(),
    //             FileType::Rust(file) => file.get_path(),
    //             #[cfg(feature = "c_lang")]
    //             FileType::C(file) => file.get_path(),
    //         }
    //     }
    //     fn get_functions(&self) -> &Vec<FunctionType> {
    //         match self {
    //             FileType::Python(file) => file.get_functions(),
    //             FileType::Rust(file) => file.get_functions(),
    //             #[cfg(feature = "c_lang")]
    //             FileType::C(file) => file.get_functions(),
    //         }
    //     }
    // }

    #[test]
    fn test_file() {
        let file: PythonFile = File::new("test.py".to_owned(), vec![]);

        
        let file: FileType = file.into();
        // file.
        // let file = Box::new(file);
        // file.
        // assert_eq!(file.
        // assert_eq!(file.get_functions().len(), 2);
        // assert_eq!(file.get_functions()[0].get_name(), "test");
        // assert_eq!(file.get_functions()[1].get_name(), "test2");
    }
    
}


// pub mod new_new_ideas {

//     use std::{fmt::{self, Formatter, Display, Debug}, error::Error, collections::HashMap};

//     use chrono::{FixedOffset, DateTime};
//     use enum_dispatch::enum_dispatch;

//     use crate::{languages::{Language, Function, rust::{self, RustFunction}, python::{self, PythonFunction} , LanguageFilter}, Filter};

//     #[cfg(feature = "c_lang")]
//     use crate::languages::c;

//     use super::Directions;
//     #[derive(Debug, Clone)]
//     pub struct File<T: Function +  Display + Debug > {
//         pub(crate) path: String,
//         pub(crate) functions: Vec<T>,
//         pub(crate) current_position: usize,
//     }

//     impl<T: Function + Display + Debug> File<T> {
//         pub fn new(path: String, functions: Vec<T>) -> File<T> {
//             File { path, functions, current_position: 0 }
//         }
//     }

//     impl <T: Function +  Display + Debug>Display for File<T> {
//         fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//             for (i, function) in self.functions.iter().enumerate() {
//                 write!(
//                     f,
//                     "{}",
//                     match i {
//                         0 => "",
//                         _ => "\n...\n",
//                     },
//                 )?;
//                 let previous = match i {
//                     0 => None,
//                     _ => self.functions.get(i - 1),
//                 };
//                 let next = self.functions.get(i + 1);
//                 function.fmt_with_context(f, Box::new(previous), Box::new(next))?;
//             }
//             Ok(())
//         }
//     }



//     // #[enum_dispatch]
//     pub trait FileTrait {
//         type Item: Function + Display + Debug + Clone;
//         fn get_path(&self) -> &str;
//         fn get_functions(&self) -> &Vec<Self::Item>;
//         fn filter_by(&self, filter: Filter) -> Result<File<Self::Item>, Box<dyn Error>> {
//             let mut functions: Vec<Self::Item> = self.get_functions().clone();
//             match filter {
//                 Filter::PLFilter(pl_filter) => {
//                     functions = functions.into_iter().filter(|function| {function.matches(&pl_filter)}).collect();
//                 }
//                 Filter::FunctionInLines(start, end) => {
//                     functions = functions
//                         .into_iter()
//                         .filter(|function| {
//                             let (start_line, end_line) = function.get_lines();
//                             start_line >= start && end_line <= end
//                         })
//                         .collect();
//                 }
//                 _ => {
//                     return Err(format!("Filter {:?} not implemented", filter).into());
//                 }

//             }

//             Ok(File { path: self.get_path().to_string(), functions, current_position: 0 })
//         }
//     }

//     type PythonFile = File<PythonFunction>;
//     impl FileTrait for PythonFile {
//         type Item = PythonFunction;
 
//         fn get_path(&self) -> &str {
//             &self.path
//         }
//         fn get_functions(&self) -> &Vec<PythonFunction> {
//             &self.functions
//         }
//     }
        

//     type RustFile = File<RustFunction>;

//     impl FileTrait for RustFile {
//         type Item = RustFunction;
//         fn get_path(&self) -> &str {
//             &self.path
//         }
//         fn get_functions(&self) -> &Vec<RustFunction> {
//             &self.functions
//         }
//     }

//     #[cfg(feature = "c_lang")]
//     pub type CFile = File<c::Function>;

//     #[cfg(feature = "c_lang")]
//     impl FileTrait<c::Function> for CFile {
//         fn get_path(&self) -> &str {
//             &self.path
//         }
//         fn get_functions(&self) -> &Vec<c::Function> {
//             &self.functions
//         }
//     }
//     impl <T: Function +  Display + Debug + Clone>Iterator for File<T> {
//         type Item = T;
//         fn next(&mut self) -> Option<Self::Item> {
//             self.functions.get(self.current_position).map(|function| {
//                 self.current_position += 1;
//                 function.clone()
//             })
//         }

//     }


//     #[derive(Debug, Clone)]
//     pub struct Commit {
//         commit_hash: String,
//         python_files: Vec<PythonFile>,
//         rust_files: Vec<RustFile>,
//         #[cfg(feature = "c_lang")]
//         c_files: Vec<CFile>,
//         pub(crate) date: DateTime<FixedOffset>,
//         current_iter_pos: usize,
//         current_pos: usize,
//         author: String,
//         email: String,
//         message: String,
//     }



//     impl Commit {
//         /// Create a new `Commit` with the given `commit_hash`, functions, and date.
//         pub fn new(
//             commit_hash: String,
//             python_files: Vec<PythonFile>,
//             rust_files: Vec<RustFile>,
//             #[cfg(feature = "c_lang")] c_files: Vec<CFile>,
//             date: &str,
//             author: String,
//             email: String,
//             message: String,
//         ) -> Self {
//             Self {
//                 commit_hash,
//                 python_files,
//                 rust_files,
//                 #[cfg(feature = "c_lang")]
//                 c_files,
//                 date: DateTime::parse_from_rfc2822(&date).expect("Failed to parse date"),
//                 current_pos: 0,
//                 current_iter_pos: 0,
//                 author,
//                 email,
//                 message,
//             }
//         }
    
//         /// sets the current file to the next file if possible
//         pub fn move_forward(&mut self) {
//             if self.current_pos >= self.files.len() - 1 {
//                 return;
//             }
//             self.current_pos += 1;
//         }
    
//         /// sets the current file to the previous file if possible
//         pub fn move_back(&mut self) {
//             if self.current_pos == 0 {
//                 return;
//             }
//             self.current_pos -= 1;
//         }
    
//         /// returns a hashmap containing the commits metadata
//         /// inlcuding the `commit hash`, `date`, and `file`
//         pub fn get_metadata(&self) -> HashMap<String, String> {
//             let mut map = HashMap::new();
//             map.insert("commit hash".to_string(), self.commit_hash.clone());
//             map.insert("date".to_string(), self.date.to_rfc2822());
//             map.insert(
//                 "file".to_string(),
//                 self.files[self.current_pos].name.clone(),
//             );
//             map
//         }
    
//         /// returns the current file
//         pub fn get_file<T: Function>(&self) -> &File<T> {
//             &self.files[self.current_pos]
//         }
    
//         /// returns the current file (mutable)
//         pub fn get_file_mut(&mut self) -> &mut File {
//             &mut self.files[self.current_pos]
//         }
    
//         /// tells you in which directions you can move through the files in the commit
//         pub fn get_move_direction(&self) -> Directions {
//             match self.current_pos {
//                 0 if self.files.len() == 1 => Directions::None,
//                 0 => Directions::Forward,
//                 x if x == self.files.len() - 1 => Directions::Back,
//                 _ => Directions::Both,
//             }
//         }
    
//         /// returns a new `Commit` by filtering the current one by the filter specified (does not modify the current one).
//         ///
//         /// valid filters are: `Filter::FunctionInLines`, and `Filter::FileAbsolute`, `Filter::FileRelative`, and `Filter::Directory`.
//         pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
//             match filter {
//                 Filter::FileAbsolute(_)
//                 | Filter::FileRelative(_)
//                 | Filter::Directory(_)
//                 | Filter::FunctionInLines(..)
//                 | Filter::PLFilter(_) => {}
//                 Filter::None => {
//                     return Ok(self.clone());
//                 }
//                 _ => Err("Invalid filter")?,
//             }
//             #[cfg(feature = "parallel")]
//             let t = self.files.iter();
//             #[cfg(not(feature = "parallel"))]
//             let t = self.files.iter();
//             let vec: Vec<_> = t
//                 .filter(|f| match filter {
//                     Filter::FileAbsolute(file) => f.name == *file,
//                     Filter::FileRelative(file) => f.name.ends_with(file),
//                     Filter::Directory(dir) => f.name.contains(dir),
//                     Filter::FunctionInLines(..) | Filter::PLFilter(_) => f.filter_by(filter).is_ok(),
//                     Filter::None => true,
//                     _ => false,
//                 })
//                 .cloned()
//                 .collect();
    
//             if vec.is_empty() {
//                 return Err("No files found for filter")?;
//             }
//             Ok(Self {
//                 commit_hash: self.commit_hash.clone(),
//                 files: vec,
//                 date: self.date,
//                 current_pos: 0,
//                 current_iter_pos: 0,
//                 author: self.author.clone(),
//                 email: self.email.clone(),
//                 message: self.message.clone(),
//             })
//         }
//     }
// }
/// This is used to store each individual file in a commit and the associated functions in that file.
#[derive(Debug, Clone)]
pub struct File {
    /// The name of the file
    pub(crate) name: String,
    pub(crate) functions: FileType,
    pub(crate) current_pos: usize,
}

impl File {
    /// Create a new file with the given name and functions
    pub const fn new(name: String, functions: FileType) -> Self {
        Self {
            name,
            functions,
            current_pos: 0,
        }
    }

    /// This is used to get the functions in the file
    pub const fn get_functions(&self) -> &FileType {
        &self.functions
    }

    /// This is used to get the functions in the file (mutable)
    pub fn get_functions_mut(&mut self) -> &mut FileType {
        &mut self.functions
    }

    pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
        match filter {
            Filter::FunctionInLines(..) | Filter::PLFilter(_) => {}
            Filter::None => return Ok(self.clone()),
            _ => return Err("Filter not available")?,
        }
        let mut new_file = self.clone();
        new_file.functions = match &self.functions {
            FileType::Rust(functions, _) => {
                let mut vec = Vec::new();
                for function in functions {
                    match &filter {
                        Filter::FunctionInLines(start, end) => {
                            if function.lines.0 >= *start && function.lines.1 <= *end {
                                vec.push(function.clone());
                            }
                        }
                        Filter::PLFilter(LanguageFilter::Rust(filter)) => {
                            if filter.matches(function) {
                                vec.push(function.clone());
                            }
                        }
                        _ => return Err("Filter not available")?,
                    }
                }
                if vec.is_empty() {
                    return Err("No functions found for filter")?;
                }
                FileType::Rust(vec, 0)
            }

            FileType::Python(functions, _) => {
                let mut vec = Vec::new();
                for function in functions {
                    match &filter {
                        Filter::FunctionInLines(start, end) => {
                            if function.lines.0 >= *start && function.lines.1 <= *end {
                                vec.push(function.clone());
                            }
                        }
                        Filter::PLFilter(LanguageFilter::Python(filter)) => {
                            if filter.matches(function) {
                                vec.push(function.clone());
                            }
                        }
                        _ => return Err("Filter not available")?,
                    }
                }
                if vec.is_empty() {
                    return Err("No functions found for filter")?;
                }
                FileType::Python(vec, 0)
            }
            #[cfg(feature = "c_lang")]
            FileType::C(functions, _) => {
                let mut vec = Vec::new();
                for function in functions {
                    match &filter {
                        Filter::FunctionInLines(start, end) => {
                            if function.lines.0 >= *start && function.lines.1 <= *end {
                                vec.push(function.clone());
                            }
                        }
                        Filter::PLFilter(LanguageFilter::C(filter)) => {
                            if filter.matches(function) {
                                vec.push(function.clone());
                            }
                        }
                        _ => return Err("Filter not available")?,
                    }
                }
                if vec.is_empty() {
                    return Err("No functions found for filter")?;
                }
                FileType::C(vec, 0)
            }
        };
        match &new_file.functions {
            #[cfg(feature = "c_lang")]
            FileType::C(functions, _) => {
                if functions.is_empty() {
                    return Err("No functions found for filter")?;
                }
            }
            FileType::Python(functions, _) => {
                if functions.is_empty() {
                    return Err("No functions found for filter")?;
                }
            }
            FileType::Rust(functions, _) => {
                if functions.is_empty() {
                    return Err("No functions found for filter")?;
                }
            }
        }
        Ok(new_file)
    }

    /// This is will get the current function in the file
    pub fn get_current_function(&self) -> Option<FunctionType> {
        self.functions.get(self.current_pos)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // write!(f, "{}", self.name)
        match &self.functions {
            FileType::Python(python, _) => {
                for (i, function) in python.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => python.get(i - 1),
                    };
                    let next = python.get(i + 1);
                    function.fmt_with_context(f, previous,next);
                }
            }
            FileType::Rust(rust, _) => {
                for (i, function) in rust.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => rust.get(i - 1),
                    };
                    let next = rust.get(i + 1);
                    function.fmt_with_context(f, previous,next);
                }
            }
            #[cfg(feature = "c_lang")]
            FileType::C(c, _) => {
                for (i, function) in c.iter().enumerate() {
                    write!(
                        f,
                        "{}",
                        match i {
                            0 => "",
                            _ => "\n...\n",
                        },
                    )?;
                    let previous = match i {
                        0 => None,
                        _ => c.get(i - 1),
                    };
                    let next = c.get(i + 1);
                    function.fmt_with_context(f, previous,next);
                }
            }
        };
        Ok(())
    }
}

pub enum FunctionType {
    Python(python::PythonFunction),
    Rust(rust::RustFunction),
    #[cfg(feature = "c_lang")]
    C(c::Function),
}

impl FunctionType {
    pub const fn get_lines(&self) -> (usize, usize) {
        match self {
            Self::Python(python) => python.lines,
            Self::Rust(rust) => rust.lines,
            #[cfg(feature = "c_lang")]
            Self::C(c) => c.lines,
        }
    }
}

impl Iterator for File {
    type Item = FunctionType;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current_pos;
        self.current_pos += 1;

        match &self.functions {
            FileType::Python(python, _) => {
                python.get(current).map(|f| FunctionType::Python(f.clone()))
            }
            FileType::Rust(rust, _) => rust.get(current).map(|f| FunctionType::Rust(f.clone())),
            #[cfg(feature = "c_lang")]
            FileType::C(c, _) => c.get(current).map(|f| FunctionType::C(f.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    /// The python language
    Python(Vec<python::PythonFunction>, usize),
    /// The rust language
    Rust(Vec<rust::RustFunction>, usize),
    #[cfg(feature = "c_lang")]
    /// c language
    C(Vec<c::Function>, usize),
}

impl Iterator for FileType {
    type Item = FunctionType;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Python(python, pos) => python.get(*pos).map(|f| {
                *pos += 1;
                FunctionType::Python(f.clone())
            }),
            Self::Rust(rust, pos) => rust.get(*pos).map(|f| {
                *pos += 1;
                FunctionType::Rust(f.clone())
            }),
            #[cfg(feature = "c_lang")]
            Self::C(c, pos) => c.get(*pos).map(|f| {
                *pos += 1;
                FunctionType::C(f.clone())
            }),
        }
    }
}

impl FileType {
    pub fn get(&self, index: usize) -> Option<FunctionType> {
        match self {
            Self::Rust(rust, _) => rust
                .get(index)
                .map(|function| FunctionType::Rust(function.clone())),
            #[cfg(feature = "c_lang")]
            Self::C(c, _) => c
                .get(index)
                .map(|function| FunctionType::C(function.clone())),
            Self::Python(python, _) => python
                .get(index)
                .map(|function| FunctionType::Python(function.clone())),
        }
    }
    #[cfg(feature = "c_lang")]
    pub fn get_current<
        T: Clone + From<python::Function> + From<c::Function> + From<rust::RustFunction>,
    >(
        &self,
    ) -> Vec<T> {
        match self {
            Self::Python(python, _pos) => python
                .iter()
                .map(|function| T::from(function.clone()))
                .collect(),
            Self::Rust(rust, _pos) => rust
                .iter()
                .map(|function| T::from(function.clone()))
                .collect(),
            Self::C(c, _pos) => c.iter().map(|function| T::from(function.clone())).collect(),
        }
    }

    #[cfg(not(feature = "c_lang"))]
    pub fn get_current<T: Clone + From<python::PythonFunction> + From<rust::RustFunction>>(&self) -> Vec<T> {
        match self {
            Self::Python(python, _pos) => python
                .iter()
                .map(|function| T::from(function.clone()))
                .collect(),
            Self::Rust(rust, _pos) => rust
                .iter()
                .map(|function| T::from(function.clone()))
                .collect(),
        }
    }
}

/// This holds information like date and commit `commit_hash` and also the list of function found in the commit.
#[derive(Debug, Clone)]
pub struct Commit {
    commit_hash: String,
    files: Vec<File>,
    pub(crate) date: DateTime<FixedOffset>,
    current_iter_pos: usize,
    current_pos: usize,
    author: String,
    email: String,
    message: String,
}

impl Commit {
    /// Create a new `Commit` with the given `commit_hash`, functions, and date.
    pub fn new(
        commit_hash: String,
        files: Vec<File>,
        date: &str,
        author: String,
        email: String,
        message: String,
    ) -> Self {
        Self {
            commit_hash,
            files,
            date: DateTime::parse_from_rfc2822(date).expect("Failed to parse date"),
            current_pos: 0,
            current_iter_pos: 0,
            author,
            email,
            message,
        }
    }

    /// sets the current file to the next file if possible
    pub fn move_forward(&mut self) {
        if self.current_pos >= self.files.len() - 1 {
            return;
        }
        self.current_pos += 1;
    }

    /// sets the current file to the previous file if possible
    pub fn move_back(&mut self) {
        if self.current_pos == 0 {
            return;
        }
        self.current_pos -= 1;
    }

    /// returns a hashmap containing the commits metadata
    /// inlcuding the `commit hash`, `date`, and `file`
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("commit hash".to_string(), self.commit_hash.clone());
        map.insert("date".to_string(), self.date.to_rfc2822());
        map.insert(
            "file".to_string(),
            self.files[self.current_pos].name.clone(),
        );
        map
    }

    /// returns the current file
    pub fn get_file(&self) -> &File {
        &self.files[self.current_pos]
    }

    /// returns the current file (mutable)
    pub fn get_file_mut(&mut self) -> &mut File {
        &mut self.files[self.current_pos]
    }

    /// tells you in which directions you can move through the files in the commit
    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.files.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.files.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    /// returns a new `Commit` by filtering the current one by the filter specified (does not modify the current one).
    ///
    /// valid filters are: `Filter::FunctionInLines`, and `Filter::FileAbsolute`, `Filter::FileRelative`, and `Filter::Directory`.
    pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
        match filter {
            Filter::FileAbsolute(_)
            | Filter::FileRelative(_)
            | Filter::Directory(_)
            | Filter::FunctionInLines(..)
            | Filter::PLFilter(_) => {}
            Filter::None => {
                return Ok(self.clone());
            }
            _ => Err("Invalid filter")?,
        }
        #[cfg(feature = "parallel")]
        let t = self.files.iter();
        #[cfg(not(feature = "parallel"))]
        let t = self.files.iter();
        let vec: Vec<_> = t
            .filter(|f| match filter {
                Filter::FileAbsolute(file) => f.name == *file,
                Filter::FileRelative(file) => f.name.ends_with(file),
                Filter::Directory(dir) => f.name.contains(dir),
                Filter::FunctionInLines(..) | Filter::PLFilter(_) => f.filter_by(filter).is_ok(),
                Filter::None => true,
                _ => false,
            })
            .cloned()
            .collect();

        if vec.is_empty() {
            return Err("No files found for filter")?;
        }
        Ok(Self {
            commit_hash: self.commit_hash.clone(),
            files: vec,
            date: self.date,
            current_pos: 0,
            current_iter_pos: 0,
            author: self.author.clone(),
            email: self.email.clone(),
            message: self.message.clone(),
        })
    }
}

impl Iterator for Commit {
    type Item = File;
    fn next(&mut self) -> Option<Self::Item> {
        // get the current function without removing it
        let function = self.files.get(self.current_iter_pos).cloned();
        self.current_iter_pos += 1;
        function
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.files[self.current_pos])?;
        Ok(())
    }
}

/// This struct holds the a list of commits and the function that were looked up for each commit.
#[derive(Debug, Clone)]
pub struct FunctionHistory {
    pub(crate) name: String,
    pub(crate) commit_history: Vec<Commit>,

    current_iter_pos: usize,
    current_pos: usize,
}

impl FunctionHistory {
    // creates a new `FunctionHistory` from a list of commits
    pub fn new(name: String, commit_history: Vec<Commit>) -> Self {
        Self {
            name,
            commit_history,
            current_iter_pos: 0,
            current_pos: 0,
        }
    }
    /// This will return a vector of all the commit hashess in the history.
    pub fn list_commit_hashes(&self) -> Vec<&str> {
        self.commit_history
            .iter()
            .map(|c| c.commit_hash.as_ref())
            .collect()
    }

    /// this will move to the next commit if possible
    pub fn move_forward(&mut self) {
        if self.current_pos >= self.commit_history.len() - 1 {
            return;
        }
        self.current_pos += 1;
        self.commit_history[self.current_pos].current_iter_pos = 0;
        self.commit_history[self.current_pos].current_pos = 0;
    }

    /// this will move to the previous commit if possible
    pub fn move_back(&mut self) {
        if self.current_pos == 0 {
            return;
        }
        self.current_pos -= 1;
        self.commit_history[self.current_pos].current_iter_pos = 0;
        self.commit_history[self.current_pos].current_pos = 0;
    }

    /// this will move to the next file in the current commit if possible
    pub fn move_forward_file(&mut self) {
        self.commit_history[self.current_pos].move_forward();
    }

    /// this will move to the previous file in the current commit if possible
    pub fn move_back_file(&mut self) {
        self.commit_history[self.current_pos].move_back();
    }

    /// this returns some metadata about the current commit
    /// including the `commit hash`, `date`, and `file`
    pub fn get_metadata(&self) -> HashMap<String, String> {
        self.commit_history[self.current_pos].get_metadata()
    }

    /// returns a mutable reference to the current commit
    pub fn get_mut_commit(&mut self) -> &mut Commit {
        &mut self.commit_history[self.current_pos]
    }

    /// returns a reference to the current commit
    pub fn get_commit(&self) -> &Commit {
        &self.commit_history[self.current_pos]
    }

    /// returns the directions in which ways you can move through the commit history
    pub fn get_move_direction(&self) -> Directions {
        match self.current_pos {
            0 if self.commit_history.len() == 1 => Directions::None,
            0 => Directions::Forward,
            x if x == self.commit_history.len() - 1 => Directions::Back,
            _ => Directions::Both,
        }
    }

    /// tells you in which directions you can move through the files in the current commit
    pub fn get_commit_move_direction(&self) -> Directions {
        self.commit_history[self.current_pos].get_move_direction()
    }

    /// returns a new `FunctionHistory` by filtering the current one by the filter specified (does not modify the current one).
    /// All filter are valid
    ///
    /// # examples
    /// ```rust
    /// use git_function_history::{get_function_history, Filter, FileType};
    ///
    /// let history = get_function_history("new", FileType::None, Filter::None).unwrap();
    ///
    /// history.filter_by(Filter::Directory("app".to_string())).unwrap();
    /// ```
    pub fn filter_by(&self, filter: &Filter) -> Result<Self, Box<dyn Error>> {
        #[cfg(feature = "parallel")]
        let t = self.commit_history.par_iter();
        #[cfg(not(feature = "parallel"))]
        let t = self.commit_history.iter();
        let vec: Vec<Commit> = t
            .filter(|f| match filter {
                Filter::FunctionInLines(..)
                | Filter::Directory(_)
                | Filter::FileAbsolute(_)
                | Filter::PLFilter(_)
                | Filter::FileRelative(_) => f.filter_by(filter).is_ok(),
                Filter::CommitHash(commit_hash) => &f.commit_hash == commit_hash,
                Filter::Date(date) => &f.date.to_rfc2822() == date,
                Filter::DateRange(start, end) => {
                    let start = match DateTime::parse_from_rfc2822(start) {
                        Ok(date) => date,
                        Err(_) => return false,
                    };
                    let end = match DateTime::parse_from_rfc2822(end) {
                        Ok(date) => date,
                        Err(_) => return false,
                    };
                    f.date >= start || f.date <= end
                }

                Filter::Author(author) => &f.author == author,
                Filter::AuthorEmail(email) => &f.email == email,
                Filter::Message(message) => f.message.contains(message),
                Filter::None => true,
            })
            .cloned()
            .collect();

        if vec.is_empty() {
            return Err("No history found for the filter")?;
        }
        Ok(Self {
            commit_history: vec,
            name: self.name.clone(),
            current_pos: 0,
            current_iter_pos: 0,
        })
    }
}

impl Display for FunctionHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.commit_history[self.current_pos])?;
        Ok(())
    }
}

impl Iterator for FunctionHistory {
    type Item = Commit;
    fn next(&mut self) -> Option<Self::Item> {
        self.commit_history
            .get(self.current_iter_pos)
            .cloned()
            .map(|c| {
                self.current_iter_pos += 1;
                c
            })
    }
}

/// Options returned when you use `get_move_direction`
/// It tells you which way you could move through the commits or files
pub enum Directions {
    /// You can only move forward
    Forward,
    /// You can only move back
    Back,
    /// You can't move in any direction
    None,
    /// You can move in both directions
    Both,
}

trait ErrorToOption<FileType> {
    fn to_option(self) -> Option<FileType>;
}

impl<FileType> ErrorToOption<FileType> for Result<FileType, Box<dyn Error>> {
    fn to_option(self) -> Option<FileType> {
        match self {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}
