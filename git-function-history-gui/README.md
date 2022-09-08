# [![crates.io](https://img.shields.io/crates/v/git-function-history-gui.svg?label=latest%20version)](https://crates.io/crates/git-function-history-gui) [![Crates.io](https://img.shields.io/crates/d/git-function-history-gui?label=crates.io%20downloads)](https://crates.io/crates/git-function-history-gui)

# git function history GUI

A GUI frontend for the [git function history library](https://crates.io/crates/git_function_history).

## Installation

cargo install git-function-history-gui

### Note

Under Linux you may need to install the following packages:

- libclang-dev
- libgtk-3-dev
- libxcb-render0-dev
- libxcb-shape0-dev
- libxcb-xfixes0-dev
- libspeechd-dev
- libxkbcommon-dev
- libssl-dev

On ubuntu you can install them with:
`sudo apt-get install -y libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

## Usage

When you run the program, you will see a window, like this: (the title bar/decorations vary by platform)

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/startup.png" width="400">

The app is split into three sections:

- The top section shows the current output of whatever command you have entered in the command bar, this section is called the viewing pane.

- The middle section is where you build and run commands, it is called the command bar.

- The bottom section shows the current status of the app, and any errors that have occurred, and on the right hand side there is as a button to change the theme.

### Command bar

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar.png" width="400">

Even though the command bar is the middle section, we will start with it because you cannot use the top section without first building a command in the command bar.

The leftmost part of the command bar is the command selector, it is a drop down menu that allows you to select the command you want to build.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_selector.png" width="100">

The commands are:

- `filter`: This command allows you to filter the output of the previous command.

- `search`: This command builds a search query for a function in a git repository.

- `list`: This command is used to list commit hashes or dates for each commit in a git repository.

#### Command bar - search

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_search.png" width="400">

The first thing you'll see is a text box, this is where you enter the name of the function you want to search for.

The next thing you'll see is a drop down menu, this is the search file selector, it allows you to select what type of file you want to search in.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_file.png" width="100">

The options are:

- `None`: This option will search all rust files in the repository (very expensive).

- `Relative`: This will search all file ending with the filename specified in the text box.

- `Absolute`: This option will search the exact file specified in the text box.

If you select `Relative` or `Absolute` then you will see a text box appear, this is where you enter the filename.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_file_input.png" width="400">

After that there is another dropdown menu to filter the search (before it is run) to save time.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_filter.png" width="100">

The options are:

- `None`: This option will not filter the search.

- `Commit Hash`: This option will filter the search to only the commit with the commit hash specified in the text box.

- `Date`: This option will filter the search to only the commit closest to the date specified in the text box.

`Date Range`: This option will filter the search to only the commits between the two dates specified in the text boxes.

If you select `Commit Hash` or `Date` then you will see a text box appear, this is where you enter the commit hash or date, with `Date Range` you will see two text boxes appear, these are where you enter the start and end dates.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_filter_input.png" width="400">

After that there is `Go` button, this will run the command and display the output in the viewing pane (after the command has finished).

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_go.png" width="400">

#### Command bar - filter

Once you have ran a command and gotten some output in the viewing pane,  you can filter by switching the first drop down to filter.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/filter_bar.png" width="400">

The options vary by if you have already filtered by lets say a date, then yo u won't be able to filter by a date range or a commit hash.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/filter_bar_selector.png" width="100">

All the options are:

- `None`: This option will not filter your output.

- `Commit Hash`: This option will filter the output to only the commit with the commit hash specified in the text box.

- `Date`: This option will filter the output to only the commit closest to the date specified in the text box.

- `Date Range`: This option will filter the output to only the commits between the two dates specified in the text boxes.

- `function in function`: This option will filter the output to only the commits that contain the function you specified in your search has a parent function specified in the text box.

- `function in lines`: This option will filter the output to only the commits that contain the function you specified in your search in the lines specified in the text boxes.

- `function in block`: This option will filter the output to only the commits that contain the function you specified in your search in the block specified in the text box. Valid block types are:
  - `extern`
  - `impl`
  - `trait`

Every option has a text box, except for `None`.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/filter_bar_input.png" width="400">

After that there is `Go` button, this will run the command and display the output in the viewing pane (after the command has finished).

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/filter_bar_go.png" width="400">

#### Command bar - list

The list command is used to list commit hashes or dates for each commit in a git repository.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_list.png" width="400">

Once you select the list command, you will see a drop down menu, this is the list type selector, it allows you to select what type of list you want to build.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_list_type.png" width="100">

The options are:

- `dates`: This option will list the date of each commit in the repository.

- `commit hashes`: This option will list the commit hash of each commit in the repository.

After that there is `Go` button, this will run the command and display the output in the viewing pane (after the command has finished).

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/command_bar_list_go.png" width="400">

### Viewing pane

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane.png" width="400">

The viewing pane is where the output of the command you have built in the command bar is displayed.

when you first open the app, the viewing pane will be or you filtered to the point where there is no output, so you will see a message saying `Nothing to show Please select a command`.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_empty.png" width="400">

If its showing you the a function whole history in the repository, as opposed to just a single commit, then the top of the viewing pane will have a left and right arrow )a button will be disabled if there is no more history to show in that direction
), these are used to navigate between commits along with the commit hash and date.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_history_arrows.png" width="400">

Below that is the is where you can see all the all instances of the function in the commit along with tall left and right arrows (each button will only be shown if you can go back and forward through the files in the commit), these are used to navigate between the files that contain the function.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_history_files.png" width="400">

If you filtered to a single commit, then you will see the commit hash and date at the top of the viewing pane.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_commit.png" width="400">

Below that is the is where you can see all the all instances of the function in the commit along with tall left and right arrows (each button will only be shown if you can go back and forward through the files in the commit), these are used to navigate between the files that contain the function.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_commit_files.png" width="400">

If you list the commit hashes or dates, then you will see a list of all the commit hashes or dates in the repository.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/viewing_pane_list.png" width="400">

### Status bar

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/status_bar.png" width="400">

The status bar is where you can see the status of the app, it will tell you if the app is loading or if there was an error or everything is fine.

On the right hand side there is as a button to change the theme.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/git-function-history-gui/resources/screenshots/status_bar_theme.png" width="100">

## Known issues

- [windows]: App crashes with ```error: process didn't exit successfully: `target\debug\git-function-history-gui.exe` (exit code: 0xc0000374, STATUS_HEAP_CORRUPTION)```, This is an issue with the underlying GUI library being used it might vary depending on the hardware you use.

## Future plans

- [ ] Add a way to filter by file name(s)
- [ ] Add a way to filter by more git stuff (author, email, etc)
- [ ] Add a way to filter by lifetime whether of the block, parent function, or the function itself
- [ ] Add a way to filter by block name
- [ ] Add a way to search multiple files at once

Note: most of these features need to be added to the underlying library first.