use std::ops::Deref;
use tree_sitter::{Language as TsLanguage, Node, Query, QueryError, Range};
// TODO: better api less boxing and more results
// TODO: better way to do variable assigned to function or just abondon it? (the problem is with
// languages that allow mutliple assignments how do you match up only the identifiers that
// corespond to functions)
// TODO: we could probably use tree sitter tags?

#[allow(missing_debug_implementations)]
pub struct InstatiatedLanguage<'a> {
    search_name: &'a str,
    language: LanguageInformation,
    run_query: QueryFunction,
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
// TODO: hide in docs?
trait InstantiateHelper<Type> {
    fn instiate(&self, search: Box<str>) -> QueryFunction;
}

// TODO: hide in docs?
pub trait Assoc {
    type Type;
}
impl<T: IdentifierQuery> InstantiateHelper<Identifier> for T {
    fn instiate(
        &self,
        search: Box<str>,
    ) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync> {
        let query = Query::new(&self.language(), &self.query_string().to_string()).unwrap();
        let method_field = query
            .capture_index_for_name(&self.query_name().to_string())
            .unwrap();
        Box::new(move |node, code| {
            let name = &*search;
            let mut query_cursor = tree_sitter::QueryCursor::new();
            let matches = query_cursor.matches(&query, node, code);
            let ranges = matches
                .filter(move |m| {
                    m.captures
                        .iter()
                        .any(|c| c.index == method_field && c.node.utf8_text(code).unwrap() == name)
                })
                .map(|m| m.captures[0].node.range());

            ranges.collect()
        })
    }
}
impl<T: TreeSitterQuery> InstantiateHelper<TreeSitter> for T {
    fn instiate(
        &self,
        search: Box<str>,
    ) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync> {
        let query = Query::new(
            &self.language(),
            &self.query_string_function(search.as_ref()),
        )
        .unwrap();
        Box::new(move |node, code| {
            let mut query_cursor = tree_sitter::QueryCursor::new();
            let matches = query_cursor.matches(&query, node, code);
            let ranges = matches.map(|m| m.captures[0].node.range());
            ranges.collect()
        })
    }
}

impl<T: Assoc + InstantiateHelper<T::Type> + HasLanguageInformation> SupportedLanguage for T {
    fn instiate(
        &self,
        search: Box<str>,
    ) -> Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync> {
        self.instiate(search)
    }
}
type QueryFunction = Box<dyn for<'x, 'y> Fn(Node<'x>, &'y [u8]) -> Box<[Range]> + Send + Sync>;

pub trait SupportedLanguage: HasLanguageInformation {
    fn instiate(&self, search: Box<str>) -> QueryFunction;
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
            .map(|l| InstatiatedLanguage::new(name, l.language_info(), l.instiate(name.into())))
            .collect())
    }
}
impl<'a> InstatiatedLanguage<'a> {
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
    ($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:literal=>$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl HasLanguageInformation for $name {

            fn language_name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> TsLanguage {
                $tslang
            }
        }
        impl Assoc for $name {
            type Type = Identifier;
        }
        impl IdentifierQuery for $name {
            fn query_name(&self) -> impl ToString {
                $query_name
            }
            fn query_string(&self) -> impl ToString {
                $query
            }
        }

    };
($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:ident->$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl HasLanguageInformation for $name {

            fn language_name(&self) -> &'static str {
                stringify!($name)
            }

            fn file_exts(&self) -> &'static [&'static str] {
                &[$(stringify!($ext)),+]
            }

            fn language(&self) -> TsLanguage {
                $tslang
            }
        }
        impl Assoc for $name {
            type Type = TreeSitter;
        }
        impl TreeSitterQuery for $name {
            fn query_string_function(&self, $query_name: &str) -> String {
                format!($query)
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

#[cfg(feature = "ruby")]
// TODO: also query for anonymous functions assigned to variables
construct_language!(Ruby(tree_sitter_ruby::language()).[rb] ?= "method-name" =>
"((method name: (identifier) @method-name) @method-definition)");

#[cfg(feature = "c-sharp")]
// TODO: also query for anonymous functions assigned to variables
construct_language!(CSharp(tree_sitter_c_sharp::language()).[cs] ?= "method-name" =>
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
construct_language!(Go(tree_sitter_go::language()).[go] ?= "method-name" =>
"((function_declaration name: (identifier) @method-name) @method-definition)");
#[cfg(feature = "rust")]
construct_language!(Rust(tree_sitter_rust::language()).[rs] ?= "method-name" =>

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
construct_language!(Python(tree_sitter_python::language()).[py]?= "method-name" =>

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
construct_language!(Java(tree_sitter_java::language()).[java]?="method-name" =>
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
construct_language!(OCaml(tree_sitter_ocaml::language_ocaml()).[ml]?="method-name" =>
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
        #[cfg(feature = "c-sharp")]
        &CSharp,
        #[cfg(feature = "go")]
        &Go,
        #[cfg(feature = "ruby")]
        &Ruby,
    ]
}
