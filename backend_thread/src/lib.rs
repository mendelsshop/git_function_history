use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    thread,
    time::Duration,
};

use git_function_history::get_function_history;
use types::{FilterType, FullCommand, HistoryFilter, CommmitFilterValue, CommitOrFileFilter};

use crate::types::{CommandResult, Index, ListType, Status};

pub mod types;

pub fn command_thread(
    rx_t: Receiver<FullCommand>,
    tx_t: Sender<(CommandResult, Status)>,
    log: bool,
) {
    thread::spawn(move || {
        loop {
            // TODO: change all the commented out println! to log
            match rx_t.recv_timeout(Duration::from_millis(100)) {
                Err(a) => match a {
                    RecvTimeoutError::Timeout => {
                        if log {
                            log::trace!("thread timeout");
                        }
                    }
                    RecvTimeoutError::Disconnected => {
                        if log {
                            log::warn!("thread disconnected");
                        }
                        panic!("channel disconnected");
                    }
                },
                Ok(msg) => match msg {
                    FullCommand::List(list_type) => {
                        if log {
                            log::info!("list");
                        }
                        match list_type {
                            ListType::Commits => {
                                match git_function_history::get_git_commits() {
                                    Ok(commits) => {
                                        if log {
                                            log::info!("found {} commits", commits.len());
                                        }
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
                                        if log {
                                            log::info!("found {} dates", dates.len());
                                        }
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
                        if log {
                            log::info!("Searching for {} in {:?}", name, file);
                        }
                        match get_function_history(&name, file, filter) {
                            Ok(functions) => {
                                let hist_len = functions.history.len();
                                let commit_len = if hist_len > 0 {
                                    functions.history[0].functions.len()
                                } else {
                                    0
                                };
                                if log {
                                    log::info!("Found functions",);
                                }
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
                                if log {
                                    log::info!("Found functions",);
                                }
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
                                if log {
                                    log::info!("Found functions",);
                                }
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
                                if log {
                                    log::info!("Found functions",);
                                }
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
                                if log {
                                    log::info!("Found functions",);
                                }
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
                        FilterType::CommitOrFile(filter, t) => {
                            match t {
                                CommmitFilterValue::Commit(com) => {
                                    match filter {
                                        CommitOrFileFilter::FunctionInFunction(function) => {
                                            let t = match com.get_function_with_parent(&function) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for function {}",
                                                            function
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            let hist_len = t.functions.len();
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::Commit(
                                                    t,
                                                    Index(hist_len, 0),
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                        CommitOrFileFilter::FunctionInBlock(block) => {
                                            let t = match com.get_function_from_block(block) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for block {}",
                                                            block
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            let hist_len = t.functions.len();
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::Commit(
                                                    t,
                                                    Index(hist_len, 0),
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                        CommitOrFileFilter::FunctionInLines(line1, line2) => {
                                            let t = match com.get_function_in_lines(line1, line2) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for lines {}-{}",
                                                            line1, line2
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            let hist_len = t.functions.len();
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::Commit(
                                                    t,
                                                    Index(hist_len, 0),
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                    }
                                }
                                CommmitFilterValue::File(file) => {
                                    match filter {
                                        CommitOrFileFilter::FunctionInFunction(function) => {
                                            let t = match file.get_function_with_parent(&function) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for function {}",
                                                            function
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::File(
                                                    t,
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                        CommitOrFileFilter::FunctionInBlock(block) => {
                                            let t = match file.get_function_from_block(block) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for block {}",
                                                            block
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::File(
                                                    t,
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                        CommitOrFileFilter::FunctionInLines(line1, line2) => {
                                            let t = match file.get_functin_in_lines(line1, line2) {
                                                Some(functions) => functions,
                                                None => {
                                                    tx_t.send((
                                                        CommandResult::None,
                                                        Status::Error(format!(
                                                            "No functions found for lines {}-{}",
                                                            line1, line2
                                                        )),
                                                    ))
                                                    .unwrap();
                                                    return;
                                                }
                                            
                                            };
                                            if log {
                                                log::info!("Found functions",);
                                            }
                                            tx_t.send((
                                                CommandResult::File(
                                                    t,
                                                ),
                                                Status::Ok(Some("Found functions".to_string())),
                                            ))
                                            .unwrap();
                                        }
                                    }
                                }
                                }
                            }
                        }
                    
                },
            }
        }
    });
}
