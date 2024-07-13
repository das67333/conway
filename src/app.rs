use crate::{Config, Engine, FpsLimiter};
use eframe::egui::{
    load::SizedTexture, pos2, Button, CentralPanel, Checkbox, Color32, ColorImage, Context,
    DragValue, Frame, Image, Margin, Rect, RichText, Slider, Stroke, TextureHandle, TextureOptions,
    Ui, Vec2, Widget,
};
use std::time::Instant;

pub struct App {
    ctx: Context,
    life_size: f64,             // Side length of Conway's square field.
    simulation_steps_log2: u32, // Number of Conway's GoL updates per frame.
    zoom: f64,                  // Current zoom rate.
    life: Box<dyn Engine>,      // Conway's GoL engine.
    life_rect: Option<Rect>,    // Part of the window displaying Conway's GoL field.
    texture: TextureHandle,     // Texture handle of Conway's GoL field.
    viewport_buf: Vec<f64>,
    viewport_pos_x: f64, // Position (in the Conway's GoL field) of the left top corner of the viewport.
    viewport_pos_y: f64,
    last_update_duration: f64, // Duration of the last life update in seconds.
    is_paused: bool,           // Flag indicating whether the simulation is paused.
    generation: u64,           // Current generation number.
    do_one_step: bool,         // Do one step and pause.
    pause_after_updates: bool, // Flag indicating whether to pause after a certain number of updates.
    updates_before_pause: u64, // Number of updates left before stopping.
    fps_limiter: FpsLimiter,   // Limits the frame rate to a certain value.
    show_verbose_stats: bool,
    adaptive_field_brightness: bool,
}

