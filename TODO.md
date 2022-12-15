# TODO

- Library
  - [/] add more tests
  - [/] add more examples
  - [x] add documentation to new methods
  - [x] add more `get_*` methods whether it `get_mut_*` etc
  - [x] add more filter like to filter to a certain file
  - [ ] decide when to use par_* (parallelization) vs plain methods cause of the overhead of spawning threads and the like

- GUI
  - [x] fix `thread '<unnamed>' panicked at 'channel disconnected', function_history_backend_thread/src/lib.rs:33:25` error (handling when the channel is disconnected at the end of the program)
  - [x] add new documentation for the new filters and fix some old documentation that talks about filter  and files etc
- TUI
  - [x] use a proper input box for the edit bar, so that delete and scrolling the input works
  - [x] finish documentation
  - [x] add icons for taskbars/launching the app

- General
  - [x] add the new filters to the GUI and TUI
  - [/] clean up the code
  - [/] add more logging in the code (and remove the `println!`s)
  - [ ] add more and better ways to filter dates
  - [x] add filters for git specific stuff like author, committer, etc
  - [ ] ability to get a git repo from a url using something like git clone
  - [/] add support for other languages (currently only supports rust)
  - [x] save search queries and filters to a file
  - [ ] rework the way filters and filefilters are handled ie maybe use a builder pattern
  - [/] remove all potentially panicking code

- release 7.0:
  - python:
    - [x] save parent function and classes
    - [ ] save kwargs and varargs etc using the args enum and be able to filter by all args or just kwargs etc
  - ruby:
    - [ ] save kwargs and varargs etc using the args enum and be able to filter by all args or just kwargs etc
  - gui:
    - [ ] make the list of dates clickable so when you click on a date/commit it will automatically run a search for that date/commit
    - [ ] make list command a table wiht rows and columns for date, commit, author, message, etc
    - [ ] (possibly) use tree sitter to provide syntax highlighting
  - tui:
    - [ ] make list command a table wiht rows and columns for date, commit, author, message, etc
    - [ ] (possibly) use tree sitter to provide syntax highlighting
  - lib:
    - [x] possibly stop using Commmad::new("git") and use https://crates.io/crates/git_rs OR https://crates.io/crates/rs-git-lib OR  https://crates.io/crates/gitoxide, to imporve performance with program compibilty assistant sevice on windows

    - [] move language module into its own crate
