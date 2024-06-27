#![warn(clippy::all)]

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(800., 600.)),
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| Box::new(conway::App::new(&cc.egui_ctx))),
    )
    .unwrap();
}
