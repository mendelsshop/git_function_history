use eframe::{epaint::Vec2, run_native};
use git_function_history_gui::MyEguiApp;
use std::sync::mpsc;
fn main() {
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    function_history_backend_thread::command_thread(rx_t, tx_t, true);
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(800.0, 600.0)),
        transparent: true,
        ..Default::default()
    };
    run_native(
        "Git Function History",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, (tx_m, rx_m)))),
    );
}
