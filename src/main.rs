#![warn(clippy::all)]

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod app;
mod engine;

use eframe::egui;
pub use engine::hashlife::ConwayFieldHash256;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(800., 600.)),
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| {
            Box::new(app::App::new_otca(
                &cc.egui_ctx,
                2,
                [[0; 4], [1, 1, 1, 0], [0; 4], [0; 4]],
            ))
            // Box::new(app::App::new_otca(&cc.egui_ctx, 2, [[1]]))
        }),
    )
    .unwrap();
}
