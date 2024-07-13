use std::sync::{Mutex, OnceLock};

use eframe::egui::Color32;

pub struct Config {
    otca_depth: u32,
    zoom_step: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            otca_depth: 2,
            zoom_step: 1.1,
        }
    }
}

impl Config {
    pub const MAX_FPS: f64 = 60.;

    pub const SCROLL_SCALE: f32 = -50.;
    pub const SUPERSAMPLING: f64 = 0.7;

    pub const FRAME_MARGIN: f32 = 20.;
    pub const CONTROL_PANEL_WIDTH: f32 = 400.;
    pub const TEXT_SIZE: f32 = 16.;
    pub const TEXT_COLOR: Color32 = Color32::BLACK;
    pub const BUTTON_STROKE_WIDTH: f32 = 3.;
    pub const BUTTON_STROKE_COLOR: Color32 = Color32::DARK_GRAY;
    pub const BUTTON_FILL_COLOR: Color32 = Color32::LIGHT_GRAY;

    pub const GAP_ABOVE_STATS: f32 = 50.;

    fn get() -> &'static Mutex<Config> {
        static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
        CONFIG.get_or_init(|| Mutex::new(Config::default()))
    }

    pub fn get_otca_depth() -> u32 {
        Self::get().lock().unwrap().otca_depth
    }

    pub fn set_otca_depth(depth: u32) {
        Self::get().lock().unwrap().otca_depth = depth;
    }

    pub fn get_zoom_step() -> f32 {
        Self::get().lock().unwrap().zoom_step
    }

    pub fn set_zoom_step(step: f32) {
        Self::get().lock().unwrap().zoom_step = step;
    }
}
