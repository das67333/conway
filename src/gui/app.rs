use super::{BrightnessStrategy, Config, FpsLimiter};
use crate::{Engine, HashLifeEngine, Topology};
use eframe::egui::{
    CentralPanel, Color32, ColorImage, Context, Frame, Key, Margin, Rect, TextureHandle,
    TextureOptions,
};
use std::time::Instant;

pub struct App {
    pub(super) life_engine: Box<dyn Engine>, // Conway's GoL engine.
    pub(super) is_paused: bool,              // Flag indicating whether the simulation is paused.
    pub(super) pause_after_updates: bool, // Flag indicating whether to pause after a certain number of updates.
    pub(super) updates_before_pause: u64, // Number of updates left before stopping.
    pub(super) do_one_step: bool,         // Do one step and pause.
    pub(super) simulation_steps_log2: u32, // Number of Conway's GoL updates per frame.
    pub(super) topology: Topology,        // Topology of the field.
    pub(super) filename_save: String,     // The name of the file to save the field to.
    pub(super) generation: u64,           // Current generation number.
    pub(super) last_update_duration: f64, // Duration of the last life update in seconds.
    pub(super) viewport_size: f64,        // Size of the viewport in cells.
    pub(super) viewport_pos_x: f64, // Position (in the Conway's GoL field) of the left top corner of the viewport.
    pub(super) viewport_pos_y: f64,
    pub(super) viewport_buf: Vec<f64>,
    pub(super) texture: TextureHandle, // Texture handle of Conway's GoL field.
    pub(super) life_rect: Option<Rect>, // Part of the window displaying Conway's GoL field.
    pub(super) fps_limiter: FpsLimiter, // Limits the frame rate to a certain value.
    pub(super) brightness_strategy: BrightnessStrategy, // Strategy for normalizing brightness.

    pub(super) otca_depth: u32,
    pub(super) max_fps: f64,
    pub(super) zoom_step: f32,
    pub(super) supersampling: f64,
    pub(super) show_verbose_stats: bool,
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        let top_pattern = Config::TOP_PATTERN.iter().map(|row| row.to_vec()).collect();
        let life = HashLifeEngine::from_recursive_otca_metapixel(Config::OTCA_DEPTH, top_pattern);
        // let life = crate::PatternObliviousEngine::random(7, None);
        Self {
            viewport_size: 2f64.powi(life.side_length_log2() as i32),
            life_engine: Box::new(life),
            is_paused: true,
            pause_after_updates: false,
            updates_before_pause: 0,
            do_one_step: false,
            simulation_steps_log2: 0,
            topology: Topology::Unbounded,
            filename_save: "conway.mc".to_string(),
            generation: 0,
            last_update_duration: 0.,
            viewport_pos_x: 0.,
            viewport_pos_y: 0.,
            viewport_buf: vec![],
            texture: ctx.load_texture(
                "Conway's GoL field",
                ColorImage::default(),
                TextureOptions::default(),
            ),
            life_rect: None,
            fps_limiter: FpsLimiter::default(),
            brightness_strategy: BrightnessStrategy::Linear,
            otca_depth: Config::OTCA_DEPTH,
            max_fps: Config::MAX_FPS,
            zoom_step: Config::ZOOM_STEP,
            supersampling: Config::SUPERSAMPLING,
            show_verbose_stats: false,
        }
    }

    pub fn reset_viewport(&mut self) {
        self.life_engine = Box::new(HashLifeEngine::from_recursive_otca_metapixel(
            self.otca_depth,
            Config::TOP_PATTERN.iter().map(|row| row.to_vec()).collect(),
        ));

        self.is_paused = true;
        self.pause_after_updates = false;
        self.updates_before_pause = 0;
        self.do_one_step = false;
        self.simulation_steps_log2 = 0;
        self.topology = Topology::Unbounded;
        self.filename_save = "conway.mc".to_string();
        self.generation = 0;
        self.last_update_duration = 0.;
        self.viewport_size = 2f64.powi(self.life_engine.side_length_log2() as i32);
        self.viewport_pos_x = 0.;
        self.viewport_pos_y = 0.;
    }

    pub fn reset_appearance(&mut self) {
        self.brightness_strategy = BrightnessStrategy::Linear;
        self.max_fps = Config::MAX_FPS;
        self.zoom_step = Config::ZOOM_STEP;
        self.supersampling = Config::SUPERSAMPLING;
        self.show_verbose_stats = false;
    }

    fn update_engine(&mut self) {
        if self.pause_after_updates && self.updates_before_pause == 0 {
            self.is_paused = true;
            self.do_one_step = false;
        }
        if self.is_paused && !self.do_one_step {
            return;
        }

        let timer = Instant::now();
        {
            let [dx, dy] = self
                .life_engine
                .update(self.simulation_steps_log2, self.topology);

            self.viewport_pos_x += dx as f64;
            self.viewport_pos_y += dy as f64;
        }
        // updating frame counter
        self.last_update_duration = timer.elapsed().as_secs_f64();

        self.generation += 1 << self.simulation_steps_log2;
        if self.pause_after_updates {
            self.updates_before_pause -= 1;
        }
        self.do_one_step = false;
    }

    fn update_viewport(&mut self, ctx: &Context, life_rect: Rect) {
        ctx.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                if life_rect.contains(pos) {
                    if input.pointer.primary_down() {
                        let p = input.pointer.delta() / life_rect.size();
                        self.viewport_pos_x -= self.viewport_size * p.x as f64;
                        self.viewport_pos_y -= self.viewport_size * p.y as f64;
                    }

                    if input.raw_scroll_delta.y != 0. {
                        let zoom_change = self
                            .zoom_step
                            .powf(input.raw_scroll_delta.y / Config::SCROLL_SCALE);
                        let p =
                            (pos - life_rect.left_top()) * (1. - zoom_change) / life_rect.size();
                        self.viewport_pos_x += self.viewport_size * p.x as f64;
                        self.viewport_pos_y += self.viewport_size * p.y as f64;
                        self.viewport_size *= zoom_change as f64;
                    }

                    if !matches!(self.topology, Topology::Unbounded) {
                        let life_size = 2f64.powi(self.life_engine.side_length_log2() as i32);
                        self.viewport_size = self.viewport_size.min(life_size);
                        let lim = life_size - self.viewport_size;
                        self.viewport_pos_x = self.viewport_pos_x.min(lim).max(0.);
                        self.viewport_pos_y = self.viewport_pos_y.min(lim).max(0.);
                    }
                }
            }
            if input.key_pressed(Key::Space) {
                self.do_one_step = true;
            }
            if input.key_pressed(Key::E) && !input.modifiers.ctrl {
                self.is_paused = !self.is_paused;
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // full-window panel
        CentralPanel::default()
            .frame(
                Frame::default()
                    .inner_margin(Margin::same(Config::FRAME_MARGIN))
                    .fill(Color32::LIGHT_GRAY),
            )
            .show(ctx, |ui| {
                // TODO: power-efficient mode?
                ctx.request_repaint();

                // updating and drawing the field
                if let Some(life_rect) = self.life_rect {
                    self.update_viewport(ctx, life_rect);
                }

                self.draw(ui);

                self.update_engine();
            });

        self.fps_limiter.sleep(self.max_fps);
    }
}
