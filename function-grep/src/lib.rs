#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::missing_panics_doc)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
use core::fmt;

use filter::InstantiatedFilter;
use supported_languages::InstatiatedLanguage;
use tree_sitter::{LanguageError, QueryError, Range, Tree};
#[allow(missing_debug_implementations)]
pub enum SupportedLanguages {
    All,
    Many(Vec<String>),
    Single(String),
}

mod filter;
/// For adding new language support, and some predefined support for certain languages,
pub mod supported_languages;

//fn run_query<'a>(
//    query: &Query,
//    name: &str,
//    field: u32,
//    node: Node<'a>,
//    code: &'a [u8],
//) -> Result<Box<[Range]>, QueryError> {
//    let mut query_cursor = tree_sitter::QueryCursor::new();
//    let matches = query_cursor.matches(query, node, code);
//    let ranges = matches
//        .filter(|m| {
//            m.captures
//                .iter()
//                .any(|c| c.index == field && c.node.utf8_text(code).unwrap() == name)
//        })
//        .map(|m| m.captures[0].node.range());
//
//    Ok(ranges.collect())
//}

/// Errors that we may give back.
#[derive(Debug)]
pub enum Error {
    /// If there is no language that has this file extension or the file does not have a file
    /// extension.
    FileTypeUnkown(String),
    /// If tree sitter fails to parse the file.
    ParseError(String),
    /// If tree sitter doesn't like the grammer for given language
    GrammarLoad(&'static str, LanguageError),
    /// If tree sitter doesn't like the query from a given [`SupportedLanguage`].
    InvalidQuery(&'static str, QueryError),
    /// If there are no result after filtering.
    NoSuchResultsForFilter,
    /// If there are no result after searching.
    NoResultsForSearch,
}

/// Tries to find the appropiate language for the given file extension [`ext`] based on the list of
/// languages [`langs`] provided.
///
/// # Errors
/// If there is no language for this file extension in the provided language list.
pub fn get_file_type_from_file_ext<'a>(
    ext: &str,
    langs: &'a [&'a InstatiatedLanguage<'a>],
) -> Result<&'a InstatiatedLanguage<'a>, Error> {
    langs
        .iter()
        .find(|lang| lang.file_exts().contains(&ext))
        .copied()
        .ok_or_else(|| Error::FileTypeUnkown(ext.to_string()))
}

/// Tries to find the appropiate language for the given file [`file_name`] based on the list of
/// languages [`langs`] provided.
/// This works by obtaining the extension from the file path and using
/// [`get_file_type_from_file_ext`].
///
/// # Errors
/// If there is no file extension for this file name, or there is no language for this file in the provided language list.
pub fn get_file_type_from_file<'a>(
    file_name: &str,
    langs: &'a [&'a InstatiatedLanguage<'a>],
) -> Result<&'a InstatiatedLanguage<'a>, Error> {
    file_name
        .rsplit_once('.')
        .ok_or_else(|| Error::FileTypeUnkown(file_name.to_string()))
        .map(|(_, ext)| ext)
        .and_then(|ext| get_file_type_from_file_ext(ext, langs))
}

#[derive(Debug, Clone)]
/// The result of finding function with a given name.
/// Use [`Self::search_file`] or [`Self::search_file_with_name`] to do the searching.
pub struct ParsedFile {
    // I believe we cannot store something refernceing the tree, so we cannot directly store the
    // results of the query, but just their ranges so in the [`filter`] method we use the tree to
    // obtain the correct nodes from their ranges
    file: Box<str>,
    file_name: Option<Box<str>>,
    function_name: Box<str>,
    // TODO: maybe each supported language could define filters
    // if so we would store InstatiatedLanguage here
    language_type: Box<str>,
    tree: Tree,
    results: Box<[Range]>,
}

impl ParsedFile {
    #[must_use]
    pub fn new(
        file: &str,
        function_name: &str,
        language_type: &str,
        tree: Tree,
        results: Box<[Range]>,
    ) -> Self {
        Self {
            file: file.into(),
            function_name: function_name.into(),
            language_type: language_type.into(),
            tree,
            results,
            file_name: None,
        }
    }

