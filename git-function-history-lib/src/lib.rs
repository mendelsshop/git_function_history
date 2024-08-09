#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
#![allow(
    clippy::must_use_candidate,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cognitive_complexity,
    clippy::float_cmp,
    clippy::return_self_not_must_use,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::too_many_lines
)]
/// code and function related language

/// Different types that can extracted from the result of `get_function_history`.
pub mod types;
macro_rules! get_item_from {
    ($oid:expr, $repo:expr, $typs:ident) => {
        gix::hash::ObjectId::from($oid)
            .attach(&$repo)
            .object()
            .map_err(|_| "Could not find object")?
            .$typs()
            .map_err(|_| format!("Could not find {} from object", stringify!($typs)))?
    };
}

macro_rules! get_item_from_oid_option {
    ($oid:expr, $repo:expr, $typs:ident) => {
        gix::hash::ObjectId::from($oid.id)
            .attach(&$repo)
            .object()
            .ok()?
            .$typs()
            .ok()
    };
}
use chrono::{DateTime, Utc};
use function_grep::{
    supported_languages::{InstatiateMap, InstatiatedLanguage, SupportedLanguage},
    ParsedFile,
};
use git_function_history_proc_macro::enumstuff;
// use languages::LanguageFilter;

#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use gix::{objs, prelude::ObjectIdExt, ObjectId, Tree};
use std::{error::Error, ops::Sub};

pub use types::{Commit, FunctionHistory};

/// Different filetypes that can be used to ease the process of finding functions using `get_function_history`.
/// path separator is `/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFilterType {
    /// When you have a absolute path to a file.
    Absolute(String),
    /// When you have a relative path to a file and or want to find look in all files match a name (aka `ends_with`).
    Relative(String),
    /// When you want to filter only files in a specific directory
    Directory(String),
    /// When you don't know the path to a file.
    None,
}

