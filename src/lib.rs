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
    clippy::missing_errors_doc
)]

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::fs::File;
use std::{error::Error, process::Command};

// read languages.json and parse the json to a const/static
lazy_static! {
    #[derive(Debug)]
    // this is for when we support multiple languages
    pub static ref LANGUAGES: Value = serde_json::from_reader(File::open(&"languages.json").unwrap()).unwrap();
    pub static ref CAPTURE_IN_QUOTE: Regex = Regex::new(r#"(["|'](?:\\["|']|[^"|'])*['|"])"#).unwrap();
    pub static ref CAPTURE_IN_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();
    pub static ref CAPTURE_MULTI_LINE_COMMENT: Regex = Regex::new(r#"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/"#).unwrap();
}
#[derive(Debug)]
pub struct Commit {
    pub id: String,
    pub contents: String,
    pub date: String,
}

impl Commit {
    const fn new(id: String, contents: String, date: String) -> Self {
        Self { id, contents, date }
    }
}

#[derive(Debug)]
pub struct FunctionHistory {
    pub name: String,
    pub history: Vec<Commit>,
}

impl FunctionHistory {
    pub fn get_by_commit_id(&self, id: &str) -> Option<&Commit> {
        self.history.iter().find(|c| c.id == id)
    }

    pub fn get_by_date(&self, date: &str) -> Option<&Commit> {
        self.history.iter().find(|c| c.date == date)
    }

    pub fn get_date_range(&self, start: &str, end: &str) -> Vec<&Commit> {
        // TODO: import chrono and use it to compare dates
        todo!("get_date_range({}-{})", start, end);
    }
}

/// Checks if git is installed if its not it will error out with `git is not installed`.
/// <br>
/// If not it will get all the commits along with the date.
/// <br>
/// It the creates a vector of `Commit` structs.
/// <br>
/// it goes the command output and splits it into the commit id, and date.
/// <br>
/// Using the `find_funtions_in_commit` it will find all the functions matching the name in the commit.
/// <br>
/// It will then create a new `Commit` struct with the id, date, and the the functions.
/// <br>
/// It will then return the vector of `Commit` structs if contents of any of the commits is not empty.
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
    };
    for commit in String::from_utf8_lossy(&commits.stdout).split('\n') {
        let commit = commit.split(',').collect::<Vec<&str>>();
        if commit.len() == 2 {
            match find_function_in_commit(commit[0], file_path, name) {
                Ok(contents) => {
                    file_history.history.push(Commit::new(
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
) -> Result<String, Box<dyn Error>> {
    let file_contents = find_file_in_commit(commit, file_path)?;
    let mut contents: String = "".to_string();
    let funtion_keyword = "fn".to_string();
    // create a regex to that finds the the function keyword followed by any amount of whitespace followed by the function name that returns the index after the function name
    let fn_regex = Regex::new(format!(r"{}[\s]*{}", funtion_keyword, name).as_str()).unwrap();
    let points = turn_three_vecs_into_one(
        get_points_from_regex(&CAPTURE_IN_QUOTE, &file_contents),
        get_points_from_regex(&CAPTURE_MULTI_LINE_COMMENT, &file_contents),
        get_points_from_regex(&CAPTURE_IN_COMMENT, &file_contents),
    );
    let mut function_range = Vec::new();
    fn_regex.find_iter(&file_contents).for_each(|m| {
        match get_body(&file_contents, &points, m.start()) {
            t if t != 0 => {
                function_range.push((m.start(), t));
            }
            _ => {}
        }
    });
    for (start, end) in function_range {
        if !contents.is_empty() {
            contents.push_str("\n...\n");
        }
        // TODO: if function dstarts in middle of a line capture from the start of the line
        contents += &file_contents[start..end];
    }
    if contents.is_empty() {
        return Err(String::from("Function not found"))?;
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

fn turn_three_vecs_into_one(
    vec1: Vec<(usize, usize)>,
    vec2: Vec<(usize, usize)>,
    vec3: Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut points: Vec<(usize, usize)> = Vec::new();
    for (start, end) in vec1 {
        points.push((start, end));
    }
    for (start, end) in vec2 {
        points.push((start, end));
    }
    for (start, end) in vec3 {
        points.push((start, end));
    }
    points
}

fn get_body(contents: &str, points: &[(usize, usize)], start_point: usize) -> usize {
    let mut brace_count = 0usize;
    for (index, char) in contents.chars().enumerate() {
        if index < start_point {
            continue;
        }
        if points
            .iter()
            .any(|&(start, end)| index >= start && index < end)
        {
            continue;
        }
        if char == '{' {
            brace_count += 1;
        } else if char == '}' {
            brace_count -= 1;
            if brace_count == 0 {
                return index + 1;
            }
        } else if char == ';' {
            return index+1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn found_function() {
        let output = get_function("empty_test", "src/test_functions.rs");
        assert!(output.is_ok());
        let output = output.unwrap();
        // assert!(output.last().unwrap().date == "Tue Aug 9 13:02:28 2022 -0400");
        for i in output.history {
            println!("{}\n{}", i.date, i.contents);
        }
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
