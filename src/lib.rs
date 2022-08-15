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
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::fmt::Write;
use std::fs::File;
use std::{error::Error, process::Command};
pub use things::{Block, BlockType, CommitFunctions, Function, FunctionHistory};
use things::{InternalBlock, InternalFunctions, Points};

// read languages.json and parse the json to a const/static
lazy_static! {
    #[derive(Debug)]
    // this is for when we support multiple languages
    pub (crate)static ref LANGUAGES: Value = serde_json::from_reader(File::open(&"languages.json").unwrap()).unwrap();
    pub (crate) static ref CAPTURE_FUNCTION: Regex = Regex::new(r#".*\bfn\s*(?P<name>[^\s<>]+)(?P<lifetime><[^<>]+>)?\s*\("#).unwrap();
    // this regex look for string chars and comments
    pub (crate) static ref CAPTURE_NOT_NEEDED: Regex = Regex::new(r#"(["](?:\\["]|[^"])*["])|(//.*)|(/\*[^*]*\*+(?:[^/*][^*]*\*+)*/)|(['][^\\'][']|['](?:\\(?:'|x[[:xdigit:]]{2}|u\{[[:xdigit:]]{1,6}\}|n|t|r)|\\\\)['])"#).unwrap();
    pub (crate) static ref CAPTURE_BLOCKS: Regex = Regex::new(r#"(.*\bimpl\s*(?P<lifetime_impl><[^<>]+>)?\s*(?P<name_impl>[^\s<>]+)\s*(<[^<>]+>)?\s*(?P<for>for\s*(?P<for_type>[^\s<>]+)\s*(?P<for_lifetime><[^<>]+>)?)?\s*(?P<wher_impl>where*[^{]+)?\{)|(.*\btrait\s+(?P<name_trait>[^\s<>]+)\s*(?P<lifetime_trait><[^<>]+>)?\s*(?P<wher_trait>where[^{]+)?\{)|(.*\bextern\s*(?P<extern>".+")?\s*\{)"#).unwrap();
}

/// Checks if git is installed if its not it will error out with `git is not installed`.
/// <br>
/// If not it will get all the commits along with the date.
/// <br>
/// It the creates a vector of `CommitFunctions` structs.
/// <br>
/// it goes the command output and splits it into the commit id, and date.
/// <br>
/// Using the `find_funtions_in_commit` it will find all the functions matching the name in the commit.
/// <br>
/// It will then create a new `CommitFunctions` struct with the id, date, and the the functions.
/// <br>
/// It will then return the vector of `CommitFunctions` structs if contents of any of the commits is not empty.
/// <br>
/// If not it will error out with `no history found`.
///
/// # example
///
/// ```
/// use git_function_history::get_function;
/// let t = get_function("test_function", "src/test_functions.rs");
/// ```
pub fn get_function(name: &str, file_path: &str) -> Result<FunctionHistory, Box<dyn Error>> {
    // check if git is installed
    Command::new("git")
        .arg("--version")
        .output()
        .expect("git is not installed");
    // get the commit hitory
    let commits = Command::new("git")
        .args(r#"log --pretty=%H,%ad"#.split(' '))
        .output()?;
    // if the stderr is not empty return the stderr
    if !commits.stderr.is_empty() {
        Err(String::from_utf8_lossy(&commits.stderr))?;
    }
    let mut file_history = FunctionHistory {
        name: name.to_string(),
        history: Vec::new(),
        current_pos: 0
    };
    for commit in String::from_utf8_lossy(&commits.stdout).split('\n') {
        let commit = commit.split(',').collect::<Vec<&str>>();
        if commit.len() == 2 {
            match find_function_in_commit(commit[0], file_path, name) {
                Ok(contents) => {
                    file_history.history.push(CommitFunctions::new(
                        commit[0].to_string(),
                        contents,
                        commit[1].to_string(),
                    ));
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }
    // get the commit hitory
    // chck if the file_history is empty if it is return an error
    if file_history.history.is_empty() {
        Err("No history found")?;
    }
    Ok(file_history)
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
                function_range.push(InternalFunctions {
                    range: Points {
                        x: cap.start(),
                        y: t.0,
                    },
                    name: get_function_name(&blank_content[cap.start()..cap.end()]),
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
                });
            }
            _ => {
                continue;
            }
        }
    }
    for t in function_range {
        if t.name != name {
            continue;
        }
        let mut function = Function {
            name: t.name.clone(),
            contents: String::new(),
            block: None,
            function: None,
            lines: (0, 0),
        };
        // check if block is in range
        let current_block = block_range.iter().find(|x| t.range.in_other(&x.full));

        if let Some(block) = current_block {
            function.block = Some(Block {
                name: None,
                top: file_contents[block.start.x..block.start.y].to_string(),
                bottom: file_contents[block.end.x..block.end.y].to_string(),
                block_type: block.types,
            });
        };
        function.contents = file_contents[t.range.x..t.range.y].to_string();
        let top_line: usize = file_contents[t.range.x..t.range.y].split_once(':').unwrap().0.parse().unwrap();
        // println!("{}", top_line);
        let bottom_line = match file_contents[t.range.x..t.range.y].rsplit_once('\n') {
            Some(line) => {
                line.1.split_once(':').unwrap().0.parse().unwrap()},
            None => top_line,
        };
        function.lines = (top_line, bottom_line);
        contents.push(function);
    }
    if contents.is_empty() {
        Err("No functions found")?;
    }
    Ok(contents)
}

fn get_points_from_regex(regex: &Regex, file_contents: &str) -> Vec<(usize, usize)> {
    let mut points: Vec<(usize, usize)> = Vec::new();
    regex.find_iter(file_contents).for_each(|m| {
        points.push((m.start(), m.end()));
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
    function_header = function_header.split_once("fn ").unwrap().1;
    for char in function_header.chars() {
        if char == '(' || char == '<' || char.is_whitespace() {
            break;
        }
        name.push(char);
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn found_function() {
        let output = get_function("empty_test", "src/test_functions.rs");
        assert!(output.is_ok());
        let output = output.unwrap();
        println!("{}", output.history[0]);
    }
    #[test]
    fn git_installed() {
        let output = get_function("empty_test", "src/test_functions.rs");
        // assert that err is "not git is not installed"
        if output.is_err() {
            assert_ne!(output.unwrap_err().to_string(), "git is not installed");
        }
    }

    #[test]
    fn not_found_function() {
        let output = get_function("not_a_function", "src/test_functions.rs");
        assert!(output.is_err());
    }
}
