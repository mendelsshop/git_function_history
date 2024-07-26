use std::collections::HashMap;

use tree_sitter::Node;

pub enum AttributeType {
    String,
    Number,
    Boolean,
    Other(String),
}

#[derive(Hash, Eq, PartialEq)]
pub struct Attribute(String);

use crate::SupportedLanguages;
pub trait Filter: HasFilterInformation {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String>;
    fn to_filter(&self, s: &str) -> Result<InstantiatedFilter, String> {
        let filter = self.parse_filter(s)?;
        Ok(InstantiatedFilter {
            filter_information: self.filter_info(),
            filter_function: filter,
        })
    }
}
pub struct FilterInformation {
    /// the name of the filter (so users can find the filter)
    filter_name: String,
    /// describes what the filter does and how it parses
    description: String,
    /// what languages this filter works on
    supported_languages: SupportedLanguages,

    attributes: HashMap<Attribute, AttributeType>,
}

impl FilterInformation {
    pub const fn supported_languages(&self) -> &SupportedLanguages {
        &self.supported_languages
    }

    pub const fn attributes(&self) -> &HashMap<Attribute, AttributeType> {
        &self.attributes
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn filter_name(&self) -> &str {
        &self.filter_name
    }
}
pub trait HasFilterInformation {
    /// the name of the filter (so users can find the filter)
    fn filter_name(&self) -> String;
    /// describes what the filter does and how it parses
    fn description(&self) -> String;
    /// what languages this filter works on
    fn supported_languages(&self) -> SupportedLanguages;
    fn attributes(&self) -> HashMap<Attribute, AttributeType>;
    // TODO: have filter creation informaton about types and fields for uis
    fn filter_info(&self) -> FilterInformation {
        FilterInformation {
            filter_name: self.filter_name(),
            attributes: self.attributes(),
            description: self.description(),
            supported_languages: self.supported_languages(),
        }
    }
}
type FilterFunction = Box<dyn FnMut(&Node<'_>) -> bool>;

// TODO: make our own FromStr that also requires the proggramer to sepcify that attributes each
// filter has and their type so that we can make macro that creates parser, and also so that we can
// communicate to a gui (or tui) that labels, and types of each input
pub struct InstantiatedFilter {
    filter_information: FilterInformation,
    filter_function: FilterFunction,
}

impl InstantiatedFilter {
    pub fn filter(&mut self, node: &Node<'_>) -> bool {
        (self.filter_function)(node)
    }

    pub const fn attributes(&self) -> &HashMap<Attribute, AttributeType> {
        self.filter_information.attributes()
    }

    pub fn description(&self) -> &str {
        self.filter_information.description()
    }

    pub fn filter_name(&self) -> &str {
        self.filter_information.filter_name()
    }

    pub const fn supported_languages(&self) -> &SupportedLanguages {
        self.filter_information.supported_languages()
    }
}

pub struct FunctionInLines;

fn number<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    position: &str,
) -> Result<usize, String> {
    substring.next().ok_or_else(||format! ("invalid options for function_in_lines filter\nexpected {format}\n missing {position} [number]"))
                .and_then(|end| end.parse().map_err(|_| format! ("invalid options for function_in_lines filter\nexpected {format}\n cannot parse {position} [number]")))
}
fn extra<'a>(substring: &mut impl Iterator<Item = &'a str>, format: &str) -> Result<(), String> {
    substring.next().map_or(Ok(()), |extra| Err(format!( "invalid options for function_in_lines filter\nexpected {format}\n, found extra stuff after {format}: {extra}")))
}
fn label<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    label: &str,
) -> Result<(), String> {
    substring.next().ok_or_else(||format! ("invalid options for function_in_lines filter\nexpected {format}\n missing label {label}"))
                .and_then(|l| {
                    if label == l {
                        Ok(())
                    } else {
                        Err(format!("invalid options for function_in_lines filter\n expected {format}\nexpected {label}, found {l}")) 
                    }
                }
        )
}
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
    fn filter_name(&self) -> String {
        "function_in_lines".to_string()
    }
    fn supported_languages(&self) -> SupportedLanguages {
        SupportedLanguages::All
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
        HashMap::from([(Attribute("start".to_string()), AttributeType::Number)])
    }
}
