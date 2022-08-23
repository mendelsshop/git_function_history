pub mod types;
use std::{sync::mpsc, time::Duration};

use eframe::egui::{Label, TopBottomPanel, Visuals};
use eframe::{
    self,
    egui::{self, Button, Context, Layout},
};
use git_function_history::{FileType, Filter};
use types::{Command, CommandResult, FullCommand, ListType, Status};

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
}

impl MyEguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        channels: (
            mpsc::Sender<FullCommand>,
            mpsc::Receiver<(CommandResult, Status)>,
        ),
    ) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        // channels.0.send(FullCommand::List(ListType::Commits)).unwrap();
        Self {
            dark_theme: true,
            command: Command::Search,
            input_buffer: String::new(),
            cmd_output: CommandResult::None,
            status: Status::default(),
            list_type: ListType::default(),
            channels,
        }
    }

    pub(crate) fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
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
                    ui.add(Label::new("Loading..."));
                }
                Status::Ok(a) => match a {
                    Some(a) => {
                        ui.add(Label::new(format!("Ok: {}", a)));
                    }
                    None => {
                        ui.add(Label::new("Ready"));
                    }
                },
                Status::Error(a) => {
                    ui.add(Label::new(a));
                }
            }
            ui.add_space(20.);
        });
        self.render_top_panel(ctx, frame);
        egui::TopBottomPanel::bottom("commnad_builder").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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
                            CommandResult::History(_t) => {
                                // Options
                                // 1. by date
                                // 2. by commit hash
                                // 3. in date range
                                // 4. function in block
                                // 5. function in lines
                                // 6. function in function
                            }
                            CommandResult::Commit(_t) => {
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
                        ui.text_edit_singleline(&mut self.input_buffer);
                        let resp = ui.add(Button::new("Go"));
                        if resp.clicked() {
                            // get file if any
                            // get filters if any
                            self.status = Status::Loading;
                            self.channels
                                .0
                                .send(FullCommand::Search(
                                    self.input_buffer.clone(),
                                    FileType::None,
                                    Filter::None,
                                ))
                                .unwrap();
                            self.input_buffer.clear();
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
            egui::ScrollArea::vertical()
                .max_height(f32::INFINITY)
                .max_width(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.channels.1.recv_timeout(Duration::from_millis(100)) {
                        Ok(timeout) => match timeout {
                            (_, Status::Error(e)) => {
                                self.status = Status::Error(e);
                            }
                            (t, Status::Ok(msg)) => {
                                println!("received");
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
                    match &self.cmd_output {
                        CommandResult::History(t) => {
                            // TODO: keep track of commit and file index
                            // TODO: add buttons to switch between files and commits
                            ui.add(Label::new(format!("Function: {}", t.name)));
                            if !t.history.is_empty() {
                                if !t.history[0].functions.is_empty() {
                                    ui.add(Label::new(format!(
                                        "Date: {}\nCommit Hash: {}",
                                        t.history[0].date, t.history[0].id,
                                    )));
                                    if !t.history[0].functions[0].functions.is_empty() {
                                        ui.add(Label::new(format!(
                                            "{}",
                                            t.history[0].functions[0]
                                        )));
                                    } else {
                                        ui.add(Label::new("No history Found"));
                                    }
                                } else {
                                    ui.add(Label::new("No history Found"));
                                }
                            } else {
                                ui.add(Label::new("No history Found"));
                            }
                        }
                        CommandResult::Commit(_t) => {}
                        CommandResult::File(_t) => {
                            ui.add(Label::new("File:"));
                        }
                        CommandResult::String(t) => {
                            for line in t {
                                if !line.is_empty() {
                                    ui.add(Label::new(line));
                                }
                            }
                        }
                        CommandResult::None => {
                            ui.add(Label::new("Nothing to show"));
                            ui.add(Label::new("Please select a command"));
                        }
                    }
                });
        });
    }
}