#[inline(never)]
fn normalize_brightness(v: &[f64]) -> Vec<u8> {
    // TODO: improve performance
    let u = v
        .iter()
        .filter_map(|&x| if x == 0. { None } else { Some(x) })
        .collect::<Vec<_>>();
    if u.iter().all(|&x| x == u[0]) {
        let mut k = 1.;
        if !u.is_empty() {
            k = 1. / u[0];
        }
        return v.iter().map(|&x| (x / k) as u8 * u8::MAX).collect();
    }
    let m = u.iter().sum::<f64>() / u.len() as f64;
    let dev = (u.iter().map(|&x| (x - m) * (x - m)).sum::<f64>() / (u.len() - 1) as f64).sqrt();
    v.iter()
        .map(|&x| (((x - m + dev * 0.5) / dev).clamp(0., 1.) * u8::MAX as f64) as u8)
        .collect()
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        let life = crate::HashLifeEngine::from_recursive_otca_metapixel(
            Config::get_otca_depth(),
            [[0; 4], [1, 1, 1, 0], [0; 4], [0; 4]],
        );
        // let life = crate::PatternObliviousEngine::random(7, None);
        App {
            ctx: ctx.clone(),
            life_size: 2f64.powi(life.side_length_log2() as i32),
            simulation_steps_log2: 0,
            zoom: 1.,
            life: Box::new(life),
            life_rect: None,
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
            fps_limiter: FpsLimiter::new(Config::MAX_FPS),
            show_verbose_stats: false,
            adaptive_field_brightness: true,
        }
    }

    fn update_field(&mut self) {
        if self.pause_after_updates && self.updates_before_pause == 0 {
            self.is_paused = true;
            self.do_one_step = false;
        }
        if self.is_paused && !self.do_one_step {
            return;
        }

        let timer = Instant::now();
        self.life.update(self.simulation_steps_log2);
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
                ui.label(new_text(&format!(
                    "Generation: {:>10e}",
                    self.generation as f64
                )));

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
                                .clamp_range(0..=self.life.side_length_log2() - 1),
                        )
                    })
                    .inner
                });

                ui.horizontal(|ui| {
                    if ui.add(new_button("Reset all")).clicked() {
                        *self = Self::new(&self.ctx);
                    }

                    ui.label(new_text("with OTCA depth: "));
                    let mut depth = Config::get_otca_depth();
                    ui.add(DragValue::new(&mut depth).clamp_range(1..=5));
                    Config::set_otca_depth(depth);
                });

                ui.label(new_text(&format!(
                    "Last field update: {:.3} ms",
                    self.last_update_duration * 1e3
                )));

                ui.label(new_text(&format!(
                    "FPS: {:3}",
                    self.fps_limiter.fps().round() as u32
                )));
                ui.horizontal(|ui| {
                    ui.label(new_text("Max FPS: "));
                    ui.add(
                        Slider::new(self.fps_limiter.max_fps_mut(), 5.0..=480.0).logarithmic(true),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(new_text("Zoom step: "));
                    let mut step = Config::get_zoom_step();
                    ui.add(Slider::new(&mut step, 1.0..=2.0));
                    Config::set_zoom_step(step);
                });

                ui.add(Checkbox::new(
                    &mut self.adaptive_field_brightness,
                    new_text("Adaptive field brightness"),
                ));
                ui.add_space(Config::GAP_ABOVE_STATS);

                ui.add(Checkbox::new(
                    &mut self.show_verbose_stats,
                    new_text("Verbose stats (can drop FPS)"),
                ));
                ui.label(new_text(&self.life.stats(self.show_verbose_stats)));
            });
            // to adjust the bounds of the control panel
            ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
        });
    }

    fn draw_gol_field(&mut self, ui: &mut Ui, size_px: f32) {
        // RETRIEVING A PART OF THE FIELD THAT SLIGHTLY EXCEEDS VIEWPORT
        // desired size of texture in pixels
        let mut resolution = size_px as f64 * Config::SUPERSAMPLING;
        // top left viewport coordinate in cells
        let mut x = self.life_size * self.viewport_pos_x;
        let mut y = self.life_size * self.viewport_pos_y;
        // size of viewport in cells
        let mut size_c = self.life_size * self.zoom;
        self.life.fill_texture(
            &mut x,
            &mut y,
            &mut size_c,
            &mut resolution,
            &mut self.viewport_buf,
        );

        let gray = normalize_brightness(&self.viewport_buf);
        let ci = ColorImage::from_gray([resolution as usize; 2], &gray);
        // TODO: NEAREST when close, LINEAR when far away
        let texture_options = if size_c > resolution {
            TextureOptions::LINEAR
        } else {
            TextureOptions::NEAREST
        };
        self.texture.set(ci, texture_options);
        let vp_x = (self.viewport_pos_x * self.life_size - x) / size_c;
        let vp_y = (self.viewport_pos_y * self.life_size - y) / size_c;
        let vp = pos2(vp_x as f32, vp_y as f32);
        let vp_s = Vec2::splat((self.zoom * self.life_size / size_c) as f32);

        let source = SizedTexture::new(self.texture.id(), [size_px; 2]);
        let uv = Rect::from_points(&[vp, vp + vp_s]);
        let response = ui
            .vertical_centered(|ui: &mut Ui| Widget::ui(Image::new(source).uv(uv), ui))
            .response;
        self.life_rect.replace(response.rect);
    }

    fn update_viewport(&mut self, ctx: &Context, life_rect: Rect) {
        ctx.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                if life_rect.contains(pos) {
                    if input.raw_scroll_delta.y != 0. {
                        let zoom_change = Config::get_zoom_step()
                            .powf(input.raw_scroll_delta.y / Config::SCROLL_SCALE);
                        self.viewport_pos_x += self.zoom
                            * ((pos.x - life_rect.left_top().x) * (1. - zoom_change)
                                / life_rect.size().x) as f64;
                        self.viewport_pos_y += self.zoom
                            * ((pos.y - life_rect.left_top().y) * (1. - zoom_change)
                                / life_rect.size().y) as f64;
                        self.zoom *= zoom_change as f64;
                    }

                    if input.pointer.primary_down() && input.pointer.delta() != Vec2::ZERO {
                        self.viewport_pos_x -=
                            input.pointer.delta().x as f64 / life_rect.size().x as f64 * self.zoom;
                        self.viewport_pos_y -=
                            input.pointer.delta().y as f64 / life_rect.size().y as f64 * self.zoom;
                    }
                    self.viewport_pos_x = self.viewport_pos_x.min(1. - self.zoom).max(0.);
                    self.viewport_pos_y = self.viewport_pos_y.min(1. - self.zoom).max(0.);
                    self.zoom = self.zoom.min(1.);
                }
            }
            // if input.key_pressed(Key::Space) {
            //     self.state = !self.state;
            // }
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

                self.update_field();
            });

        self.fps_limiter.delay();
    }
}
