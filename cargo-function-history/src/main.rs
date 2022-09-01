use std::{
    cell::RefCell, env, error::Error, process::exit, rc::Rc, sync::mpsc, thread, time::Duration,
};

use cargo_function_history::{app::{App, CommandResult, ui::Status}, start_ui, types::{HistoryFilter, FullCommand, FilterType, ListType}};
use git_function_history::get_function_history;

fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_args();
    let history = match config.function_name {
        string if string.is_empty() => None,
        string => match config.file_name {
            file if file.is_empty() => match get_function_history(
                &string,
                git_function_history::FileType::None,
                git_function_history::Filter::None,
            ) {
                Ok(functions) => Some(functions),
                Err(_err) => None,
            },
            file => match get_function_history(
                &string,
                git_function_history::FileType::Absolute(file),
                git_function_history::Filter::None,
            ) {
                Ok(functions) => Some(functions),
                Err(_err) => None,
            },
        },
    };
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    thread::spawn(move || {
        loop {
            // TODO: do somethung like in inputs::events.rs and send a tick if there is no input
            // so that the reciver doesnt have to use recv_timeout, and it might speed up the app
            // TODO: change all the commented out println! to log
            match rx_t.recv_timeout(Duration::from_millis(100)) {
                Err(a) => {
                    match a {
                        mpsc::RecvTimeoutError::Timeout => {
                            // // println!("thread timeout");
                        }
                        mpsc::RecvTimeoutError::Disconnected => {
                            panic!("channel disconnected");
                        }
                    }
                }
                Ok(msg) => match msg {
                    FullCommand::List(list_type) => {
                        // println!("list");
                        match list_type {
                            ListType::Commits => {
                                match git_function_history::get_git_commits() {
                                    Ok(commits) => {
                                        // println!("found {} commits", commits.len());
                                        tx_t.send((
                                            CommandResult::String(commits),
                                            Status::Ok(Some("Found commits dates".to_string())),
                                        ))
                                        .unwrap();
                                    }
                                    Err(err) => {
                                        tx_t.send((
                                            CommandResult::None,
                                            Status::Error(err.to_string()),
                                        ))
                                        .unwrap();
                                    }
                                };
                            }
                            ListType::Dates => {
                                match git_function_history::get_git_dates() {
                                    Ok(dates) => {
                                        // println!("found {} dates", dates.len());
                                        tx_t.send((
                                            CommandResult::String(dates),
                                            Status::Ok(Some("Found dates".to_string())),
                                        ))
                                        .unwrap();
                                    }
                                    Err(err) => {
                                        tx_t.send((
                                            CommandResult::None,
                                            Status::Error(err.to_string()),
                                        ))
                                        .unwrap();
                                    }
                                };
                            }
                        }
                    }
                    FullCommand::Search(name, file, filter) => {
                        // println!("Searching for {} in {:?}", name, file);
                        match git_function_history::get_function_history(&name, file, filter) {
                            Ok(functions) => {
                                let hist_len = functions.history.len();
                                let _commit_len = if hist_len > 0 {
                                    functions.history[0].functions.len()
                                } else {
                                    0
                                };
                                // println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        functions,
                                        
                                        
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            Err(err) => {
                                tx_t.send((CommandResult::None, Status::Error(err.to_string())))
                                    .unwrap();
                            }
                        };
                    }
                    FullCommand::Filter(t) => match t {
                        FilterType::History(functions, history) => match functions {
                            HistoryFilter::Date(date) => {
                                match history.get_by_date(&date) {
                                    Some(functions) => {
                                        tx_t.send((
                                            CommandResult::Commit(
                                                functions.clone(),
                                            ),
                                            Status::Ok(Some("Found functions".to_string())),
                                        ))
                                        .unwrap();
                                    }
                                    None => {
                                        tx_t.send((
                                            CommandResult::None,
                                            Status::Error(format!(
                                                "No functions found for date {}",
                                                date
                                            )),
                                        ))
                                        .unwrap();
                                    }
                                };
                            }
                            HistoryFilter::CommitId(commit) => {
                                match history.get_by_commit_id(&commit) {
                                    Some(functions) => {
                                        tx_t.send((
                                            CommandResult::Commit(
                                                functions.clone(),
                                                
                                            ),
                                            Status::Ok(Some("Found functions".to_string())),
                                        ))
                                        .unwrap();
                                    }
                                    None => {
                                        tx_t.send((
                                            CommandResult::None,
                                            Status::Error(format!(
                                                "No functions found for commit {}",
                                                commit
                                            )),
                                        ))
                                        .unwrap();
                                    }
                                };
                            }
                            HistoryFilter::DateRange(frst, scd) => {
                                let t = history.get_date_range(&frst, &scd);
                                let hist_len = t.history.len();
                                let _commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                // println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        
                                        
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInBlock(block) => {
                                let t = history.get_all_functions_in_block(block);
                                let hist_len = t.history.len();
                                let _commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                // println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        
                                        
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInLines(line1, line2) => {
                                let t = history.get_all_functions_line(line1, line2);
                                let hist_len = t.history.len();
                                let _commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                // println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        
                                        
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInFunction(function) => {
                                let t = history.get_all_function_with_parent(&function);
                                let hist_len = t.history.len();
                                let _commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                // println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        
                                        
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                        },
                        FilterType::CommitOrFile(_commit, _t) => {}
                    },
                },
            }
        }
    });
    let app = Rc::new(RefCell::new(App::new(history, (tx_m, rx_m)))); // TODO app is useless for now
    start_ui(app)?;
    Ok(())
}

fn usage() -> ! {
    // println!("Usage: cargo function-history [function-name]:[filename] <options>");
    // println!("Available options:");
    // println!("  --help - show this message");
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
                    // println!("Error:\n\tUnknown argument: {}\n\tTip: use --help to see available arguments.", arg.1);
                    exit(1);
                }
            }
        }
    });
    config
}
