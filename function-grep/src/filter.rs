use general_filters::{FunctionInImpl, FunctionInLines, FunctionWithParameterRust};
use std::{collections::HashMap, fmt, hash::Hash};

mod filter_parsers;
mod general_filters;
use tree_sitter::Node;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum AttributeType {
    String,
    Number,
    Boolean,
    Other(String),
}

impl fmt::Display for AttributeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "String"),
            Self::Number => write!(f, "Number"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Other(arg0) => write!(f, "{arg0}"),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Attribute(String);
impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use crate::SupportedLanguages;
pub trait Filter: HasFilterInformation {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String>;
    // TODO: make way to parse from hashmap of attribute to string
    fn to_filter(&self, s: &str) -> Result<InstantiatedFilter, String> {
        let filter = self.parse_filter(s)?;
        let info = self.filter_info();
        let info = FilterInformation {
            supported_languages: info.supported_languages.into(),
            description: info.description,
            attributes: info.attributes,
            filter_name: info.filter_name,
        };
        Ok(InstantiatedFilter {
            filter_information: info,
            filter_function: filter,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilterInformation<Supports> {
    /// the name of the filter (so users can find the filter)
    filter_name: String,
    /// describes what the filter does and how it parses
    description: String,
    /// what languages this filter works on
    supported_languages: Supports,

    attributes: HashMap<Attribute, AttributeType>,
}

impl<Supports> fmt::Display for FilterInformation<Supports> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter {}", self.filter_name)
    }
}

impl<Supports: Hash> Hash for FilterInformation<Supports> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filter_name.hash(state);
        self.description.hash(state);
        self.supported_languages.hash(state);
    }
}
impl<Supports: PartialEq> PartialEq for FilterInformation<Supports> {
    fn eq(&self, other: &Self) -> bool {
        self.filter_name == other.filter_name
            && self.description == other.description
            && self.supported_languages == other.supported_languages
    }
}
impl<Supports: PartialEq> Eq for FilterInformation<Supports> {}
impl<Supports> FilterInformation<Supports> {
    #[must_use]
    pub const fn supported_languages(&self) -> &Supports {
        &self.supported_languages
    }

    #[must_use]
    pub const fn attributes(&self) -> &HashMap<Attribute, AttributeType> {
        &self.attributes
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn filter_name(&self) -> &str {
        &self.filter_name
    }
}
pub trait HasFilterInformation {
    type Supports: Into<SupportedLanguages>;
    /// the name of the filter (so users can find the filter)
    fn filter_name(&self) -> String;
    /// describes what the filter does and how it parses
    fn description(&self) -> String;
    /// what languages this filter works on
    fn supports(&self) -> Self::Supports;
    fn attributes(&self) -> HashMap<Attribute, AttributeType>;
    // TODO: have filter creation informaton about types and fields for uis
    fn filter_info(&self) -> FilterInformation<Self::Supports> {
        FilterInformation {
            filter_name: self.filter_name(),
            attributes: self.attributes(),
            description: self.description(),
            supported_languages: self.supports(),
        }
    }
}
type FilterFunction = Box<dyn Fn(&Node<'_>) -> bool + Send + Sync>;

// TODO: make our own FromStr that also requires the proggramer to sepcify that attributes each
// filter has and their type so that we can make macro that creates parser, and also so that we can
// communicate to a gui (or tui) that labels, and types of each input
pub struct InstantiatedFilter {
    filter_information: FilterInformation<SupportedLanguages>,
    filter_function: FilterFunction,
}

impl PartialEq for InstantiatedFilter {
    fn eq(&self, other: &Self) -> bool {
        self.filter_information == other.filter_information
    }
}
impl Eq for InstantiatedFilter {}

impl fmt::Display for InstantiatedFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filter_information)
    }
}
impl std::fmt::Debug for InstantiatedFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstantiatedFilter")
            .field("filter_information", &self.filter_information)
            .finish()
    }
}

impl InstantiatedFilter {
    #[must_use]
    pub fn filter(&self, node: &Node<'_>) -> bool {
        (self.filter_function)(node)
    }

    #[must_use]
    pub const fn attributes(&self) -> &HashMap<Attribute, AttributeType> {
        self.filter_information.attributes()
    }

    #[must_use]
    pub fn description(&self) -> &str {
        self.filter_information.description()
    }

    #[must_use]
    pub fn filter_name(&self) -> &str {
        self.filter_information.filter_name()
    }

    #[must_use]
    pub const fn supported_languages(&self) -> &SupportedLanguages {
        self.filter_information.supported_languages()
    }
}

pub struct All;
impl From<All> for SupportedLanguages {
    fn from(_: All) -> Self {
        Self::All
    }
    // add code here
}
pub struct Language(String);
impl From<Language> for SupportedLanguages {
    fn from(value: Language) -> Self {
        Self::Single(value.0)
    }
}
// impl Language for
macro_rules! default_filters {
    ($($filter:ident),*) => {S
        HashMap::from([$(($filter.filter_info().filter_name().to_string(), &$filter as &'static dyn Filter)),*])
    };
}
pub struct Many<'a> {
    pub name: String,
    pub filters: HashMap<String, &'a dyn Filter<Supports = Language>>,
}

pub enum FilterType<'a> {
    All(&'a dyn Filter<Supports = All>),
    Many(Many<'a>),
}
impl<'a> FilterType<'a> {
    fn filter_name(&self) -> String {
        todo!()
    }

    fn supports(&self) -> SupportedLanguages {
        todo!()
    }

    // add code here
}
pub struct Filters<'a> {
    filters: HashMap<String, FilterType<'a>>,
}

impl Filters<'static> {
    pub fn default() -> Self {
        Self {
            filters: HashMap::from([
                (
                    FunctionInLines.filter_info().filter_name().to_string(),
                    FilterType::All(&FunctionInLines as &'static dyn Filter<Supports = All>),
                ),
                (
                    FunctionInImpl.filter_info().filter_name().to_string(),
                    FilterType::Many(Many {
                        name: "function_in_impl".to_string(),
                        filters: HashMap::from([(
                            "Rust".to_string(),
                            &FunctionInImpl as &'static dyn Filter<Supports = Language>,
                        )]),
                    }),
                ),
                (
                    FunctionWithParameterRust
                        .filter_info()
                        .filter_name()
                        .to_string(),
                    FilterType::Many(Many {
                        name: "function_with_parameter".to_string(),
                        filters: HashMap::from([(
                            FunctionWithParameterRust.supports().0,
                            &FunctionWithParameterRust as &'static dyn Filter<Supports = Language>,
                        )]),
                    }),
                ),
            ]),
        }
    }
}

impl<'a> Filters<'a> {
    pub fn add_filter(&mut self, filter: impl Into<FilterType<'a>>) -> Result<(), String> {
        let filter = filter.into();
        let name = filter.filter_name().clone();
        {
            let this = self.filters.get_mut(&name);
            match this {
                Some(filters) => match filters {
                    FilterType::All(_) => Err("cannot add to an all filter".to_string()),
                    FilterType::Many(Many { filters, .. }) => merge_filters(filters, filter),
                },
                None => {
                    self.filters.insert(name, filter);
                    Ok(())
                }
            }
        }
    }
}

fn merge_filters<'a>(
    hash_map: &mut HashMap<String, &'a dyn Filter<Supports = Language>>,
    filter: FilterType<'a>,
) -> Result<(), String> {
    todo!()
}
