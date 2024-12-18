use std::{ops::Deref, str, sync::atomic::AtomicUsize};
use tree_sitter::{Language as TsLanguage, Node, Query, QueryError, Range};
use tree_sitter_tags::{Tag, TagsConfiguration, TagsContext};
// TODO: better api less boxing and more results
// TODO: better way to do variable assigned to function or just abondon it? (the problem is with
// languages that allow mutliple assignments how do you match up only the identifiers that
// corespond to functions)
// TODO: we could probably use tree sitter tags?

#[allow(missing_debug_implementations)]
pub struct InstantiatedLanguage<'a> {
    search_name: &'a str,
    language: LanguageInformation,
    run_query: QueryFunction,
}

#[derive(Debug)]
pub enum InstantiationError {
    NoMatchingField(String),
    Query(QueryError),
    Tags(tree_sitter_tags::Error),
}
pub trait HasLanguageInformation {
    /// The name of this language
    fn language_name(&self) -> &'static str;
    /// The list of file extensions used for this language.
    fn file_exts(&self) -> &'static [&'static str];
    /// The [`tree_sitter::Language`] for this language
    fn language(&self) -> TsLanguage;
    fn language_info(&self) -> LanguageInformation {
        LanguageInformation {
            language_name: self.language_name(),
            file_exts: self.file_exts(),
            language: self.language(),
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct LanguageInformation {
    language_name: &'static str,
    file_exts: &'static [&'static str],
    language: TsLanguage,
}
#[allow(missing_debug_implementations)]
// TODO: find cleaner (no double parse) way to use this, "fork" tree sitter tags, or make your own
// standard
pub trait TreeSitterTags: Assoc<Type = Tags> + HasLanguageInformation {
    fn tag_query(&self) -> impl ToString;
}
#[allow(missing_debug_implementations)]
pub trait TreeSitterQuery: Assoc<Type = TreeSitter> + HasLanguageInformation {
    // TODO: type saftey for query
    /// Given an identifier(name)
    /// this should produce a string that is the sexp of a query
    /// that finds all matches of function-like things with given name
    /// # Example:
    /// ```rust
    /// fn query(name: &str) -> String {
    ///     format!("((function_item
    ///   name: (identifier) @method-name)
    ///   @method-definition
    /// (#eq? @method-name {name}))
    /// ((let_declaration
    ///   pattern: (identifier) @method-name
    ///   value: (closure_expression)) @method-definition
    /// (#eq? @method-name {name}))
    /// ((const_item
    ///   name: (identifier) @method-name
    ///   value: (closure_expression)) @method-definition
    /// (#eq? @method-name {name}))
    /// ((static_item
    ///   name: (identifier) @method-name
    ///   value: (closure_expression)) @method-definition
    /// (#eq? @method-name {name}))")
    /// }
    /// ```
    fn query_string_function(&self, name: &str) -> String;
}

#[allow(missing_debug_implementations)]
pub trait IdentifierQuery: Assoc<Type = Identifier> + HasLanguageInformation {
    fn query_string(&self) -> impl ToString;
    fn query_name(&self) -> impl ToString;
}

// TODO: hide in docs?
#[allow(missing_debug_implementations)]
pub struct Identifier;
// TODO: hide in docs?
#[allow(missing_debug_implementations)]
pub struct TreeSitter;
#[allow(missing_debug_implementations)]
pub struct Tags;
// TODO: hide in docs?
trait InstantiateHelper<Type> {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError>;
}

// TODO: hide in docs?
pub trait Assoc {
    type Type;
}
impl<T: IdentifierQuery> InstantiateHelper<Identifier> for T {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError> {
        Query::new(&self.language(), &self.query_string().to_string())
            .map_err(InstantiationError::Query)
            .and_then(|query: Query| {
                query
                    .capture_index_for_name(&self.query_name().to_string())
                    .ok_or_else(|| {
                        InstantiationError::NoMatchingField(self.query_name().to_string())
                    })
                    .map(|method_field| -> QueryFunction {
                        Box::new(move |node, code| {
                            let name = &*search;
                            let mut query_cursor = tree_sitter::QueryCursor::new();
                            let matches = query_cursor.matches(&query, node, code);
                            let ranges = matches
                                .filter(move |m| {
                                    m.captures.iter().any(|c| {
                                        c.index == method_field
                                            && c.node.utf8_text(code).unwrap_or("") == name
                                    })
                                })
                                .map(|m| m.captures[0].node.range());

                            ranges.collect()
                        })
                    })
            })
    }
}
impl<T: TreeSitterQuery> InstantiateHelper<TreeSitter> for T {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError> {
        Query::new(
            &self.language(),
            &self.query_string_function(search.as_ref()),
        )
        .map(|query| -> QueryFunction {
            Box::new(move |node, code| {
                let mut query_cursor = tree_sitter::QueryCursor::new();
                let matches = query_cursor.matches(&query, node, code);

                let ranges = matches.map(|m| m.captures[0].node.range());
                ranges.collect()
            })
        })
        .map_err(InstantiationError::Query)
    }
}
struct TagsConfigurationThreadSafe(TagsConfiguration);
unsafe impl Send for TagsConfigurationThreadSafe {}
unsafe impl Sync for TagsConfigurationThreadSafe {}
impl TagsConfigurationThreadSafe {
    pub fn generate_tags<'a>(
        &'a self,
        context: &'a mut TagsContext,
        source: &'a [u8],
        cancellation_flag: Option<&'a AtomicUsize>,
    ) -> Result<
        (
            impl Iterator<Item = Result<Tag, tree_sitter_tags::Error>> + 'a,
            bool,
        ),
        tree_sitter_tags::Error,
    > {
        context.generate_tags(&self.0, source, cancellation_flag)
    }
    pub fn syntax_type_name(&self, id: u32) -> &str {
        self.0.syntax_type_name(id)
    }
}
impl<T: TreeSitterTags> InstantiateHelper<Tags> for T {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError> {
        TagsConfiguration::new(self.language(), &self.tag_query().to_string(), "")
            .map_err(InstantiationError::Tags)
            .map(|tag_config| -> QueryFunction {
                let tag_config = TagsConfigurationThreadSafe(tag_config);
                Box::new(move |node, code| {
                    let mut tag_context = TagsContext::new();
                    let name = &*search;
                    // TODO: don't double parse
                    tag_config
                        .generate_tags(&mut tag_context, code, None)
                        .map(|tags| {
                            let ranges = tags
                                .0
                                .filter_map(Result::ok)
                                .filter(|tag| {
                                    ["method", "function"]
                                        .contains(&tag_config.syntax_type_name(tag.syntax_type_id))
                                        && str::from_utf8(&code[tag.name_range.clone()])
                                            .unwrap_or("")
                                            == name
                                })
                                .filter_map(|tag| {
                                    node.descendant_for_byte_range(tag.range.start, tag.range.end)
                                        .map(|node| node.range())
                                });
                            ranges.collect()
                        })
                        .unwrap_or_else(|_| vec![].into_boxed_slice())
                })
            })
    }
}

