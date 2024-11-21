use general_filters::{FunctionInImpl, FunctionInLines, FunctionWithParameterRust};
use std::{
    collections::{hash_map, HashMap},
    fmt::{self, Display},
    hash::Hash,
};

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
// TODO: should we have way to specify an attribute as being optional
pub type Attributes = HashMap<Attribute, AttributeType>;
use crate::SupportedLanguages;
pub trait Filter: HasFilterInformation {
    fn parse_filter(&self, s: &str) -> Result<FilterFunction, String>;
    // TODO: make way to parse from hashmap of attribute to string
    fn to_filter(&self, s: &str) -> Result<InstantiatedFilter<Self::Supports>, String> {
        let filter = self.parse_filter(s)?;
        let info = self.filter_info();
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

    attributes: Attributes,
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
    pub const fn attributes(&self) -> &Attributes {
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
    fn attributes(&self) -> Attributes;
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
// TODO: make something that builds on this like FilterType
pub struct InstantiatedFilter<Supports> {
    filter_information: FilterInformation<Supports>,
    filter_function: FilterFunction,
}

impl<Supports: PartialEq> PartialEq for InstantiatedFilter<Supports> {
    fn eq(&self, other: &Self) -> bool {
        self.filter_information == other.filter_information
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl<Supports: PartialEq> Eq for InstantiatedFilter<Supports> {}

impl<Supports> fmt::Display for InstantiatedFilter<Supports> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filter_information)
    }
}
impl<Supports: std::fmt::Debug> std::fmt::Debug for InstantiatedFilter<Supports> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstantiatedFilter")
            .field("filter_information", &self.filter_information)
            .finish()
    }
}

impl<Supports> InstantiatedFilter<Supports> {
    #[must_use]
    pub fn filter(&self, node: &Node<'_>) -> bool {
        (self.filter_function)(node)
    }

