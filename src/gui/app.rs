use super::{brightness::BrightnessStrategy, config::Config, fps_limit::FpsLimiter};
use crate::{Engine, Topology};
use eframe::egui::{
    load::SizedTexture, pos2, Button, CentralPanel, Checkbox, Color32, ColorImage, Context,
    DragValue, Frame, Image, Key, Margin, Rect, RichText, Slider, Stroke, TextEdit, TextureFilter,
    TextureHandle, TextureOptions, TextureWrapMode, Ui, Vec2,
};
use std::time::Instant;

pub struct App {
    ////////////////////////////////////////////////////////////////
    generation: u64,              // Current generation number.
    life_engine: Box<dyn Engine>, // Conway's GoL engine.
    is_paused: bool,              // Flag indicating whether the simulation is paused.
    pause_after_updates: bool, // Flag indicating whether to pause after a certain number of updates.
    updates_before_pause: u64, // Number of updates left before stopping.
    do_one_step: bool,         // Do one step and pause.
    simulation_steps_log2: u32, // Number of Conway's GoL updates per frame.
    topology: Topology,        // Topology of the field.
    filename_save: String,     // The name of the file to save the field to.
    last_update_duration: f64, // Duration of the last life update in seconds.
    ////////////////////////////////////////////////////////////////
    viewport_size: f64,      // Size of the viewport in cells.
    life_rect: Option<Rect>, // Part of the window displaying Conway's GoL field.
    ctx: Context,
    texture: TextureHandle, // Texture handle of Conway's GoL field.
    viewport_buf: Vec<f64>,
    viewport_pos_x: f64, // Position (in the Conway's GoL field) of the left top corner of the viewport.
    viewport_pos_y: f64,
    fps_limiter: FpsLimiter, // Limits the frame rate to a certain value.
    brightness_strategy: BrightnessStrategy, // Strategy for normalizing brightness.
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        // be careful with deadlocks
        let depth = Config::get().otca_depth;
        let top_pattern = Config::get().top_pattern.clone();
        let life = crate::HashLifeEngine::from_recursive_otca_metapixel(depth, top_pattern);
        // let life = crate::PatternObliviousEngine::random(7, None);
        App {
            simulation_steps_log2: 0,
            viewport_size: 2f64.powi(life.side_length_log2() as i32),
            life_engine: Box::new(life),
            life_rect: None,
            ctx: ctx.clone(),
            texture: ctx.load_texture(
                "Conway's GoL field",
                ColorImage::default(),
                TextureOptions::default(),
            ),
            viewport_buf: vec![],
            viewport_pos_x: 0.,
            viewport_pos_y: 0.,
            last_update_duration: 0.,
            is_paused: true,
            generation: 0,
            do_one_step: false,
            pause_after_updates: false,
            updates_before_pause: 0,
            fps_limiter: FpsLimiter::default(),
            filename_save: "conway.mc".to_string(),
            topology: Topology::Unbounded,
            brightness_strategy: BrightnessStrategy::Linear,
        }
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

