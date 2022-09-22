# [![crates.io](https://img.shields.io/crates/v/cargo-function-history.svg?label=latest%20version)](https://crates.io/crates/cargo-function-history) [![Crates.io](https://img.shields.io/crates/d/cargo-function-history?label=crates.io%20downloads)](https://crates.io/crates/cargo-function-history)

# cargo function history

* needs at least 15*15 terminal size

A cargo frontend for the [git function history library](https://crates.io/crates/git-function-history).

## Installation

cargo install cargo-function-history

## Usage

`cargo function-history <function-name<:filename>> <options>`

or cargo-function-history `<function-name<:filename>> <options>`

### Options

- `--help`: display the help message

- `--filter-date <date>`: filter only to this date

- `--filter-commit-hash <hash>`: filter only to this commit hash

- `--filter-date-range=<date1>:<date2>`: filter to the given date range

- `--file-absolute`: search the exact file with the filename specified after the function name

- `--file-relative`: search any file ending with the filename specified after the function name

### using the tui

Once you run the the command, a tui interface will pop up.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/startup.png" width="400">

Even if you specified a search via command you ran to open the app, you will still see no function history result, because it is still loading. Once it is done loading, you will see the function history result.

Now that you've opened the app, you will see its split into 3 sections:

- The top section is the viewing pane. It shows the function history result.

- The middle section is the command pane. Here is where you enter command, which will be executed when you press enter.

- The bottom section is the status pane. It shows the status of the app.

#### command-pane

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane.png" width="400">

Even though the viewing pane comes first you generally wont be able to use the viewing pane without entering a command.

To enter editing mode press `:` then you will see the command pane change to the command input mode, and show the cursor with yellow text in the input bar

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-input.png" width="400">

To exit editing mode press `esc`.

If your command is to long to fit on the screen, you can can use the left and right arrow keys to scroll the command pane.

You can also use the up and down arrow keys to scroll through your command history.

Each command starts with one of three command types:

- `search`: search for a function

- `filter`: filter the current search

- `list`: list the commits or dates

##### command-pane-search

after typing `search` you can type the function name you want to search for.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-search-none.png" width="400">

If you want your search to only be for a certain file, you first specify if it is an absolute or relative search with `absolute` or `relative`, then the file name, or if you want to search to any file that contains a directory you can do `directory` directory-name.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-search-absolute.png" width="400">

If you want your search to also be for a certain date, commit hash, or date range, you can specify that with `date`, `commit-hash`, or `date-range`, then the date, commit hash, or date range (for the date range each date is separated by a space).

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-search-absolute-date.png" width="400">

If you only need your search to be for a certain date, commit hash, or date range, you can skip the the file name and filetype.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-search-date.png" width="400">

Then press enter to execute the command, and after executing the command you will see the search result in the viewing pane.

##### command-pane-filter

after running a search (or another filter) typing `filter`, lets you build a command to filter the current output.

The options vary by if you have already filtered by lets say a date, then you won't be able to filter by a date range or a commit hash.

All the options are:

- `commit`: This option will filter the output to only the commit with the commit hash specified after the `commit` keyword

- `date`: This option will filter the output to only the commit closest to the date specified after the `date` keyword

- `date-range`: This option will filter the output to only the commits between the two space separated dates specified after the `date-range` keyword

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-filter-date.png" width="400">

- `parent`: This option will filter the output to only the commits that contain the function you specified in your search has a parent function specified after the `parent` keyword
- `line-range`: This option will filter the output to only the commits that contain the function you specified in your search in the lines specified after the `line-range` keyword

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-filter-lines.png" width="400">

- `block`: This option will filter the output to only the commits that contain the function you specified in your search in the block specified after the `block` keyword. Valid block types are:
  - `extern`
  - `impl`
  - `trait`

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-filter-block.png" width="400">

- `file-absolute`: This option will filter the output to only the commits that contain the exact file you specified in your search in the file specified after the `file-absolute` keyword

- `file-relative`: This option will filter the output to only the commits that contain any file that ends with the file you specified in your search in the file specified after the `file-relative` keyword

- `directory`: This option will filter the output to only the commits that contain any file that contains the directory you specified in your search in the directory specified after the `directory` keyword

After entering the command, press enter to execute the command, and after executing the command you will see the search result in the viewing pane.

##### command-pane-list

After typing `list` you can type the type of list you want to see with `commits` or `dates`.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/command-pane-list-commits.png" width="400">

Then press enter to execute the command, and after executing the command you will see the list result in the viewing pane.

[//]: # (explain what the different keys do in edit mode based of of https://github.com/sayanarijit/tui-input/blob/main/src/backend/crossterm.rs#L12)

#### viewing pane

To navigate the viewing pane you need to be in viewing mode. To enter viewing mode from editing mode press `esc`.

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/viewing-pane.png" width="400">

To scroll the file in the viewing pane you can use the the `up` or `k` and `down` or `j` keys.

To move to the next or previous commit you can use the `right` or `l` and `left` or `h` keys.

To move through files in a commit you can use the `shift` + `right` or `l` and `shift` + `left` or `h` keys.

At the top it will show the commit hash, date, and time of the commit.
With the file name under that.

The file will be shown with the function you searched for with the line numbers.

#### status pane

The status pane is used to display the status of the program.

There are 4 different status types:

- `error`: This status type is used to display errors that occur during the program execution

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/status-pane-err.png" width="400">

- `warning`: This status type is used to display warnings that occur during the program execution
- `ok`: This status type is used to show when a command has been executed successfully

<img src="https://raw.githubusercontent.com/mendelsshop/git_function_history/main/cargo-function-history/resources/screenshots/status-pane-ok.png" width="400">

- `loading`: This status type is used to show when the program is loading

## Note

When specifying dates please use the RFC 2822 format, e.g. `Mon, 15 Feb 2021 15:04:05 +0000`, please put underscores instead of spaces like `Mon,_15_Feb_2021_15:04:05_+0000`.
