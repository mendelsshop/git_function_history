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
    clippy::return_self_not_must_use,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::too_many_lines
)]
/// code and function related language
pub mod languages;
/// Different types that can extracted from the result of `get_function_history`.
pub mod types;
macro_rules! get_item_from {
    ($oid:expr, $repo:expr, $typs:ident) => {
        git_repository::hash::ObjectId::from($oid)
            .attach(&$repo)
            .object()?
            .$typs()?
    };
}

macro_rules! get_item_from_oid_option {
    ($oid:expr, $repo:expr, $typs:ident) => {
        git_repository::hash::ObjectId::from($oid)
            .attach(&$repo)
            .object()
            .ok()?
            .$typs()
            .ok()
    };
}
#[cfg(feature = "cache")]
use cached::proc_macro::cached;
use chrono::{DateTime, NaiveDateTime, Utc};
use languages::{rust, LanguageFilter, PythonFile, RubyFile, RustFile};
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use git_repository::{objs, prelude::ObjectIdExt, ObjectId};
use std::{error::Error, ops::Sub};

// #[cfg(feature = "c_lang")]
// use languages::CFile;
#[cfg(feature = "unstable")]
use languages::GoFile;

pub use {
    languages::Language,
    types::{Commit, FileType, FunctionHistory},
};

