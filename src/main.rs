mod app;
mod protocol;
mod visualizer;
mod worker;

use app::KlsApp;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Kelly KLS Companion"),
        ..Default::default()
    };

    eframe::run_native(
        "Kelly KLS Companion",
        options,
        Box::new(|cc| Ok(Box::new(KlsApp::new(cc)))),
    )
}
