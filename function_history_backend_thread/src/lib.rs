use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    thread,
    time::Duration,
};

use git_function_history::get_function_history;
use types::FullCommand;

use crate::types::{CommandResult, ListType, Status};

pub mod types;

pub fn command_thread(
    rx_t: Receiver<FullCommand>,
    tx_t: Sender<(CommandResult, Status)>,
    log: bool,
) {
    thread::spawn(move || loop {
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
                    break;
                }
            },
            Ok(msg) => {
                let msg = match msg {
                    FullCommand::List(list_type) => {
                        if log {
                            log::info!("list");
                        }
                        match list_type {
                            ListType::Commits => {
                                match git_function_history::get_git_commit_hashes() {
                                    Ok(commits) => {
                                        if log {
                                            log::info!("found {} commits", commits.len());
                                        }
                                        (
                                            CommandResult::String(commits),
                                            Status::Ok(Some("Found commits dates".to_string())),
                                        )
                                    }
                                    Err(err) => {
                                        (CommandResult::None, Status::Error(err.to_string()))
                                    }
                                }
                            }
                            ListType::Dates => match git_function_history::get_git_dates() {
                                Ok(dates) => {
                                    if log {
                                        log::info!("found {} dates", dates.len());
                                    }
                                    (
                                        CommandResult::String(dates),
                                        Status::Ok(Some("Found dates".to_string())),
                                    )
                                }
                                Err(err) => (CommandResult::None, Status::Error(err.to_string())),
                            },
                        }
                    }
                    FullCommand::Search(name, file, filter) => {
                        if log {
                            log::info!("Searching for {} in {:?}", name, file);
                        }
                        match get_function_history(&name, file, filter) {
                            Ok(functions) => {
                                if log {
                                    log::info!("Found functions");
                                }
                                (
                                    CommandResult::History(functions),
                                    Status::Ok(Some("Found functions".to_string())),
                                )
                            }
                            Err(err) => (CommandResult::None, Status::Error(err.to_string())),
                        }
                    }
                    FullCommand::Filter(filter) => {
                        if let CommandResult::History(hist) = filter.thing {
                            if log {
                                log::info!("Filtering history with filter {:?}", filter.filter);
                            }
                            match hist.filter_by(filter.filter) {
                                Ok(hist) => {
                                    if log {
                                        log::info!("Filtered history");
                                    }
                                    (
                                        CommandResult::History(hist),
                                        Status::Ok(Some("Filtered history".to_string())),
                                    )
                                }
                                Err(err) => (CommandResult::None, Status::Error(err.to_string())),
                            }
                        } else {
                            (
                                CommandResult::None,
                                Status::Error("Can't filter this".to_string()),
                            )
                        }
                    }
                };
                tx_t.send(msg).unwrap();
            }
        }
    });
}
