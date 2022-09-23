# TODO

- Library
  - [/] add more tests
  - [/] add more examples
  - [x] add documentation to new methods
  - [x] add more `get_*` methods whether it `get_mut_*` etc
  - [x] add more filter like to filter to a certain file

- GUI
  - [x] fix `thread '<unnamed>' panicked at 'channel disconnected', function_history_backend_thread/src/lib.rs:33:25` error (handling when the channel is disconnected at the end of the program)
  - [x] add new documentation for the new filters and fix some old documentation that talks about filter commitfunctions and files etc
- TUI
  - [x] use a proper input box for the edit bar, so that delete and scrolling the input works
  - [x] finish documentation
  - [ ] add icons for taskbars/launching the app

- General
  - [x] add the new filters to the GUI and TUI
  - [/] clean up the code
  - [x] add more logging in the code (and remove the `println!`s)
  - [x] bump versions when all else is done and publish to crates.io

- version 7.0
  - [ ] add more and better ways to filter dates
  - [ ] add filters for git specific stuff like author, committer, etc
  - [ ] ability to get a git repo from a url using something like git clone
