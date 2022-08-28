# [![crates.io](https://img.shields.io/crates/v/git-function-history-gui.svg?label=latest%20version)](https://crates.io/crates/git-function-history-gui) [![Crates.io](https://img.shields.io/crates/d/git-function-history-gui?label=crates.io%20downloads)](https://crates.io/crates/git-function-history-gui)

# git function history GUI

A GUI frontend for the [git function history library](https://crates.io/crates/git_function_history).

## Known issues

- [windows]: App crashes with ```error: process didn't exit successfully: `target\debug\git-function-history-gui.exe` (exit code: 0xc0000374, STATUS_HEAP_CORRUPTION)```, This is an issue with the underlying GUI library being used it might vary depending on the hardware you use.
