use std::ops::Deref;

use tree_sitter::{Language as TsLanguage, Node, Query, QueryError, Range};
// TODO: better api less boxing and more results

#[allow(missing_debug_implementations)]
pub struct InstatiatedLanguage<'a> {
    name: &'static str,
    search_name: &'a str,
    file_exts: &'static [&'static str],
    language: TsLanguage,
    run_query: Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync>,
}

pub trait Instatiate {
    fn instiate(
        &self,
        search: Box<str>,
    ) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync>;
}

pub trait InstatiateMap<'a> {
    fn instatiate_map(self, name: &'a str) -> Result<Vec<InstatiatedLanguage<'a>>, QueryError>;
}
impl<'a, T, U> InstatiateMap<'a> for T
where
    T: IntoIterator<Item = U>,
    U: Deref<Target = &'a dyn SupportedLanguage>,
{
    fn instatiate_map(self, name: &'a str) -> Result<Vec<InstatiatedLanguage<'a>>, QueryError> {
        Ok(self
            .into_iter()
            .map(|l| {
                InstatiatedLanguage::new(
                    l.name(),
                    name,
                    l.file_exts(),
                    l.language(),
                    l.instiate(name.into()),
                )
            })
            .collect())
    }
}
// impl<'a, T, U> Instatiate<'a> for T
// where
//     T: IntoIterator<Item = U>,
//     U: Deref<Target = &'a dyn SupportedLanguage>,
// {
//     fn instatiate_map(self, name: &'a str) -> Result<Vec<InstatiatedLanguage<'a>>, QueryError> {
//         self.into_iter()
//             .map(|l| l.instatiate(name))
//             .collect::<Result<Vec<_>, _>>()
//     }
// }
impl<'a> InstatiatedLanguage<'a> {
    pub(crate) fn new(
        name: &'static str,
        search_name: &'a str,
        file_exts: &'static [&'static str],
        language: TsLanguage,
        run_query: Box<dyn Fn(Node<'_>, &'_ [u8]) -> Box<[Range]> + Send + Sync>,
    ) -> Self {
        Self {
            name,
            search_name,
            file_exts,
            language,
            run_query,
        }
    }

    pub fn run_query(&self, node: Node<'_>, code: &'_ [u8]) -> Box<[Range]> {
        (self.run_query)(node, code)
    }
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn file_exts(&self) -> &'static [&'static str] {
        self.file_exts
    }

    #[must_use]
    pub fn language(&self) -> &TsLanguage {
        &self.language
    }

    pub(crate) fn search_name(&self) -> &str {
        self.search_name
    }
}

pub trait SupportedLanguage: Send + Sync + Instatiate {
    /// The name of this language
    fn name(&self) -> &'static str;
    /// The list of file extensions used for this language.
    fn file_exts(&self) -> &'static [&'static str];
    /// The [`tree_sitter::Language`] for this language
    fn language(&self) -> TsLanguage;
    /*     // TODO: type saftey for query
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
    ////// ``` }
     */
    // TODO: either make at trait creation time the query be the actual tree sitter query not a
    // string represtion of it, and when we do the actual search we look for capture with
    // @method-name = name
    // or we make an instiate method that takes the name creates the query and returns new trait
    // with query not as string, because after running some tests in git_function_history with
    // flamegraph I see that a lot of time is spent making queries from string
    // but we're probably going to need both b/c if we go the instation route the the trait/thing its
    // returning is basically the other option
}

#[macro_export]
/// Use to more easily make new [`SupportedLanguage`]s.
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
/// construct_language!(C(tree_sitter_c::language()).[c h]?=
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
    ($name:ident($tslang:expr).[$($ext:ident)+]?=$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl Instatiate for $name {
fn instiate(&self, name: Box<str>) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync>
    where
        Self: Sized {
     let query =   Query::new(&$tslang, $query).unwrap();
            let method_field = query.capture_index_for_name("method-name").unwrap();
           Box::new( move|node, code|  {
                let name = &*name;
                let mut query_cursor = tree_sitter::QueryCursor::new();
                let matches = query_cursor.matches(&query, node, code);
                let ranges = matches
                    .filter(move |m| {
                        m.captures
                            .iter()
                            .find(|c| {
                                c.index == method_field && c.node.utf8_text(code).unwrap() == name
                            })
                            .is_some()
                    })
                    .map(|m| m.captures[0].node.range());

                ranges.collect()
            })
    }
    }
        impl SupportedLanguage for $name {

            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> TsLanguage {
                $tslang
            }
        }

        impl <'a> std::ops::Deref for $name {
            type Target = dyn SupportedLanguage;

           fn deref(&self) -> &Self::Target {
            self
        }
        }
    };
($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:ident->$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl Instatiate for $name {
fn instiate(&self, $query_name: Box<str>) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync> {

     let query =   Query::new(&$tslang, &format!($query, )).unwrap();
    Box::new(move|node, code|{
        let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, node, code);
    let ranges = matches.map(|m| m.captures[0].node.range());

        ranges.collect()
        })}
        }
        impl SupportedLanguage for $name {

            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> TsLanguage {
                $tslang
            }
        }

        impl <'a> std::ops::Deref for $name {
            type Target = dyn SupportedLanguage;

           fn deref(&self) -> &Self::Target {
            self
        }
        }
    };
}

#[cfg(feature = "c")]

construct_language!(C(tree_sitter_c::language()).[c h]?=
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

#[cfg(feature = "rust")]
construct_language!(Rust(tree_sitter_rust::language()).[rs]?=

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
construct_language!(Python(tree_sitter_python::language()).[py]?=

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
construct_language!(Java(tree_sitter_java::language()).[java]?=
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
construct_language!(OCaml(tree_sitter_ocaml::language_ocaml()).[ml]?=
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
    ]
}
