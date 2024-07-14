use std::sync::{Mutex, MutexGuard, OnceLock};

use eframe::egui::Color32;

pub struct Config {
    pub otca_depth: u32,
    pub max_fps: f64,
    pub zoom_step: f32,
    pub supersampling: f32,
    pub adaptive_field_brightness: bool,
    pub show_verbose_stats: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            otca_depth: 2,
            max_fps: 60.,
            zoom_step: 2.,
            supersampling: 0.7,
            adaptive_field_brightness: true,
            show_verbose_stats: false,
        }
    }
}

impl Config {
    pub const SCROLL_SCALE: f32 = -50.;

    pub const FRAME_MARGIN: f32 = 20.;
    pub const CONTROL_PANEL_WIDTH: f32 = 400.;
    pub const TEXT_SIZE: f32 = 16.;
    pub const TEXT_COLOR: Color32 = Color32::BLACK;
    pub const BUTTON_STROKE_WIDTH: f32 = 3.;
    pub const BUTTON_STROKE_COLOR: Color32 = Color32::DARK_GRAY;
    pub const BUTTON_FILL_COLOR: Color32 = Color32::LIGHT_GRAY;
    pub const FILENAME_INPUT_FIELD_SIZE: [f32; 2] = [80., 20.];

    pub const GAP_ABOVE_STATS: f32 = 50.;

    pub fn get<'a>() -> MutexGuard<'a, Config> {
        static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
        CONFIG
            .get_or_init(|| Mutex::new(Config::default()))
            .lock()
            .unwrap()
    }

    pub fn reset() {
        *Self::get() = Config::default();
    }
}