impl<T: Assoc + InstantiateHelper<T::Type> + HasLanguageInformation> SupportedLanguage for T {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError> {
        self.instantiate(search)
    }
}
// TODO: maybe make this fallable
type QueryFunction = Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync>;

pub trait SupportedLanguage: HasLanguageInformation {
    fn instantiate(&self, search: Box<str>) -> Result<QueryFunction, InstantiationError>;
    fn to_language<'a>(
        &self,
        search: &'a str,
    ) -> Result<InstantiatedLanguage<'a>, InstantiationError> {
        self.instantiate(search.into())
            .map(|f| InstantiatedLanguage::new(search, self.language_info(), f))
    }
}

pub trait InstantiateMap<'a> {
    fn instantiate_map(
        self,
        name: &'a str,
    ) -> Result<Vec<InstantiatedLanguage<'a>>, InstantiationError>;
}
impl<'a, T, U> InstantiateMap<'a> for T
where
    T: IntoIterator<Item = U>,
    U: Deref<Target = &'a dyn SupportedLanguage>,
{
    fn instantiate_map(
        self,
        name: &'a str,
    ) -> Result<Vec<InstantiatedLanguage<'a>>, InstantiationError> {
        self.into_iter().map(|l| l.to_language(name)).collect()
    }
}
impl<'a> InstantiatedLanguage<'a> {
    pub(crate) fn new(
        search_name: &'a str,
        language: LanguageInformation,
        run_query: QueryFunction,
    ) -> Self {
        Self {
            search_name,
            language,
            run_query,
        }
    }

