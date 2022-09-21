
# [![Clippy check + test](https://github.com/mendelsshop/git_function_history/actions/workflows/cargo_clippy_lib.yml/badge.svg)](https://github.com/mendelsshop/git_function_history/actions/workflows/cargo_clippy_lib.yml) [![crates.io](https://img.shields.io/crates/v/git_function_history.svg?label=latest%20version)](https://crates.io/crates/git_function_history) [![Crates.io](https://img.shields.io/crates/d/git_function_history?label=crates.io%20downloads)](https://crates.io/crates/git_function_history)

# git function history

Show the git history of a function or method.
Use the latest (beta) version by putting `"git_function_history" = { git = 'https://github.com/mendelsshop/git_function_history' }` in your cargo.toml under `[dependencies]` section.
Use the latest [crates.io](https://crates.io/crates/git_function_history) (also beta) by putting `git_function_history = "0.5.4"` in your cargo.toml under `[dependencies]` section.

Parser vs Regex approach benchmarks:
| approach| expensive| relative| date-range |
| --- | --- | --- | --- |
|regex| 313 second(s) | 22 second(s) | 8 second(s) |
|parser| 22 second(s) | 21 second(s)| 1 second(s) |
