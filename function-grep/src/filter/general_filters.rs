use std::collections::HashMap;

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
        let fst = substring.next().ok_or("invalid options for function_in_lines filter\nexpected [number] [number], start: [number] end: [number], or end: [number] start: [number]")?;
        match fst {
            "start:" => {
                let format = "start: [number] end: [number]";
                let start = number(&mut substring, format, "start:")?;
                label(&mut substring, format, "end:")?;
                let end = number(&mut substring, format, "end:")?;
                extra(&mut substring, format)?;
                Ok((start, end))
            }
            "end:" => {
                let format = "end: [number] start: [number]";
                let end = number(&mut substring, format, "end:")?;
                label(&mut substring, format, "start:")?;
                let start = number(&mut substring, format, "start:")?;
                extra(&mut substring, format)?;
                Ok((start, end))
            }
            n => {
                if let Ok(start) = n.parse() {
                    let end = number(&mut substring, "[number] [number]", "second")?;
                    extra(&mut substring, "[number] [number]")?;
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
        todo!()
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
                    .any(|c| c.node.utf8_text(text_provider).unwrap() == name)
            })
        }))
    }
}

fn parse_with_param(s: &str) -> Result<String, String> {
    let mut substring = s.split(' ').filter(|s| *s != " ");
    let fst = substring.next().ok_or(
        "invalid options for function_in_lines filter\nexpected [string] or name: [string]",
    )?;
    match fst {
        "start:" => {
            let format = "name: [string]";
            let name = string(&mut substring, format, "name:")?;
            extra(&mut substring, format)?;
            Ok(name)
        }

        name => {
            extra(&mut substring, "[string]")?;
            Ok(name.to_string())
        }
    }
}
