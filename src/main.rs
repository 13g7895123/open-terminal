#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod config;
mod launcher;

fn main() -> eframe::Result {
    let cfg = config::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Open Terminal")
            .with_inner_size([680.0, 320.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Open Terminal",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new(cfg)))),
    )
}
