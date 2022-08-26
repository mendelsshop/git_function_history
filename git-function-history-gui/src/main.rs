use git_function_history_gui::{
    types::{CommandResult, FilterType, FullCommand, HistoryFilter, Index, ListType, Status},
    MyEguiApp,
};
use std::{sync::mpsc, thread, time::Duration};

use eframe::{epaint::Vec2, run_native};
fn main() {
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();

    thread::spawn(move || {
        loop {
            match rx_t.recv_timeout(Duration::from_millis(100)) {
                Ok(msg) => match msg {
                    FullCommand::List(list_type) => {
                        println!("list");
                        match list_type {
                            ListType::Commits => {
                                match git_function_history::get_git_commits() {
                                    Ok(commits) => {
                                        println!("found {} commits", commits.len());
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
                                        println!("found {} dates", dates.len());
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
                        println!("Searching for {} in {:?}", name, file);
                        match git_function_history::get_function_history(&name, file, filter) {
                            Ok(functions) => {
                                let hist_len = functions.history.len();
                                let commit_len = if hist_len > 0 {
                                    functions.history[0].functions.len()
                                } else {
                                    0
                                };
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        functions,
                                        Index(hist_len, 0),
                                        Index(commit_len, 0),
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
                                                Index(functions.functions.len(), 0),
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
                                                Index(functions.functions.len(), 0),
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
                                let commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        Index(hist_len, 0),
                                        Index(commit_len, 0),
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInBlock(block) => {
                                let t = history.get_all_functions_in_block(block);
                                let hist_len = t.history.len();
                                let commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        Index(hist_len, 0),
                                        Index(commit_len, 0),
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInLines(line1, line2) => {
                                let t = history.get_all_functions_line(line1, line2);
                                let hist_len = t.history.len();
                                let commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        Index(hist_len, 0),
                                        Index(commit_len, 0),
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                            HistoryFilter::FunctionInFunction(function) => {
                                let t = history.get_all_function_with_parent(&function);
                                let hist_len = t.history.len();
                                let commit_len = if hist_len > 0 {
                                    t.history[0].functions.len()
                                } else {
                                    0
                                };
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(
                                        t,
                                        Index(hist_len, 0),
                                        Index(commit_len, 0),
                                    ),
                                    Status::Ok(Some("Found functions".to_string())),
                                ))
                                .unwrap();
                            }
                        },
                        FilterType::CommitOrFile(_commit, _t) => {}
                    },
                },
                Err(a) => {
                    match a {
                        mpsc::RecvTimeoutError::Timeout => {
                            // println!("thread timeout");
                        }
                        mpsc::RecvTimeoutError::Disconnected => {
                            panic!("channel disconnected");
                        }
                    }
                }
            };
        }
    });
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(800.0, 600.0));
    run_native(
        "Git Function History",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, (tx_m, rx_m)))),
    );
    // thread.join().unwrap();
}
