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

pub mod things;
use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;
use std::{error::Error, process::Command};
pub use things::{Block, BlockType, CommitFunctions, File, Function, FunctionHistory};
use things::{FunctionBlock, InternalBlock, InternalFunctions, Points};

// read languages.json and parse the json to a const/static
lazy_static! {
    #[derive(Debug)]
    // this is for when we support multiple languages
    pub (crate) static ref CAPTURE_FUNCTION: Regex = Regex::new(r#".*\bfn\s*(?P<name>[^\s<>]+)(?P<lifetime><[^<>]+>)?\s*\("#).expect("failed to compile regex");
    // this regex look for string chars and comments
    pub (crate) static ref CAPTURE_NOT_NEEDED: FancyRegex = FancyRegex::new(r#"(["](?:\\["]|[^"])*["])|(//.*)|(/\*[^*]*\*+(?:[^/*][^*]*\*+)*/)|(['][^\\'][']|['](?:\\(?:'|x[[:xdigit:]]{2}|u\{[[:xdigit:]]{1,6}\}|n|t|r)|\\\\)['])|(r(?P<hashes>[#]*)".*?"\k<hashes>)"#).expect("failed to compile regex");
    pub (crate) static ref CAPTURE_BLOCKS: Regex = Regex::new(r#"(.*\bimpl\s*(?P<lifetime_impl><[^<>]+>)?\s*(?P<name_impl>[^\s<>]+)\s*(<[^<>]+>)?\s*(?P<for>for\s*(?P<for_type>[^\s<>]+)\s*(?P<for_lifetime><[^<>]+>)?)?\s*(?P<wher_impl>where*[^{]+)?\{)|(.*\btrait\s+(?P<name_trait>[^\s<>]+)\s*(?P<lifetime_trait><[^<>]+>)?\s*(?P<wher_trait>where[^{]+)?\{)|(.*\bextern\s*(?P<extern>".+")?\s*\{)"#).expect("failed to compile regex");
}
#[derive(Debug, Clone)]
pub enum FileType {
    /// When you have a absolute path to a file.
    Absolute(String),
    /// When you have a relative path to a file and or want to find look in all files match a name.
    Relative(String),
    /// When you don't know the path to a file.
    None,
}

/// This is filter enum is used when you only want to lookup a function with the filter
/// it is different from the from the all the filters in the things module, because those filters are after the fact,
/// and require that you already found all the functions in the file. Making using this filter most probably faster.
#[derive(Debug, Clone)]
pub enum Filter {
    /// When you want to filter by a commit hash.
    CommitId(String),
    /// When you want to filter by a specific date (in rfc2822 format).
    Date(String),
    /// When you want to filter from one ate to another date (both in rfc2822 format).
    DateRange(String, String),
    /// When you want to filter by nothing.
    None,
}

// TODO: document this
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
/// It will then go through the functions and find the ones that match the name als getting the blocks that enclose that function.
/// <br>
/// It will then return a `FunctionHistory` struct with all the commits with files that have functions that match the name.
/// <br>
/// If no histoy is is available it will error out with `no history found`.
///
/// # examples
///
/// ```
/// use git_function_history::{get_function_history, Filter, FileType};
/// let t = get_function_history("empty_test", FileType::Absolute("src/test_functions.rs"), Filter::None);
/// ```
#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
pub fn get_function_history(
    name: &str,
    file: FileType,
    filter: Filter,
) -> Result<FunctionHistory, Box<dyn Error>> {
    // check if git is installed
    Command::new("git")
        .arg("--version")
        .output()
        .expect("git is not installed");
    // get the commit hitory
    let mut command = Command::new("git");
    command.arg("log");
    command.arg("--pretty=%H;%aD");
    command.arg("--date=rfc2822");
    match filter {
        Filter::CommitId(id) => {
            command.arg(id);
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
        Filter::None => {}
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
            let id = parts.next().expect("no id found in git command output");
            let date = parts
                .next()
                .expect("date is missing from git command output");
            (id, date)
        })
        .collect::<Vec<_>>();
    let mut file_history = FunctionHistory::new(String::from(name), Vec::new());
    match file {
        FileType::Absolute(path) => {
            if !path.ends_with(".rs") {
                return Err("not a rust file")?;
            }
            for commit in commits {
                match find_function_in_commit(commit.0, &path, name) {
                    Ok(contents) => {
                        file_history.history.push(CommitFunctions::new(
                            commit.0.to_string(),
                            vec![File::new(path.to_string(), contents)],
                            commit.1,
                        ));
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        FileType::Relative(path) => {
            if !path.ends_with(".rs") {
                return Err("not a rust file")?;
            }
            for commit in commits {
                match find_function_in_commit_with_relative_path(commit.0, name, &path) {
                    Ok(contents) => {
                        file_history.history.push(CommitFunctions::new(
                            commit.0.to_string(),
                            contents,
                            commit.1,
                        ));
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        FileType::None => {
            for commit in commits {
                match find_function_in_commit_with_unkown_file(commit.0, name) {
                    Ok(contents) => {
                        file_history.history.push(CommitFunctions::new(
                            commit.0.to_string(),
                            contents,
                            commit.1,
                        ));
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
    }
    if file_history.history.is_empty() {
        Err("No history found")?;
    }
    Ok(file_history)
}

/// List all the commits date in the git history (in rfc2822 format).
pub fn get_git_dates() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(&["log", "--pretty=%aD", "--date", "rfc2822"])
        .output()?;
    let output = String::from_utf8(output.stdout)?;
    let output = output
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    Ok(output)
}

/// List all the commit hashes in the git history.
pub fn get_git_commits() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git").args(&["log", "--pretty=%H"]).output()?;
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
    let mut contents: String = "".to_string();
    // add line numbers to the file contents
    for (i, line) in file_contents.lines().enumerate() {
        writeln!(contents, "{}: {}", i + 1, line)?;
    }
    let file_contents = contents;
    let mut contents: Vec<Function> = Vec::new();
    let points = get_points_from_regex(&CAPTURE_NOT_NEEDED, &file_contents);
    let blank_content = blank_out_range(&file_contents, &points);
    let mut function_range = Vec::new();
    for cap in CAPTURE_FUNCTION.find_iter(&blank_content) {
        // get the function name
        match get_body(&blank_content, cap.end(), true) {
            t if t.0 != 0 => {
                let top_line: usize = file_contents[cap.start()..t.0]
                    .split_once(':')
                    .unwrap_to_error("line is not indexed with a line number please file an issue at https://github.com/mendelsshop/git_function_history/")?
                    .0
                    .parse()?;
                let bottom_line = match file_contents[cap.start()..t.0].rsplit_once('\n') {
                    Some(line) => line.1.split_once(':').unwrap_to_error("line is not indexed with a line number please file an issue at https://github.com/mendelsshop/git_function_history/")?.0.parse()?,
                    None => top_line,
                };
                function_range.push(InternalFunctions {
                    range: Points {
                        x: cap.start(),
                        y: t.0,
                    },
                    name: get_function_name(&blank_content[cap.start()..cap.end()]),
                    file_line: Points {
                        x: top_line,
                        y: bottom_line,
                    },
                });
            }
            _ => {
                continue;
            }
        }
    }
    let mut block_range = Vec::new();
    for cap in CAPTURE_BLOCKS.find_iter(&blank_content) {
        // get the function name
        match get_body(&blank_content, cap.end() - 1, false) {
            t if t.0 != 0 => {
                let top_line: usize = file_contents[cap.start()..t.0]
                    .split_once(':')
                    .unwrap_to_error("line is not indexed with a line number please file an issue at https://github.com/mendelsshop/git_function_history/")?
                    .0
                    .parse()?;
                let bottom_line = match file_contents[cap.start()..t.0].rsplit_once('\n') {
                    Some(line) => line.1.split_once(':').unwrap_to_error("line is not indexed with a line number please file an issue at https://github.com/mendelsshop/git_function_history/")?.0.parse()?,
                    None => top_line,
                };
                block_range.push(InternalBlock {
                    // range:
                    start: Points {
                        x: cap.start(),
                        y: cap.end(),
                    },
                    full: Points {
                        x: cap.start(),
                        y: t.0,
                    },
                    end: Points { x: t.1, y: t.0 },
                    types: match CAPTURE_BLOCKS.captures(&file_contents[cap.start()..cap.end()]) {
                        Some(types) => {
                            if types.name("extern").is_some() {
                                BlockType::Extern
                            } else if types.name("name_impl").is_some() {
                                BlockType::Impl
                            } else if types.name("name_trait").is_some() {
                                BlockType::Trait
                            } else {
                                BlockType::Unknown
                            }
                        }
                        None => BlockType::Unknown,
                    },
                    file_line: Points {
                        x: top_line,
                        y: bottom_line,
                    },
                });
            }
            _ => {
                continue;
            }
        }
    }
    for t in &function_range {
        if t.name != name {
            continue;
        }
        let mut function = Function {
            name: t.name.clone(),
            contents: String::new(),
            block: None,
            function: None,
            lines: (t.file_line.x, t.file_line.y),
        };
        // check if block is in range
        let current_block = block_range.iter().find(|x| t.range.in_other(&x.full));
        let function_ranges = Points {
            x: t.file_line.x,
            y: t.file_line.y,
        };
        function.function = match function_range
            .iter()
            .filter(|other| function_ranges.in_other(&other.file_line))
            .map(|fns| FunctionBlock {
                name: fns.name.clone(),
                top: file_contents
                    .lines()
                    .nth(fns.file_line.x - 1)
                    .unwrap_or_else(|| {
                        panic!(
                            "could not get line {} in file {} from commit: {}",
                            fns.file_line.y - 1,
                            file_path,
                            name
                        )
                    })
                    .to_string(),
                bottom: file_contents
                    .lines()
                    .nth(fns.file_line.y - 1)
                    .unwrap_or_else(|| {
                        panic!(
                            "could not get line {} in file {} from commit: {}",
                            fns.file_line.y - 1,
                            file_path,
                            name
                        )
                    })
                    .to_string(),
                lines: (fns.file_line.x, fns.file_line.y),
            })
            .collect::<Vec<FunctionBlock>>()
        {
            vec if vec.is_empty() => None,
            vec => Some(vec),
        };
        if let Some(block) = current_block {
            function.block = Some(Block {
                name: None,
                top: file_contents[block.start.x..block.start.y].to_string(),
                bottom: file_contents[block.end.x..block.end.y].to_string(),
                block_type: block.types,
                lines: (block.file_line.x, block.file_line.y),
            });
        };
        function.contents = file_contents[t.range.x..t.range.y].to_string();
        contents.push(function);
    }
    if contents.is_empty() {
        Err("No functions found")?;
    }
    Ok(contents)
}

fn get_points_from_regex(regex: &FancyRegex, file_contents: &str) -> Vec<(usize, usize)> {
    let mut points: Vec<(usize, usize)> = Vec::new();
    regex.find_iter(file_contents).for_each(|m| {
        points.push((
            m.as_ref().expect("regex did not work").start(),
            m.as_ref().expect("regex did not work").end(),
        ));
    });
    points
}

fn get_body(contents: &str, start_point: usize, semi_colon: bool) -> (usize, usize) {
    let mut last_newline = 0;
    let mut brace_count = 0;
    let mut found_end = 0;
    for (index, char) in contents.chars().enumerate() {
        if index < start_point {
            continue;
        }
        if found_end != 0 && char == '\n' {
            return (index, last_newline);
        }
        if char == '{' {
            brace_count += 1;
        } else if char == '}' {
            brace_count -= 1;
            if brace_count == 0 {
                found_end = index;
            }
        } else if char == ';' && brace_count == 0 && semi_colon {
            found_end = index;
        } else if char == '\n' {
            last_newline = index;
        }
    }
    (contents.len(), last_newline)
}

fn blank_out_range(contents: &str, ranges: &Vec<(usize, usize)>) -> String {
    let mut new_contents = contents.to_string();
    for (start, end) in ranges {
        new_contents.replace_range(start..end, &" ".repeat(end - start));
    }
    new_contents
}

fn get_function_name(mut function_header: &str) -> String {
    let mut name = String::new();
    function_header = function_header
        .split_once("fn ")
        .unwrap_or(("", function_header))
        .1;
    for char in function_header.chars() {
        if char == '(' || char == '<' || char.is_whitespace() {
            break;
        }
        name.push(char);
    }
    name
}

fn find_function_in_commit_with_unkown_file(
    commit: &str,
    name: &str,
) -> Result<Vec<File>, Box<dyn Error>> {
    // get a list of all the files in the repository
    let mut files = Vec::new();
    let command = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", commit])
        .output()?;
    if !command.stderr.is_empty() {
        Err(String::from_utf8_lossy(&command.stderr))?;
    }
    let file_list = String::from_utf8_lossy(&command.stdout).to_string();
    for file in file_list.split('\n') {
        if file.ends_with(".rs") {
            files.push(file.to_string());
        }
    }
    let mut returns = Vec::new();
    for file in files {
        match find_function_in_commit(commit, &file, name) {
            Ok(functions) => returns.push(File::new(file, functions)),
            Err(_) => continue,
        }
    }
    Ok(returns)
}

fn find_function_in_commit_with_relative_path(
    commit: &str,
    name: &str,
    relative_path: &str,
) -> Result<Vec<File>, Box<dyn Error>> {
    // get a list of all the files in the repository
    let mut files = Vec::new();
    let command = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", commit])
        .output()?;
    if !command.stderr.is_empty() {
        Err(String::from_utf8_lossy(&command.stderr))?;
    }
    let file_list = String::from_utf8_lossy(&command.stdout).to_string();
    for file in file_list.split('\n') {
        if file.ends_with(relative_path) {
            files.push(file.to_string());
        }
    }
    let mut returns = Vec::new();
    for file in files {
        match find_function_in_commit(commit, &file, name) {
            Ok(functions) => returns.push(File::new(file, functions)),
            Err(_) => continue,
        }
    }
    Ok(returns)
}

trait UwrapToError<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>>;
}

impl<T> UwrapToError<T> for Option<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>> {
        match self {
            Some(val) => Ok(val),
            None => Err(message.to_string().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn found_function() {
        let output = get_function_history(
            "empty_test",
            FileType::Absolute("src/test_functions.rs".to_string()),
            Filter::None,
        );
        match &output {
            Ok(functions) => {
                println!("{}", functions.history[0]);
            }
            Err(e) => println!("{}", e),
        }
        assert!(output.is_ok());
    }
    #[test]
    fn git_installed() {
        let output = get_function_history(
            "empty_test",
            FileType::Absolute("src/test_functions.rs".to_string()),
            Filter::None,
        );
        // assert that err is "not git is not installed"
        if output.is_err() {
            assert_ne!(output.unwrap_err().to_string(), "git is not installed");
        }
    }

    #[test]
    fn not_found_function() {
        let output = get_function_history(
            "Not_a_function",
            FileType::Absolute("src/test_functions.rs".to_string()),
            Filter::None,
        );
        assert!(output.is_err());
    }

    #[test]
    fn not_rust_file() {
        let output = get_function_history(
            "empty_test",
            FileType::Absolute("src/test_functions.txt".to_string()),
            Filter::None,
        );
        let path = std::env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        assert!(output.is_err());
        assert_eq!(output.unwrap_err().to_string(), "not a rust file");
    }
    #[test]
    fn test() {
        let output = get_function_history(
            "empty_test",
            FileType::None,
            Filter::DateRange(
                "17 Aug 2022 11:27:23 -0400".to_owned(),
                "19 Aug 2022 23:45:52 +0000".to_owned(),
            ),
        );
        match &output {
            Ok(functions) => {
                println!("{}", functions.history[0]);
            }
            Err(e) => println!("{}", e),
        }
        let path = std::env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        assert!(output.is_ok());
    }
}
