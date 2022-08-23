use git_function_history_gui::{
    types::{CommandResult, FullCommand, ListType, Status},
    MyEguiApp,
};
use std::{sync::mpsc, thread, time::Duration};

use eframe::{epaint::Vec2, run_native};
fn main() {
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();

    let thread = thread::spawn(move || {
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
                                println!("Found functions",);
                                tx_t.send((
                                    CommandResult::History(functions),
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
                    FullCommand::Filter() => {
                        println!("filter");
                    }
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
    thread.join().unwrap();
}
