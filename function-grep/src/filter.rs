use general_filters::{FunctionInImpl, FunctionInLines, FunctionWithParameter};
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
        Ok(InstantiatedFilter {
            filter_information: self.filter_info(),
            filter_function: filter,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilterInformation {
    /// the name of the filter (so users can find the filter)
    filter_name: String,
    /// describes what the filter does and how it parses
    description: String,
    /// what languages this filter works on
    supported_languages: SupportedLanguages,

    attributes: HashMap<Attribute, AttributeType>,
}

impl fmt::Display for FilterInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter {}", self.filter_name)
    }
}

impl Hash for FilterInformation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filter_name.hash(state);
        self.description.hash(state);
        self.supported_languages.hash(state);
    }
}
impl PartialEq for FilterInformation {
    fn eq(&self, other: &Self) -> bool {
        self.filter_name == other.filter_name
            && self.description == other.description
            && self.supported_languages == other.supported_languages
    }
}
impl Eq for FilterInformation {}
impl FilterInformation {
    #[must_use]
    pub const fn supported_languages(&self) -> &SupportedLanguages {
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
type FilterFunction = Box<dyn Fn(&Node<'_>) -> bool + Send + Sync>;

// TODO: make our own FromStr that also requires the proggramer to sepcify that attributes each
// filter has and their type so that we can make macro that creates parser, and also so that we can
// communicate to a gui (or tui) that labels, and types of each input
pub struct InstantiatedFilter {
    filter_information: FilterInformation,
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

macro_rules! default_filters {
    ($($filter:ident),*) => {
        HashMap::from([$(($filter.filter_info().filter_name().to_string(), &$filter as &'static dyn Filter)),*])
    };
}
pub enum FilterType<'a> {
    All(&'a dyn Filter),
    Many(HashMap<String, &'a dyn Filter>),
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
                    FilterType::All(&FunctionInLines as &'static dyn Filter),
                ),
                (
                    FunctionInImpl.filter_info().filter_name().to_string(),
                    FilterType::Many(HashMap::from([(
                        "Rust".to_string(),
                        &FunctionInImpl as &'static dyn Filter,
                    )])),
                ),
                (
                    FunctionWithParameter
                        .filter_info()
                        .filter_name()
                        .to_string(),
                    FilterType::Many(
                        match FunctionWithParameter.filter_info().supported_languages() {
                            SupportedLanguages::All => unreachable!(),
                            SupportedLanguages::Many(vec) => {
                                HashMap::from_iter(vec.into_iter().map(|lang| {
                                    (
                                        lang.to_string(),
                                        &FunctionWithParameter as &'static dyn Filter,
                                    )
                                }))
                            }
                            SupportedLanguages::Single(_) => unreachable!(),
                        },
                    ),
                ),
            ]),
        }
    }
}
impl<'a> Filters<'a> {
    pub fn add_filter(&mut self, filter: &'a dyn Filter) -> Result<(), String> {
        let result = self
            .filters
            .get_mut(&filter.filter_name())
            .map(|filters| match filters {
                FilterType::All(_) => Err("cannot add to an all filter".to_string()),
                FilterType::Many(hash_map) => merge_filters(hash_map, filter),
            })
            .or_else(|| {
                self.filters
                    .insert(filter.filter_name(), to_filter_type(filter));
                Some(Ok(()))
            });
        match result {
            Some(res) => res,
            None => Ok(()),
        }
    }
}

fn merge_filters<'a>(
    hash_map: &mut HashMap<String, &'a dyn Filter>,
    filter: &'a dyn Filter,
) -> Result<(), String> {
    todo!()
}

fn to_filter_type<'a>(filter: &'a dyn Filter) -> FilterType<'a> {
    todo!()
}
