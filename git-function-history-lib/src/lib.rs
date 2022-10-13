#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
#![allow(
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::match_wildcard_for_single_variants,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cognitive_complexity,
    clippy::float_cmp,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::return_self_not_must_use
)]
pub mod languages;

/// Different types that can extracted from the result of `get_function_history`.
pub mod types;

use languages::{rust, LanguageFilter};

use languages::{PythonFile, RustFile};
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
pub use types::FileType;

#[cfg(feature = "c_lang")]
use languages::CFile;

use std::{error::Error, process::Command};
pub use types::{Commit, FunctionHistory};

pub use crate::languages::Language;

/// Different filetypes that can be used to ease the process of finding functions using `get_function_history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFilterType {
    /// When you have a absolute path to a file.
    Absolute(String),
    /// When you have a relative path to a file and or want to find look in all files match a name.
    Relative(String),
    /// When you want to filter only files in a specific directory
    Directory(String),
    /// When you don't know the path to a file.
    None,
}

// TODO: Add support for filtering by generic parameters, lifetimes, and return types.
/// This is filter enum is used when you want to lookup a function with the filter of filter a previous lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// When you want to filter by a commit hash.
    CommitHash(String),
    /// When you want to filter by a specific date (in rfc2822 format).
    Date(String),
    /// When you want to filter from one ate to another date (both in rfc2822 format).
    DateRange(String, String),
    /// When you have a absolute path to a file.
    FileAbsolute(String),
    /// When you have a relative path to a file and or want to find look in all files match a name.
    FileRelative(String),
    /// When you want to filter only files in a specific directory
    Directory(String),
    /// when you want to filter by function that are in between specific lines
    FunctionInLines(usize, usize),
    /// when you want to filter by a any commit author name that contains a specific string
    Author(String),
    /// when you want to filter by a any commit author email that contains a specific string
    AuthorEmail(String),
    // when you want to filter by a a commit message that contains a specific string
    Message(String),
    /// when you want to filter by proggramming language filter
    PLFilter(LanguageFilter),
    /// When you want to filter by nothing.
    None,
}

