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

use std::fmt::Write;
use std::{process::Command, error::Error};

#[derive(Debug)]
pub struct Commit {
    pub id: String,
    pub contents: String,
    pub date: String,
}

impl Commit {
    const fn new(id: String, contents: String, date: String) -> Self {
        Self {
            id,
            contents,
            date,
        }
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
pub fn get_function(name: &str, file_path: &str) -> Result<Vec<Commit>, Box<dyn Error>> {
    // check if git is installed
    Command::new("git")
        .arg("--version")
        .output().expect("git is not installed");
    // get the commit hitory
    let commits = Command::new("git")
        .args(r#"log --pretty=%H,%ad"#.split(' '))
        .output()?;
    // if the stderr is not empty return the stderr
    if !commits.stderr.is_empty() {
        Err(String::from_utf8_lossy(&commits.stderr))?;
    }
    let mut file_history: Vec<Commit> = Vec::new();
    for commit in String::from_utf8_lossy(&commits.stdout).split('\n') {
        let commit = commit.split(',').collect::<Vec<&str>>();
        if commit.len() == 2 {
            match find_function_in_commit(commit[0], file_path, name) {
                Ok(contents) => {
                    file_history.push(Commit::new(
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
    if file_history.is_empty() {
        Err(String::from("No history found"))?;
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

fn find_function_in_commit(commit: &str, file_path: &str, name: &str) -> Result<String, Box<dyn Error>> {
    let file_contents = find_file_in_commit(commit, file_path)?;
    let file_contents = file_contents.split('\n');
    // TODO: dont hard code how functions are declared
    // for 2 reasons 1. every language has different ways of declaring functions 2. there could be more space / new line between the function keyword and the function name
    let fn_name = format!("fn {}", name);
    let mut found = false;
    let mut contents = String::new();
    let mut brace_count = 0;
    for (i, line) in file_contents.into_iter().enumerate() {
        // for each line check if the line contains the function name set found to true and add the i: line to the contents
        if line.contains(&fn_name) {
            found = true;
            writeln!(contents,"{i}: {}\n", line) ?;
            // split line line on the fn name
            match line.split_once(&fn_name){
                Some(line) => 
                {for lines in line.1.chars() {
                    if lines == '{' {
                        brace_count += 1;
                    } else if lines == '}' {
                        brace_count -= 1;
                    }
                }; continue },
                None => continue,
            };}
            // loop through the line and check if the line contains { or } and add the line to the contents
            for lines in line.chars() {
                if lines == '{' {
                    brace_count += 1;
                } else if lines == '}' {
                    brace_count -= 1;
                }
            } 
            if found {
                writeln!(contents,"{i}: {}\n", line) ?;
            }

            if brace_count == 0 && found {
                found = false;
            }
        
        
    }
    if contents.is_empty() {
        return Err(String::from("Function not found"))?;
    }
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn found_function() {
        let output = get_function("empty_test", "src/test_functions.rs");
        // check if output ik ok and not err
        assert!(output.is_ok());
        let output = output.unwrap();
        assert!(output.last().unwrap().date == "Tue Aug 9 13:02:28 2022 -0400");
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