    #[must_use]
    pub const fn attributes(&self) -> &Attributes {
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
    pub const fn supported_languages(&self) -> &Supports {
        self.filter_information.supported_languages()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct All;
impl From<All> for SupportedLanguages {
    fn from(_: All) -> Self {
        Self::All
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Language(String);
impl From<Language> for SupportedLanguages {
    fn from(value: Language) -> Self {
        Self::Single(value.0)
    }
}
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
// impl Language for
// TODO: rework macro to work with the new language per filter system
macro_rules! default_filters {
    ($($filter:ident),*) => {S
        HashMap::from([$(($filter.filter_info().filter_name().to_string(), &$filter as &'static dyn Filter)),*])
    };
}
#[derive(Debug, PartialEq, Eq)]
pub struct Many<T: Info<Supported = Language>> {
    pub name: String,
    pub filters: HashMap<String, T>,
}

// TODO: merge with HasFilterInformation
pub trait Info {
    type Supported;
    fn filter_name(&self) -> String;
}
// TODO: parameterize over something that is hkt might not work

pub type FilterType<'a> =
    SingleOrMany<&'a dyn Filter<Supports = All>, &'a dyn Filter<Supports = Language>>;
pub type InstantiatedFilterType =
    SingleOrMany<InstantiatedFilter<All>, InstantiatedFilter<Language>>;
#[derive(Debug, PartialEq, Eq)]
pub enum SingleOrMany<A: Info<Supported = All>, M: Info<Supported = Language>> {
    All(A),
    Many(Many<M>),
    Single(M),
}
impl<A: Info<Supported = All>, M: Info<Supported = Language>> SingleOrMany<A, M> {
    #[must_use]
    pub fn filter_name(&self) -> String {
        match self {
            Self::All(f) => f.filter_name(),
            Self::Many(many) => many.name.clone(),
            Self::Single(s) => s.filter_name(),
        }
    }

    #[must_use]
    pub fn supports(&self) -> SupportedLanguages {
        match self {
            Self::All(_) => SupportedLanguages::All,
            Self::Many(many) => SupportedLanguages::Many(many.filters.keys().cloned().collect()),
            // TODO: make Info carry not just the supported type but also a value for it
            Self::Single(_s) => todo!(),
        }
    }
}
impl<A: Info<Supported = All>, M: Info<Supported = Language>> Display for SingleOrMany<A, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.filter_name())
    }
}
impl<'a> FilterType<'a> {
    #[must_use]
    pub fn specific(&self, s: &str) -> Option<Self> {
        match self {
            Self::All(_) | Self::Single(_) => None,
            Self::Many(m) => m.filters.get(s).map(|x| Self::Single(*x)),
        }
    }

    pub fn to_filter(&self, s: &str) -> Result<InstantiatedFilterType, String> {
        match self {
            SingleOrMany::All(a) => a.to_filter(s).map(SingleOrMany::All),
            SingleOrMany::Single(a) => a.to_filter(s).map(SingleOrMany::Single),
            SingleOrMany::Many(Many { name, filters }) => filters
                .iter()
                .map(|(name, f)| f.to_filter(s).map(|f| (name.clone(), f)))
                .collect::<Result<HashMap<_, _>, _>>()
                .map(|filters| {
                    SingleOrMany::Many(Many {
                        name: name.to_string(),
                        filters,
                    })
                }),
        }
    }
}
pub struct Filters<'a> {
    filters: HashMap<String, FilterType<'a>>,
}
impl<T> Info for &dyn Filter<Supports = T>
where
    SupportedLanguages: From<T>,
{
    type Supported = T;

    fn filter_name(&self) -> String {
        self.filter_info().filter_name().to_string()
    }
}
impl<T> Info for InstantiatedFilter<T>
where
    SupportedLanguages: From<T>,
{
    type Supported = T;

    fn filter_name(&self) -> String {
        self.filter_name().to_string()
    }
}
impl<'a> IntoIterator for Filters<'a> {
    type Item = (String, FilterType<'a>);

    type IntoIter = hash_map::IntoIter<String, FilterType<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.filters.into_iter()
    }
}

impl Filters<'static> {
    #[must_use]
    pub fn default() -> Self {
        Self {
            filters: HashMap::from([
                (
                    FunctionInLines.filter_info().filter_name().to_string(),
                    SingleOrMany::All(&FunctionInLines as &'static dyn Filter<Supports = All>),
                ),
                (
                    FunctionInImpl.filter_info().filter_name().to_string(),
                    SingleOrMany::Many(Many {
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
                    SingleOrMany::Many(Many {
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
        let filter: FilterType<'a> = filter.into();
        let name = filter.filter_name();
        if let Some(filters) = self.filters.get_mut(&name) {
            match filters {
                SingleOrMany::All(_) => Err("cannot add with or to an all filter".to_string()),
                _ if matches!(filter, SingleOrMany::All(_)) => {
                    Err("cannot add with or to an all filter".to_string())
                }
                SingleOrMany::Many(filter1) => match filter {
                    SingleOrMany::All(_) => unreachable!(),
                    SingleOrMany::Many(filter2) => try_extend_filter(filter1, filter2),
                    SingleOrMany::Single(filter2) => try_add_filter(filter1, filter2),
                },
                SingleOrMany::Single(filter1) => match filter {
                    SingleOrMany::All(_) => unreachable!(),
                    SingleOrMany::Many(mut filter2) => {
                        try_add_filter(&mut filter2, *filter1)?;
                        *filters = SingleOrMany::Many(filter2);
                        Ok(())
                    }
                    SingleOrMany::Single(filter2) if filter1.supports() == filter2.supports() => {
                        Err(format!(
                            "cannot add duplicate filter {name} for {}",
                            filter1.supports()
                        ))
                    }
                    SingleOrMany::Single(filter2) => {
                        *filters = SingleOrMany::Many(Many {
                            name,
                            filters: HashMap::from([
                                (filter1.supports().0, *filter1),
                                (filter2.supports().0, filter2),
                            ]),
                        });
                        Ok(())
                    }
                },
            }
        } else {
            self.filters.insert(name, filter);
            Ok(())
        }
    }
    #[must_use]
    pub fn get_filter(&self, name: &str) -> Option<&FilterType<'a>> {
        self.filters.get(name)
    }
}

fn try_add_filter<'a>(
    filters: &mut Many<&'a (dyn Filter<Supports = Language>)>,
    filter: &'a (dyn Filter<Supports = Language>),
) -> Result<(), String> {
    let mut status = Ok(());
    filters
        .filters
        .entry(filter.supports().0)
        .and_modify(|_| {
            status = Err(format!(
                "cannot add duplicate filter {} for {}",
                filter.filter_name(),
                filter.supports()
            ));
        })
        .or_insert(filter);
    status
}

fn try_extend_filter<'a>(
    filters: &mut Many<&'a (dyn Filter<Supports = Language>)>,
    new_filters: Many<&'a (dyn Filter<Supports = Language>)>,
) -> Result<(), String> {
    new_filters
        .filters
        .into_values()
        .try_for_each(|filter| try_add_filter(filters, filter))
}