/// Valid filters are: `Filter::CommitId`, `Filter::Date`, `Filter::DateRange`.
///
/// Checks if git is installed if its not it will error out with `git is not installed`.
/// <br>
/// It then goes and creates a git log command based on the filters that you pass in.
/// <br>
/// Then it matches on the filetype, if its not none it will check that the file ends with .rs if not it will error out with `file is not a rust file`.
/// <br>
/// If its an absolute it will only for a file matching the exact path from te root of the repo.
/// <br>
/// If its a relative it will look for a that ends with the name of the file.
/// <br>
/// If its none it will look for all files in the repo that end in .rs.
/// Note: using `FilteType::None` will take a long time to run (especially if you no filters).
/// <br>
/// It will then go through the file and find all the functions and blocks in the file.
/// <br>
/// It will then go through the functions and find the ones that match the name also getting the blocks that enclose that function.
/// <br>
/// It will then return a `FunctionHistory` struct with all the commits with files that have functions that match the name.
/// <br>
/// If no histoy is is available it will error out with `no history found`.
///
/// # examples
///
/// ```
/// use git_function_history::{get_function_history, Filter, FileFilterType, Language};
/// let t = get_function_history("empty_test", &FileFilterType::Absolute("src/test_functions.rs".to_string()), &Filter::None, &Language::Rust).unwrap();
/// ```
#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
pub fn get_function_history(
    name: &str,
    file: &FileFilterType,
    filter: &Filter,
    langs: &languages::Language,
) -> Result<FunctionHistory, Box<dyn Error + Send + Sync>> {
    // chack if name is empty
    if name.is_empty() {
        Err("function name is empty")?;
    }
    // check if git is installed
    Command::new("git").arg("--version").output()?;
    // get the commit hitory
    let mut command = Command::new("git");
    command.arg("log");
    command.arg("--pretty=%H;%aD;%aN;%aE;%s");
    command.arg("--date=rfc2822");
    match filter {
        Filter::CommitHash(hash) => {
            command.arg(hash);
            command.arg("-n");
            command.arg("1");
        }
        Filter::Date(date) => {
            command.arg("--since");
            command.arg(date);
            command.arg("--max-count=1");
        }
        Filter::DateRange(start, end) => {
            command.arg("--since");
            command.arg(start);
            command.arg("--until");
            command.arg(end);
        }
        Filter::Author(author) => {
            command.arg("--author");
            command.arg(author);
        }
        Filter::AuthorEmail(email) => {
            command.arg("--author");
            command.arg(email);
        }
        Filter::Message(message) => {
            command.arg("--grep");
            command.arg(message);
        }
        Filter::None => {}
        _ => {
            Err("filter not supported")?;
        }
    }
    let output = command.output()?;
    if !output.stderr.is_empty() {
        return Err(String::from_utf8(output.stderr)?)?;
    }
    let stdout = String::from_utf8(output.stdout)?;
    let commits = stdout
        .lines()
        .map(|line| {
            let mut parts = line.split(';');
            let id = parts
                .next()
                .unwrap_to_error("no id found in git command output");
            let date = parts
                .next()
                .unwrap_to_error("date is missing from git command output");
            let author = parts
                .next()
                .unwrap_to_error("author is missing from git command output");
            let email = parts
                .next()
                .unwrap_to_error("email is missing from git command output");
            let message = parts
                .next()
                .unwrap_to_error("message is missing from git command output");
            Ok((id?, date?, author?, email?, message?))
        })
        .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?;

    let mut file_history = FunctionHistory::new(String::from(name), Vec::new());
    let err = "no history found".to_string();
    // check if file is a rust file
    if let FileFilterType::Absolute(path) | FileFilterType::Relative(path) = &file {
        match langs {
            #[cfg(feature = "c_lang")]
            Language::C => {
                if !path.ends_with(".c") && !path.ends_with(".h") {
                    Err(format!("file is not a c file: {}", path))?;
                }
            }
            Language::Python => {
                if !path.ends_with(".py") {
                    Err(format!("file is not a python file: {}", path))?;
                }
            }
            Language::Rust => {
                if !path.ends_with(".rs") {
                    Err(format!("file is not a rust file: {}", path))?;
                }
            }
            Language::All => {
                if !path.ends_with(".rs")
                    || !path.ends_with(".py")
                    || !path.ends_with(".c")
                    || !path.ends_with(".h")
                {
                    Err(format!("file is not supported: {}", path))?;
                }
            }
        }
    }
    #[cfg(feature = "parallel")]
    let t = commits.par_iter();
    #[cfg(not(feature = "parallel"))]
    let t = commits.iter();
    file_history.commit_history = t
        .filter_map(|commit| match &file {
            FileFilterType::Absolute(_)
            | FileFilterType::Relative(_)
            | FileFilterType::None
            | FileFilterType::Directory(_) => {
                match find_function_in_commit_with_filetype(commit.0, name, file, *langs) {
                    Ok(contents) => Some(Commit::new(
                        commit.0.to_string(),
                        contents,
                        commit.1,
                        commit.2.to_string(),
                        commit.3.to_string(),
                        commit.4.to_string(),
                    )),
                    Err(_) => None,
                }
            }
        })
        .collect();
    if file_history.commit_history.is_empty() {
        return Err(err)?;
    }
    Ok(file_history)
}

/// used for the `get_function_history` macro internally (you don't have to touch this)
pub struct MacroOpts<'a> {
    pub name: &'a str,
    pub file: FileFilterType,
    pub filter: Filter,
    pub language: Language,
}

impl Default for MacroOpts<'_> {
    fn default() -> Self {
        Self {
            name: "",
            file: FileFilterType::None,
            filter: Filter::None,
            language: Language::All,
        }
    }
}

/// macro to get the history of a function
/// wrapper around the `get_function_history` function
///
/// # examples
/// ```rust
/// use git_function_history::{get_function_history, languages::Language, Filter, FileFilterType};
/// git_function_history::get_function_history!(name = "main", file = FileFilterType::Relative("src/main.rs".to_string()), filter = Filter::None, language = Language::Rust);
/// ```
///
/// everything is optional but the name, and in no particular order
///
/// ```rust
/// use git_function_history::{get_function_history, FileFilterType};
/// git_function_history::get_function_history!(name = "main", file = FileFilterType::Relative("src/main.rs".to_string()));
/// ```
///
/// ```rust
///
/// use git_function_history::{get_function_history, Filter, FileFilterType};
/// git_function_history::get_function_history!(name = "main", filter = Filter::None, file = FileFilterType::Relative("src/main.rs".to_string()));
/// ```
///
/// Default values are:
///
/// - file: `FileFilterType::None`
/// - filter: `Filter::None`
/// - language: `Language::All`
#[macro_export]
macro_rules! get_function_history {
    ($($variant:ident = $value:expr),*) => {{
        let mut opts = $crate::MacroOpts::default();
        $(
            opts.$variant = $value;
        )*
        get_function_history(
            &opts.name,
            &opts.file,
            &opts.filter,
            &opts.language
        )
    }};
}

