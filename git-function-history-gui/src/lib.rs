use std::{sync::mpsc, time::Duration};

use eframe::{
    self,
    egui::{self, Button, Layout, Sense, SidePanel},
    epaint::Vec2,
};
use eframe::{
    egui::{Label, TextEdit, TopBottomPanel, Visuals},
    epaint::Color32,
};
use function_history_backend_thread::types::{
    Command, CommandResult, FilterType, FullCommand, HistoryFilterType, ListType, Status,
};
use git_function_history::{
    types::Directions, BlockType, CommitFunctions, FileType, Filter, FunctionHistory,
};

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
    file_type: FileType,
    history_filter_type: HistoryFilterType,
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
            file_type: FileType::None,
            filter: Filter::None,
            history_filter_type: HistoryFilterType::None,
        }
    }

    fn draw_commit(commit: &mut CommitFunctions, ctx: &egui::Context, show: bool) {
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
        match commit.get_move_direction() {
            Directions::None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(Label::new(commit.get_file().to_string()));
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
                        .show(ui, |ui| ui.add(Label::new(commit.get_file().to_string())));
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
                            ui.add(Label::new(commit.get_file().to_string()));
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
                            ui.add(Label::new(commit.get_file().to_string()));
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
                        history.get_metadata()["commit hash"],
                        history.get_metadata()["date"]
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

                match r_resp {
                    Some(r_resp) => {
                        if r_resp.clicked() {
                            history.move_forward();
                        }
                    }
                    None => {}
                }
                match l_resp {
                    Some(l_resp) => {
                        if l_resp.clicked() {
                            history.move_back();
                        }
                    }
                    None => {}
                }
            });
        });
        Self::draw_commit(history.get_mut_commit(), ctx, false);
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if self.dark_theme {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(20.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(
                    Layout::left_to_right(eframe::emath::Align::Center),
                    |ui| match &self.status {
                        Status::Loading => {
                            ui.colored_label(Color32::BLUE, "Loading...");
                        }
                        Status::Ok(a) => match a {
                            Some(a) => {
                                ui.colored_label(Color32::LIGHT_GREEN, format!("Ok: {}", a));
                            }
                            None => {
                                ui.colored_label(Color32::GREEN, "Ready");
                            }
                        },
                        Status::Warning(a) => {
                            ui.colored_label(Color32::LIGHT_RED, format!("Warn: {}", a));
                        }
                        Status::Error(a) => {
                            ui.colored_label(Color32::LIGHT_RED, format!("Error: {}", a));
                        }
                    },
                );
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
                let max = ui.available_width() / 6.0;
                egui::ComboBox::from_id_source("command_combo_box")
                    .selected_text(self.command.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.command, Command::Filter, "filter");
                        ui.selectable_value(&mut self.command, Command::Search, "search");
                        ui.selectable_value(&mut self.command, Command::List, "list");
                    });
                match self.command {
                    Command::Filter => {
                        match &self.cmd_output {
                            CommandResult::History(_) => {
                                // Options 1. by date 2. by commit hash 3. in date range 4. function in block 5. function in lines 6. function in function
                                let text = match &self.history_filter_type {
                                    HistoryFilterType::None => "filter type".to_string(),
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
                                            HistoryFilterType::FunctionInBlock(String::new()),
                                            "function in block",
                                        );
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::FunctionInLines(
                                                String::new(),
                                                String::new(),
                                            ),
                                            "function in lines",
                                        );
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::FunctionInFunction(String::new()),
                                            "function in function",
                                        );
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::FileAbsolute(String::new()),
                                            "file absolute",
                                        );
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::FileRelative(String::new()),
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
                                    });
                                match &mut self.history_filter_type {
                                    HistoryFilterType::DateRange(line1, line2)
                                    | HistoryFilterType::FunctionInLines(line1, line2) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(line1));
                                        });
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(line2));
                                        });
                                    }
                                    HistoryFilterType::Date(dir)
                                    | HistoryFilterType::CommitHash(dir)
                                    | HistoryFilterType::FunctionInBlock(dir)
                                    | HistoryFilterType::FunctionInFunction(dir)
                                    | HistoryFilterType::FileAbsolute(dir)
                                    | HistoryFilterType::FileRelative(dir)
                                    | HistoryFilterType::Directory(dir) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(dir));
                                        });
                                    }
                                    HistoryFilterType::None => {
                                        // do nothing
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
                                            Some(Filter::CommitHash(commit_hash.to_string()))
                                        }
                                        HistoryFilterType::DateRange(date1, date2) => Some(
                                            Filter::DateRange(date1.to_string(), date2.to_string()),
                                        ),
                                        HistoryFilterType::FunctionInBlock(block) => Some(
                                            Filter::FunctionInBlock(BlockType::from_string(block)),
                                        ),
                                        HistoryFilterType::FunctionInLines(line1, line2) => {
                                            let fn_in_lines = (
                                                match line1.parse::<usize>() {
                                                    Ok(x) => x,
                                                    Err(e) => {
                                                        self.status =
                                                            Status::Error(format!("{}", e));
                                                        return;
                                                    }
                                                },
                                                match line2.parse::<usize>() {
                                                    Ok(x) => x,
                                                    Err(e) => {
                                                        self.status =
                                                            Status::Error(format!("{}", e));
                                                        return;
                                                    }
                                                },
                                            );
                                            Some(Filter::FunctionInLines(
                                                fn_in_lines.0,
                                                fn_in_lines.1,
                                            ))
                                        }
                                        HistoryFilterType::FunctionInFunction(function) => {
                                            Some(Filter::FunctionWithParent(function.to_string()))
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
                                    };
                                    if let Some(filter) = filter {
                                        self.channels
                                            .0
                                            .send(FullCommand::Filter(FilterType {
                                                thing: self.cmd_output.clone(),
                                                filter,
                                            }))
                                            .unwrap();
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
                        ui.horizontal(|ui| {
                            // set the width of the input field
                            ui.set_min_width(4.0);
                            ui.set_max_width(max);
                            ui.add(TextEdit::singleline(&mut self.input_buffer));
                        });

                        let text = match &self.file_type {
                            FileType::Directory(_) => "directory",
                            FileType::Absolute(_) => "absolute",
                            FileType::Relative(_) => "relative",
                            _ => "file type",
                        };
                        egui::ComboBox::from_id_source("search_file_combo_box")
                            .selected_text(text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.file_type, FileType::None, "None");
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileType::Relative(String::new()),
                                    "Relative",
                                );
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileType::Absolute(String::new()),
                                    "Absolute",
                                );
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileType::Directory(String::new()),
                                    "Directory",
                                );
                            });
                        match &mut self.file_type {
                            FileType::None => {}
                            FileType::Relative(dir)
                            | FileType::Absolute(dir)
                            | FileType::Directory(dir) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(dir));
                                });
                            }
                        }
                        // get filters if any
                        let text = match &self.filter {
                            Filter::CommitHash(_) => "commit hash".to_string(),
                            Filter::DateRange(..) => "date range".to_string(),
                            Filter::Date(_) => "date".to_string(),
                            _ => "filter type".to_string(),
                        };
                        egui::ComboBox::from_id_source("search_search_filter_combo_box")
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
                        match &mut self.filter {
                            Filter::None => {}
                            Filter::CommitHash(thing) | Filter::Date(thing) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(thing));
                                });
                            }
                            Filter::DateRange(start, end) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(start));
                                });
                                ui.add(Label::new("-"));
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(end));
                                });
                            }
                            _ => {}
                        }
                        let resp = ui.add(Button::new("Go"));
                        if resp.clicked() {
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::Search(
                                    self.input_buffer.clone(),
                                    self.file_type.clone(),
                                    self.filter.clone(),
                                ))
                                .unwrap();
                        }
                    }
                    Command::List => {
                        egui::ComboBox::from_id_source("list_type")
                            .selected_text(self.list_type.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.list_type, ListType::Dates, "dates");
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
                                .unwrap();
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // check if the channel has a message and if so set it to self.command
            match self.channels.1.recv_timeout(Duration::from_millis(100)) {
                Ok(timeout) => match timeout {
                    (_, Status::Error(e)) => {
                        log::info!("got error last command failed with {}", e);
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
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for line in t {
                                if !line.is_empty() {
                                    ui.add(Label::new(line.to_string()));
                                }
                            }
                        });
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
