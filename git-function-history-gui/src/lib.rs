mod types;

use std::{collections::HashMap, sync::mpsc, time::Duration};

use eframe::{
    self,
    egui::{self, Button, Label, Layout, Sense, SidePanel, TextEdit, TopBottomPanel, Visuals},
    epaint::{Color32, Vec2},
};
use function_history_backend_thread::types::{
    Command, CommandResult, FilterType, FullCommand, ListType, SearchType, Status,
};
use git_function_history::{types::Directions, Commit, FileFilterType, Filter, FunctionHistory};
use itertools::Itertools;
use types::HistoryFilterType;

// TODO: stop cloning everyting and use references instead
pub struct MyEguiApp {
    command: Command,
    dark_theme: bool,
    input_buffer: String,
    cmd_output: CommandResult,
    status: Status,
    list_type: ListType,
    channels: (
        mpsc::Sender<FullCommand>,
        mpsc::Receiver<(CommandResult, Status)>,
    ),
    filter: Filter,
    file_type: FileFilterType,
    history_filter_type: types::HistoryFilterType,
    current_commit: String,
    do_commit: bool,
}

impl MyEguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        channels: (
            mpsc::Sender<FullCommand>,
            mpsc::Receiver<(CommandResult, Status)>,
        ),
    ) -> Self {
        Self {
            dark_theme: true,
            command: Command::Search,
            input_buffer: String::new(),
            cmd_output: CommandResult::None,
            status: Status::default(),
            list_type: ListType::default(),
            channels,
            file_type: FileFilterType::None,
            filter: Filter::None,
            history_filter_type: types::HistoryFilterType::None,
            current_commit: String::new(),
            do_commit: false,
        }
    }

    fn draw_config_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("My Window")
            .open(&mut true)
            .show(ctx, |ui| {
                if ui.button("cancel").clicked() {
                    self.current_commit = String::new();
                }
            });
    }

    fn draw_commit(commit: &mut Commit, ctx: &egui::Context, show: bool) {
        if show {
            TopBottomPanel::top("date_id").show(ctx, |ui| {
                ui.add(Label::new(format!(
                    "Commit: {}",
                    commit.get_metadata()["commit hash"]
                )));
                ui.add(Label::new(format!(
                    "Date: {}",
                    commit.get_metadata()["date"]
                )));
            });
        }
        TopBottomPanel::top("file_name").show(ctx, |ui| {
            ui.add(Label::new(format!(
                "File {}",
                commit.get_metadata()["file"]
            )));
        });
        let file = commit.get_file().map(|x| x.to_string()).unwrap_or_else(|| "error occured could not retrieve file please file a bug report  to https://github.com/mendelsshop/git_function_history/issues".to_string());
        match commit.get_move_direction() {
            Directions::None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(Label::new(file));
                        });
                });
            }
            Directions::Forward => {
                // split the screen in two parts, most of it is for the content, the and leave a small part for the right arrow
                log::debug!("found at least one file index beginning");
                let resp = egui::SidePanel::right("right_arrow")
                    .show(ctx, |ui| {
                        ui.set_width(0.5);
                        ui.add_sized(
                            Vec2::new(ui.available_width(), ui.available_height()),
                            Button::new("->"),
                        )
                    })
                    .inner;
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| ui.add(Label::new(file)));
                });
                if resp.clicked() {
                    commit.move_forward();
                }
            }
            Directions::Back => {
                log::debug!("found at least one file index end");
                // split the screen in two parts, leave a small part for the left arrow and the rest for the content
                let resp = SidePanel::left("right_button")
                    .show(ctx, |ui| {
                        ui.set_width(1.0);
                        ui.add_sized(
                            Vec2::new(ui.available_width(), ui.available_height()),
                            Button::new("<-"),
                        )
                    })
                    .inner;
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(Label::new(file));
                        });
                });
                if resp.clicked() {
                    commit.move_back();
                }
            }
            Directions::Both => {
                log::debug!("found at least one file index middle");
                // split screen into 3 parts, leave a small part for the left arrow, the middle part for the content and leave a small part for the right arrow
                let l_resp = SidePanel::left("left_arrow")
                    .show(ctx, |ui| {
                        ui.set_width(1.0);
                        ui.add_sized(
                            Vec2::new(ui.available_width(), ui.available_height()),
                            Button::new("<-"),
                        )
                    })
                    .inner;
                let r_resp = egui::SidePanel::right("right_arrows")
                    .show(ctx, |ui| {
                        ui.set_width(1.0);
                        ui.add_sized(
                            Vec2::new(ui.available_width(), ui.available_height()),
                            Button::new("->"),
                        )
                    })
                    .inner;
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(Label::new(file));
                        });
                });
                if l_resp.clicked() {
                    commit.move_back();
                } else if r_resp.clicked() {
                    commit.move_forward();
                }
            }
        }
    }

    fn draw_history(history: &mut FunctionHistory, ctx: &egui::Context) {
        // split the screen top and bottom into two parts, leave small part for the left arrow commit hash and right arrow and the rest for the content
        // create a 3 line header
        TopBottomPanel::top("control history").show(ctx, |ui| {
            ui.set_height(2.0);
            ui.horizontal(|ui| {
                let mut max = ui.available_width();
                let l_resp = match history.get_move_direction() {
                    Directions::Forward => {
                        ui.add_sized(Vec2::new(2.0, 2.0), Button::new("<-").sense(Sense::hover()));
                        None
                    }
                    _ => Some(
                        // add a left arrow button that is disabled
                        ui.add_sized(Vec2::new(2.0, 2.0), Button::new("<-")),
                    ),
                };
                max -= ui.available_width();
                ui.add_sized(
                    Vec2::new(ui.available_width() - max, 2.0),
                    Label::new(format!(
                        "{}\n{}",
                        history
                            .get_metadata()
                            .get("commit hash")
                            .map_or("could not retrieve commit hash", |x| x.as_str()),
                        history
                            .get_metadata()
                            .get("date")
                            .map_or("could not retieve date", |x| x.as_str()),
                    )),
                );

                let r_resp = match history.get_move_direction() {
                    Directions::Back => {
                        ui.add_sized(Vec2::new(2.0, 2.0), Button::new("->").sense(Sense::hover()));
                        None
                    }
                    _ => {
                        // add a right arrow button that is disabled
                        Some(ui.add_sized(Vec2::new(2.0, 2.0), Button::new("->")))
                    }
                };

                if let Some(r_resp) = r_resp {
                    if r_resp.clicked() {
                        history.move_forward();
                    }
                }
                if let Some(l_resp) = l_resp {
                    if l_resp.clicked() {
                        history.move_back();
                    }
                }
            });
        });
        if let Some(x) = history.get_mut_commit() {
            Self::draw_commit(x, ctx, false)
        } else {
            TopBottomPanel::top("no_commit_found").show(ctx, |ui| {
                ui.add(Label::new("no commit found"));
            });
        }
    }
}
macro_rules! draw_text_input {
    ($ui:expr, $max:expr,  $($field:expr)+) => {{
        $($ui.horizontal(|ui| {
            // set the width of the input field
            ui.set_min_width(4.0);
            ui.set_max_width($max);
            ui.add(TextEdit::singleline($field));
        });)*
    }};
}
macro_rules! draw_selecction {
    () => {};
}
impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if self.dark_theme {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        if self.do_commit {
            self.draw_config_window(ctx);
            if self.current_commit.is_empty() {
                self.do_commit = false;
            }
        } else {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.add_space(20.);
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
                        match &self.status {
                            Status::Loading => {
                                ui.colored_label(Color32::BLUE, "Loading...");
                            }
                            Status::Ok(a) => match a {
                                Some(a) => {
                                    ui.colored_label(Color32::LIGHT_GREEN, format!("Ok: {a}"));
                                }
                                None => {
                                    ui.colored_label(Color32::GREEN, "Ready");
                                }
                            },
                            Status::Warning(a) => {
                                ui.colored_label(Color32::LIGHT_RED, format!("Warn: {a}"));
                            }
                            Status::Error(a) => {
                                ui.colored_label(Color32::LIGHT_RED, format!("Error: {a}"));
                            }
                        }
                    });
                    // controls
                    ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                        let theme_btn = ui.add(Button::new({
                            if self.dark_theme {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        }));
                        if theme_btn.clicked() {
                            self.dark_theme = !self.dark_theme;
                        }
                    });
                });

                ui.add_space(20.);
            });
            egui::TopBottomPanel::bottom("commnad_builder").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::ScrollArea::horizontal()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let max = ui.available_width() / 6.0;
                            egui::ComboBox::from_id_source("command_combo_box")
                                .selected_text(self.command.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.command,
                                        Command::Filter,
                                        "filter",
                                    );
                                    ui.selectable_value(
                                        &mut self.command,
                                        Command::Search,
                                        "search",
                                    );
                                    ui.selectable_value(&mut self.command, Command::List, "list");
                                });
                            match self.command {
                                Command::Filter => {
                                    match &self.cmd_output {
                                        CommandResult::History(_) => {
                                            // Options 1. by date 2. by commit hash 3. in date range 4. function in block 5. function in lines 6. function in function
                                            let text = match &self.history_filter_type {
                                                HistoryFilterType::None => {
                                                    "filter type".to_string()
                                                }
                                                a => a.to_string(),
                                            };
                                            egui::ComboBox::from_id_source("history_combo_box")
                                                .selected_text(text)
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::Date(String::new()),
                                                        "by date",
                                                    );
                                                    ui.selectable_value(
                                                    &mut self.history_filter_type,
                                                    HistoryFilterType::CommitHash(String::new()),
                                                    "by commit hash",
                                                );
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::DateRange(
                                                            String::new(),
                                                            String::new(),
                                                        ),
                                                        "in date range",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::FileAbsolute(
                                                            String::new(),
                                                        ),
                                                        "file absolute",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::FileRelative(
                                                            String::new(),
                                                        ),
                                                        "file relative",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::Directory(String::new()),
                                                        "directory",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.history_filter_type,
                                                        HistoryFilterType::None,
                                                        "none",
                                                    );
                                                    function_grep::filter::Filters::default()
                                                        .into_iter()
                                                        .sorted_by_cached_key(|(key, _)| {
                                                            key.clone()
                                                        })
                                                        .for_each(|filter| {
                                                            ui.selectable_value(
                                                                &mut self.history_filter_type,
                                                                HistoryFilterType::PL(
                                                                    if let function_grep::filter::FilterType::All(filter) =
                                                                        filter.1
                                                                    {
                                                                        filter
                                                                            .attributes()
                                                                            .into_iter()
                                                                            .map(|(attr, kind)| {
                                                                                (
                                                                                    attr.to_string(
                                                                                    ),
                                                                                    kind.to_string(
                                                                                    ),
                                                                                )
                                                                            })
                                                                            .collect()
                                                                    } else {
                                                                        HashMap::new()
                                                                    },
                                                                    filter.1,
                                                                ),
                                                                filter.0,
                                                            );
                                                        })
                                                });
                                            match &mut self.history_filter_type {
                                                HistoryFilterType::DateRange(line1, line2) => {
                                                    draw_text_input!(ui,max, line1 line2)
                                                }
                                                HistoryFilterType::Date(dir)
                                                | HistoryFilterType::CommitHash(dir)
                                                | HistoryFilterType::FileAbsolute(dir)
                                                | HistoryFilterType::FileRelative(dir)
                                                | HistoryFilterType::Directory(dir) => {
                                                    draw_text_input!(ui, max, dir)
                                                }
                                                HistoryFilterType::None => {
                                                    // do nothing
                                                }
                                                HistoryFilterType::PL(inputs, filters) => {
                                                    inputs.iter_mut().for_each(|(desc, field)| {
                                                        ui.horizontal(|ui| {
                                                            ui.set_min_width(4.0);
                                                            ui.set_max_width(max);
                                                            ui.label(desc.to_string());
                                                            ui.add(TextEdit::singleline(field));
                                                        });
                                                    if let function_grep::filter::FilterType::Many(_) = filters {

                                                            // TODO: update history filter type
                                                            // hashmap to keep track of if a field
                                                            // is removed
                                            let resp = ui.add(Button::new("-"));
                                            if resp.clicked() {
                                                            }
                                                        }
                                                    });
                                                    if let function_grep::filter::FilterType::Many(_) = filters {
                                            let resp = ui.add(Button::new("add field"));
                                            if resp.clicked() {
                                                            let total = inputs.len()+ 1;
                                                            inputs.insert(format!("field{total}"), String::new());
                                                        }
                                                    }
                                                }
                                            }
                                            let resp = ui.add(Button::new("Go"));
                                            if resp.clicked() {
                                                self.status = Status::Loading;
                                                let filter = match &self.history_filter_type {
                                                    HistoryFilterType::Date(date) => {
                                                        Some(Filter::Date(date.to_string()))
                                                    }
                                                    HistoryFilterType::CommitHash(commit_hash) => {
                                                        Some(Filter::CommitHash(
                                                            commit_hash.to_string(),
                                                        ))
                                                    }
                                                    HistoryFilterType::DateRange(date1, date2) => {
                                                        Some(Filter::DateRange(
                                                            date1.to_string(),
                                                            date2.to_string(),
                                                        ))
                                                    }
                                                    HistoryFilterType::FileAbsolute(file) => {
                                                        Some(Filter::FileAbsolute(file.to_string()))
                                                    }
                                                    HistoryFilterType::FileRelative(file) => {
                                                        Some(Filter::FileRelative(file.to_string()))
                                                    }
                                                    HistoryFilterType::Directory(dir) => {
                                                        Some(Filter::Directory(dir.to_string()))
                                                    }
                                                    HistoryFilterType::None => {
                                                        self.status = Status::Ok(None);
                                                        None
                                                    }
                                                    HistoryFilterType::PL(input, filter) => {
                                                        let filter = filter.to_filter(
                                                            &input
                                                                .values()
                                                                .cloned()
                                                                .collect::<Vec<_>>()
                                                                .join(" "),
                                                        );
                                                        filter
                                                            .inspect_err(|e| {
                                                                self.status =
                                                                    Status::Error(e.to_string())
                                                            })
                                                            .ok()
                                                            .map(Filter::PLFilter)
                                                    }
                                                };
                                                if let Some(filter) = filter {
                                                    self.channels
                                                        .0
                                                        .send(FullCommand::Filter(FilterType {
                                                            thing: self.cmd_output.clone(),
                                                            filter,
                                                        }))
                                                        .expect("could not send message in thread");
                                                }
                                            }
                                        }

                                        _ => {
                                            ui.add(Label::new("No filters available"));
                                        }
                                    }
                                }
                                Command::Search => {
                                    ui.add(Label::new("Function Name:"));

                                    draw_text_input!(ui, max, &mut self.input_buffer);
                                    let text = match &self.file_type {
                                        FileFilterType::Directory(_) => "directory",
                                        FileFilterType::Absolute(_) => "absolute",
                                        FileFilterType::Relative(_) => "relative",
                                        _ => "file type",
                                    };
                                    egui::ComboBox::from_id_source("search_file_combo_box")
                                        .selected_text(text)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.file_type,
                                                FileFilterType::None,
                                                "None",
                                            );
                                            ui.selectable_value(
                                                &mut self.file_type,
                                                FileFilterType::Relative(String::new()),
                                                "Relative",
                                            );
                                            ui.selectable_value(
                                                &mut self.file_type,
                                                FileFilterType::Absolute(String::new()),
                                                "Absolute",
                                            );
                                            ui.selectable_value(
                                                &mut self.file_type,
                                                FileFilterType::Directory(String::new()),
                                                "Directory",
                                            );
                                        });
                                    match &mut self.file_type {
                                        FileFilterType::None => {}
                                        FileFilterType::Relative(dir)
                                        | FileFilterType::Absolute(dir)
                                        | FileFilterType::Directory(dir) => {
                                            draw_text_input!(ui, max, dir)
                                        }
                                    }
                                    // get filters if any
                                    let text = match &self.filter {
                                        Filter::CommitHash(_) => "commit hash".to_string(),
                                        Filter::DateRange(..) => "date range".to_string(),
                                        Filter::Date(_) => "date".to_string(),
                                        _ => "filter type".to_string(),
                                    };
                                    egui::ComboBox::from_id_source(
                                        "search_search_filter_combo_box",
                                    )
                                    .selected_text(text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.filter, Filter::None, "None");
                                        ui.selectable_value(
                                            &mut self.filter,
                                            Filter::CommitHash(String::new()),
                                            "Commit Hash",
                                        );
                                        ui.selectable_value(
                                            &mut self.filter,
                                            Filter::Date(String::new()),
                                            "Date",
                                        );
                                        ui.selectable_value(
                                            &mut self.filter,
                                            Filter::DateRange(String::new(), String::new()),
                                            "Date Range",
                                        );
                                    });

                                    // let
                                    match &mut self.filter {
                                        Filter::None => {}
                                        Filter::CommitHash(thing) | Filter::Date(thing) => {
                                            draw_text_input!(ui, max, thing)
                                        }
                                        Filter::DateRange(start, end) => {
                                            draw_text_input!(ui, max, start);
                                            ui.add(Label::new("-"));
                                            draw_text_input!(ui, max, end)
                                        }
                                        _ => {}
                                    }
                                    //let text = self.language.to_string();
                                    //egui::ComboBox::from_id_source("search_language_combo_box")
                                    //    .selected_text(text)
                                    //    .show_ui(ui, |ui| {
                                    //        ui.selectable_value(
                                    //            &mut self.language,
                                    //            Language::Rust,
                                    //            "Rust",
                                    //        );
                                    //        #[cfg(feature = "c_lang")]
                                    //        ui.selectable_value(
                                    //            &mut self.language,
                                    //            Language::C,
                                    //            "C",
                                    //        );
                                    //        ui.selectable_value(
                                    //            &mut self.language,
                                    //            Language::Python,
                                    //            "Python",
                                    //        );
                                    //        ui.selectable_value(
                                    //            &mut self.language,
                                    //            Language::All,
                                    //            "All",
                                    //        );
                                    //    });
                                    let resp = ui.add(Button::new("Go"));
                                    if resp.clicked() {
                                        self.status = Status::Loading;
                                        self.channels
                                            .0
                                            .send(FullCommand::Search(SearchType::new(
                                                self.input_buffer.clone(),
                                                self.file_type.clone(),
                                                std::mem::replace(&mut self.filter, Filter::None),
                                            )))
                                            .expect("could not send message in thread");
                                    }
                                }
                                Command::List => {
                                    egui::ComboBox::from_id_source("list_type")
                                        .selected_text(self.list_type.to_string())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.list_type,
                                                ListType::Dates,
                                                "dates",
                                            );
                                            ui.selectable_value(
                                                &mut self.list_type,
                                                ListType::Commits,
                                                "commits",
                                            );
                                        });
                                    let resp = ui.add(Button::new("Go"));
                                    if resp.clicked() {
                                        self.status = Status::Loading;
                                        self.channels
                                            .0
                                            .send(FullCommand::List(self.list_type))
                                            .expect("could not send message in thread");
                                    }
                                }
                            }
                        });
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                // check if the channel has a message and if so set it to self.command
                match self.channels.1.recv_timeout(Duration::from_millis(100)) {
                    Ok(timeout) => match timeout {
                        (_, Status::Error(e)) => {
                            let e = e.split_once("why").unwrap_or((&e, ""));
                            let e = format!(
                                "error recieved last command didn't work; {}{}",
                                e.0,
                                e.1.split_once("why").unwrap_or(("", "")).0,
                            );
                            log::warn!("{}", e);
                            self.status = Status::Error(e);
                        }
                        (t, Status::Ok(msg)) => {
                            log::info!("got results of last command");
                            self.status = Status::Ok(msg);
                            self.cmd_output = t;
                        }
                        _ => {}
                    },
                    Err(e) => match e {
                        mpsc::RecvTimeoutError::Timeout => {}
                        mpsc::RecvTimeoutError::Disconnected => {
                            panic!("Disconnected");
                        }
                    },
                }
                // match self.commmand and render based on that
                match &mut self.cmd_output {
                    CommandResult::History(t) => {
                        Self::draw_history(t, ctx);
                    }

                    CommandResult::String(t) => {
                        let resp = ui.button("go");
                        egui::ScrollArea::vertical()
                            .max_height(f32::INFINITY)
                            .max_width(f32::INFINITY)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for line in t {
                                    if !line.is_empty() {
                                        // ui.add(Button::new(line.to_string()));
                                        ui.selectable_value(
                                            &mut self.current_commit,
                                            line.to_string(),
                                            line.to_string(),
                                        );
                                    }
                                }
                            });
                        if resp.clicked() {
                            // show a popup window
                            // let response = ui.button("Open popup");
                            // let popup_id = ui.make_persistent_id("my_unique_id");
                            // // if response.clicked() {
                            //     ui.memory().toggle_popup(popup_id);
                            // // }
                            // egui::popup::popup_below_widget(ui, popup_id, &resp, |ui| {
                            //     ui.set_min_width(200.0); // if you want to control the size
                            //     ui.label("Some more info, or things you can select:");
                            //     ui.label("…");
                            // });
                            // egui::Area::new("my_area")
                            //     .fixed_pos(egui::pos2(32.0, 32.0))
                            //     .show(ctx, |ui| {
                            //         ui.label("Floating text!");
                            //     });
                            self.do_commit = true;
                        }
                    }
                    CommandResult::None => match &self.status {
                        Status::Loading => {
                            ui.add(Label::new("Loading..."));
                        }
                        _ => {
                            ui.add(Label::new("Nothing to show"));
                            ui.add(Label::new("Please select a command"));
                        }
                    },
                };
            });
        }
    }
}
