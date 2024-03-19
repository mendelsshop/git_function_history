#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(missing_debug_implementations, clippy::missing_panics_doc)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
use core::fmt;

use supported_languages::SupportedLanguage;
use tree_sitter::{Language, LanguageError, Node, QueryError, Range, Tree};

pub mod supported_languages;

fn run_query<'a>(
    query_str: &'a str,
    lang: Language,
    node: Node<'a>,
    code: &'a [u8],
) -> Result<Box<[Range]>, QueryError> {
    let query = tree_sitter::Query::new(lang, query_str)?;
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, node, code);
    let ranges = matches.map(|m| m.captures[0].node.range());
    Ok(ranges.collect())
}

#[derive(Debug)]
pub enum Error {
    FileTypeUnkown(String),
    FileTypeNotSupported(String),
    ParseError(String),
    GrammarLoad(&'static str, LanguageError),
    InvalidQuery(&'static str, QueryError),
}

///
/// # Errors
fn get_file_type_from_ext<'a>(
    ext: &str,
    langs: &'a [&'a dyn SupportedLanguage],
) -> Result<&'a dyn SupportedLanguage, Error> {
    langs
        .iter()
        .find(|lang| lang.file_exts().contains(&ext))
        .copied()
        .ok_or_else(|| Error::FileTypeUnkown(ext.to_string()))
}

///
/// # Errors
pub fn get_file_type_from_file<'a>(
    file: &str,
    langs: &'a [&'a dyn SupportedLanguage],
) -> Result<&'a dyn SupportedLanguage, Error> {
    file.rsplit_once('.')
        .ok_or_else(|| Error::FileTypeUnkown(file.to_string()))
        .map(|(_, ext)| ext)
        .and_then(|ext| get_file_type_from_ext(ext, langs))
}

#[derive(Debug, Clone)]
pub struct ParsedFile<'a> {
    // I believe we cannot store something refernceing the tree, so we cannot directly store the
    // results of the query, but just their ranges so in the [`filter`] method we use the tree to
    // obtain the correct nodes from their ranges
    file: &'a str,
    function_name: &'a str,
    // TODO: maybe each supported language could define filters
    // if so we would store dyn SupportedLanguage here
    language_type: &'a str,
    tree: Tree,
    results: Box<[Range]>,
}

impl IntoIterator for ParsedFile<'_> {
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

impl fmt::Display for ParsedFile<'_> {
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

#[derive(Debug)]
pub struct NoResultsForFilter;

impl<'a> ParsedFile<'a> {
    #[must_use]
    pub fn new(
        file: &'a str,
        function_name: &'a str,
        language_type: &'a str,
        tree: Tree,
        results: Box<[Range]>,
    ) -> Self {
        Self {
            file,
            function_name,
            language_type,
            tree,
            results,
        }
    }

    // TODO: maybe only make this hidden and expose a filter method that takes in some sort of
    // filter trait
    ///
    /// # Errors
    /// If there filter [f] filters out all the results of this file
    pub fn filter(&self, f: fn(&Node<'_>) -> bool) -> Result<Self, NoResultsForFilter> {
        let root = self.tree.root_node();
        let ranges: Box<[Range]> = self
            .ranges()
            .filter_map(|range| root.descendant_for_point_range(range.start_point, range.end_point))
            .filter(f)
            .map(|n| n.range())
            .collect();
        if ranges.is_empty() {
            return Err(NoResultsForFilter);
        }
        let clone = Self {
            results: ranges,
            ..self.clone()
        };
        Ok(clone)
    }

    #[must_use]
    pub fn language(&self) -> &str {
        self.language_type
    }

    #[must_use]
    pub fn search_name(&self) -> &str {
        self.function_name
    }

    fn ranges(&self) -> impl Iterator<Item = &Range> {
        self.results.iter()
    }
}

///
/// # Errors
pub fn search_file<'a>(
    code: &'a str,
    language: &'a dyn SupportedLanguage,
    name: &'a str,
) -> Result<ParsedFile<'a>, Error> {
    let code_bytes = code.as_bytes();
    let mut parser = tree_sitter::Parser::new();
    let ts_lang = language.language();
    parser
        .set_language(ts_lang)
        .map_err(|lang_err| Error::GrammarLoad(language.name(), lang_err))?;
    let parsed = parser
        .parse(code, None)
        .ok_or_else(|| Error::ParseError(code.to_string()))?;

    let query_str = language.query(name);
    let node = parsed.root_node();
    let command_ranges = run_query(&query_str, ts_lang, node, code_bytes)
        .map_err(|query_err| Error::InvalidQuery(language.name(), query_err))?;

    Ok(ParsedFile::new(
        code,
        name,
        language.name(),
        parsed,
        command_ranges,
    ))
}
