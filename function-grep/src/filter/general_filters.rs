use std::collections::{HashMap, HashSet};

use tree_sitter::{Node, Query, QueryCursor};

use super::{
    filter_parsers::{extra, label, number, string},
    All, Attribute, AttributeType, Attributes, Filter, FilterFunction, HasFilterInformation,
    Language,
};

pub struct FunctionInLines;
impl FunctionInLines {
    fn from_str(s: &str) -> Result<(usize, usize), String> {
        let mut substring = s.split(' ').filter(|s| *s != " ");
        let filter = "function_with_parameters";
        let fst = substring.next().ok_or("invalid options for function_in_lines filter\nexpected [number] [number], start: [number] end: [number], or end: [number] start: [number]")?;
        match fst {
            "start:" => {
                let format = "start: [number] end: [number]";
                let start = number(&mut substring, format, "start:", filter)?;
                label(&mut substring, format, "end:", filter)?;
                let end = number(&mut substring, format, "end:", filter)?;
                extra(&mut substring, format, filter)?;
                Ok((start, end))
            }
            "end:" => {
                let format = "end: [number] start: [number]";
                let end = number(&mut substring, format, "end:", filter)?;
                label(&mut substring, format, "start:", filter)?;
                let start = number(&mut substring, format, "start:", filter)?;
                extra(&mut substring, format, filter)?;
                Ok((start, end))
            }
            n => {
                if let Ok(start) = n.parse() {
                    let end = number(&mut substring, "[number] [number]", "second", filter)?;
                    extra(&mut substring, "[number] [number]", filter)?;
                    Ok((start, end))
                } else {
                    Err(format!("invalid options for function_in_lines filter\nexpected [number] [number], start: [number] end: [number], or end: [number] start: [number]\ngiven {n}"))
                }
            }
        }
    }
}
impl Filter for FunctionInLines {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String> {
        let (start, end) = Self::from_str(s)?;
        Ok(Box::new(move |node: &Node<'_>, _| {
            node.range().start_point.row >= start && node.range().end_point.row <= end
        }))
    }
}
impl HasFilterInformation for FunctionInLines {
    type Supports = All;
    fn filter_name(&self) -> String {
        "function_in_lines".to_string()
    }
    fn supports(&self) -> Self::Supports {
        All
    }
    fn description(&self) -> String {
        "filter: function_in_lines
filters to only functions within the specified lines
format:
\t[number] [number]
\tstart: [number] end: [number]
\tend: [number] start: [number]"
            .to_string()
    }
    fn attributes(&self) -> Attributes {
        HashMap::from([
            (Attribute("start".to_string()), AttributeType::Number),
            (Attribute("end".to_string()), AttributeType::Number),
        ])
    }
}

pub struct FunctionInImpl;

impl Filter for FunctionInImpl {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String> {
        if !s.is_empty() {
            return Err(format!("invalid options for function_in_impl, this filter accepts not options, but got {s}"));
        }

        Ok(Box::new(move |node: &Node<'_>, _| {
            node.parent().is_some_and(|parent| {
                parent
                    .parent()
                    .is_some_and(|parent| parent.grammar_name() == "impl_item")
            })
        }))
    }
}

impl HasFilterInformation for FunctionInImpl {
    type Supports = Language;
    fn filter_name(&self) -> String {
        "function_in_impl".to_string()
    }

    fn description(&self) -> String {
        "find if any functions are in an impl block".to_string()
    }

    fn supports(&self) -> Self::Supports {
        Language("Rust".to_string())
    }

    fn attributes(&self) -> Attributes {
        HashMap::new()
    }
}

pub struct TreeSitterQueryFilter;
impl HasFilterInformation for TreeSitterQueryFilter {
    fn filter_name(&self) -> String {
        "tree_sitter_query".to_string()
    }

    fn description(&self) -> String {
        "filter using an arbritrary tree sitter query".to_string()
    }

    fn supports(&self) -> Self::Supports {
        All
    }

    fn attributes(&self) -> Attributes {
        HashMap::from([(Attribute("query".to_string()), AttributeType::String)])
    }

    type Supports = All;
}
impl Filter for TreeSitterQueryFilter {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String> {
        let mut languages: HashMap<tree_sitter::Language, Result<Query, ()>> = HashMap::new();
        let query: String = {
            let mut substring = s.split(' ').filter(|s| *s != " ").peekable();
            let fst = substring.peek().ok_or(
                "invalid options for tree_sitter_query filter\nexpected [string] or name: [string]",
            )?;
            match *fst {
                "query:" => {
                    substring.next();
                    substring.collect::<Vec<_>>().join(" ")
                }

                _ => substring.collect::<Vec<_>>().join(" "),
            }
        };
        Ok(Box::new(move |node: &Node<'_>, code| {
            let language = node.language();
            let language: &tree_sitter::Language = &language;
            let mut cursor = QueryCursor::new();
            cursor.set_max_start_depth(Some(0));
            let text_provider = code.as_bytes();
            match languages
                .entry(language.clone())
                .or_insert(Query::new(language, &query).map_err(|_| ()))
            {
                Ok(query) => cursor
                    .matches(&query, *node, text_provider)
                    .next()
                    .is_some(),
                Err(_) => false,
            }
        }))
    }
}
pub struct FunctionWithParameterRust;
pub struct FunctionWithParameterPython;