/// List all the commits date in the git history (in rfc2822 format).
pub fn get_git_dates() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["log", "--pretty=%aD", "--date", "rfc2822"])
        .output()?;
    let output = String::from_utf8(output.stdout)?;
    let output = output
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    Ok(output)
}

/// List all the commit hashes in the git history.
pub fn get_git_commit_hashes() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git").args(["log", "--pretty=%H"]).output()?;
    let output = String::from_utf8(output.stdout)?;
    let output = output
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    Ok(output)
}

fn find_file_in_commit(commit: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
    let commit_history = Command::new("git")
        .args(format!("show {}:{}", commit, file_path).split(' '))
        .output()?;
    if !commit_history.stderr.is_empty() {
        Err(String::from_utf8_lossy(&commit_history.stderr))?;
    }
    Ok(String::from_utf8_lossy(&commit_history.stdout).to_string())
}

fn find_function_in_commit_with_filetype(
    commit: &str,
    name: &str,
    filetype: &FileFilterType,
    langs: Language,
) -> Result<Vec<FileType>, Box<dyn Error>> {
    // get a list of all the files in the repository
    let mut files = Vec::new();
    let command = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "--full-tree", commit])
        .output()?;
    if !command.stderr.is_empty() {
        Err(String::from_utf8_lossy(&command.stderr))?;
    }
    let file_list = String::from_utf8_lossy(&command.stdout).to_string();
    for file in file_list.split('\n') {
        match filetype {
            FileFilterType::Relative(ref path) => {
                if file.ends_with(path) {
                    files.push(file);
                }
            }
            FileFilterType::Absolute(ref path) => {
                if file == path {
                    files.push(file);
                }
            }
            FileFilterType::Directory(ref path) => {
                if path.contains(path) {
                    files.push(file);
                }
            }
            FileFilterType::None => {
                match langs {
                    #[cfg(feature = "c_lang")]
                    Language::C => {
                        if file.ends_with(".c") || file.ends_with(".h") {
                            files.push(file);
                        }
                    }
                    Language::Python => {
                        if file.ends_with(".py") {
                            files.push(file);
                        }
                    }
                    Language::Rust => {
                        if file.ends_with(".rs") {
                            files.push(file);
                        }
                    }
                    Language::All => {
                        if file.ends_with(".rs")
                            || file.ends_with(".py")
                            // TODO: use cfg!() macro to check if c_lang is enabled
                            || file.ends_with(".c")
                            || file.ends_with(".h")
                        {
                            files.push(file);
                        }
                    }
                }
            }
        }
    }
    let err = "no function found".to_string();
    #[cfg(feature = "parellel")]
    let t = files.par_iter();
    #[cfg(not(feature = "parellel"))]
    let t = files.iter();
    let returns: Vec<FileType> = t
        .filter_map(|file| find_function_in_file_with_commit(commit, file, name, langs).ok())
        .collect();
    if returns.is_empty() {
        Err(err)?;
    }
    Ok(returns)
}

