use eframe::egui::{TopBottomPanel, Visuals, Label};
use eframe::{
    self,
    egui::{self, Button, Context, Layout},
    epaint::Vec2,
    run_native,
};
use git_function_history::{File, CommitFunctions, FunctionHistory, FileType, Filter};
fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(800.0, 600.0));
    run_native(
        "Git Function History",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
}

#[derive(Default)]
struct MyEguiApp {
    command: Command,
    dark_theme: bool,
    input_buffer: String,
    cmd_output: CommandResult,
    status: Status,
    list_type: ListType,
}
impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            dark_theme: true,
            command: Command::Search,
            input_buffer: String::new(),
            cmd_output: CommandResult::None,
            status: Status::default(),
            list_type: ListType::default(),
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
                Status::Ok(a) => {
                    match a {
                        Some(a) => {
                            ui.add(Label::new(format!("Ok: {}", a)));
                        }
                        None => {
                            ui.add(Label::new("Ready"));
                        }
                    }
                }
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
                    ui.selectable_value(&mut self.command, Command::List, "history");
                });
                match self.command {
                    Command::Filter => {
                        match &self.cmd_output {
                            CommandResult::History(t) => {
                                // Options 
                                // 1. by date
                                // 2. by commit hash
                                // 3. in date range
                                // 4. function in block
                                // 5. function in lines
                                // 6. function in function
                            }
                            CommandResult::Commit(t) => {
                                // Options 
                                // 1. function in block
                                // 2. function in lines
                                // 3. function in function
                            }
                            CommandResult::File(t) => {
                                // Options 
                                // 1. function in block
                                // 2. function in lines
                                // 3. function in function
                            }
                            _ => {
                                ui.add(Label::new("No filters available"));
                            }
                        }
                    },
                    Command::Search => {
                        ui.add(Label::new("Function Name:"));
                        let text_input = ui.text_edit_singleline(&mut self.input_buffer);
                        if text_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            // get file if any
                            // get filters if any
                            self.status = Status::Loading;
                            match git_function_history::get_function_history(&self.input_buffer, FileType::None, Filter::None) {
                                Ok(t) => {
                                    self.cmd_output = CommandResult::History(t);
                                    self.status = Status::Ok(None);
                                }
                                Err(t) => {
                                    self.status = Status::Error(t.to_string());
                                }
                            }
                            self.input_buffer.clear();

                        }
                    },
                    Command::List => { 
                        egui::ComboBox::from_id_source("list_type")
                        .selected_text(self.list_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.list_type, ListType::Dates, "dates");
                            ui.selectable_value(&mut self.list_type, ListType::Commits, "commits");
                        });
                        let resp = ui.add(Button::new("Go"));
                        if resp.clicked() {
                            match self.list_type {
                                ListType::Dates => {
                                    self.status = Status::Loading;
                                    match git_function_history::get_git_dates(){
                                        Ok(dates) => {self.cmd_output = CommandResult::String(dates);
                                            self.status = Status::Ok(Some("Found commits dates".to_string()));    
                                        },
                                        Err(err) => {
                                            self.status=Status::Error(err.to_string());
                                            self.cmd_output = CommandResult::None;
                                        }
                                    };
                                    
                                }
                                ListType::Commits => {
                                    self.status = Status::Loading;
                                    match git_function_history::get_git_commits(){
                                        Ok(commits) => {self.cmd_output = CommandResult::String(commits);
                                            self.status = Status::Ok(Some("Found commits hashes".to_string()));    
                                        },
                                        Err(err) => {
                                            self.status=Status::Error(err.to_string());
                                            self.cmd_output = CommandResult::None;
                                        }
                                    };

                                }
                            }
                        }
                    },
                }
            });
            
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(f32::INFINITY).max_width(f32::INFINITY).auto_shrink([false, false]).show(ui, |ui| {
                match &self.cmd_output {
                    CommandResult::None => {
                        ui.add(Label::new("Nothing to show"));
                        ui.add(Label::new("Please select a command"));

                    },
                    CommandResult::String(ref s) => {
                        for line in s {
                            if !line.is_empty() {
                                ui.add(Label::new(line));
                            }
                        }
                    }
                    _ => {},
                }
            });
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Filter,
    List,
    Search,
}

impl Default for Command {
    fn default() -> Self {
        Command::Search
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Filter => write!(f, "filter"),
            Command::List => write!(f, "list"),
            Command::Search => write!(f, "search"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Dates,
    Commits,
}

impl std::fmt::Display for ListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListType::Dates => write!(f, "dates"),
            ListType::Commits => write!(f, "commits"),
        }
    }
}

impl Default for ListType {
    fn default() -> Self {
        ListType::Dates
    }
}

pub enum CommandResult {
    History(FunctionHistory),
    Commit(CommitFunctions),
    File(File),
    String(Vec<String>),
    None,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult::None
    }
}

enum Status {
    Ok(Option<String>),
    Error(String),
    Loading,
}

impl Default for Status {
    fn default() -> Self {
        Status::Ok(None)
    }
}