    #[must_use]
    pub fn run_query(&self, node: Node<'_>, code: &'_ [u8]) -> Box<[Range]> {
        (self.run_query)(node, code)
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.language.language_name
    }

    #[must_use]
    pub const fn file_exts(&self) -> &'static [&'static str] {
        self.language.file_exts
    }

    #[must_use]
    pub const fn language(&self) -> &TsLanguage {
        &self.language.language
    }

    pub(crate) const fn search_name(&self) -> &str {
        self.search_name
    }
}

#[macro_export]
/// Use to more easily make new [`SupportedLanguage`]s.
///
/// First provide the name (which is used as the type of the language), followed by the tree sitter
/// languge in parenthesis, next you put the file extensions in brackets with a leading .
/// to specify the query we use ?= variable -> string literal query.
/// In the query you when you want use the variable just do {variable}.
///
/// Example:
/// ```rust
/// use function_grep::construct_language;
/// use function_grep::supported_languages::SupportedLanguage;
/// use tree_sitter::Language;
/// construct_language!(C(tree_sitter_c::LANGUAGE).[c h]?=
///    name ->  "((function_definition
///  declarator:
///  (function_declarator declarator: (identifier) @method-name))
///  @method-definition
///  (#eq? @method-name {name}))
/// ((declaration declarator:
///  (function_declarator declarator: (identifier) @method-name))
///  @method-definition
///  (#eq? @method-name {name}))"
/// );
/// ```
macro_rules! construct_language {
    ($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:literal=>$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl $crate::supported_languages::HasLanguageInformation for $name {

            fn language_name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> tree_sitter::Language {
                $tslang.into()
            }
        }
        impl $crate::supported_languages::Assoc for $name {
            type Type = $crate::supported_languages::Identifier;
        }
        impl $crate::supported_languages::IdentifierQuery for $name {
            fn query_name(&self) -> impl ToString {
                $query_name
            }
            fn query_string(&self) -> impl ToString {
                $query
            }
        }

    };
($name:ident($tslang:expr).[$($ext:ident)+]?=$tags:expr ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl $crate::supported_languages::HasLanguageInformation for $name {

            fn language_name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> tree_sitter::Language {
                $tslang.into()
            }
        }
        impl $crate::supported_languages::Assoc for $name {
            type Type = $crate::supported_languages::Tags;
        }
        impl $crate::supported_languages::TreeSitterTags for $name {
            fn tag_query(&self) -> impl ToString {
                $tags
            }
        }

    };
($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:ident->$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl $crate::supported_languages::HasLanguageInformation for $name {

            fn language_name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> tree_sitter::Language {
                $tslang.into()
            }
        }
        impl $crate::supported_languages::Assoc for $name {
            type Type = $crate::supported_languages::TreeSitter;
        }
        impl $crate::supported_languages::TreeSitterQuery for $name {
            fn query_string_function(&self, $query_name: &str) -> String {
                format!($query)
            }
        }
    };
}

#[cfg(feature = "c")]