fn find_function_in_file_with_commit(
    commit: &str,
    file_path: &str,
    name: &str,
    langs: Language,
) -> Result<FileType, Box<dyn Error>> {
    let fc = find_file_in_commit(commit, file_path)?;
    match langs {
        Language::Rust => {
            let functions = rust::find_function_in_file(&fc, name)?;
            Ok(FileType::Rust(RustFile::new(
                file_path.to_string(),
                functions,
            )))
        }
        #[cfg(feature = "c_lang")]
        Language::C => {
            let functions = languages::c::find_function_in_file(&fc, name)?;
            Ok(FileType::C(CFile::new(file_path.to_string(), functions)))
        }
        Language::Python => {
            let functions = languages::python::find_function_in_file(&fc, name)?;
            Ok(FileType::Python(PythonFile::new(
                file_path.to_string(),
                functions,
            )))
        }
        Language::All => match file_path.split('.').last() {
            Some("rs") => {
                let functions = rust::find_function_in_file(&fc, name)?;
                Ok(FileType::Rust(RustFile::new(
                    file_path.to_string(),
                    functions,
                )))
            }
            #[cfg(feature = "c_lang")]
            Some("c" | "h") => {
                let functions = languages::c::find_function_in_file(&fc, name)?;
                Ok(FileType::C(CFile::new(file_path.to_string(), functions)))
            }
            Some("py") => {
                let functions = languages::python::find_function_in_file(&fc, name)?;
                Ok(FileType::Python(PythonFile::new(
                    file_path.to_string(),
                    functions,
                )))
            }
            _ => Err("unknown file type")?,
        },
    }
}

trait UnwrapToError<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>>;
}

impl<T> UnwrapToError<T> for Option<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        match self {
            Some(val) => Ok(val),
            None => Err(message.to_string().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::languages::FileTrait;

    use super::*;
    #[test]
    fn found_function() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Relative("src/test_functions.rs".to_string()),
            &Filter::None,
            &languages::Language::Rust,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{}", functions);
            }
            Err(e) => println!("{}", e),
        }
        assert!(output.is_ok());
    }
    #[test]
    fn git_installed() {
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Absolute("src/test_functions.rs".to_string()),
            &Filter::None,
            &languages::Language::Rust,
        );
        // assert that err is "not git is not installed"
        if output.is_err() {
            assert_ne!(output.unwrap_err().to_string(), "git is not installed");
        }
    }

    #[test]
    fn not_found() {
        let output = get_function_history(
            "Not_a_function",
            &FileFilterType::Absolute("src/test_functions.rs".to_string()),
            &Filter::None,
            &languages::Language::Rust,
        );
        match &output {
            Ok(output) => println!("{}", output),
            Err(error) => println!("{}", error),
        }
        assert!(output.is_err());
    }

    #[test]
    fn not_rust_file() {
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Absolute("src/test_functions.txt".to_string()),
            &Filter::None,
            &languages::Language::Rust,
        );
        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("file is not a rust file"));
    }
    #[test]
    fn test_date() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::None,
            &Filter::DateRange(
                "27 Sep 2022 11:27:23 -0400".to_owned(),
                "04 Oct 2022 23:45:52 +0000".to_owned(),
            ),
            &languages::Language::Rust,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{}", functions);
            }
            Err(e) => println!("-{}-", e),
        }
        assert!(output.is_ok());
    }

    #[test]
    fn expensive_tes() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::None,
            &Filter::None,
            &languages::Language::All,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{}", functions);
                functions.get_commit().files.iter().for_each(|file| {
                    println!("{}", file);
                });
            }
            Err(e) => println!("{}", e),
        }
        assert!(output.is_ok());
    }

    #[test]
    fn python() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Relative("src/test_functions.py".to_string()),
            &Filter::DateRange(
                "03 Oct 2022 11:27:23 -0400".to_owned(),
                "04 Oct 2022 23:45:52 +0000".to_owned(),
            ),
            &languages::Language::Python,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{}", functions);
            }
            Err(e) => println!("{}", e),
        }
        assert!(output.is_ok());
        let output = output.unwrap();
        let commit = output.get_commit();
        let file = commit.get_file();
        let _functions = file.get_functions();
    }

    #[test]
    #[cfg(feature = "c_lang")]
    fn c_lang() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Relative("src/test_functions.c".to_string()),
            &Filter::DateRange(
                "03 Oct 2022 11:27:23 -0400".to_owned(),
                "05 Oct 2022 23:45:52 +0000".to_owned(),
            ),
            &languages::Language::C,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{}", functions);
            }
            Err(e) => println!("{}", e),
        }
        assert!(output.is_ok());
    }

    #[test]
    fn parse_commit() {
        let commit_hash = "d098bba8be70106060f7250b80add703b7673d0e";
        let now = Utc::now();
        let t = find_function_in_commit_with_filetype(
            commit_hash,
            "empty_test",
            &FileFilterType::None,
            languages::Language::All,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        assert!(t.is_ok());
    }
}
