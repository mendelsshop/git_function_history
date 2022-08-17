use git_function_history::get_function;
use std::{env, process::exit};

fn main() {
    let config = parse_args();
    let function = get_function(&config.function_name, &config.file_name);
    if function.is_err() {
        eprintln!("Error:\n\t{}", function.unwrap_err());
        exit(1);
    }
    // TODO: print the latest commit and use cross-term to get keyborad input for the arrow keys to go to the next commit or previous commit
    // and use up and down arrow keys to scroll through a commit if the commit is too long to fit on the screen
    println!("{}", function.unwrap());
}

fn usage() -> ! {
    println!("Usage: cargo function-history [function-name]:[filename] <options>");
    println!("Available options:");
    println!("  --help - show this message");
    exit(1)
}

#[derive(Debug)]
struct Config {
    file_name: String,
    function_name: String,
}

fn parse_args() -> Config {
    let mut config = Config {
        file_name: String::new(),
        function_name: String::new(),
    };
    if env::args().count() < 2 {
        usage();
    }
    env::args().enumerate().skip(1).for_each(|arg| {
        if arg.0 == 1 {
            match arg.1.split_once(':') {
                Some(string_tuple) => {
                    config.file_name = string_tuple.1.replace('\\', "/");
                    config.function_name = string_tuple.0.to_string();
                }
                _ => {
                    println!("Error:\n\tExpected funtion-name:file-name.\n\tFound function-name.\n\tTip: make sure to separate the function-name and file-name with a colon (:).");
                    exit(1);
                }
            }
        } else {
            match arg.1.as_str() {
                "--help" => {
                    usage();
                }
                _ => {
                    println!("Error:\n\tUnknown argument: {}\n\tTip: use --help to see available arguments.", arg.1);
                    exit(1);
                }
            }
        }
    });
    config
}
