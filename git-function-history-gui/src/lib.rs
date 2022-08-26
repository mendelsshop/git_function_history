pub mod types;
use std::{sync::mpsc, time::Duration};

use eframe::{
    self,
    egui::{self, Button, Context, Layout, Sense, SidePanel},
    epaint::Vec2,
};
use eframe::{
    egui::{Label, TextEdit, TopBottomPanel, Visuals},
    epaint::Color32,
};
use git_function_history::{CommitFunctions, FileType, Filter, FunctionHistory};
use types::{Command, CommandResult, FileTypeS, FilterS, FullCommand, Index, ListType, Status};
// TODO: use a logger instead of print statements
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
    filter: FilterS,
    file_type: FileTypeS,
    file_input_abs: String,
    file_input_rel: String,
    filter_input_id: String,
    filter_input_date: String,
    filter_input_date_range: (String, String),
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
            filter: FilterS::None,
            file_input_abs: String::new(),
            file_input_rel: String::new(),
            filter_input_id: String::new(),
            filter_input_date: String::new(),
            filter_input_date_range: (String::new(), String::new()),
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
                    let resp = egui::SidePanel::right("right_arrow").show(ctx, |ui| {
                         ui.set_width(0.5);
                         ui.add_sized(Vec2::new(ui.available_width(), ui.available_height()), Button::new("->"))
                    }).inner;
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
                    let resp = 
                    SidePanel::left("right_button").show(ctx, |ui| {
                        ui.set_width(1.0);
                        ui.add_sized(Vec2::new(ui.available_width(), ui.available_height()), Button::new("<-"))
                    }).inner;
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
                let l_resp = 
                SidePanel::left("left_arrow").show(ctx, |ui| {
                    ui.set_width(1.0);
                    ui.add_sized(Vec2::new(ui.available_width(), ui.available_height()), Button::new("<-"))
                }).inner;
                let r_resp = egui::SidePanel::right("right_arrows").show(ctx, |ui| {
                    ui.set_width(1.0);
                    ui.add_sized(Vec2::new(ui.available_width(), ui.available_height()), Button::new("->"))
               }).inner;
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
                    }
                    else if r_resp.clicked() {
                        i = *is + 1;
  
                    } else {
                        i = * is;
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
                            ui.add_sized(Vec2::new(2.0, 2.0), Button::new("<-").sense(
                                Sense::hover()
                            ));
                            None
                        }
                        _ => Some(
                            // add a left arrow button that is disabled
                            ui.add_sized(Vec2::new(2.0, 2.0), Button::new("<-")),
                        ),
                    };
                    max -= ui.available_width();
                    ui.add_sized(
                        Vec2::new(ui.available_width()-max, 2.0),
                        Label::new(format!(
                            "{}\n{}",
                            history.0.history[history.1.1].id,
                            history.0.history[history.1.1].date
                        )),
                    );
                    
                    let r_resp = match history.1 {
                        Index(len, i) if *i == *len - 1 => {
                            ui.add_sized(Vec2::new(2.0, 2.0), Button::new("->").sense(
                                Sense::hover()
                            ));
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
        Self::draw_commit((&history.0.history[history.1.1], history.2), ctx, false);
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
                            CommandResult::History(_t, _, _) => {
                                // Options
                                // 1. by date
                                // 2. by commit hash
                                // 3. in date range
                                // 4. function in block
                                // 5. function in lines
                                // 6. function in function
                            }
                            CommandResult::Commit(_t, _) => {
                                // Options
                                // 1. function in block
                                // 2. function in lines
                                // 3. function in function
                            }
                            CommandResult::File(_t) => {
                                // Options
                                // 1. function in block
                                // 2. function in lines
                                // 3. function in function
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

                        // get file if any
                        egui::ComboBox::from_id_source("search_file_combo_box")
                            .selected_text(self.file_type.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.file_type, FileTypeS::None, "None");
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileTypeS::Relative,
                                    "Relative",
                                );
                                ui.selectable_value(
                                    &mut self.file_type,
                                    FileTypeS::Absolute,
                                    "Absolute",
                                );
                            });
                        match self.file_type {
                            FileTypeS::None => {}
                            FileTypeS::Relative => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(&mut self.file_input_rel));
                                });
                            }
                            FileTypeS::Absolute => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(&mut self.file_input_abs));
                                });
                            }
                        }
                        // get filters if any
                        egui::ComboBox::from_id_source("search_filter_combo_box")
                            .selected_text(self.filter.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.filter, FilterS::None, "None");
                                ui.selectable_value(
                                    &mut self.filter,
                                    FilterS::CommitId,
                                    "Commit Hash",
                                );
                                ui.selectable_value(&mut self.filter, FilterS::Date, "Date");
                                ui.selectable_value(
                                    &mut self.filter,
                                    FilterS::DateRange,
                                    "Date Range",
                                );
                            });
                        match self.filter {
                            FilterS::None => {}
                            FilterS::CommitId => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(&mut self.filter_input_id));
                                });
                            }
                            FilterS::Date => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(&mut self.filter_input_date));
                                });
                            }
                            FilterS::DateRange => {
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(
                                        &mut self.filter_input_date_range.0,
                                    ));
                                });
                                ui.add(Label::new("-"));
                                ui.horizontal(|ui| {
                                    // set the width of the input field
                                    ui.set_min_width(4.0);
                                    ui.set_max_width(max);
                                    ui.add(TextEdit::singleline(
                                        &mut self.filter_input_date_range.1,
                                    ));
                                });
                            }
                        }
                        let resp = ui.add(Button::new("Go"));
                        if resp.clicked() {
                            let file = match self.file_type {
                                FileTypeS::None => FileType::None,
                                FileTypeS::Relative => {
                                    FileType::Relative(self.file_input_rel.clone())
                                }
                                FileTypeS::Absolute => {
                                    FileType::Absolute(self.file_input_abs.clone())
                                }
                            };
                            let filter = match self.filter {
                                FilterS::None => Filter::None,
                                FilterS::CommitId => Filter::CommitId(self.filter_input_id.clone()),
                                FilterS::Date => Filter::Date(self.filter_input_date.clone()),
                                FilterS::DateRange => Filter::DateRange(
                                    self.filter_input_date_range.0.clone(),
                                    self.filter_input_date_range.1.clone(),
                                ),
                            };
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::Search(self.input_buffer.clone(), file, filter))
                                .unwrap();
                            self.input_buffer.clear();
                            self.file_input_rel.clear();
                            self.file_input_abs.clear();
                            self.filter_input_id.clear();
                            self.filter_input_date.clear();
                            self.filter_input_date_range.0.clear();
                            self.filter_input_date_range.1.clear();
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