/// Different filetypes that can be used to ease the process of finding functions using `get_function_history`.
/// path separator is `/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFilterType {
    /// When you have a absolute path to a file.
    Absolute(String),
    /// When you have a relative path to a file and or want to find look in all files match a name (aka ends_with).
    Relative(String),
    /// When you want to filter only files in a specific directory
    Directory(String),
    /// When you don't know the path to a file.
    None,
}

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
/// If its none it will look for all files in the repo that end in supported files (depends on features)
/// Note: using `FilteType::None` will take a long time to run (especially if you no filters).
/// <br>
/// It will then go through the file and find all the functions and blocks in the file.
/// <br>
/// It will then go through the functions and find the ones that match the name also getting the blocks that enclose that function.
/// <br>
/// It will then return a `FunctionHistory` struct with all the commits with files that have functions that match the name.
/// <br>
/// If no histoy is is available it will error out with `no history found`, and possibly a reason why.
///
/// # examples
///
/// ```
/// use git_function_history::{get_function_history, Filter, FileFilterType, Language};
/// let t = get_function_history("empty_test", &FileFilterType::Absolute("src/test_functions.rs".to_string()), &Filter::None, &Language::Rust).unwrap();
/// ```
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
    // if filter is date list all the dates and find the one that is closest to the date set that to closest_date and when using the first filter check if the date of the commit is equal to the closest_date
    let repo = git_repository::discover(".")?;
    let mut tips = vec![];
    let head = repo.head_commit()?;
    tips.push(head.id);
    let commit_iter = repo.rev_walk(tips);
    let commits = commit_iter
        .all()?
        .filter_map(|i| match i {
            Ok(i) => get_item_from_oid_option!(i, &repo, try_into_commit),
            Err(_) => None,
        })
        .collect::<Vec<_>>();
    // find the closest date by using get_git_dates_commits_oxide
    let closest_date = match filter {
        Filter::Date(date) => {
            let date = DateTime::parse_from_rfc2822(date)?.with_timezone(&Utc);
            let date_list = get_git_info()?;
            date_list
                .iter()
                .min_by_key(|elem| {
                    elem.date.sub(date).num_seconds().abs()
                    // elem.0.signed_duration_since(date)
                })
                .map(|elem| elem.hash.clone())
                .unwrap_to_error_sync("no commits found")?
        }
        Filter::Author(_)
        | Filter::AuthorEmail(_)
        | Filter::Message(_)
        | Filter::DateRange(..)
        | Filter::None
        | Filter::CommitHash(_) => String::new(),
        _ => Err("invalid filter")?,
    };
    match file {
        FileFilterType::Absolute(file) | FileFilterType::Relative(file) => {
            // vaildate that the file makes sense with language
            let is_supported = langs.get_file_endings().iter().any(|i| file.ends_with(i));
            if !is_supported {
                Err(format!("file {file} is not a {} file", langs.get_names()))?;
            }
        }
        FileFilterType::Directory(_) | FileFilterType::None => {}
    }
    let commits = commits
        .iter()
        .filter_map(|i| {
            let tree = i.tree().ok()?.id;
            let time = i.time().ok()?;
            let time = DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(time.seconds_since_unix_epoch.into(), 0),
                Utc,
            );
            let authorinfo = i.author().ok()?;
            let author = authorinfo.name.to_string();
            let email = authorinfo.email.to_string();
            let messages = i.message().ok()?;
            let mut message = messages.title.to_string();
            if let Some(i) = messages.body {
                message.push_str(i.to_string().as_str());
            }
            let commit = i.id().to_hex().to_string();
            let metadata = (message, commit, author, email, time);
            Some((tree, metadata))
        })
        .filter(|(_, metadata)| {
            match filter {
                Filter::CommitHash(hash) => *hash == metadata.1,
                Filter::Date(_) => metadata.1 == closest_date,
                Filter::DateRange(start, end) => {
                    // let date = metadata.4.seconds_since_unix_epoch;
                    let date = metadata.4;
                    let start = DateTime::parse_from_rfc2822(start)
                        .map(|i| i.with_timezone(&Utc))
                        .expect("failed to parse start date");
                    let end = DateTime::parse_from_rfc2822(end)
                        .map(|i| i.with_timezone(&Utc))
                        .expect("failed to parse end date");
                    start <= date && date <= end
                }
                Filter::Author(author) => *author == metadata.2,
                Filter::AuthorEmail(email) => *email == metadata.3,
                Filter::Message(message) => {
                    metadata.0.contains(message)
                        || message.contains(&metadata.0)
                        || message == &metadata.0
                }
                Filter::None => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "parallel")]
    let commits = commits.into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let commits = commits.iter();
    let commits = commits
        .filter_map(|i| {
            let tree = sender(i.0, name, *langs, file);
            match tree {
                Ok(tree) => {
                    if tree.is_empty() {
                        None?;
                    }
                    Some(Commit::new(
                        &i.1 .1,
                        tree,
                        &i.1 .4.to_rfc2822(),
                        &i.1 .2,
                        &i.1 .3,
                        &i.1 .0,
                    ))
                }
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>();
    if commits.is_empty() {
        Err("no history found")?;
    }
    let fh = FunctionHistory::new(name.to_string(), commits);
    Ok(fh)
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

fn sender(
    id: ObjectId,
    name: &str,
    langs: Language,
    file: &FileFilterType,
) -> Result<Vec<FileType>, Box<dyn std::error::Error>> {
    let repo = git_repository::discover(".")?;
    let object = repo.find_object(id)?;
    let tree = object.try_into_tree()?;
    traverse_tree(&tree, &repo, name, "", langs, file)
}

fn traverse_tree(
    tree: &git_repository::Tree<'_>,
    repo: &git_repository::Repository,
    name: &str,
    path: &str,
    langs: Language,
    filetype: &FileFilterType,
) -> Result<Vec<FileType>, Box<dyn std::error::Error>> {
    let treee_iter = tree.iter();
    let mut files: Vec<(String, String)> = Vec::new();
    let mut ret = Vec::new();
    for i in treee_iter {
        let i = i?;
        match &i.mode() {
            objs::tree::EntryMode::Tree => {
                let new = get_item_from!(i.oid(), &repo, try_into_tree);
                let path_new = format!("{path}/{}", i.filename());
                ret.extend(traverse_tree(&new, repo, name, &path_new, langs, filetype)?);
            }
            objs::tree::EntryMode::Blob => {
                let file = format!("{path}/{}", i.filename());
                match &filetype {
                    FileFilterType::Relative(ref path) => {
                        if !file.ends_with(path) {
                            continue;
                        }
                    }
                    FileFilterType::Absolute(ref path) => {
                        if &file == path {
                            continue;
                        }
                    }
                    FileFilterType::Directory(ref path) => {
                        if !file.contains(path) {
                            continue;
                        }
                    }
                    FileFilterType::None => match langs {
                        // #[cfg(feature = "c_lang")]
                        // Language::C => {
                        //     if file.ends_with(".c") || file.ends_with(".h") {
                        //         files.push(file);
                        //     }
                        // }
                        #[cfg(feature = "unstable")]
                        Language::Go => {
                            if !file.ends_with(".go") {
                                continue;
                            }
                        }
                        Language::Python => {
                            if !file.ends_with(".py") {
                                continue;
                            }
                        }
                        Language::Rust => {
                            if !file.ends_with(".rs") {
                                continue;
                            }
                        }
                        Language::Ruby => {
                            if !file.ends_with(".rb") {
                                continue;
                            }
                        }
                        Language::All => {
                            cfg_if::cfg_if! {
                                if #[cfg(feature = "c_lang")] {
                                    if !(file.ends_with(".c") || file.ends_with(".h") || !file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb")) {
                                        continue;
                                    }
                                }
                                else if #[cfg(feature = "unstable")] {
                                    if !(file.ends_with(".go")  || file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb")){
                                        continue
                                    }
                                }
                                else if #[cfg(all(feature = "unstable", feature = "c_lang"))] {
                                    if !(file.ends_with(".go") || file.ends_with(".c") || file.ends_with(".h") || file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb")) {
                                        continue;
                                    }
                                }
                                else {
                                    if !(file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb")) {
                                        continue;
                                    }
                                }

                            }
                        }
                    },
                }
                let obh = repo.find_object(i.oid())?;
                let objref = objs::ObjectRef::from_bytes(obh.kind, &obh.data)?;
                let blob = objref.into_blob();
                if let Some(blob) = blob {
                    files.push((file, String::from_utf8_lossy(blob.data).to_string()));
                }
            }
            _ => {}
        }
    }
    ret.extend(find_function_in_files_with_commit(
        files,
        name.to_string(),
        langs,
    ));

    Ok(ret)
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
            opts.name,
            &opts.file,
            &opts.filter,
            &opts.language
        )
    }};
}

