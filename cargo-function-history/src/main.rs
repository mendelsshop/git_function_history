use std::{cell::RefCell, env, error::Error, process::exit, rc::Rc};

use cargo_function_history::{app::App, start_ui};
use git_function_history::{get_function, get_all_functions};

fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_args();
    let history = match config.function_name {
        string if string.is_empty() => {None}
        string => {
            match config.file_name {
                file if file.is_empty() => {
                    match get_all_functions(&string) {
                        Ok(functions) => Some(functions),
                        Err(err) => None,
                    }
                }
                file => {
                    match get_function(&string, &file) {
                        Ok(functions) => Some(functions),
                        Err(err) => None,
                    }
                }
            }
        }
    };
    let app = Rc::new(RefCell::new(App::new(history))); // TODO app is useless for now
    start_ui(app)?;
    Ok(())
}

fn usage() -> ! {
    println!("Usage: cargo function-history [function-name]:[filename] <options>");
    println!("Available options:");
    println!("  --help - show this message");
    exit(1);
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
    env::args().enumerate().skip(1).for_each(|arg| {
        if arg.0 == 1 {
            match arg.1.split_once(':') {
                Some(string_tuple) => {
                    config.file_name = string_tuple.1.replace('\\', "/");
                    config.function_name = string_tuple.0.to_string();
                }
                None => {
                    config.function_name = arg.1.to_string();
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
