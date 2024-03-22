use std::ops::Deref;

use tree_sitter::{Language as TsLanguage, Query, QueryError};

#[derive(Debug)]
pub struct InstatiatedLanguage<'a> {
    name: &'static str,
    search_name: &'a str,
    file_exts: &'static [&'static str],
    language: TsLanguage,
    query: Query,
}

pub trait Instatiate<'a> {
    fn instatiate_map(self, name: &'a str) -> Result<Vec<InstatiatedLanguage<'a>>, QueryError>;
}

impl<'a, T, U> Instatiate<'a> for T
where
    T: IntoIterator<Item = U>,
    U: Deref<Target = &'a dyn SupportedLanguage>,
{
    fn instatiate_map(self, name: &'a str) -> Result<Vec<InstatiatedLanguage<'a>>, QueryError> {
        self.into_iter()
            .map(|l| l.instatiate(name))
            .collect::<Result<Vec<_>, _>>()
    }
}
impl<'a> InstatiatedLanguage<'a> {
    pub(crate) fn new(
        name: &'static str,
        search_name: &'a str,
        file_exts: &'static [&'static str],
        language: TsLanguage,
        query: Query,
    ) -> Self {
        Self {
            search_name,
            name,
            file_exts,
            language,
            query,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn file_exts(&self) -> &'static [&'static str] {
        self.file_exts
    }

    pub fn language(&self) -> TsLanguage {
        self.language
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub(crate) fn search_name(&self) -> &str {
        self.search_name
    }
}

pub trait SupportedLanguage: Send + Sync {
    /// The name of this language
    fn name(&self) -> &'static str;
    /// The list of file extensions used for this language.
    fn file_exts(&self) -> &'static [&'static str];
    /// The [`tree_sitter::Language`] for this language
    fn language(&self) -> TsLanguage;
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
    // TODO: either make at trait creation time the query be the actual tree sitter query not a
    // string represtion of it, and when we do the actual search we look for capture with
    // @method-name = name
    // or we make an instiate method that takes the name creates the query and returns new trait
    // with query not as string, because after running some tests in git_function_history with
    // flamegraph I see that a lot of time is spent making queries from string
    // but we're probably going to need both b/c if we go the instation route the the trait/thing its
    // returning is basically the other option
    fn query(&self, name: &str) -> String;

    fn instatiate<'a>(&self, name: &'a str) -> Result<InstatiatedLanguage<'a>, QueryError> {
        Query::new(self.language(), &self.query(name)).map(|query| {
            InstatiatedLanguage::new(self.name(), name, self.file_exts(), self.language(), query)
        })
    }
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
    ($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:ident->$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl SupportedLanguage for $name {
            fn query(&self, $query_name: &str) -> String {
                format!($query)
            }

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
construct_language!(Rust(tree_sitter_rust::language()).[rs]?=name->

            "((function_item
  name: (identifier) @method-name)
  @method-definition
(#eq? @method-name {name}))
((let_declaration
  pattern: (identifier) @method-name
  value: (closure_expression)) @method-definition
(#eq? @method-name {name}))
((const_item
  name: (identifier) @method-name
  value: (closure_expression)) @method-definition
(#eq? @method-name {name}))
((static_item
  name: (identifier) @method-name
  value: (closure_expression)) @method-definition
(#eq? @method-name {name}))"
);

#[cfg(feature = "python")]
construct_language!(Python(tree_sitter_python::language()).[py]?=name->

            "((function_definition
 name: (identifier) @method-name)
 @method-definition 
(#eq? @method-name {name}))
((assignment 
 left: ((identifier) @method-name) 
 right: (lambda)) @method-definition 
(#eq? @method-name {name}))
"
);

#[cfg(feature = "java")]
construct_language!(Java(tree_sitter_java::language()).[java]?=name->
"((method_declaration
 name: (identifier) @method-name)
 @method-definition
(#eq? @method-name {name}))
((local_variable_declaration
 declarator: (variable_declarator
 name: (identifier) @method-name
 value: (lambda_expression)))
 @method-definition
(#eq? @method-name {name}))
((field_declaration
 declarator: (variable_declarator
 name: (identifier) @method-name
 value: (lambda_expression)))
 @method-definition
(#eq? @method-name {name}))"
);

#[cfg(feature = "ocaml")]
construct_language!(OCaml(tree_sitter_ocaml::language_ocaml()).[ml]?=name->
"((value_definition
 (let_binding pattern: (value_name) @method-name (parameter)))
 @method-defintion
(#eq? @method-name {name}))
((value_definition
 (let_binding pattern: (parenthesized_operator) @method-name (parameter)))
 @method-defintion
(#eq? @method-name {name}))
((value_definition
 (let_binding pattern: (value_name) @method-name body: (function_expression)))
 @method-defintion
(#eq? @method-name {name}))
((value_definition
 (let_binding pattern: (value_name) @method-name body: (fun_expression)))
 @method-defintion
(#eq? @method-name {name}))");

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
