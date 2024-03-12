#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(missing_debug_implementations, clippy::missing_panics_doc)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
use supported_languages::SupportedLanguage;
use tree_sitter::{Language, LanguageError, Node, QueryError, Range};

pub mod supported_languages;

fn run_query(
    query_str: &str,
    lang: Language,
    node: &Node<'_>,
    code: &[u8],
) -> Result<Vec<Range>, QueryError> {
    let query = tree_sitter::Query::new(lang, query_str)?;
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, *node, code);
    let ranges = matches.map(|m| m.captures[0].node.range());
    Ok(ranges.collect::<Vec<_>>())
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
fn get_file_type_from_ext(
    ext: &str,
    langs: &[impl SupportedLanguage],
) -> Result<impl SupportedLanguage, Error> {
    langs
        .iter()
        .find(|lang| lang.file_exts().contains(&ext))
        .copied()
        .ok_or_else(||Error::FileTypeUnkown(ext.to_string()))
}

///
/// # Errors
pub fn get_file_type_from_file(
    file: &str,
    langs: &[impl SupportedLanguage],
) -> Result<impl SupportedLanguage, Error> {
    file.rsplit_once('.')
        .ok_or_else(|| Error::FileTypeUnkown(file.to_string()))
        .map(|(_, ext)| ext)
        .and_then(|ext| get_file_type_from_ext(ext, langs))
}


///
/// # Errors
pub fn search_file<'a>(
    code: &'a str,
    language: impl SupportedLanguage,
    name: &'a str,
) -> Result<impl Iterator<Item = (String, Range)> + 'a, Error> {
    let code_bytes = code.as_bytes();
    let mut parser = tree_sitter::Parser::new();
    let ts_lang = language.language();
    parser
        .set_language(ts_lang)
        .map_err(|lang_err| Error::GrammarLoad(language.name(), lang_err))?;
    let parsed = parser
        .parse(code, None)
        .ok_or_else(|| Error::ParseError(code.to_string()))?;

    let binding = parsed.root_node();
    let query_str = language.query(name);
    let command_ranges = run_query(&query_str, ts_lang, &binding, code_bytes)
        .map_err(|query_err| Error::InvalidQuery(language.name(), query_err))?;

    let lines = code.lines().enumerate();
    let texts = command_ranges.into_iter().map(move |range| {
        (
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
                .join("\n"),
            range,
        )
    });
    Ok(texts)
}
