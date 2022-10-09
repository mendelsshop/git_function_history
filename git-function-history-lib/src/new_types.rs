// repo: struct {history: Vec<commit>}
// commit: struct {commit_hash: string commit_date: string ... files: Vec<filetype>}
// filetype: enum {python(python_file), rust(rust_file), ...} all variants implement a common trait
// FileTrait trait: {
//     get_file_name(&self) -> string
//     get_function(&self) -> Vec<implents function trait>
//     ... 
// }

// File: struct {file_name: string functions: Vec<implemnts function trait>} 
// python_file: File where functions is a Vec<python_function>
// rust_file: File where functions is a Vec<rust_function>


// FunctionTrait trait: {
//     fmt_with_context(&self, prev option<self type>, next option<self type>) -> io::Result<()>
//     ...
// }

// functiontrait is implemented by python_function and rust_function, and is not object safe.

use std::{fmt::{self, Display}};

pub fn fmt_with_context<T: FunctionTrait + Display>(current: T, prev: Option<T>, next: Option<T>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match (prev, next) {
        (Some(prev), Some(next)) => {
            if prev.get_total_lines() == current.get_total_lines() && next.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?
            } else if prev.get_total_lines() == current.get_total_lines()  {
                write!(f, "{}", current.get_body())?;
                write!(f, "{}", current.get_bottoms().join("\n"))?;
            
            } else if next.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_tops().join("\n"))?;
                write!(f, "{}", current.get_body())?;
            } else {
                write!(f, "{}", current)?;
            }
            
        },
        (Some(prev), None) => {
            if prev.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?
            } else {
                write!(f, "{}", current)?;
            }
        

        },
        (None, Some(next)) => {
            if next.get_total_lines() == current.get_total_lines() {
                write!(f, "{}", current.get_body())?
            } else {
                write!(f, "{}", current)?;
            }
        

        },
        (None, None) => {
            // print the function
            write!(f, "{}", current)?;
        }
    }
    Ok(())
}
use crate::languages::rust::RustFunction;
use crate::languages::python::PythonFunction;
pub trait FunctionTrait: fmt::Debug + fmt::Display {
    fn get_tops(&self) -> Vec<String>;
    fn get_lines(&self) -> (usize, usize);
    fn get_total_lines(&self) -> (usize, usize);
    fn get_name(&self) -> String;
    fn get_bottoms(&self) -> Vec<String>;
    fn get_body(&self) -> String;

}

// functiontrait is not object safe, so we can't implement it for a trait object ie box<dyn FunctionTrait>
trait FileTrait {
    fn get_file_name(&self) -> String;
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>>;
}

impl FunctionTrait for RustFunction {
    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        match &self.block {
            Some(block) => {
                tops.push(block.top.clone());
            }
            None => {}
        }
        for parent in &self.function {
            tops.push(parent.top.clone());
        }
        tops
    }
    fn get_lines(&self) -> (usize, usize) {
        self.lines
    }
    fn get_total_lines(&self) -> (usize, usize) {
        match &self.block {
            Some(block) => (block.lines.0, self.lines.1),
            None => {
                let mut start = self.lines.0;
                let mut end = self.lines.1;
                for parent in &self.function {
                    if parent.lines.0 < start {
                        start = parent.lines.0;
                        end = parent.lines.1;
                    }
                }
                (start, end)
            }
        }

    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        match &self.block {
            Some(block) => {
                bottoms.push(block.bottom.clone());
            }
            None => {}
        }
        for parent in &self.function {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }
    fn get_body(&self) -> String {
        self.contents.clone()
    }
}

impl FunctionTrait for PythonFunction {
    fn get_tops(&self) -> Vec<String> {
        let mut tops = Vec::new();
        match &self.class {
            Some(block) => {
                tops.push(block.top.clone());
            }
            None => {}
        }
        for parent in &self.parent {
            tops.push(parent.top.clone());
        }
        tops
    }
    fn get_lines(&self) -> (usize, usize) {
        self.lines
    }
    fn get_total_lines(&self) -> (usize, usize) {
        match &self.class {
            Some(block) => (block.lines.0, self.lines.1),
            None => {
                let mut start = self.lines.0;
                let mut end = self.lines.1;
                for parent in &self.parent {
                    if parent.lines.0 < start {
                        start = parent.lines.0;
                        end = parent.lines.1;
                    }
                }
                (start, end)
            }
        }

    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_bottoms(&self) -> Vec<String> {
        let mut bottoms = Vec::new();
        match &self.class {
            Some(block) => {
                bottoms.push(block.bottom.clone());
            }
            None => {}
        }
        for parent in &self.parent {
            bottoms.push(parent.bottom.clone());
        }
        bottoms
    }
    fn get_body(&self) -> String {
        self.body.clone()
    }


}

pub struct File<T: FunctionTrait + Clone > {
    file_name: String,
    functions: Vec<T>,
}



// impl FileTrait for File<RustFunction> {
//     type FunctionType = RustFunction;
//     fn get_file_name(&self) -> String {
//         self.file_name.clone()
//     }
//     fn get_functions(&self) -> Vec<Self::FunctionType> {
//         self.functions.clone()
//     }
// }

impl FileTrait for File<PythonFunction> {
    // type FunctionType = PythonFunction;
    fn get_file_name(&self) -> String {
        self.file_name.clone()
    }
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>> {
        self.functions.clone().iter().cloned().map(|x| Box::new(x) as Box<dyn FunctionTrait>).collect()
    }
}

impl FileTrait for File<RustFunction> {
    // type FunctionType = RustFunction;
    fn get_file_name(&self) -> String {
        self.file_name.clone()
    }
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>> {
        self.functions.clone().iter().cloned().map(|x| Box::new(x) as Box<dyn FunctionTrait>).collect()
    }
}

impl From<RustFunction> for Box<dyn FunctionTrait> {
    fn from(f: RustFunction) -> Self {
        Box::new(f)
    }
}

pub enum FileType  {
    Rust(File<RustFunction>),
    Python(File<PythonFunction>),
}

impl FileTrait for FileType {

    fn get_file_name(&self) -> String {
        match self {
            FileType::Rust(file) => file.get_file_name(),
            FileType::Python(file) => file.get_file_name(),
        }
    }
    fn get_functions(&self) -> Vec<Box<dyn FunctionTrait>> {
        match self {
            FileType::Rust(file) => file.get_functions(),
            FileType::Python(file) => file.get_functions(),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::languages::{rust::find_function_in_commit, python};

    use super::*;

    #[test]
    fn it_works() {
        let fc = r#"
        fn main() {
            println!("Hello, world!");
        }
        "#;
        let rf = find_function_in_commit(fc, "main").unwrap();
        let f = File {
            file_name: "test.rs".to_string(),
            functions: rf,
        };
        let fts: FileType = FileType::Rust(f);

        println!("{}", fts.get_file_name());
        println!("{:?}", fts.get_functions());

        let fc = r#"def main():
    print("Hello, world!")

def main2():
    print("Hello, world!")
        "#;
        let rf = python::find_function_in_commit(fc, "main").unwrap();
        let f = File {
            file_name: "test.py".to_string(),
            functions: rf,
        };
        let ft: FileType = FileType::Python(f);
        let mut vec = Vec::new();
        vec.push(ft);
        vec.push(fts);



    }
}
