use tree_sitter::Language;

pub trait SupportedLanguage: Send + Sync {
    /// The name of this language
    fn name(&self) -> &'static str;
    /// The list of file extensions used for this language.
    fn file_exts(&self) -> &'static [&'static str];
    /// The [`tree_sitter::Language`] for this language
    fn language(&self) -> Language;
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
    fn query(&self, name: &str) -> String;
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

            fn language(&self) -> Language {
                $tslang
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