pub fn get_git_info() -> Result<Vec<CommitInfo>, Box<dyn Error + Send + Sync>> {
    let repo = git_repository::discover(".")?;
    let mut tips = vec![];
    let head = repo.head_commit()?;
    tips.push(head.id);
    let commit_iter = repo.rev_walk(tips);
    let commits = commit_iter.all()?.filter_map(|i| match i {
        Ok(i) => get_item_from_oid_option!(i, &repo, try_into_commit).map(|i| {
            let author = match i.author() {
                Ok(author) => author,
                Err(_) => return None,
            };
            let message = match i.message() {
                Ok(message) => message,
                Err(_) => return None,
            };
            let mut msg = message.title.to_string();
            if let Some(msg_body) = message.body {
                msg.push_str(&msg_body.to_string());
            }

            Some(CommitInfo {
                date: match i.time().map(|x| {
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(x.seconds_since_unix_epoch.into(), 0),
                        Utc,
                    )
                }) {
                    Ok(i) => i,
                    Err(_) => return None,
                },
                hash: i.id.to_string(),
                author_email: author.email.to_string(),
                author: author.name.to_string(),
                message: msg,
            })
        }),
        Err(_) => None,
    });
    let commits = commits.flatten();
    Ok(commits.collect())
}

pub struct CommitInfo {
    pub date: DateTime<Utc>,
    pub hash: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
}

