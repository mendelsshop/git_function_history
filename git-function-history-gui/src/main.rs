use eframe::{epaint::Vec2, run_native};
use git_function_history_gui::MyEguiApp;
use image::ImageFormat::Png;
use std::sync::mpsc;
fn main() {
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    simple_file_logger::init_logger(
        "git-function-history-gui",
        simple_file_logger::LogLevel::Info,
    )
    .expect("could not intialize logger");

    const ICON: &[u8] = include_bytes!("../resources/icon1.png");
    let icon =
        image::load_from_memory_with_format(ICON, Png).expect("could not load image for icon");
    function_history_backend_thread::command_thread(rx_t, tx_t, true);
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(800.0, 600.0)),
        transparent: true,
        icon_data: Some(eframe::IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_bytes(),
        }),
        ..Default::default()
    };
    run_native(
        "Git Function History",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, (tx_m, rx_m)))),
    );
}