    fn draw_control_panel(&mut self, ui: &mut Ui) {
        let new_text = |text: &str| {
            RichText::new(text)
                .color(Config::TEXT_COLOR)
                .size(Config::TEXT_SIZE)
        };

        let new_button = |text: &str| {
            Button::new(new_text(text))
                .fill(Config::BUTTON_FILL_COLOR)
                .stroke(Stroke::new(
                    Config::BUTTON_STROKE_WIDTH,
                    Config::BUTTON_STROKE_COLOR,
                ))
        };

        let aw = ui.available_width();
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(new_text(&format!("Generation: {}", self.generation as f64)));

                let text = if self.is_paused { "Play" } else { "Pause" };
                if ui.add(new_button(text)).clicked() {
                    self.is_paused = !self.is_paused;
                }

                ui.add_enabled(self.is_paused, |ui: &mut Ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.pause_after_updates, new_text("Pause after "));
                        ui.add_enabled(self.pause_after_updates, |ui: &mut Ui| {
                            ui.add(DragValue::new(&mut self.updates_before_pause));
                            ui.label(new_text(" updates"))
                        });
                    });

                    if ui.add(new_button("Next step")).clicked() {
                        self.do_one_step = true;
                    }
                    ui.horizontal(|ui: &mut Ui| {
                        ui.label(new_text("Step size: 2^"));
                        ui.add(
                            DragValue::new(&mut self.simulation_steps_log2)
                                .range(0..=self.life_engine.side_length_log2() - 1),
                        )
                    });

                    ui.horizontal(|ui| {
                        ui.label(new_text("Topology: "));
                        ui.radio_value(
                            &mut self.topology,
                            Topology::Unbounded,
                            new_text("Unbounded"),
                        );
                        ui.radio_value(&mut self.topology, Topology::Torus, new_text("Torus"))
                    });

                    ui.horizontal(|ui| {
                        if ui.add(new_button("Save to file")).clicked() {
                            self.life_engine.save_as_mc(&self.filename_save);
                        }

                        ui.label(new_text("named: "));
                        ui.add_sized(
                            Config::FILENAME_INPUT_FIELD_SIZE,
                            TextEdit::singleline(&mut self.filename_save),
                        )
                    })
                    .inner
                });

                ui.horizontal(|ui| {
                    if ui.add(new_button("Reset field")).clicked() {
                        *self = Self::new(&self.ctx);
                    }

                    ui.label(new_text("with OTCA depth: "));
                    ui.add(DragValue::new(&mut Config::get().otca_depth).range(1..=5));
                });

                ui.label(new_text(&format!(
                    "\nLast field update: {:.3} ms",
                    self.last_update_duration * 1e3
                )));

                ui.label(new_text(&format!(
                    "FPS: {:3}",
                    self.fps_limiter.fps().round() as u32
                )));
                ui.horizontal(|ui| {
                    ui.label(new_text("Max FPS: "));
                    ui.add(Slider::new(&mut Config::get().max_fps, 5.0..=480.0).logarithmic(true));
                });
                ui.horizontal(|ui| {
                    ui.label(new_text("Zoom step: "));
                    ui.add(Slider::new(&mut Config::get().zoom_step, 1.0..=4.0));
                });
                ui.horizontal(|ui| {
                    ui.label(new_text("Supersampling: "));
                    ui.add(Slider::new(&mut Config::get().supersampling, 0.1..=2.0));
                });

                ui.horizontal(|ui| {
                    ui.label(new_text("Brightness: "));
                    ui.radio_value(
                        &mut self.brightness_strategy,
                        BrightnessStrategy::Linear,
                        new_text("Linear"),
                    );
                    ui.radio_value(
                        &mut self.brightness_strategy,
                        BrightnessStrategy::Golly,
                        new_text("Golly"),
                    );
                    ui.radio_value(
                        &mut self.brightness_strategy,
                        BrightnessStrategy::Custom,
                        new_text("Custom"),
                    );
                });

                if ui.add(new_button("Reset config")).clicked() {
                    Config::reset();
                }

                ui.add_space(Config::GAP_ABOVE_STATS);

                ui.add(Checkbox::new(
                    &mut Config::get().show_verbose_stats,
                    new_text("Verbose stats (can drop FPS)"),
                ));
                ui.label(new_text(
                    &self.life_engine.stats(Config::get().show_verbose_stats),
                ));
            });
            // to adjust the bounds of the control panel
            ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
        });
    }

    fn draw_gol_field(&mut self, ui: &mut Ui, size_px: f32) {
        // Retrieving a part of the field that slightly exceeds viewport.
        // desired size of texture in pixels
        let mut resolution = (size_px * Config::get().supersampling) as f64;
        // top left viewport coordinate in cells
        let (mut x, mut y) = (self.viewport_pos_x, self.viewport_pos_y);
        // size of the subregion of the field that will be retrieved;
        // is going to be increased from `viewport_size`
        let mut size = self.viewport_size;
        // `step_size` is the number of cells per pixel side
        self.life_engine.fill_texture(
            &mut x,
            &mut y,
            &mut size,
            &mut resolution,
            &mut self.viewport_buf,
        );

        let gray = self
            .brightness_strategy
            .transform(resolution as usize, &self.viewport_buf);

        let ci = ColorImage::from_gray([resolution as usize; 2], &gray);
        let texture_options = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Linear,
            wrap_mode: TextureWrapMode::ClampToEdge,
        };
        self.texture.set(ci, texture_options);
        let vp_x = (self.viewport_pos_x - x) / size;
        let vp_y = (self.viewport_pos_y - y) / size;
        let vp = pos2(vp_x as f32, vp_y as f32);
        let vp_s = Vec2::splat((self.viewport_size / size) as f32);

        let source = SizedTexture::new(self.texture.id(), [size_px; 2]);
        let uv = Rect::from_points(&[vp, vp + vp_s]);
        let image = Image::from_texture(source).uv(uv);
        let response = ui.vertical_centered(|ui| ui.add(image)).response;
        self.life_rect.replace(response.rect);
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
                        let zoom_change = Config::get()
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

                let area = ui.available_size();
                let size_px = area
                    .y
                    .min(area.x - Config::CONTROL_PANEL_WIDTH - Config::FRAME_MARGIN);
                ui.horizontal(|ui| {
                    self.draw_control_panel(ui);
                    ui.add_space(ui.available_width() - size_px);
                    self.draw_gol_field(ui, size_px);
                });

                self.update_engine();
            });

        self.fps_limiter.sleep();
    }
}
