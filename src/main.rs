mod app;
mod brightness;
mod config;
mod draw;
mod field_source;
mod fps_limit;

pub use app::App;
use brightness::BrightnessStrategy;
pub use config::Config;
use field_source::FieldSource;
use fps_limit::FpsLimiter;

fn main() {
    use eframe::egui::{vec2, ViewportBuilder};

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(vec2(1280., 800.))
            .with_min_inner_size(vec2(640.0, 360.0)),
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(&cc.egui_ctx)))),
    )
    .unwrap();
}
