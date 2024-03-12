use super::SupportedLanguage;

#[derive(Copy, Clone, Debug)]
pub struct C;

impl SupportedLanguage for C {
    fn file_exts(&self) -> &'static [&'static str] {
        &["c", "h"]
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_c::language()
    }
    fn query(&self, name: &str) -> String {
        format!(
            "((function_definition
 declarator:
 (function_declarator declarator: (identifier) @method-name))
 @method-definition
 (#eq? @method-name {name}))
((declaration declarator:
 (function_declarator declarator: (identifier) @method-name))
 @method-definition
 (#eq? @method-name {name}))"
        )
    }

    fn name(&self) -> &'static str {
        "c"
    }
}
