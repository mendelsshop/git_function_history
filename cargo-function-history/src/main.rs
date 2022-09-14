use std::{cell::RefCell, env, error::Error, process::exit, rc::Rc, sync::mpsc};

use cargo_function_history::{app::App, start_ui};
use function_history_backend_thread::types::FullCommand;
use git_function_history::{FileType, Filter};
use log::info;

fn main() -> Result<(), Box<dyn Error>> {
    simple_file_logger::init_logger("cargo_function_history", simple_file_logger::LogLevel::Info)?;
    info!("Starting cargo function history");
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    function_history_backend_thread::command_thread(rx_t, tx_t, true);
    info!("started command thread");
    let config = parse_args();
    match config.function_name {
        string if string.is_empty() => {}
        string => tx_m.send(FullCommand::Search(string, config.file_type, config.filter))?,
    };
    let app = Rc::new(RefCell::new(App::new((tx_m, rx_m))));
    start_ui(app)?;
    Ok(())
}

fn usage() -> ! {
    println!("Usage: cargo function-history <function-name<:filename>> <options>");
    println!("Available options:");
    println!("  --help - show this message");
    println!("  --file-absolute - search the exact file with the filename specified after the function name");
    println!("  --file-relative - search any file ending with the filename specified after the function name");
    println!("  --filter-date=<date> - filter to the given date");
    println!("  --filter-commit-hash=<hash> - filter to the given commit hash");
    println!(" --filter-date-range=<date1>:<date2> - filter to the given date range");
    exit(1);
}

#[derive(Debug)]
struct Config {
    function_name: String,
    filter: Filter,
    file_type: FileType,
}

fn parse_args() -> Config {
    let mut config = Config {
        function_name: String::new(),
        filter: Filter::None,
        file_type: FileType::None,
    };
    env::args().enumerate().skip(1).for_each(|arg| {
        if arg.0 == 1 {
            println!("{}", arg.1);
            match arg.1.split_once(':') {
                Some(string_tuple) => {
                    config.file_type = FileType::Relative(string_tuple.1.replace('\\', "/"));
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
                "--file-absolute" => {
                    match &config.file_type {
                        FileType::None => {
                            eprintln!("Error no file name specified");
                            exit(1);
                        }
                        FileType::Relative(path) => {
                            config.file_type = FileType::Absolute(path.to_string());

                        }
                        _ => {}
                    }
                }
                "--file-relative" => {
                    match &config.file_type {
                        FileType::None => {
                            eprintln!("Error no file name specified");
                            exit(1);
                        }
                        FileType::Absolute(path) => {
                            config.file_type = FileType::Relative(path.to_string());
                        }
                        _ => {}
                    }
                }
                string if string.starts_with("--filter-date=") => {
                    let date = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no date specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::Date(date.to_string());
                }
                string if string.starts_with("--filter-commit-hash=") => {
                    let hash = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no commit hash specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::CommitId(hash.to_string());
                }
                string if string.starts_with("--date-range=") => {
                    let date_range = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no date range specified");
                            exit(1);
                        }
                    };
                    let date_range = match date_range.split_once(':') {
                        Some(string_tuple) => string_tuple,
                        None => {
                            eprintln!("Error no end date specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::DateRange(date_range.0.to_string(), date_range.1.to_string());
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
