use std::{
    cell::RefCell,
    io::stdout,
    rc::Rc,
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    thread,
    time::Duration,
};

use crate::{app::ui, types::Index};
use app::{state::AppState, ui::Status, App, AppReturn, CommandResult};
use eyre::Result;
use git_function_history::get_function_history;
use inputs::{events::Events, InputEvent};
use tui::{backend::CrosstermBackend, Terminal};
use types::{FilterType, FullCommand, HistoryFilter, ListType};

pub mod app;
pub mod inputs;
pub mod types;
pub fn start_ui(app: Rc<RefCell<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // User event handler
    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    loop {
        let mut app = app.borrow_mut();

        // Render
        terminal.draw(|rect| ui::draw(rect, &mut app))?;

        // Handle inputs
        match &mut app.state() {
            AppState::Editing => {
                match events.next()? {
                    InputEvent::Input(key) => app.do_edit_action(key),
                    InputEvent::Tick => {}
                };
            }
            _ => {
                let result = match events.next()? {
                    InputEvent::Input(key) => app.do_action(key),
                    InputEvent::Tick => AppReturn::Continue,
                };
                // Check if we should exit
                if result == AppReturn::Exit {
                    break;
                }
            }
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

pub fn command_thread(rx_t: Receiver<FullCommand>, tx_t: Sender<(CommandResult, Status)>) {
    thread::spawn(move || {
        loop {
            // TODO: change all the commented out println! to log
            match rx_t.recv_timeout(Duration::from_millis(100)) {
                Err(a) => match a {
                    RecvTimeoutError::Timeout => {
                        log::trace!("thread timeout");
                    }
                    RecvTimeoutError::Disconnected => {
                        log::warn!("thread disconnected");
                        panic!("channel disconnected");
                    }
                },
                Ok(msg) => match msg {
                    FullCommand::List(list_type) => {
                        log::info!("list");
                        match list_type {
                            ListType::Commits => {
                                match git_function_history::get_git_commits() {
                                    Ok(commits) => {
                                        log::info!("found {} commits", commits.len());
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
                                        log::info!("found {} dates", dates.len());
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
                        log::info!("Searching for {} in {:?}", name, file);
                        match get_function_history(&name, file, filter) {
                            Ok(functions) => {
                                let hist_len = functions.history.len();
                                let commit_len = if hist_len > 0 {
                                    functions.history[0].functions.len()
                                } else {
                                    0
                                };
                                log::info!("Found functions",);
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
                                log::info!("Found functions",);
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
                                log::info!("Found functions",);
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
                                log::info!("Found functions",);
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
                                log::info!("Found functions",);
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
            }
        }
    });
}
