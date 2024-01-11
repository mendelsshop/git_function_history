use eframe::{
    egui::{IconData, ViewportBuilder},
    epaint::vec2,
    run_native,
};
use git_function_history_gui::MyEguiApp;
use image::ImageFormat::Png;
use std::sync::{mpsc, Arc};
fn main() -> eframe::Result<()> {
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    simple_file_logger::init_logger!("git-function-history-gui")
        .expect("could not intialize logger");
    const ICON: &[u8] = include_bytes!("../resources/icon1.png");
    let icon =
        image::load_from_memory_with_format(ICON, Png).expect("could not load image for icon");
    function_history_backend_thread::command_thread(rx_t, tx_t, true);
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(Arc::new(IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_bytes(),
            }))
            .with_transparent(true)
            .with_inner_size(vec2(800.0, 600.0)),
        ..Default::default()
    };
    run_native(
        "Git Function History",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, (tx_m, rx_m)))),
    )
}