fn find_function_in_file_with_commit(
    file_path: &str,
    fc: &str,
    name: &str,
    langs: Language,
) -> Result<FileType, Box<dyn Error>> {
    let file = match langs {
        Language::Rust => {
            let functions = rust::find_function_in_file(fc, name)?;
            FileType::Rust(RustFile::new(file_path.to_string(), functions))
        }
        // #[cfg(feature = "c_lang")]
        // Language::C => {
        //     let functions = languages::c::find_function_in_file(fc, name)?;
        //     FileType::C(CFile::new(file_path.to_string(), functions))
        // }
        #[cfg(feature = "unstable")]
        Language::Go => {
            let functions = languages::go::find_function_in_file(fc, name)?;
            FileType::Go(GoFile::new(file_path.to_string(), functions))
        }
        Language::Python => {
            let functions = languages::python::find_function_in_file(fc, name)?;
            FileType::Python(PythonFile::new(file_path.to_string(), functions))
        }
        Language::Ruby => {
            let functions = languages::ruby::find_function_in_file(fc, name)?;
            FileType::Ruby(RubyFile::new(file_path.to_string(), functions))
        }
        Language::All => match file_path.split('.').last() {
            Some("rs") => {
                let functions = rust::find_function_in_file(fc, name)?;
                FileType::Rust(RustFile::new(file_path.to_string(), functions))
            }
            // #[cfg(feature = "c_lang")]
            // Some("c" | "h") => {
            //     let functions = languages::c::find_function_in_file(fc, name)?;
            //     FileType::C(CFile::new(file_path.to_string(), functions))
            // }
            Some("py" | "pyw") => {
                let functions = languages::python::find_function_in_file(fc, name)?;
                FileType::Python(PythonFile::new(file_path.to_string(), functions))
            }
            #[cfg(feature = "unstable")]
            Some("go") => {
                let functions = languages::go::find_function_in_file(fc, name)?;
                FileType::Go(GoFile::new(file_path.to_string(), functions))
            }
            Some("rb") => {
                let functions = languages::ruby::find_function_in_file(fc, name)?;
                FileType::Ruby(RubyFile::new(file_path.to_string(), functions))
            }
            _ => Err("unknown file type")?,
        },
    };
    Ok(file)
}

#[cfg_attr(feature = "cache", cached)]
// function that takes a vec of files paths and there contents and a function name and uses find_function_in_file_with_commit to find the function in each file and returns a vec of the functions
fn find_function_in_files_with_commit(
    files: Vec<(String, String)>,
    name: String,
    langs: Language,
) -> Vec<FileType> {
    #[cfg(feature = "parallel")]
    let t = files.par_iter();
    #[cfg(not(feature = "parallel"))]
    let t = files.iter();
    t.filter_map(|(file_path, fc)| {
        find_function_in_file_with_commit(file_path, fc, &name, langs).ok()
    })
    .collect()
}

