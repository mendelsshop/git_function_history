# git_function_history_proc-macro

[![Crates.io](https://img.shields.io/crates/v/git_function_history-proc-macro.svg)](https://crates.io/crates/git_function_history-proc-macro)
[![Docs.rs](https://docs.rs/git_function_history-proc-macro/badge.svg)](https://docs.rs/git_function_history-proc-macro)
![msrv](https://raw.githubusercontent.com/mendelsshop/git_function_history/main/resources/git_function_history-proc-macro.svg)

This crate provides a procedural derive macro `enum_stuff` along with the attribue `enumstuff` which is used to skip fields or variants for the crate [`git_function_history`](https://crates.io/crates/git_function_history), that makes it easier to parse list of strings to filter types provided by the `git_function_history` crate, along with some other stuff for types that derive it. This makes it easier for consumer of the `git_function_history` crate to create UIs by providing a way to turn user commands into filters and commands the `git_function_history` crate can understand.
To see an example of the features provided by this crate in use look at [`cargo-function-history`](https://github.com/mendelsshop/git_function_history/tree/main/cargo-function-history).
