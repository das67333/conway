#![warn(clippy::all)]

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1280., 720.))
            .with_min_inner_size(egui::vec2(640.0, 360.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| Box::new(conway::App::new(&cc.egui_ctx))),
    )
    .unwrap();
}
