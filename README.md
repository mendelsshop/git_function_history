# ![Custom badge](https://img.shields.io/endpoint?color=green&url=https%3A%2F%2Fraw.githubusercontent.com%2Fmendelsshop%2Fgit_function_history%2Fstats%2Floc.json) ![Custom badge](https://img.shields.io/endpoint?color=green&url=https%3A%2F%2Fraw.githubusercontent.com%2Fmendelsshop%2Fgit_function_history%2Fstats%2Fdownloads.json)

# git function history

Parser (main) vs Regex approach benchmarks:
| approach| expensive| relative| date-range |
| --- | --- | --- | --- |
|regex| 313 second(s) | 22 second(s) | 8 second(s) |
|parser| 22 second(s) | 21 second(s)| 1 second(s) |

* These benchmarks were done in debug mode on a Ryzen 7 5700u with 16Gb of ram.

## Structure of this project

* [git-function-history-lib](https://github.com/mendelsshop/git_function_history/tree/main/git-function-history-lib) - the library itself
* [function_history_backend_thread](https://github.com/mendelsshop/git_function_history/tree/main/function_history_backend_thread) - a threading middleware for the library (used by the GUI and TUI)
* [cargo-function-history](https://github.com/mendelsshop/git_function_history/tree/main/cargo-function-history) - a cargo subcommand that uses the library (TUI)
* [git-function-history-gui](https://github.com/mendelsshop/git_function_history/tree/main/git-function-history-gui) - the GUI frontend
