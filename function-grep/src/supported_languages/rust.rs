use super::SupportedLanguage;

#[derive(Copy, Clone, Debug)]
pub struct Rust;

impl SupportedLanguage for Rust {
    fn file_exts(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_rust::language()
    }
    fn query(&self, name: &str) -> String {
        format!(
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
        )
    }

    fn name(&self) -> &'static str {
        "rust"
    }
}
