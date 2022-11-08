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
    clippy::module_name_repetitions
)]
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
        // use git_repository;
        git_repository::hash::ObjectId::from($oid)
            .attach(&$repo)
            // .map(|id| id.attach(&repo))?
            .object()
            .ok()?
            .$typs()
            .ok()
    };
}
// static mut file_count: usize = 0;
#[cfg(feature = "cache")]
use cached::proc_macro::cached;
use chrono::{DateTime, NaiveDateTime, Utc};
use languages::{rust, LanguageFilter, PythonFile, RubyFile, RustFile};
use rayon::prelude::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crossbeam_channel::{bounded, Receiver, Sender};
use git_repository::{prelude::ObjectIdExt, ObjectId};
use std::{
    error::Error,
    process::Command,
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration, fmt, path::PathBuf,
};

// #[cfg(feature = "c_lang")]
// use languages::CFile;
#[cfg(feature = "unstable")]
use languages::GoFile;

pub use {
    languages::Language,
    types::{Commit, FileType, FunctionHistory},
};

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
/// let t = get_function_history("empty_test", &FileFilterType::Absolute("src/test_functions.rs".to_string()), &Filter::None, language::Rust).unwrapexit();
/// ```
#[allow(clippy::too_many_lines)]
// TODO: split this function into smaller functions
pub fn get_function_history(
    name: String,
    file: &FileFilterType,
    filter: &Filter,
    langs: languages::Language,
) -> Result<FunctionHistory, Box<dyn Error + Send + Sync>> {
    // chack if name is empty
    if name.is_empty() {
        Err("function name is empty")?;
    }
    let repo = git_repository::discover(".")?;
    let mut tips = vec![];
    let head = repo.head_commit().unwrapexit();
    tips.push(head.id);
    let commit_iter = repo.rev_walk(tips);
    let commits = commit_iter
        .all()
        .unwrapexit()
        .filter_map(|i| match i {
            Ok(i) => get_item_from_oid_option!(i, &repo, try_into_commit),
            Err(_) => None,
        })
        .collect::<Vec<_>>();
    let closest_date = (0, Utc::now());

    let commits = commits
        .iter()
        .map(|i| {
            let tree = i.tree().unwrapexit().id;
            let time = i.time().unwrapexit();
            let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time.seconds_since_unix_epoch.into(), 0), Utc);
            let authorinfo = i.author().unwrapexit();
            let author = authorinfo.name.to_string();
            let email = authorinfo.email.to_string();
            let messages = i.message().unwrapexit();
            let mut message = messages.title.to_string();
            if let Some(i) = messages.body {
                message.push_str(i.to_string().as_str());
            }
            let commit = i.id().to_hex().to_string();
            let metadata = (message, commit, author, email, time);

            (tree, metadata)
        })
        .filter(|(_, metadata)| {
            match filter {
                Filter::CommitHash(hash) => *hash == metadata.1,
                Filter::Date(date) => {
                    // if closest_date
                    // let date = date.parse::<DateTime<Utc>>().unwrapexit();
                    // let cmp = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(metadata.4.seconds_since_unix_epoch.into(), 0), Utc);
                    false
                }
                Filter::DateRange(start, end) => {
                    // let date = metadata.4.seconds_since_unix_epoch;
                    let date = metadata.4;
                    let start = DateTime::parse_from_rfc2822(start)
                        .map(|i| i.with_timezone(&Utc))
                        .unwrapexit();
                    let end = DateTime::parse_from_rfc2822(end)
                        .map(|i| i.with_timezone(&Utc))
                        .unwrapexit();
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
    let (tx, rx) = bounded(0);
    let cloned_name = name.clone();
    spawn(move || receiver(name.clone(), langs.clone(), rx));
    let commits = commits.into_par_iter().filter_map(|i| {
        // let name_c = name.clone();
        // println!("{}: ",i.1.1);
        let txt = tx.clone();
        let tree = spawn(move || sender(i.0,  txt));
        let tree = tree.join().unwrapexit();
        // println!("done");
        match tree {
            Ok(tree) => {
                // for item in tree {
                //     println!("file: {}\n{}", item.0, item.1);
                // }
                Some(Commit::new(i.1.1, tree, &i.1.4.to_rfc2822(), i.1.2, i.1.3, i.1.0))

            }
            Err(_) => {
                // println!("error");
                None
            }
        }
    }).collect::<Vec<_>>();
    if commits.is_empty() {
        Err("no history found")?;
    }
    let fh = FunctionHistory::new(cloned_name, commits);
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
    c: Sender<Arc<Mutex<(ObjectId, Vec<FileType>, bool)>>>,
) -> Result<Vec<FileType>, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let id = Arc::new(Mutex::new((id, Vec::new(), false)));
        c.send(Arc::clone(&id))?;
        loop {
            {
                let id = id.lock().unwrapexit();
                if id.2 {
                    return Ok(id.1.clone());
                }
            }
            sleep(Duration::from_millis(1000));
        }
    }
}

fn receiver(
    name: String,
    langs: Language,
    c: Receiver<Arc<Mutex<(ObjectId, Vec<FileType>,  bool)>>>,
) {
    let repo = git_repository::discover(".").unwrapexit();
    for r in c {
        let mut data = r.lock().unwrapexit();
        let (objid, _, _ ) = data.clone();
        let object = repo.find_object(objid).unwrapexit();
        let tree = object.into_tree();
        // println!("traverse_tree");
        let files = traverse_tree(&tree, &repo, &name,"".to_string(), langs).unwrapexit();
        // println!("tree traverse_tree");
        data.1 = files;
        data.2 = true;
    }
}

fn traverse_tree(
    tree: &git_repository::Tree<'_>,
    repo: &git_repository::Repository,
    name: &str,
    path: String,
    langs: Language,
) -> Result<Vec<FileType>, Box<dyn std::error::Error>> {
    // println!("traverse_tree in");
    let treee_iter = tree.iter();
    let mut files: Vec<(String, String)> = Vec::new();
    let mut ret = Vec::new();
    for i in treee_iter {
        let i = i.unwrapexit();
        match &i.mode() {
            git_object::tree::EntryMode::Tree => {
                let new = get_item_from!(i.oid(), &repo, try_into_tree);
                let path_new = PathBuf::from(path.clone()).join(i.filename().to_string());
                // println!("new tree");
                ret.extend(traverse_tree(&new, repo, name, path_new.to_str().unwrapexit().to_string(), langs)?);
                // println!("new tree done");
            }
            git_object::tree::EntryMode::Blob => {
                if !i.filename().to_string().ends_with(".rs") {
                    continue;
                }
                // println!("{path}/{}", i.filename().to_string());
                // unsafe {
                //     COUNT += 1;
                // }
                let obh = repo.find_object(i.oid())?;
                // i.inner.
                let objref = git_object::ObjectRef::from_bytes(obh.kind, &obh.data)?;
                let blob = objref.into_blob();
                if let Some(blob) = blob {
                    files.push((
                        i.filename().to_string(),
                        String::from_utf8_lossy(blob.data).to_string(),
                    ));
                }
            }
            _ => {
                println!("unknown");
            }
        }
    }
    // println!("parsing");
    ret.extend(find_function_in_files_with_commit(
        files,
        name.to_string(),
        langs,
    ));
    // println!("traverse_tree out");
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
            opts.name.to_string(),
            &opts.file,
            &opts.filter,
            opts.language
        )
    }};
}
#[inline]
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
#[inline]
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
#[inline]
fn find_file_in_commit(commit: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
    let commit_history = Command::new("git")
        .args(format!("show {commit}:{file_path}").split(' '))
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
    if file_list.is_empty() {
        return Err(format!("no files found for commit {commit} in git ouput"))?;
    }
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
            FileFilterType::None => match langs {
                // #[cfg(feature = "c_lang")]
                // Language::C => {
                //     if file.ends_with(".c") || file.ends_with(".h") {
                //         files.push(file);
                //     }
                // }
                #[cfg(feature = "unstable")]
                Language::Go => {
                    if file.ends_with(".go") {
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
                Language::Ruby => {
                    if file.ends_with(".rb") {
                        files.push(file);
                    }
                }
                Language::All => {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "c_lang")] {
                            if file.ends_with(".c") || file.ends_with(".h") || file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb") {
                                files.push(file);
                            }
                        }
                        else if #[cfg(feature = "unstable")] {
                            if file.ends_with(".go")  || file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb"){
                                files.push(file);
                            }
                        }
                        else if #[cfg(all(feature = "unstable", feature = "c_lang"))] {
                            if file.ends_with(".c") || file.ends_with(".h") || file.ends_with(".go") || file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb") {
                                files.push(file);
                            }
                        }
                        else {
                            if file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".rb") {
                                files.push(file);
                            }
                        }

                    }
                }
            },
        }
    }
    if files.is_empty() {
        return Err(format!(
            "no files found for commit {} in matching the languages specified",
            commit
        ))?;
    }
    let err = "no function found".to_string();
    // organize the files into hierarchical structure of directories (vector of vectors) by splitting the path
    let mut file_tree: Vec<(String, Vec<&str>)> = Vec::new();
    for file in files.clone() {
        let mut file_path = file.split('/').collect::<Vec<&str>>();
        let name = match file_path.pop() {
            Some(name) => name,
            None => continue,
        };
        let file_path = file_path.join("/");
        let mut file_tree_index_found = false;
        for (file_tree_index, file_tree_path) in file_tree.iter().enumerate() {
            if (file_tree_path).0 == file_path {
                file_tree_index_found = true;
                file_tree[file_tree_index].1.push(name);
                break;
            }
        }
        if !file_tree_index_found {
            file_tree.push((file_path, vec![name]));
        }
    }

    #[cfg(feature = "parallel")]
    let t = file_tree.par_iter();
    #[cfg(not(feature = "parallel"))]
    let t = file_tree.iter();
    let stuffs = t
        .filter_map(|(path, files)| {
            let mut file_types = Vec::new();
            for file in files {
                let file_path = format!("{path}/{file}");
                let file_contents = find_file_in_commit(commit, &file_path).ok()?;

                file_types.push((file_path, file_contents));
            }

            match find_function_in_files_with_commit(file_types, name.to_string(), langs) {
                vec if !vec.is_empty() => Some(vec),
                _ => None,
            }
        })
        .flat_map(|x| x)
        .collect::<Vec<_>>();
    if stuffs.is_empty() {
        Err(err)?;
    }
    Ok(stuffs)
}

