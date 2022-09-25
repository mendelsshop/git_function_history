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
/// Different types that can extracted from the result of `get_function_history`.
pub mod types;
use ra_ap_syntax::{
    ast::{self, HasDocComments, HasGenericParams, HasName},
    AstNode, SourceFile, SyntaxKind,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use std::{collections::HashMap, error::Error, process::Command};
pub use types::{
    Block, BlockType, CommitFunctions, File, Function, FunctionBlock, FunctionHistory,
};

/// Different filetypes that can be used to ease the process of finding functions using `get_function_history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
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
    // when you want to filter by function that are in a specific block (impl, trait, extern)
    FunctionInBlock(BlockType),
    // when you want to filter by function that are in between specific lines
    FunctionInLines(usize, usize),
    // when you want filter by a function that has a parent function of a specific name
    FunctionWithParent(String),
    /// when you want to filter by a any commit author name that contains a specific string
    Author(String),
    /// when you want to filter by a any commit author email that contains a specific string
    AuthorEmail(String),
    // when you want to filter by a a commit message that contains a specific string
    Message(String),
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
/// use git_function_history::{get_function_history, Filter, FileType};
/// let t = get_function_history("empty_test", FileType::Absolute("src/test_functions.rs".to_string()), Filter::None);
/// ```
#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
pub fn get_function_history(
    name: &str,
    file: &FileType,
    filter: Filter,
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
    if let FileType::Absolute(path) | FileType::Relative(path) = &file {
        if !path.ends_with(".rs") {
            Err("file is not a rust file")?;
        }
    }
    file_history.commit_history = commits
        .par_iter()
        .filter_map(|commit| {
            match &file {
                FileType::Absolute(path) => match find_function_in_commit(commit.0, path, name) {
                    Ok(contents) => Some(CommitFunctions::new(
                        commit.0.to_string(),
                        vec![File::new(path.to_string(), contents)],
                        commit.1,
                        commit.2.to_string(),
                        commit.3.to_string(),
                        commit.4.to_string(),
                    )),
                    Err(_) => None,
                },

                FileType::Relative(_) => {
                    match find_function_in_commit_with_filetype(commit.0, name, file) {
                        Ok(contents) => Some(CommitFunctions::new(
                            commit.0.to_string(),
                            contents,
                            commit.1,
                            commit.2.to_string(),
                            commit.3.to_string(),
                            commit.4.to_string(),
                        )),
                        Err(_) => {
                            // err = e.to_string();
                            None
                        }
                    }
                }

                FileType::None | FileType::Directory(_) => {
                    match find_function_in_commit_with_filetype(commit.0, name, file) {
                        Ok(contents) => Some(CommitFunctions::new(
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
            }
        })
        .collect();
    if file_history.commit_history.is_empty() {
        return Err(err)?;
    }
    Ok(file_history)
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

#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
fn find_function_in_commit(
    commit: &str,
    file_path: &str,
    name: &str,
) -> Result<Vec<Function>, Box<dyn Error>> {
    let file_contents = find_file_in_commit(commit, file_path)?;
    let mut functions = Vec::new();
    get_function_asts(name, &file_contents, &mut functions);
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
    let mut hist = Vec::new();
    for f in &functions {
        let stuff = get_stuff(f, &file_contents, &map);
        let generics = get_genrerics_and_lifetime(f);
        let mut parent = f.syntax().parent();
        let mut parent_fn: Vec<FunctionBlock> = Vec::new();
        let mut parent_block = None;
        while let Some(p) = parent.into_iter().next() {
            if p.kind() == SyntaxKind::SOURCE_FILE {
                break;
            }
            ast::Fn::cast(p.clone()).map_or_else(
                || {
                    if let Some(block) = ast::Impl::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, &file_contents, &map);
                        let generics = get_genrerics_and_lifetime(&block);
                        parent_block = Some(Block {
                            name: block.self_ty().map(|ty| ty.to_string()),
                            lifetime: generics.1,
                            generics: generics.0,
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Impl,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    } else if let Some(block) = ast::Trait::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, &file_contents, &map);
                        let generics = get_genrerics_and_lifetime(&block);
                        parent_block = Some(Block {
                            name: block.name().map(|ty| ty.to_string()),
                            lifetime: generics.1,
                            generics: generics.0,
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Trait,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    } else if let Some(block) = ast::ExternBlock::cast(p.clone()) {
                        let attr = get_doc_comments_and_attrs(&block);
                        let stuff = get_stuff(&block, &file_contents, &map);
                        parent_block = Some(Block {
                            name: block.abi().map(|ty| ty.to_string()),
                            lifetime: Vec::new(),
                            generics: Vec::new(),
                            top: stuff.1 .0,
                            bottom: stuff.1 .1,
                            block_type: BlockType::Extern,
                            lines: (stuff.0 .0, stuff.0 .1),
                            attributes: attr.1,
                            doc_comments: attr.0,
                        });
                    }
                },
                |function| {
                    let stuff = get_stuff(&function, &file_contents, &map);
                    let generics = get_genrerics_and_lifetime(&function);
                    let attr = get_doc_comments_and_attrs(&function);
                    parent_fn.push(FunctionBlock {
                        name: function.name().unwrap().to_string(),
                        lifetime: generics.1,
                        generics: generics.0,
                        top: stuff.1 .0,
                        bottom: stuff.1 .1,
                        lines: (stuff.0 .0, stuff.0 .1),
                        return_type: function.ret_type().map(|ty| ty.to_string()),
                        arguments: match function.param_list() {
                            Some(args) => args
                                .params()
                                .map(|arg| arg.to_string())
                                .collect::<Vec<String>>(),
                            None => Vec::new(),
                        },
                        attributes: attr.1,
                        doc_comments: attr.0,
                    });
                },
            );
            parent = p.parent();
        }
        let attr = get_doc_comments_and_attrs(f);
        let mut start = stuff.0 .0;
        let bb = match map[&start] {
            0 => 0,
            x => x + 1,
        };
        let contents: String = file_contents[bb..f.syntax().text_range().end().into()]
            .to_string()
            .lines()
            .map(|l| {
                start += 1;
                format!("{}: {}\n", start, l,)
            })
            .collect();
        let contents = contents.trim_end().to_string();
        let function = Function {
            name: f.name().unwrap().to_string(),
            contents,
            block: parent_block,
            function: parent_fn,
            return_type: f.ret_type().map(|ty| ty.to_string()),
            arguments: match f.param_list() {
                Some(args) => args
                    .params()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>(),
                None => Vec::new(),
            },
            lifetime: generics.1,
            generics: generics.0,
            lines: (stuff.0 .0, stuff.0 .1),
            attributes: attr.1,
            doc_comments: attr.0,
        };
        hist.push(function);
    }
    if hist.is_empty() {
        Err("no function found")?;
    }
    Ok(hist)
}

fn get_function_asts(name: &str, file: &str, functions: &mut Vec<ast::Fn>) {
    let parsed_file = SourceFile::parse(file).tree();
    parsed_file
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .filter(|function| function.name().unwrap().text() == name)
        .for_each(|function| functions.push(function));
}

fn get_stuff<T: AstNode>(
    block: &T,
    file: &str,
    map: &HashMap<usize, &usize>,
) -> ((usize, usize), (String, String), (usize, usize)) {
    let start = block.syntax().text_range().start();
    let end = block.syntax().text_range().end();
    // get the start and end lines
    let mut found_start_brace = 0;
    let mut end_line = 0;
    let mut starts = 0;
    let mut start_line = 0;
    // TODO: combine these loops
    for (i, line) in file.chars().enumerate() {
        if line == '\n' {
            if usize::from(start) < i {
                starts = i;
                break;
            }
            start_line += 1;
        }
    }
    for (i, line) in file.chars().enumerate() {
        if line == '\n' {
            if usize::from(end) < i {
                break;
            }
            end_line += 1;
        }
        if line == '{' && found_start_brace == 0 && usize::from(start) < i {
            found_start_brace = i;
        }
    }
    if found_start_brace == 0 {
        found_start_brace = usize::from(start);
    }
    let start = map[&start_line];
    let mut start_lines = start_line;
    let mut content: String = file[(*start)..=found_start_brace].to_string();
    if &content[..1] == "\n" {
        content = content[1..].to_string();
    }
    (
        (start_line, end_line),
        (
            content
                .lines()
                .map(|l| {
                    start_lines += 1;
                    format!("{}: {}\n", start_lines, l,)
                })
                .collect::<String>()
                .trim_end()
                .to_string(),
            format!(
                "\n{}: {}",
                end_line,
                file.lines()
                    .nth(if end_line == file.lines().count() - 1 {
                        end_line
                    } else {
                        end_line - 1
                    })
                    .unwrap_or("")
            ),
        ),
        (starts, end_line),
    )
}

fn get_genrerics_and_lifetime<T: HasGenericParams>(block: &T) -> (Vec<String>, Vec<String>) {
    match block.generic_param_list() {
        None => (vec![], vec![]),
        Some(gt) => (
            gt.generic_params()
                .map(|gt| gt.to_string())
                .collect::<Vec<String>>(),
            gt.lifetime_params()
                .map(|lt| lt.to_string())
                .collect::<Vec<String>>(),
        ),
    }
}

fn get_doc_comments_and_attrs<T: HasDocComments>(block: &T) -> (Vec<String>, Vec<String>) {
    (
        block
            .doc_comments()
            .map(|c| c.to_string())
            .collect::<Vec<String>>(),
        block
            .attrs()
            .map(|c| c.to_string())
            .collect::<Vec<String>>(),
    )
}

fn find_function_in_commit_with_filetype(
    commit: &str,
    name: &str,
    filetype: &FileType,
) -> Result<Vec<File>, Box<dyn Error>> {
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
            FileType::Relative(ref path) => {
                if file.ends_with(path) {
                    files.push(file);
                }
            }
            FileType::Directory(ref path) => {
                if path.contains(path) {
                    files.push(file);
                }
            }
            FileType::None => {
                if file.ends_with(".rs") {
                    files.push(file);
                }
            }
            _ => {}
        }
    }
    let err = "no function found".to_string();
    let returns: Vec<File> = files
        .par_iter()
        .filter_map(|file| match find_function_in_commit(commit, file, name) {
            Ok(functions) => Some(File::new((*file).to_string(), functions)),
            Err(_) => None,
        })
        .collect();
    if returns.is_empty() {
        Err(err)?;
    }
    Ok(returns)
}

trait UwrapToError<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>>;
}

impl<T> UwrapToError<T> for Option<T> {
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

    use super::*;
    #[test]
    fn found_function() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileType::Relative("src/test_functions.rs".to_string()),
            Filter::None,
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
            &FileType::Absolute("src/test_functions.rs".to_string()),
            Filter::None,
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
            &FileType::Absolute("src/test_functions.rs".to_string()),
            Filter::None,
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
            &FileType::Absolute("src/test_functions.txt".to_string()),
            Filter::None,
        );
        assert!(output.is_err());
        assert_eq!(output.unwrap_err().to_string(), "file is not a rust file");
    }
    #[test]
    fn test_date() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileType::None,
            Filter::DateRange(
                "17 Aug 2022 11:27:23 -0400".to_owned(),
                "19 Aug 2022 23:45:52 +0000".to_owned(),
            ),
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
        let output = get_function_history("empty_test", &FileType::None, Filter::None);
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
}
