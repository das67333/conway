use eframe::egui::Color32;

pub struct Config;

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

    pub const WIDGET_GAP: f32 = 20.;

    pub const OTCA_DEPTH: u32 = 2;
    pub const MAX_FPS: f64 = 60.;
    pub const ZOOM_STEP: f32 = 2.;
    pub const SUPERSAMPLING: f64 = 0.7;
    pub const TOP_PATTERN: [[u8; 4]; 4] = [[0, 1, 0, 0], [0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0]];
}