trait UnwrapToError<T> {
    fn unwrap_to_error_sync(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>>;
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>>;
}

impl<T> UnwrapToError<T> for Option<T> {
    fn unwrap_to_error_sync(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        self.map_or_else(|| Err(message)?, |val| Ok(val))
    }
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>> {
        self.map_or_else(|| Err(message)?, |val| Ok(val))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::languages::{
        rust::{BlockType, RustFilter},
        FileTrait,
    };

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
                println!("{functions}");
            }
            Err(e) => println!("{e}"),
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
            &FileFilterType::None,
            &Filter::None,
            &languages::Language::Rust,
        );
        match &output {
            Ok(output) => println!("{output}"),
            Err(error) => println!("{error}"),
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
        println!("{}", output.as_ref().unwrap_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("is not a rust file"));
    }
    #[test]
    fn test_date_range() {
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
                println!("{functions}");
            }
            Err(e) => println!("-{e}-"),
        }
        assert!(output.is_ok());
    }

    #[test]
    fn tet_date() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::None,
            &Filter::Date("27 Sep 2022 00:27:23 -0400".to_owned()),
            &languages::Language::Rust,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => {
                println!("{functions}");
            }
            Err(e) => println!("-{e}-"),
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
                println!("{functions}");
                functions.get_commit().files.iter().for_each(|file| {
                    println!("file: {}", file.get_file_name());
                    println!("{file}");
                });
            }
            Err(e) => println!("{e}"),
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
                println!("{functions}");
            }
            Err(e) => println!("{e}"),
        }
        assert!(output.is_ok());
        let output = output.unwrap();
        let commit = output.get_commit();
        let file = commit.get_file();
        let _functions = file.get_functions();
    }

    // #[test]
    // #[cfg(feature = "c_lang")]
    // fn c_lang() {
    //     let now = Utc::now();
    //     let output = get_function_history(
    //         "empty_test",
    //         &FileFilterType::Relative("src/test_functions.c".to_string()),
    //         &Filter::DateRange(
    //             "03 Oct 2022 11:27:23 -0400".to_owned(),
    //             "05 Oct 2022 23:45:52 +0000".to_owned(),
    //         ),
    //         &languages::Language::C,
    //     );
    //     let after = Utc::now() - now;
    //     println!("time taken: {}", after.num_seconds());
    //     match &output {
    //         Ok(functions) => println!("{}", functions),
    //         Err(e) => println!("{}", e),
    //     }
    //     assert!(output.is_ok());
    // }
    #[test]
    #[cfg(feature = "unstable")]
    fn go() {
        let now = Utc::now();
        // sleep(Duration::from_secs(2));
        println!("go STARTING");
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Relative("src/test_functions.go".to_string()),
            &Filter::None,
            &languages::Language::Go,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        match &output {
            Ok(functions) => println!("{functions}"),
            Err(e) => println!("{e}"),
        }
        assert!(output.is_ok());
    }

    #[test]
    fn filter_by_param_rust() {
        // search for rust functions
        let mut now = Utc::now();
        let output = get_function_history!(name = "empty_test", language = Language::Rust);
        let mut after = Utc::now() - now;
        println!("time taken to search: {}", after.num_seconds());
        let output = match output {
            Ok(result) => result,
            Err(e) => panic!("{}", e),
        };
        now = Utc::now();
        let new_output = output.filter_by(&Filter::PLFilter(LanguageFilter::Rust(
            rust::RustFilter::HasParameterType(String::from("String")),
        )));
        after = Utc::now() - now;
        println!("time taken to filter {}", after.num_seconds());
        match &new_output {
            Ok(res) => println!("{res}"),
            Err(e) => println!("{e}"),
        }
        let new_output = output.filter_by(&Filter::PLFilter(LanguageFilter::Rust(
            rust::RustFilter::InBlock(BlockType::Extern),
        )));
        after = Utc::now() - now;
        println!("time taken to filter {}", after.num_seconds());
        match &new_output {
            Ok(res) => println!("{res}"),
            Err(e) => println!("{e}"),
        }
        assert!(new_output.is_ok());
    }

    #[test]
    fn test_filter_by() {
        let repo =
            get_function_history!(name = "empty_test").expect("Failed to get function history");
        let f1 = filter_by!(
            repo,
            RustFilter::InBlock(crate::languages::rust::BlockType::Impl),
            Rust
        );
        match f1 {
            Ok(_) => println!("filter 1 ok"),
            Err(e) => println!("error: {e}"),
        }
        let f2 = filter_by!(
            repo,
            Filter::CommitHash("c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0".to_string())
        );
        match f2 {
            Ok(_) => println!("filter 2 ok"),
            Err(e) => println!("error: {e}"),
        }
        let f3 = filter_by!(
            repo,
            LanguageFilter::Rust(RustFilter::InBlock(crate::languages::rust::BlockType::Impl)),
            1
        );
        match f3 {
            Ok(_) => println!("filter 3 ok"),
            Err(e) => println!("error: {e}"),
        }
    }
}
