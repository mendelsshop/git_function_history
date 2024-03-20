# [![crates.io](https://img.shields.io/crates/v/function-grep.svg?label=latest%20version)](https://crates.io/crates/function-grep) [![Crates.io](https://img.shields.io/crates/d/function-grep?label=crates.io%20downloads)](https://crates.io/crates/function-grep) ![msrv](https://raw.githubusercontent.com/mendelsshop/git_function_history/main/resources/function-grep_msrv.svg)
# function grep
Find functions with a given name in a file, powered by tree sitter.

Use the latest [crates.io](https://crates.io/crates/function-grep) by putting `function-grep = "0.1.0"` in your cargo.toml under `[dependencies]` section.

# Examples
## When you know the language 
```rust
use function_grep::{supported_languages::Rust, ParsedFile};
use tree_sitter::Point;
use tree_sitter::Range;

let results = ParsedFile::search_file("foo", "fn foo() {}\n fn bar()\n", &Rust).unwrap();
println!("{:?}", results.results());
assert_eq!(results.results(), &[Range { start_byte: 0, end_byte: 11, start_point: Point { row: 0, column: 0 }, end_point: Point { row: 0, column: 11 } }]);
assert_eq!(results.to_string(), "1: fn foo() {}".to_string())
```

## When you don't know the language 

```rust
use function_grep::{supported_languages, ParsedFile};
use tree_sitter::Point;
use tree_sitter::Range;

let results = ParsedFile::search_file_with_name("foo", "fn foo() {}\n fn bar()\n", "test.rs",  supported_languages::predefined_languages()).unwrap();
println!("{:?}", results.results());
assert_eq!(results.results(), &[Range { start_byte: 0, end_byte: 11, start_point: Point { row: 0, column: 0 }, end_point: Point { row: 0, column: 11 } }]);
assert_eq!(results.to_string(), "1: fn foo() {}".to_string())
```

## Using a custom language
```rust
use function_grep::{supported_languages::SupportedLanguage, construct_language, ParsedFile};
use tree_sitter::Point;
use tree_sitter::Range;
use tree_sitter::Language;


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
let results = ParsedFile::search_file("foo", "fn foo() {}\n fn bar()\n", &Rust).unwrap();
println!("{:?}", results.results());
assert_eq!(results.results(), &[Range { start_byte: 0, end_byte: 11, start_point: Point { row: 0, column: 0 }, end_point: Point { row: 0, column: 11 } }]);
assert_eq!(results.to_string(), "1: fn foo() {}".to_string())
```

# Predefined Languages

Theres is built in support for python, c, rust, ocaml, and java.
Each predefined language is a feature thats on by default, use no-default-fatures, to select specific languages only.
