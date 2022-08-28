pub mod types;
use std::{sync::mpsc, time::Duration};

use eframe::{
    self,
    egui::{self, Button, Context, Layout, Sense, SidePanel, Ui},
    epaint::Vec2,
};
use eframe::{
    egui::{Label, TextEdit, TopBottomPanel, Visuals},
    epaint::Color32,
};
use git_function_history::{BlockType, CommitFunctions, FileType, Filter, FunctionHistory};
use types::{
    Command, CommandResult, CommitFilterType, CommitOrFileFilter, CommmitFilterValue, FileTypeS,
    FilterType, FullCommand, HistoryFilter, HistoryFilterType, Index, ListType, SearchFilter,
    Status,
};
// TODO: use a logger instead of print statements
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
    filter: SearchFilter,
    file_type: FileTypeS,
    history_filter_type: HistoryFilterType,
    commit_filter_type: CommitFilterType,
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
            file_type: FileTypeS::None,
            filter: SearchFilter::None,
            history_filter_type: HistoryFilterType::None,
            commit_filter_type: CommitFilterType::None,
        }
    }

    pub(crate) fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(
                    Layout::left_to_right(eframe::emath::Align::Center),
                    |_ui| {},
                );
                // controls
                ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                    let close_btn = ui.add(Button::new("❌"));
                    let theme_btn = ui.add(Button::new("🌙"));
                    if theme_btn.clicked() {
                        self.dark_theme = !self.dark_theme;
                    }
                    if close_btn.clicked() {
                        frame.close();
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    fn draw_commit(commit: (&CommitFunctions, &mut Index), ctx: &egui::Context, show: bool) {
        if show {
            TopBottomPanel::top("date_id").show(ctx, |ui| {
                ui.add(Label::new(format!("Commit: {}", commit.0.id)));
                ui.add(Label::new(format!("Date: {}", commit.0.date)));
            });
        }
        let mut i = 0;
        match commit.1 {
            Index(0, _) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(Label::new("no files found"));
                });
            }
            Index(len, 0) if *len == 1 => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(Label::new(commit.0.functions[0].to_string()));
                        });
                });
            }
            Index(_, 0) => {
                // split the screen in two parts, most of it is for the content, the and leave a small part for the right arrow
                println!("found at least one file index beginning");
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
                        .show(ui, |ui| {
                            ui.add(Label::new(commit.0.functions[0].to_string()))
                        });
                });
                if resp.clicked() {
                    i = 1;
                }
            }
            Index(len, d) if *d == *len - 1 => {
                println!("found at least one file index end");
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
                            ui.add(Label::new(commit.0.functions[*len - 1].to_string()));
                        });
                });
                if resp.clicked() {
                    i = *d - 1;
                } else {
                    i = *d
                }
            }
            Index(_, is) => {
                println!("found at least one file index middle");
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
                            ui.add(Label::new(commit.0.functions[*is].to_string()));
                        });
                });
                if l_resp.clicked() {
                    i = *is - 1;
                } else if r_resp.clicked() {
                    i = *is + 1;
                } else {
                    i = *is;
                }
            }
        }
        *commit.1 = Index(commit.1 .0, i);
    }

    fn draw_history(history: (&FunctionHistory, &mut Index, &mut Index), ctx: &egui::Context) {
        // split the screen top and bottom into two parts, leave small part for the left arrow commit hash and right arrow and the rest for the content
        // create a 3 line header
        TopBottomPanel::top("control history").show(ctx, |ui| {
            ui.set_height(2.0);
            ui.horizontal(|ui| {
                let mut max = ui.available_width();
                let l_resp = match history.1 {
                    Index(_, 0) => {
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
                        history.0.history[history.1 .1].id, history.0.history[history.1 .1].date
                    )),
                );

                let r_resp = match history.1 {
                    Index(len, i) if *i == *len - 1 => {
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
                            *history.1 = Index(history.1 .0, history.1 .1 + 1);
                            // reset file index
                            *history.2 = Index(history.2 .0, 0)
                        }
                    }
                    None => {}
                }
                match l_resp {
                    Some(l_resp) => {
                        if l_resp.clicked() {
                            *history.1 = Index(history.1 .0, history.1 .1 - 1);
                            // reset file index
                            *history.2 = Index(history.2 .0, 0)
                        }
                    }
                    None => {}
                }
            });
        });
        Self::draw_commit((&history.0.history[history.1 .1], history.2), ctx, false);
    }

    fn draw_commit_file_filter(&mut self, ui: &mut Ui, t: CommmitFilterValue, max: f32) {
        // Options
        // 1. function in block
        // 2. function in lines
        // 3. function in function
        egui::ComboBox::from_id_source("commit_combo_box")
            .selected_text(self.commit_filter_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.commit_filter_type,
                    CommitFilterType::FunctionInFunction(String::new()),
                    "function with parent",
                );
                ui.selectable_value(
                    &mut self.commit_filter_type,
                    CommitFilterType::FunctionInBlock(String::new()),
                    "function in block",
                );
                ui.selectable_value(
                    &mut self.commit_filter_type,
                    CommitFilterType::FunctionInLines(String::new(), String::new()),
                    "function in lines",
                );
                ui.selectable_value(&mut self.commit_filter_type, CommitFilterType::None, "none");
            });
        match &mut self.commit_filter_type {
            CommitFilterType::FunctionInBlock(block) => {
                ui.horizontal(|ui| {
                    // set the width of the input field
                    ui.set_min_width(4.0);
                    ui.set_max_width(max);
                    ui.add(TextEdit::singleline(block));
                });
            }
            CommitFilterType::FunctionInLines(line1, line2) => {
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
            CommitFilterType::FunctionInFunction(function) => {
                ui.horizontal(|ui| {
                    // set the width of the input field
                    ui.set_min_width(4.0);
                    ui.set_max_width(max);
                    ui.add(TextEdit::singleline(function));
                });
            }
            CommitFilterType::None => {
                // do nothing
            }
        }
        let resp = ui.add(Button::new("Go"));
        if resp.clicked() {
            self.status = Status::Loading;
            match &self.commit_filter_type {
                CommitFilterType::FunctionInBlock(block) => {
                    self.channels
                        .0
                        .send(FullCommand::Filter(FilterType::CommitOrFile(
                            CommitOrFileFilter::FunctionInBlock(BlockType::from_string(block)),
                            t,
                        )))
                        .unwrap();
                }
                CommitFilterType::FunctionInLines(line1, line2) => {
                    let fn_in_lines = (
                        match line1.parse::<usize>() {
                            Ok(x) => x,
                            Err(e) => {
                                self.status = Status::Error(format!("{}", e));
                                return;
                            }
                        },
                        match line2.parse::<usize>() {
                            Ok(x) => x,
                            Err(e) => {
                                self.status = Status::Error(format!("{}", e));
                                return;
                            }
                        },
                    );
                    self.channels
                        .0
                        .send(FullCommand::Filter(FilterType::CommitOrFile(
                            CommitOrFileFilter::FunctionInLines(fn_in_lines.0, fn_in_lines.1),
                            t,
                        )))
                        .unwrap();
                }
                CommitFilterType::FunctionInFunction(function) => {
                    self.channels
                        .0
                        .send(FullCommand::Filter(FilterType::CommitOrFile(
                            CommitOrFileFilter::FunctionInFunction(function.to_string()),
                            t,
                        )))
                        .unwrap();
                }
                CommitFilterType::None => {}
            }
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if self.dark_theme {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(20.);
            match &self.status {
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
                Status::Error(a) => {
                    ui.colored_label(Color32::LIGHT_RED, format!("Error: {}", a));
                }
            }
            ui.add_space(20.);
        });
        self.render_top_panel(ctx, frame);
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
                            CommandResult::History(t, _, _) => {
                                // Options 1. by date 2. by commit hash 3. in date range 4. function in block 5. function in lines 6. function in function
                                egui::ComboBox::from_id_source("history_combo_box")
                                    .selected_text(self.history_filter_type.to_string())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::Date(String::new()),
                                            "by date",
                                        );
                                        ui.selectable_value(
                                            &mut self.history_filter_type,
                                            HistoryFilterType::CommitId(String::new()),
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
                                            HistoryFilterType::None,
                                            "none",
                                        );
                                    });
                                match &mut self.history_filter_type {
                                    HistoryFilterType::Date(date) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(date));
                                        });
                                    }
                                    HistoryFilterType::CommitId(commit) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(commit));
                                        });
                                    }
                                    HistoryFilterType::DateRange(date1, date2) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(date1));
                                        });
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(date2));
                                        });
                                    }
                                    HistoryFilterType::FunctionInBlock(block) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(block));
                                        });
                                    }
                                    HistoryFilterType::FunctionInLines(line1, line2) => {
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
                                    HistoryFilterType::FunctionInFunction(function) => {
                                        ui.horizontal(|ui| {
                                            // set the width of the input field
                                            ui.set_min_width(4.0);
                                            ui.set_max_width(max);
                                            ui.add(TextEdit::singleline(function));
                                        });
                                    }
                                    HistoryFilterType::None => {
                                        // do nothing
                                    }
                                }
                                let resp = ui.add(Button::new("Go"));
                                if resp.clicked() {
                                    self.status = Status::Loading;
                                    match &self.history_filter_type {
                                        HistoryFilterType::Date(date) => {
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::Date(date.to_string()),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
                                        HistoryFilterType::CommitId(commit_id) => {
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::CommitId(commit_id.to_string()),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
                                        HistoryFilterType::DateRange(date1, date2) => {
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::DateRange(
                                                        date1.to_string(),
                                                        date2.to_string(),
                                                    ),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
                                        HistoryFilterType::FunctionInBlock(block) => {
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::FunctionInBlock(
                                                        BlockType::from_string(block),
                                                    ),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
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
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::FunctionInLines(
                                                        fn_in_lines.0,
                                                        fn_in_lines.1,
                                                    ),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
                                        HistoryFilterType::FunctionInFunction(function) => {
                                            self.channels
                                                .0
                                                .send(FullCommand::Filter(FilterType::History(
                                                    HistoryFilter::FunctionInFunction(
                                                        function.to_string(),
                                                    ),
                                                    t.clone(),
                                                )))
                                                .unwrap();
                                        }
                                        HistoryFilterType::None => {}
                                    }
                                }
                            }
                            CommandResult::Commit(t, _) => self.draw_commit_file_filter(
                                ui,
                                CommmitFilterValue::Commit(t.clone()),
                                max,
                            ),
                            CommandResult::File(t) => {
                                self.draw_commit_file_filter(
                                    ui,
                                    CommmitFilterValue::File(t.clone()),
                                    max,
                                );
                            }
                            _ => {
                                ui.add(Label::new("No filters available"));
                            }
                        }
                    }
                    Command::Search => {
                        // TODO: use the filetypes and searchfilter enums to hold the input buffers
                        ui.add(Label::new("Function Name:"));
                        ui.horizontal(|ui| {
                            // set the width of the input field
                            ui.set_min_width(4.0);
                            ui.set_max_width(max);
                            ui.add(TextEdit::singleline(&mut self.input_buffer));
                        });

                        // get file if any
                        egui::ComboBox::from_id_source("search_file_combo_box")
                            .selected_text(self.file_type.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.file_type, FileTypeS::None, "None");
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileTypeS::Relative(String::new()),
                                    "Relative",
                                );
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileTypeS::Absolute(String::new()),
                                    "Absolute",
                                );
                            });
                        match &mut self.file_type {
                            FileTypeS::None => {}
                            FileTypeS::Relative(abc) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(abc));
                                });
                            }
                            FileTypeS::Absolute(atring) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(atring));
                                });
                            }
                        }
                        // get filters if any
                        egui::ComboBox::from_id_source("search_search_filter_combo_box")
                            .selected_text(self.filter.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.filter, SearchFilter::None, "None");
                                ui.selectable_value(
                                    &mut self.filter,
                                    SearchFilter::CommitId(String::new()),
                                    "Commit Hash",
                                );
                                ui.selectable_value(
                                    &mut self.filter,
                                    SearchFilter::Date(String::new()),
                                    "Date",
                                );
                                ui.selectable_value(
                                    &mut self.filter,
                                    SearchFilter::DateRange(String::new(), String::new()),
                                    "Date Range",
                                );
                            });
                        match &mut self.filter {
                            SearchFilter::None => {}
                            SearchFilter::CommitId(abc) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(abc));
                                });
                            }
                            SearchFilter::Date(date) => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(date));
                                });
                            }
                            SearchFilter::DateRange(start, end) => {
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
                        }
                        let resp = ui.add(Button::new("Go"));
                        if resp.clicked() {
                            let file = match &mut self.file_type {
                                FileTypeS::None => FileType::None,
                                FileTypeS::Relative(s) => {
                                    let t = FileType::Relative(s.clone());
                                    s.clear();
                                    t
                                }
                                FileTypeS::Absolute(s) => {
                                    let t = FileType::Absolute(s.clone());
                                    s.clear();
                                    t
                                }
                            };
                            let filter = match &mut self.filter {
                                SearchFilter::None => Filter::None,
                                SearchFilter::CommitId(s) => Filter::CommitId(s.clone()),
                                SearchFilter::Date(date) => Filter::Date(date.clone()),
                                SearchFilter::DateRange(date, scd) => {
                                    Filter::DateRange(date.clone(), scd.clone())
                                }
                            };
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::Search(self.input_buffer.clone(), file, filter))
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
                            match self.list_type {
                                ListType::Dates => {
                                    self.status = Status::Loading;
                                    self.channels
                                        .0
                                        .send(FullCommand::List(self.list_type))
                                        .unwrap();
                                }
                                ListType::Commits => {
                                    self.status = Status::Loading;
                                    self.channels
                                        .0
                                        .send(FullCommand::List(self.list_type))
                                        .unwrap();
                                }
                            }
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
                        self.status = Status::Error(e);
                    }
                    (t, Status::Ok(msg)) => {
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
                CommandResult::History(t, c_index, f_index) => {
                    Self::draw_history((t, c_index, f_index), ctx);
                }
                CommandResult::Commit(t, index) => {
                    ui.add(Label::new(format!("Commit: {}", t.id)));
                    ui.add(Label::new(format!("Date: {}", t.date)));
                    Self::draw_commit((t, index), ctx, true)
                }
                CommandResult::File(t) => {
                    ui.add(Label::new("File"));
                    ui.add(Label::new(t.to_string()));
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