/// This is filter enum is used when you want to lookup a function with the filter of filter a previous lookup.
// TODO: what do we actually need to derive
// TODO: allow more complex language type filters.
#[derive(enumstuff, PartialEq, Eq, Debug)]
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
    /// when you want to filter by a any commit author name that contains a specific string
    Author(String),
    /// when you want to filter by a any commit author email that contains a specific string
    AuthorEmail(String),
    // when you want to filter by a a commit message that contains a specific string
    Message(String),
    /// when you want to filter by proggramming language filter
    #[enumstuff(skip)]
    PLFilter(function_grep::filter::InstantiatedFilter),
    /// when you want to filter to only have files that are in a specific language
    Language(String),
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
/// let t = get_function_history("empty_test", &FileFilterType::Absolute("src/test_functions.rs".to_string()), &Filter::None, function_grep::supported_languages::predefined_languages()).unwrap();
/// ```
///
/// # Errors
///
/// If no files were found that match the criteria given, this will return an 'Err'
/// Or if it cannot find or read from a git repository
///
// TODO: split this function into smaller functions
// TODO: allow more complex language type filters.
// TODO: allow multiple filters.
pub fn get_function_history(
    name: &str,
    file: &FileFilterType,
    filter: &Filter,
    langs: &[&dyn SupportedLanguage],
) -> Result<FunctionHistory, Box<dyn Error + Send + Sync>> {
    // chack if name is empty
    if name.is_empty() {
        Err("function name is empty")?;
    }
    // if filter is date list all the dates and find the one that is closest to the date set that to closest_date and when using the first filter check if the date of the commit is equal to the closest_date
    // find the closest date by using get_git_dates_commits_oxide
    match filter {
        Filter::Date(date) => {
            DateTime::parse_from_rfc2822(date)?;
        }
        Filter::Author(_)
        | Filter::AuthorEmail(_)
        | Filter::Message(_)
        | Filter::None
        | Filter::CommitHash(_) => (),
        Filter::DateRange(start, end) => {
            // check if start is before end
            // vaildate that the dates are valid
            let start = DateTime::parse_from_rfc2822(start)?.with_timezone(&Utc);
            let end = DateTime::parse_from_rfc2822(end)?.with_timezone(&Utc);
            if start > end {
                Err("start date is after end date")?;
            }
        }
        _ => Err("invalid filter")?,
    }
    match file {
        FileFilterType::Absolute(file) | FileFilterType::Relative(file) => {
            // vaildate that the file makes sense with language
            let is_supported = langs
                .iter()
                .flat_map(|lang| lang.file_exts())
                .copied()
                .any(|i| ends_with_cmp_no_case(file, i));
            if !is_supported {
                Err(format!(
                    "file {file} is not a supported file, the following files are supported {}",
                    langs
                        .iter()
                        .map(|lang| format!(
                            "({} with extension(s) [{}])",
                            lang.language_name(),
                            &lang.file_exts().join(",")
                        ))
                        .collect::<Vec<_>>()
                        .join(" ")
                ))?;
            }
        }
        FileFilterType::Directory(_) | FileFilterType::None => {}
    }

    let langs1 = &*langs
        .iter()
        .flat_map(|l| l.file_exts())
        .copied()
        .collect::<Box<[_]>>();
    let repo = gix::discover(".")?;
    let th_repo = repo.clone().into_sync();
    let commit_iter = repo.rev_walk(repo.head_id().map(gix::Id::detach));
    let commit_iter = commit_iter.all()?.filter_map(|id| Some(id.ok()?.detach()));
    #[cfg(feature = "parallel")]
    let commit_iter = {
        // we have to collect here because we don't want any refrences to not send/sync structs
        let binding = commit_iter.collect::<Vec<_>>();
        binding.into_par_iter()
    };
    let commits = commit_iter.filter_map(|id| {
        let repo = th_repo.to_thread_local();
        let commit = id.id.attach(&repo).object().ok()?.try_into_commit().ok()?;
        let tree = commit.tree().ok()?.id;
        let time = commit.time().ok()?;
        let time = DateTime::from_timestamp(time.seconds, 0)?;
        let authorinfo = commit.author().ok()?;
        let author = authorinfo.name.to_string();
        let email = authorinfo.email.to_string();
        let messages = commit.message().ok()?;
        let mut message = messages.title.to_string();
        if let Some(i) = messages.body {
            message.push_str(i.to_string().as_str());
        }
        let commit = commit.id().to_hex().to_string();
        let metadata = (message, commit, author, email, time);
        Some((tree, metadata))
    });
    let langs = langs.instatiate_map(name).unwrap();
    let langs = &*langs.as_slice().iter().collect::<Box<[_]>>();
    if let Filter::Date(date) = filter {
        let date = DateTime::parse_from_rfc2822(date)?.with_timezone(&Utc);
        let commit = commits.min_by_key(|commit| (commit.1 .4.sub(date).num_seconds().abs()));
        return if let Some(i) = commit {
            let tree = sender(i.0, th_repo.to_thread_local(), langs1, langs, file)?;

            if tree.is_empty() {
                Err("empty commit found")?;
            }

            Ok(FunctionHistory::new(
                name.to_owned(),
                vec![Commit::new(
                    &i.1 .1,
                    tree,
                    &i.1 .4.to_rfc2822(),
                    &i.1 .2,
                    &i.1 .3,
                    &i.1 .0,
                )?],
            ))
        } else {
            Err("no history found")?
        };
    }
    let commits = commits.filter(|(_, metadata)| {
            match filter {
                Filter::CommitHash(hash) => *hash == metadata.1,
                Filter::Date(_) => unreachable!(),
                Filter::DateRange(start, end) => {
                    // let date = metadata.4.seconds_since_unix_epoch;
                    let date = metadata.4;
                    let start = DateTime::parse_from_rfc2822(start)
                        .map(|i| i.with_timezone(&Utc))
                        .expect("failed to parse start date, edge case shouldn't happen please file a bug to https://github.com/mendelsshop/git_function_history/issues");
                    let end = DateTime::parse_from_rfc2822(end)
                        .map(|i| i.with_timezone(&Utc))
                        .expect("failed to parse end date, edge case shouldn't happen please file a bug to https://github.com/mendelsshop/git_function_history/issues");
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
        });

    // todo use itertools to split into vec of oks and errs
    // and report some of errors if no oks and if no oks and errs report no history found
    let commits = commits
        .filter_map(|i| {
            let tree = sender(i.0, th_repo.to_thread_local(), langs1, langs, file);
            match tree {
                Ok(tree) => {
                    if tree.is_empty() {
                        None?;
                    }
                    Some(
                        Commit::new(
                            &i.1 .1,
                            tree,
                            &i.1 .4.to_rfc2822(),
                            &i.1 .2,
                            &i.1 .3,
                            &i.1 .0,
                        )
                        .ok()?,
                    )
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

fn sender(
    id: ObjectId,
    repo: gix::Repository,
    file_exts: &[&str],
    langs: &[&InstatiatedLanguage<'_>],
    file: &FileFilterType,
) -> Result<Vec<ParsedFile>, String> {
    let object = repo.find_object(id).map_err(|_| "failed to find object")?;
    let tree = object.try_into_tree();
    let binding = tree.unwrap();
    traverse_tree(&binding, &repo, String::new(), file_exts, langs, file)
}

#[inline]
fn traverse_tree(
    tree: &Tree<'_>,
    repo: &gix::Repository,
    path: String,
    file_exts: &[&str],
    langs: &[&InstatiatedLanguage<'_>],
    filetype: &FileFilterType,
) -> Result<Vec<ParsedFile>, String> {
    let mut files_exts = file_exts.iter();

    let treee_iter = tree.iter();
    let mut files: Vec<_> = Vec::new();
    let mut ret = Vec::new();
    for i in treee_iter {
        let i = i.map_err(|_| "failed to get tree entry")?;
        // TODO: what should the path seperator be?
        let file = format!(
            "{path}{}{}",
            if path.is_empty() { "" } else { "/" },
            i.filename()
        );
        match &i.mode().kind() {
            objs::tree::EntryKind::Tree => {
                let new = repo
                    .find_object(i.oid())
                    .map_err(|_| "Could not find object")?
                    .try_into_tree()
                    .map_err(|_| {
                        format!("Could not find {} from object", stringify!(try_into_tree))
                    })?;
                ret.extend(traverse_tree(&new, repo, file, file_exts, langs, filetype)?);
            }
            objs::tree::EntryKind::Blob => {
                match &filetype {
                    FileFilterType::Relative(ref path) => {
                        if !file.ends_with(path) {
                            continue;
                        }
                    }
                    FileFilterType::Absolute(ref path) => {
                        if &file != path {
                            continue;
                        }
                    }
                    FileFilterType::Directory(ref path) => {
                        if !file.contains(path) {
                            continue;
                        }
                    }
                    FileFilterType::None => {}
                }

                if !files_exts.any(|ext| ends_with_cmp_no_case(&file, ext)) {
                    continue;
                }
                let obh = repo
                    .find_object(i.oid())
                    .map_err(|_| "failed to find object")?;
                let blob = obh
                    .try_into_blob()
                    .map_err(|e| format!("could not obtain file contents of {file}: {e}"))?;
                files.push((file, String::from_utf8_lossy(&blob.data).to_string()));
            }
            _ => {}
        }
    }
    ret.extend(find_function_in_files_with_commit(files, langs));

    Ok(ret)
}

/// used for the `get_function_history` macro internally (you don't have to touch this)
pub struct MacroOpts<'a, 'b> {
    pub name: &'a str,
    pub file: FileFilterType,
    pub filter: Filter,
    pub supported_languages: Vec<&'b dyn SupportedLanguage>,
    pub default_languages: bool,
}

impl Default for MacroOpts<'_, '_> {
    fn default() -> Self {
        Self {
            name: "",
            file: FileFilterType::None,
            filter: Filter::None,
            supported_languages: vec![],
            default_languages: true,
        }
    }
}
// TODO: update macro docs
//
/// macro to get the history of a function
/// wrapper around the `get_function_history` function
///
/// # examples
/// ```rust
/// use git_function_history::{get_function_history, languages::Language, Filter, FileFilterType};
/// git_function_history::get_function_history!(name = "main", file = FileFilterType::Relative("src/main.rs".to_string()), filter = Filter::None,default_languages = false, supported_languages = vec![&function_grep::supported_languages::Rust]);
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
        let mut supported = opts.supported_languages;
        supported.extend(if opts.default_languages { function_grep::supported_languages::predefined_languages() } else { &[] });
        get_function_history(
            opts.name,
            &opts.file,
            &opts.filter,
            &supported
        )
    }};
}

/// Returns a vec of information such as author, date, email, and message for each commit
///
/// # Errors
/// wiil return `Err`if it cannot find or read from a git repository
pub fn get_git_info() -> Result<Vec<CommitInfo>, Box<dyn Error + Send + Sync>> {
    let repo = gix::discover(".")?;
    let mut tips = vec![];
    let head = repo.head_commit()?;
    tips.push(head.id);
    let commit_iter = repo.rev_walk(tips);
    let commits = commit_iter.all()?.filter_map(|i| match i {
        Ok(i) => get_item_from_oid_option!(i, &repo, try_into_commit).map(|i| {
            let Ok(author) = i.author() else { return None };
            let Ok(message) = i.message() else {
                return None;
            };
            let mut msg = message.title.to_string();
            if let Some(msg_body) = message.body {
                msg.push_str(&msg_body.to_string());
            }

            Some(CommitInfo {
                date: match i.time().map(|x| DateTime::from_timestamp(x.seconds, 0)) {
                    Ok(Some(i)) => i,
                    _ => return None,
                },
                hash: i.id,
                author_email: author.email.to_string(),
                author: author.name.to_string(),
                message: msg,
            })
        }),
        Err(_) => None,
    });
    let commits = commits.flatten();
    let commits = commits.collect::<Vec<_>>();
    Ok(commits)
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub date: DateTime<Utc>,
    pub hash: ObjectId,
    pub message: String,
    pub author: String,
    pub author_email: String,
}

#[inline]
// #[cfg_attr(feature = "cache", cached)]
// function that takes a vec of files paths and there contents and a function name and uses find_function_in_file_with_commit to find the function in each file and returns a vec of the functions
fn find_function_in_files_with_commit(
    files: Vec<(String, String)>,
    langs: &[&InstatiatedLanguage<'_>],
) -> Vec<ParsedFile> {
    // commenting out this parallelization seems to net a gain in performance with tree sitter
    //#[cfg(feature = "parallel")]
    //let t = files.par_iter();
    //#[cfg(not(feature = "parallel"))]
    let t = files.iter();
    t.filter_map(|(file_path, fc)| ParsedFile::search_file_with_name(fc, file_path, langs).ok())
        .collect()
}

fn ends_with_cmp_no_case(filename: &str, file_ext: &str) -> bool {
    let filename = std::path::Path::new(filename);
    filename
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case(file_ext))
}

trait UnwrapToError<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, String>;
}

impl<T> UnwrapToError<T> for Option<T> {
    fn unwrap_to_error(self, message: &str) -> Result<T, String> {
        self.map_or_else(|| Err(message.to_string()), |val| Ok(val))
    }
}

#[cfg(test)]
mod tests {
    //     use chrono::Utc;
    //
    use super::*;
    #[test]
    fn found_function() {
        let now = Utc::now();
        let binding = FileFilterType::Relative("src/test_functions.rs".to_string());
        let output = get_function_history(
            "empty_test",
            &binding,
            &Filter::None,
            function_grep::supported_languages::predefined_languages(),
            // &languages::Language::Rust,
        );
        let after = Utc::now() - now;
        println!("time taken: {after}");
        match &output {
            Ok(functions) => {
                println!("{functions}");
            }
            Err(e) => println!("{e}"),
        }
        assert!(output.is_ok());
    }
    //     #[test]
    //     fn git_installed() {
    //         let output = get_function_history(
    //             "empty_test",
    //             &FileFilterType::Absolute("src/test_functions.rs".to_string()),
    //             &Filter::None,
    //             &languages::Language::Rust,
    //         );
    //         // assert that err is "not git is not installed"
    //         if output.is_err() {
    //             assert_ne!(output.unwrap_err().to_string(), "git is not installed");
    //         }
    //     }
    //
    #[test]
    fn not_found() {
        let output = get_function_history(
            "Not_a_function",
            &FileFilterType::None,
            &Filter::None,
            function_grep::supported_languages::predefined_languages(),
        );
        match &output {
            Ok(output) => println!("{output}"),
            Err(error) => println!("{error}"),
        }
        assert!(output.is_err());
    }
    #[test]
    fn not_a_supported_file() {
        let output = get_function_history(
            "empty_test",
            &FileFilterType::Absolute("src/test_functions.txt".to_string()),
            &Filter::None,
            function_grep::supported_languages::predefined_languages(),
        );
        assert!(output.is_err());
        println!("{}", output.as_ref().unwrap_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("is not a supported file"));
    }
    //     #[test]
    //     fn test_date_range() {
    //         let now = Utc::now();
    //         let output = get_function_history(
    //             "empty_test",
    //             &FileFilterType::None,
    //             &Filter::DateRange(
    //                 "27 Sep 2022 11:27:23 -0400".to_owned(),
    //                 "04 Oct 2022 23:45:52 +0000".to_owned(),
    //             ),
    //             &languages::Language::Rust,
    //         );
    //         let after = Utc::now() - now;
    //         println!("time taken: {}", after);
    //         match &output {
    //             Ok(functions) => {
    //                 println!("{functions}");
    //             }
    //             Err(e) => println!("-{e}-"),
    //         }
    //         assert!(output.is_ok());
    //     }
    //
    //     #[test]
    //     fn test_date() {
    //         let now = Utc::now();
    //         let output = get_function_history(
    //             "empty_test",
    //             &FileFilterType::None,
    //             &Filter::Date("27 Sep 2022 00:27:23 -0400".to_owned()),
    //             &languages::Language::Rust,
    //         );
    //         let after = Utc::now() - now;
    //         println!("time taken: {}", after);
    //         match &output {
    //             Ok(functions) => {
    //                 // println!("{}", functions.clone().last().unwrap().date);
    //                 println!("{:?}", functions.clone().last().unwrap().get_metadata());
    //                 println!("{functions}");
    //             }
    //             Err(e) => println!("-{e}-"),
    //         }
    //         assert!(output.is_ok());
    //     }
    //

    #[test]
    fn expensive_test() {
        let now = Utc::now();
        let output = get_function_history(
            "empty_test",
            &FileFilterType::None,
            &Filter::None,
            function_grep::supported_languages::predefined_languages(),
        );
        let after = Utc::now() - now;
        println!("time taken: {after}");
        match &output {
            Ok(functions) => {
                // println!("{functions}");
                // functions.clone().into_iter().take(5).for_each(|commit|{
                //     commit
                //     .files
                //     .iter()
                //     .for_each(|file| {
                //         println!("file: {}", file.file_name().unwrap());
                //         println!("{file}");
                //     });})
            }
            Err(e) => println!("{e}"),
        }
        assert!(output.is_ok());
    }
    //
    //     #[test]
    //     fn python_whole() {
    //         let now = Utc::now();
    //         let output = get_function_history(
    //             "empty_test",
    //             &FileFilterType::Relative("src/test_functions.py".to_string()),
    //             &Filter::None,
    //             &languages::Language::Python,
    //         );
    //         let after = Utc::now() - now;
    //         println!("time taken: {}", after);
    //         match &output {
    //             Ok(functions) => {
    //                 println!("{functions}");
    //             }
    //             Err(e) => println!("{e}"),
    //         }
    //         assert!(output.is_ok());
    //         let output = output.unwrap();
    //         let commit = output.get_commit().unwrap();
    //         let file = commit.get_file().unwrap();
    //         let _functions = file.get_functions();
    //     }
    //
    //     // #[test]
    //     // fn ruby_whole() {
    //     //     let now = Utc::now();
    //     //     let output = get_function_history(
    //     //         "empty_test",
    //     //         &FileFilterType::Relative("src/test_functions.rb".to_string()),
    //     //         &Filter::None,
    //     //         &languages::Language::Ruby,
    //     //     );
    //     //     let after = Utc::now() - now;
    //     //     println!("time taken: {}", after);
    //     //     match &output {
    //     //         Ok(functions) => {
    //     //             println!("{functions}");
    //     //         }
    //     //         Err(e) => println!("{e}"),
    //     //     }
    //     //     assert!(output.is_ok());
    //     //     let output = output.unwrap();
    //     //     let commit = output.get_commit().unwrap();
    //     //     let file = commit.get_file().unwrap();
    //     //     let _functions = file.get_functions();
    //     // }
    //
    //     // #[test]
    //     // #[cfg(feature = "c_lang")]
    //     // fn c_lang() {
    //     //     let now = Utc::now();
    //     //     let output = get_function_history(
    //     //         "empty_test",
    //     //         &FileFilterType::Relative("src/test_functionsc".to_string()),
    //     //         &Filter::DateRange(
    //     //             "03 Oct 2022 11:27:23 -0400".to_owned(),
    //     //             "05 Oct 2022 23:45:52 +0000".to_owned(),
    //     //         ),
    //     //         &languages::Language::C,
    //     //     );
    //     //     let after = Utc::now() - now;
    //     //     println!("time taken: {}", after);
    //     //     match &output {
    //     //         Ok(functions) => println!("{}", functions),
    //     //         Err(e) => println!("{}", e),
    //     //     }
    //     //     assert!(output.is_ok());
    //     // }
    //     // #[test]
    //     // fn go_whole() {
    //     //     let now = Utc::now();
    //     //     let output = get_function_history(
    //     //         "empty_test",
    //     //         &FileFilterType::Relative("src/test_functions.go".to_string()),
    //     //         &Filter::None,
    //     //         &languages::Language::Go,
    //     //     );
    //     //     let after = Utc::now() - now;
    //     //     println!("time taken: {}", after);
    //     //     match &output {
    //     //         Ok(functions) => println!("{functions}"),
    //     //         Err(e) => println!("{e}"),
    //     //     }
    //     //     assert!(output.is_ok());
    //     // }
    //
    //     // #[test]
    //     // fn filter_by_param_rust() {
    //     //     // search for rust functions
    //     //     let mut now = Utc::now();
    //     //     let output = get_function_history!(name = "empty_test", language = Language::Rust);
    //     //     let mut after = Utc::now() - now;
    //     //     println!("time taken to search: {}", after);
    //     //     let output = match output {
    //     //         Ok(result) => result,
    //     //         Err(e) => panic!("{}", e),
    //     //     };
    //     //     now = Utc::now();
    //     //     let new_output = output.filter_by(&Filter::PLFilter(LanguageFilter::Rust(
    //     //         rust::RustFilter::HasParameterType(String::from("String")),
    //     //     )));
    //     //     after = Utc::now() - now;
    //     //     println!("time taken to filter {}", after);
    //     //     match &new_output {
    //     //         Ok(res) => println!("{res}"),
    //     //         Err(e) => println!("{e}"),
    //     //     }
    //     //     let new_output = output.filter_by(&Filter::PLFilter(LanguageFilter::Rust(
    //     //         rust::RustFilter::InBlock(BlockType::Extern),
    //     //     )));
    //     //     after = Utc::now() - now;
    //     //     println!("time taken to filter {}", after);
    //     //     match &new_output {
    //     //         Ok(res) => println!("{res}"),
    //     //         Err(e) => println!("{e}"),
    //     //     }
    //     //     assert!(new_output.is_ok());
    //     // }
    //
    //     // #[test]
    //     // fn test_filter_by() {
    //     //     let repo =
    //     //         get_function_history!(name = "empty_test").expect("Failed to get function history");
    //     //     let f1 = filter_by!(
    //     //         repo,
    //     //         RustFilter::InBlock(crate::languages::rust::BlockType::Impl),
    //     //         Rust
    //     //     );
    //     //     match f1 {
    //     //         Ok(_) => println!("filter 1 ok"),
    //     //         Err(e) => println!("error: {e}"),
    //     //     }
    //     //     let f2 = filter_by!(
    //     //         repo,
    //     //         Filter::CommitHash("c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0".to_string())
    //     //     );
    //     //     match f2 {
    //     //         Ok(_) => println!("filter 2 ok"),
    //     //         Err(e) => println!("error: {e}"),
    //     //     }
    //     //     let f3 = filter_by!(
    //     //         repo,
    //     //         LanguageFilter::Rust(RustFilter::InBlock(crate::languages::rust::BlockType::Impl)),
    //     //         1
    //     //     );
    //     //     match f3 {
    //     //         Ok(_) => println!("filter 3 ok"),
    //     //         Err(e) => println!("error: {e}"),
    //     //     }
    //     // }
}