impl HasFilterInformation for FunctionWithParameterRust {
    fn filter_name(&self) -> String {
        "function_with_parameter".to_string()
    }

    fn description(&self) -> String {
        "Find a function with a given parameter".to_string()
    }

    fn supports(&self) -> Self::Supports {
        Language("Rust".to_owned())
    }

    fn attributes(&self) -> Attributes {
        HashMap::from([(Attribute("name".to_string()), AttributeType::String)])
    }

    type Supports = Language;
}

impl Filter for FunctionWithParameterRust {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String> {
        let query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            "((function_item
  parameters: (parameters (parameter pattern: (identifier) @param)))
)
((let_declaration
  value: (closure_expression
  parameters: (closure_parameters [((identifier) @param)
                                   (parameter pattern: (identifier) @param)])
  ))
)
((const_item
  value: (closure_expression
  (closure_parameters [((identifier) @param)
                                   (parameter pattern: (identifier) @param)])
  )) 
)
((static_item
  value: (closure_expression
  (closure_parameters [((identifier) @param)
                                   (parameter pattern: (identifier) @param)])
  )) 
)",
        )
        .map_err(|_| String::from("Could not create tree sitter filter"))?;
        let name = parse_with_param(s)?;
        Ok(Box::new(move |node: &Node<'_>, code| {
            let mut cursor = QueryCursor::new();
            cursor.set_max_start_depth(Some(0));
            let text_provider = code.as_bytes();
            cursor.matches(&query, *node, text_provider).any(|c| {
                c.captures
                    .iter()
                    .any(|c| c.node.utf8_text(text_provider).unwrap_or("") == name)
            })
        }))
    }
}

impl HasFilterInformation for FunctionWithParameterPython {
    fn filter_name(&self) -> String {
        "function_with_parameter".to_string()
    }

    fn description(&self) -> String {
        "Find a function with a given parameter".to_string()
    }

    fn supports(&self) -> Self::Supports {
        Language("Python".to_owned())
    }

    fn attributes(&self) -> Attributes {
        HashMap::from([(Attribute("name".to_string()), AttributeType::String)])
    }

    type Supports = Language;
}

impl Filter for FunctionWithParameterPython {
    fn parse_filter(&self, _s: &str) -> Result<FilterFunction, String> {
        todo!()
    }
}

fn parse_with_param(s: &str) -> Result<String, String> {
    let mut substring = s.split(' ').filter(|s| *s != " ");
    let fst = substring.next().ok_or(
        "invalid options for function_with_parameter filter\nexpected [string] or name: [string]",
    )?;
    let filter = "function_with_parameters";
    match fst {
        "name:" => {
            let format = "name: [string]";
            let name = string(&mut substring, format, "name:", filter)?;
            extra(&mut substring, format, filter)?;
            Ok(name)
        }

        name => {
            extra(&mut substring, "[string]", filter)?;
            Ok(name.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        filter::Filter,
        supported_languages::{Rust, SupportedLanguage},
        ParsedFile,
    };

    use super::TreeSitterQueryFilter;

    #[test]
    fn tree_sitter_query() {
        let file = ParsedFile::search_file(
            include_str!("../../../git-function-history-lib/src/test_functions.rs"),
            &Rust.to_language("empty_test").unwrap(),
        )
        .unwrap();
        println!("original:\n{file}");
        let mut filter = TreeSitterQueryFilter
            .to_filter(" (function_item type_parameters: (type_parameters))")
            .unwrap();

        let filtered = file.filter_inner(&mut filter);
        assert!(filtered.is_ok());
        let filtered = filtered.unwrap();
        println!("filtered:\n{filtered}");
    }
    #[test]
    fn tree_sitter_query_with_predicate() {
        let file = ParsedFile::search_file(
            include_str!("../../../git-function-history-lib/src/test_functions.rs"),
            &Rust.to_language("empty_test").unwrap(),
        )
        .unwrap();
        println!("original:\n{file}");
        let mut filter = TreeSitterQueryFilter
            .to_filter(" (function_item type_parameters: (type_parameters (type_identifier) @type)   (#eq? @type \"T\")
)")
            .unwrap();

        let filtered = file.filter_inner(&mut filter);
        assert!(filtered.is_ok());
        let filtered = filtered.unwrap();
        println!("filtered:\n{filtered}");
    }
}
