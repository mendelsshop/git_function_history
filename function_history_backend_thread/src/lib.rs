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
                let now = std::time::Instant::now();
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
                                            Status::Ok(Some(format!(
                                                "Found commits dates took {}s",
                                                now.elapsed().as_secs()
                                            ))),
                                        )
                                    }
                                    Err(err) => (
                                        CommandResult::None,
                                        Status::Error(format!(
                                            "Error getting commits: {} took {}s",
                                            err,
                                            now.elapsed().as_secs()
                                        )),
                                    ),
                                }
                            }
                            ListType::Dates => match git_function_history::get_git_dates() {
                                Ok(dates) => {
                                    if log {
                                        log::info!("found {} dates", dates.len());
                                    }
                                    (
                                        CommandResult::String(dates),
                                        Status::Ok(Some(format!(
                                            "Found dates took {}s",
                                            now.elapsed().as_secs()
                                        ))),
                                    )
                                }
                                Err(err) => (
                                    CommandResult::None,
                                    Status::Error(format!(
                                        "Error getting dates: {} took {}s",
                                        err,
                                        now.elapsed().as_secs()
                                    )),
                                ),
                            },
                        }
                    }
                    FullCommand::Search(name, file, _filter, lang) => {
                        if log {
                            log::info!("Searching for {} in {:?}", name, file);
                        }
                        match get_function_history(&name, &file, &lang) {
                            Ok(functions) => {
                                if log {
                                    log::info!("Found functions");
                                }
                                (
                                    CommandResult::History(functions),
                                    Status::Ok(Some(format!(
                                        "Found functions took {}s",
                                        now.elapsed().as_secs()
                                    ))),
                                )
                            }
                            Err(err) => (
                                CommandResult::None,
                                Status::Error(format!(
                                    "Error getting functions: {} took {}s",
                                    err,
                                    now.elapsed().as_secs()
                                )),
                            ),
                        }
                    }
                    FullCommand::Filter(filter) => {
                        if let CommandResult::History(hist) = filter.thing {
                            if log {
                                log::info!("Filtering history with filter {:?}", filter.filter);
                            }
                            match hist.filter_by(&filter.filter) {
                                Ok(hist) => {
                                    if log {
                                        log::info!("Filtered history");
                                    }
                                    (
                                        CommandResult::History(hist),
                                        Status::Ok(Some(format!(
                                            "Filtered history took {}s",
                                            now.elapsed().as_secs()
                                        ))),
                                    )
                                }
                                Err(err) => (
                                    CommandResult::None,
                                    Status::Error(format!(
                                        "Error filtering history: {} took {}s",
                                        err,
                                        now.elapsed().as_secs()
                                    )),
                                ),
                            }
                        } else {
                            (
                                CommandResult::None,
                                Status::Error(format!(
                                    "Can't filter this took {}s",
                                    now.elapsed().as_secs()
                                )),
                            )
                        }
                    }
                };
                if log {
                    log::info!("thread finished in {}s", now.elapsed().as_secs());
                }
                tx_t.send(msg).unwrap();
            }
        }
    });
}