fn find_function_in_file_with_commit(
    file_path: &str,
    fc: &str,
    name: &str,
    langs: Language,
) -> Result<FileType, Box<dyn Error>> {
    // println!("finding function {} in file {}", name, file_path);
    // unsafe {
    //     file_count += 1;
    // }
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
            Some("py") => {
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
    // println!("files: ",);
    #[cfg(feature = "parallel")]
    let t = files.par_iter();
    // #[cfg(not(feature = "parallel"))]
    let t = files.iter();
    // println!("finding function {} in files", name);
    t.filter_map(|(file_path, fc)| {
        // println!("fparsing ile_path: {}", file_path);
        match find_function_in_file_with_commit(file_path, fc, &name, langs) {
            Ok(file) => {
                // println!("file: {:?}", file);
                Some(file)},
            Err(e) => {
                // println!("error: {}", e);
                None
            }
        }
        // println!("parsed file_path: {}", file_path);
    })
    .collect()
}

trait UnwrapToError<T> {
    fn unwrap_to_error_sync(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>>;
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>>;
}

impl<T> UnwrapToError<T> for Option<T> {
    fn unwrap_to_error_sync(self, message: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        self.map_or_else(|| Err(message.to_string().into()), |val| Ok(val))
    }
    fn unwrap_to_error(self, message: &str) -> Result<T, Box<dyn Error>> {
        self.map_or_else(|| Err(message.to_string().into()), |val| Ok(val))
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
            "empty_test".to_string(),
            &FileFilterType::Relative("src/test_functions.rs".to_string()),
            &Filter::None,
            languages::Language::Rust,
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
            "empty_test".to_owned(),
            &FileFilterType::Absolute("src/test_functions.rs".to_string()),
            &Filter::None,
            languages::Language::Rust,
        );
        // assert that err is "not git is not installed"
        if output.is_err() {
            assert_ne!(output.unwrap_err().to_string(), "git is not installed");
        }
    }

    #[test]
    fn not_found() {
        let output = get_function_history(
            "Not_a_function".to_string(),
            &FileFilterType::None,
            &Filter::None,
            languages::Language::Rust,
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
            "empty_test".to_string(),
            &FileFilterType::Absolute("src/test_functions.txt".to_string()),
            &Filter::None,
            languages::Language::Rust,
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
            "empty_test".to_string(),
            &FileFilterType::None,
            &Filter::DateRange(
                "27 Sep 2022 11:27:23 -0400".to_owned(),
                "04 Oct 2022 23:45:52 +0000".to_owned(),
            ),
            languages::Language::Rust,
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
            "empty_test".to_string(),
            &FileFilterType::None,
            &Filter::None,
            languages::Language::All,
        );
        let after = Utc::now() - now;
        println!("time taken: {}", after.num_seconds());
        // unsafe {
        //     println!("# of files parsed: {}", file_count);
        // }
        match &output {
            Ok(functions) => {
                println!("{functions}");
                functions.get_commit().files.iter().for_each(|file| {
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
            "empty_test".to_string(),
            &FileFilterType::Relative("src/test_functions.py".to_string()),
            &Filter::DateRange(
                "03 Oct 2022 11:27:23 -0400".to_owned(),
                "04 Oct 2022 23:45:52 +0000".to_owned(),
            ),
            languages::Language::Python,
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
        let output = output.unwrapexit();
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
    //         languages::Language::C,
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
        match &t {
            Ok(functions) => {
                println!("{functions:?}");
            }
            Err(e) => println!("{e}"),
        }
        assert!(t.is_ok());
    }

    #[test]
    #[cfg(feature = "unstable")]
    fn go() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Relative("src/test_functions.go".to_string()),
            &Filter::None,
            languages::Language::Go,
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

trait unwrapexit<T> {
    fn unwrapexit(self) -> T;
}

impl<T, E: fmt::Debug> unwrapexit<T> for Result<T, E> {
    fn unwrapexit(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
}

impl<T> unwrapexit<T> for Option<T> {
    fn unwrapexit(self) -> T {
        match self {
            Some(t) => t,
            None => {
                println!("None");
                std::process::exit(1);
            }
        }
    }
}


