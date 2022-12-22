
# [![Clippy check + test](https://github.com/mendelsshop/git_function_history/actions/workflows/cargo_clippy_lib.yml/badge.svg)](https://github.com/mendelsshop/git_function_history/actions/workflows/cargo_clippy_lib.yml) [![crates.io](https://img.shields.io/crates/v/git_function_history.svg?label=latest%20version)](https://crates.io/crates/git_function_history) [![Crates.io](https://img.shields.io/crates/d/git_function_history?label=crates.io%20downloads)](https://crates.io/crates/git_function_history) [![docs.rs](https://img.shields.io/docsrs/git_function_history?logo=Docs.rs)](https://docs.rs/git_function_history/latest/git_function_history) ![msrv](../resources/msrv.svg)

# git function history

Show the git history of a function or method.
Use the latest (beta) version by putting `"git_function_history" = { git = 'https://github.com/mendelsshop/git_function_history' }` in your cargo.toml under `[dependencies]` section.
Use the latest [crates.io](https://crates.io/crates/git_function_history) by putting `git_function_history = "0.6.2"` in your cargo.toml under `[dependencies]` section.

## features

- parallel: use rayon to parallelize the git log search

- --no-default-features: disable parallelism

<!-- - c-lang: adds support c (requires you to have a c compiler installed) (see the [c-lib]() docs for more information) -->

- unstable: enable some parsers that require nightly rust so run `cargo +nightly` to use them

- cache: enables caching when parsing files and folders that don't change as often.

## parsing library dependencies

| Language | Rust | Ruby | Python | Go |
|  ---  |  ---  |  ---  |  ---  |  ---  |
|Source| [ra_ap_syntax](https://crates.io/crates/ra_ap_syntax)([Rust Analyzer](https://rust-analyzer.github.io/)) | [lib-ruby-parser](https://crates.io/crates/lib-ruby-parser) | [rustpython-parser](https://crates.io/crates/rustpython-parser/)([RustPython](https://rustpython.github.io/)) | [gosyn](https://crates.io/crates/gosyn) |
| Requirements | | | | rust nightly and unstable feature |