construct_language!(C(tree_sitter_c::LANGUAGE).[c h]?=
   name ->  "((function_definition
 declarator:
 (function_declarator declarator: (identifier) @method-name))
 @method-definition
 (#eq? @method-name {name}))
((declaration declarator:
 (function_declarator declarator: (identifier) @method-name))
 @method-definition
 (#eq? @method-name {name}))"
);

#[cfg(feature = "ruby")]
// TODO: also query for anonymous functions assigned to variables
construct_language!(Ruby(tree_sitter_ruby::LANGUAGE).[rb] ?= "method-name" =>
"((method name: (identifier) @method-name) @method-definition)");

#[cfg(feature = "c-sharp")]
// TODO: also query for anonymous functions assigned to variables
construct_language!(CSharp(tree_sitter_c_sharp::LANGUAGE).[cs] ?= "method-name" =>
"((local_function_statement name: (identifier) @method-name) @method)
((method_declaration name: (identifier) @method-name) @method)"

);
#[cfg(feature = "go")]
// TODO: also query for anonymous functions assigned to variables
//  var f,y  integer  = func(x int) int {
// 	return x * x
// }, y
// x, y := 5
// fmt.Println(foo(10))
// const f f =  5;
// func main() {
// 	empty_test(1, 2, "3")
// 	fmt.Println("Hello World!")
// }
//
// // doc comment
//
// func empty_test(c, a int, b string) {
// 	fmt.Println("Hello World!")
// }
// ((var_declaration (var_spec name: (identifier) @method-name )))
// ((const_declaration))
// ((short_var_declaration))
construct_language!(Go(tree_sitter_go::LANGUAGE).[go] ?= "method-name" =>
"((function_declaration name: (identifier) @method-name) @method-definition)");
#[cfg(feature = "rust")]
construct_language!(Rust(tree_sitter_rust::LANGUAGE).[rs] ?= "method-name" =>

            "((function_item
  name: (identifier) @method-name)
  @method-definition
)
((let_declaration
  pattern: (identifier) @method-name
  value: (closure_expression)) @method-definition
)
((const_item
  name: (identifier) @method-name
  value: (closure_expression)) @method-definition
)
((static_item
  name: (identifier) @method-name
  value: (closure_expression)) @method-definition
)"
);

#[cfg(feature = "python")]
construct_language!(Python(tree_sitter_python::LANGUAGE).[py]?= "method-name" =>

            "((function_definition
 name: (identifier) @method-name)
 @method-definition 
)
((assignment 
 left: ((identifier) @method-name) 
 right: (lambda)) @method-definition 
)
"
);

#[cfg(feature = "java")]
construct_language!(Java(tree_sitter_java::LANGUAGE).[java]?="method-name" =>
"((method_declaration
 name: (identifier) @method-name)
 @method-definition
)
((local_variable_declaration
 declarator: (variable_declarator
 name: (identifier) @method-name
 value: (lambda_expression)))
 @method-definition
)
((field_declaration
 declarator: (variable_declarator
 name: (identifier) @method-name
 value: (lambda_expression)))
 @method-definition
)"
);

#[cfg(feature = "ocaml")]
construct_language!(OCaml(tree_sitter_ocaml::LANGUAGE_OCAML).[ml]?="method-name" =>
"((value_definition
 (let_binding pattern: (value_name) @method-name (parameter)))
 @method-defintion
)
((value_definition
 (let_binding pattern: (parenthesized_operator) @method-name (parameter)))
 @method-defintion
)
((value_definition
 (let_binding pattern: (value_name) @method-name body: (function_expression)))
 @method-defintion
)
((value_definition
 (let_binding pattern: (value_name) @method-name body: (fun_expression)))
 @method-defintion
)");

#[cfg(feature = "javascript")]
construct_language!(JavaScript(tree_sitter_javascript::LANGUAGE).[js]?=tree_sitter_javascript::TAGS_QUERY);
#[must_use]
/// Use this to obtain some defualt languages (what languages are presend depend of the features
/// you allow).
pub fn predefined_languages() -> &'static [&'static dyn SupportedLanguage] {
    &[
        #[cfg(feature = "rust")]
        &Rust,
        #[cfg(feature = "c")]
        &C,
        #[cfg(feature = "python")]
        &Python,
        #[cfg(feature = "java")]
        &Java,
        #[cfg(feature = "ocaml")]
        &OCaml,
        #[cfg(feature = "c-sharp")]
        &CSharp,
        #[cfg(feature = "go")]
        &Go,
        #[cfg(feature = "ruby")]
        &Ruby,
        #[cfg(feature = "javascript")]
        &JavaScript,
    ]
}
