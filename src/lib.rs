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

use std::{process::Command, error::Error};

/// Checks if git is installed if its not it will error out with `git is not installed`.
/// <br>
/// If not it creates a string for the git command to be run `format!("log -L :{}:{}", name, file_path);`.
/// <br>
/// It runs the command and if the command's stderr is not empty it will return the stderr.
/// <br>
/// If not it returns the output of the command.
/// 
/// # example
/// 
/// ```
/// get_function("test_function", "src/test_functions.rs");
/// ```
pub fn get_function(name: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
    // check if git is installed
    Command::new("git")
        .arg("--version")
        .output().expect("git is not installed");
    // prnt the git log
    let git_cmd = format!("log -L :{}:{}", name, file_path);
    let output = Command::new("git")
        .args(git_cmd.split(' '))
        .output()?;
    let output_str = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    if !stderr.is_empty() {
        Err(stderr)?;
    }
    Ok(output_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn found_function() {
        let output = get_function("empty_test", "src/test_functions.rs");
        // check if output ik ok and not err
        assert!(output.is_ok());
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
        // assert that error is "git is not installed"
        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains("no match"));
    }
}