    // TODO: maybe only make this hidden and expose a filter method that takes in some sort of
    // filter trait
    //
    /// Filters out commits not matching the filter [`f`].
    /// Returns new version of the current [`ParsedFile`] with only the results that match the
    /// filter.
    ///
    /// # Errors
    /// If the filter [`f`] filters out all the results of this file
    pub fn filter(&self, mut f: InstantiatedFilter) -> Result<Self, Error> {
        let root = self.tree.root_node();
        let ranges: Box<[Range]> = self
            .ranges()
            .filter_map(|range| root.descendant_for_point_range(range.start_point, range.end_point))
            .filter(|n| f.filter(n))
            .map(|n| n.range())
            .collect();
        if ranges.is_empty() {
            return Err(Error::NoSuchResultsForFilter);
        }
        let clone = Self {
            results: ranges,
            ..self.clone()
        };
        Ok(clone)
    }

    #[must_use]
    /// Get the name of the language used to parse this file
    pub const fn language(&self) -> &str {
        &self.language_type
    }

    #[must_use]
    /// Get the name of the function that was searched for to make this [`ParsedFile`]
    pub const fn search_name(&self) -> &str {
        &self.function_name
    }

    fn ranges(&self) -> impl Iterator<Item = &Range> {
        self.results.iter()
    }
    /// Search for all function with the name [`name`], in string [`code`] with the specified
    /// language [`language`].
    ///
    /// Note: to obtain the the language you may use [`get_file_type_from_file`] or
    /// [`get_file_type_from_file_ext`].
    /// Alternativly use [`Self::search_file_with_name`] to let us find the correct language for you.
    ///
    /// # Errors
    /// If something with tree sitter goes wrong.
    /// If the code cannot be parsed properly.
    /// If no results are found for this function name.
    pub fn search_file(code: &str, language: &InstatiatedLanguage<'_>) -> Result<Self, Error> {
        let code_bytes = code.as_bytes();
        let mut parser = tree_sitter::Parser::new();
        let ts_lang = language.language();
        parser
            .set_language(ts_lang)
            .map_err(|lang_err| Error::GrammarLoad(language.name(), lang_err))?;
        let parsed = parser
            .parse(code, None)
            .ok_or_else(|| Error::ParseError(code.to_string()))?;

        let node = parsed.root_node();
        let command_ranges = language.run_query(node, code_bytes);

        if command_ranges.is_empty() {
            return Err(Error::NoResultsForSearch);
        }
        Ok(Self::new(
            code,
            language.search_name(),
            language.name(),
            parsed,
            command_ranges,
        ))
    }

    /// Search for all function with the name [`name`], in string [`code`] with a language found
    /// from the file name [`file_name`] and the languages [`langs`].
    ///
    /// # Errors
    /// If there is no language found for the given file name.
    /// If something with tree sitter goes wrong.
    /// If the code cannot be parsed properly,
    /// If no results are found for this function name.
    pub fn search_file_with_name(
        code: &str,
        file_name: &str,
        langs: &[&InstatiatedLanguage<'_>],
    ) -> Result<Self, Error> {
        get_file_type_from_file(file_name, langs)
            .and_then(|language| Self::search_file(code, language))
            .map(|file| file.set_file_name(file_name))
    }

    fn set_file_name(mut self, file_name: &str) -> Self {
        self.file_name.replace(file_name.into());
        self
    }

    #[must_use]
    /// Get the file name of this file.
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    #[must_use]
    /// Get the [Range] of each found function.
    pub const fn results(&self) -> &[Range] {
        &self.results
    }
}

impl IntoIterator for ParsedFile {
    type Item = (Range, String);

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.ranges()
                .map(|range| {
                    (
                        *range,
                        self.file[range.start_byte..range.end_byte].to_string(),
                    )
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

impl fmt::Display for ParsedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.file.lines().enumerate();
        let texts = self
            .ranges()
            .map(move |range| {
                lines
                    .clone()
                    .filter_map(move |(line, str)| {
                        if line >= range.start_point.row && line <= range.end_point.row {
                            Some(format!("{}: {str}", line + 1))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n...\n");
        write!(f, "{texts}")
    }
}
