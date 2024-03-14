use tree_sitter::Language;

pub trait SupportedLanguage {
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

macro_rules! construct_language {
    ($name:ident($tslang:expr).[$($ext:ident)+]?=$query_name:ident->$query:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl $crate::SupportedLanguage for $name {
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
#[must_use]
pub fn predefined_languages() -> &'static [&'static dyn SupportedLanguage] {
    &[&Rust, &C, &Python]
}
