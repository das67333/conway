use super::{App, BrightnessStrategy, Config};
use crate::Topology;
use eframe::egui::{
    load::SizedTexture, pos2, Button, Checkbox, ColorImage, DragValue, Image, Rect, RichText,
    Slider, Stroke, TextEdit, TextureFilter, TextureOptions, TextureWrapMode, Ui, Vec2,
};

impl App {
    fn new_text(text: &str) -> RichText {
        RichText::new(text)
            .color(Config::TEXT_COLOR)
            .size(Config::TEXT_SIZE)
    }

    fn new_button(text: &str) -> Button {
        Button::new(Self::new_text(text))
            .fill(Config::BUTTON_FILL_COLOR)
            .stroke(Stroke::new(
                Config::BUTTON_STROKE_WIDTH,
                Config::BUTTON_STROKE_COLOR,
            ))
    }

    fn draw_viewport_controls(&mut self, ui: &mut Ui) {
        let text = if self.is_paused { "Play" } else { "Pause" };
        if ui.add(Self::new_button(text)).clicked() {
            self.is_paused = !self.is_paused;
        }

        ui.add_enabled(self.is_paused, |ui: &mut Ui| {
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.pause_after_updates,
                    Self::new_text("Pause after "),
                );
                ui.add_enabled(self.pause_after_updates, |ui: &mut Ui| {
                    ui.add(DragValue::new(&mut self.updates_before_pause));
                    ui.label(Self::new_text(" updates"))
                });
            });

            if ui.add(Self::new_button("Next step")).clicked() {
                self.do_one_step = true;
            }

            ui.horizontal(|ui: &mut Ui| {
                ui.label(Self::new_text("Step size: 2^"));
                ui.add(
                    DragValue::new(&mut self.simulation_steps_log2)
                        .range(0..=self.life_engine.side_length_log2() - 1),
                )
            });

            ui.horizontal(|ui| {
                ui.label(Self::new_text("Topology: "));
                ui.radio_value(
                    &mut self.topology,
                    Topology::Unbounded,
                    Self::new_text("Unbounded"),
                );
                ui.radio_value(&mut self.topology, Topology::Torus, Self::new_text("Torus"))
            });

            ui.horizontal(|ui| {
                if ui.add(Self::new_button("Save to file")).clicked() {
                    self.life_engine.save_as_mc(&self.filename_save);
                }

                ui.label(Self::new_text("named: "));
                ui.add_sized(
                    Config::FILENAME_INPUT_FIELD_SIZE,
                    TextEdit::singleline(&mut self.filename_save),
                )
            })
            .inner
        });

        ui.horizontal(|ui| {
            if ui.add(Self::new_button("Reset field")).clicked() {
                self.reset_viewport();
            }

            ui.label(Self::new_text("with OTCA depth: "));
            ui.add(DragValue::new(&mut self.otca_depth).range(1..=5));
        });

        ui.label(Self::new_text(&format!(
            "Generation: {}",
            self.generation as f64
        )));

        ui.label(Self::new_text(&format!(
            "\nLast field update: {:.3} ms",
            self.last_update_duration * 1e3
        )));
    }

    fn draw_appearance_controls(&mut self, ui: &mut Ui) {
        ui.label(Self::new_text(&format!(
            "FPS: {:3}",
            self.fps_limiter.fps().round() as u32
        )));

        ui.horizontal(|ui| {
            ui.label(Self::new_text("Max FPS: "));
            ui.add(Slider::new(&mut self.max_fps, 5.0..=480.0).logarithmic(true));
        });

        ui.horizontal(|ui| {
            ui.label(Self::new_text("Zoom step: "));
            ui.add(Slider::new(&mut self.zoom_step, 1.0..=4.0));
        });

        ui.horizontal(|ui| {
            ui.label(Self::new_text("Supersampling: "));
            ui.add(Slider::new(&mut self.supersampling, 0.1..=2.0));
        });

        ui.horizontal(|ui| {
            ui.label(Self::new_text("Brightness: "));
            ui.radio_value(
                &mut self.brightness_strategy,
                BrightnessStrategy::Linear,
                Self::new_text("Linear"),
            );
            ui.radio_value(
                &mut self.brightness_strategy,
                BrightnessStrategy::Golly,
                Self::new_text("Golly"),
            );
            ui.radio_value(
                &mut self.brightness_strategy,
                BrightnessStrategy::Custom,
                Self::new_text("Custom"),
            );
        });

        if ui.add(Self::new_button("Reset config")).clicked() {
            self.reset_appearance();
        }
    }

    fn draw_stats(&mut self, ui: &mut Ui) {
        ui.add(Checkbox::new(
            &mut self.show_verbose_stats,
            Self::new_text("Verbose stats (can drop FPS)"),
        ));

        ui.label(Self::new_text(
            &self.life_engine.stats(self.show_verbose_stats),
        ));
    }

    fn draw_controls(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let aw = ui.available_width();

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        self.draw_viewport_controls(ui);
                    });

                    // to adjust the bounds
                    ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
                });
            });

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        self.draw_appearance_controls(ui);

                        ui.add_space(Config::GAP_ABOVE_STATS);

                        self.draw_stats(ui);
                    });

                    // to adjust the bounds
                    ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
                });
            });
        });
    }

    fn draw_gol_field(&mut self, ui: &mut Ui, size_px: f32) {
        // Retrieving a part of the field that slightly exceeds viewport.
        // desired size of texture in pixels
        let mut resolution = size_px as f64 * self.supersampling;
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
        let response = ui.add(image);
        self.life_rect.replace(response.rect);
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        let area = ui.available_size();

        let size_px = area
            .y
            .min(area.x - Config::CONTROL_PANEL_WIDTH - Config::FRAME_MARGIN);
        ui.horizontal(|ui| {
            self.draw_controls(ui);

            ui.add_space(ui.available_width() - size_px);

            ui.vertical_centered(|ui| {
                self.draw_gol_field(ui, size_px);
            });
        });
    }
}
