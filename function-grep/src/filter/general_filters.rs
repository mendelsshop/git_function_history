use std::collections::HashMap;

use tree_sitter::Node;

use super::{
    filter_parsers::{extra, label, number},
    All, Attribute, AttributeType, Filter, FilterFunction, HasFilterInformation, Language,
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
        Ok(Box::new(move |node: &Node<'_>| {
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
    fn attributes(&self) -> HashMap<Attribute, AttributeType> {
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

        Ok(Box::new(move |node: &Node<'_>| {
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

    fn attributes(&self) -> HashMap<Attribute, AttributeType> {
        HashMap::new()
    }
}

pub struct FunctionWithParameterRust;

impl HasFilterInformation for FunctionWithParameterRust {
    fn filter_name(&self) -> String {
        "function_with_parameter".to_string()
    }

    fn description(&self) -> String {
        todo!()
    }

    fn supports(&self) -> Self::Supports {
        Language("Rust".to_owned())
    }

    fn attributes(&self) -> HashMap<Attribute, AttributeType> {
        todo!()
    }

    type Supports = Language;
}

impl Filter for FunctionWithParameterRust {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String> {
        todo!()
    }
}
