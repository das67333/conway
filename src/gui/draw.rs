use super::{field_source::FieldSource, App, BrightnessStrategy, Config};
use crate::{Engine, HashLifeEngine, NiceInt, Topology};
use eframe::egui::{
    load::SizedTexture, pos2, scroll_area::ScrollBarVisibility, Button, Color32, ColorImage,
    Context, DragValue, Frame, Image, Margin, Rect, Response, RichText, ScrollArea, Slider, Stroke,
    TextureFilter, TextureOptions, TextureWrapMode, Ui, Vec2,
};
use egui_file::{DialogType, FileDialog};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
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

    fn draw_file_dialog(
        ctx: &Context,
        ui: &mut Ui,
        button_text: &str,
        file_path: &mut Option<PathBuf>,
        file_dialog: &mut Option<FileDialog>,
        extension: &'static str,
        dialog_type: DialogType,
    ) -> Response {
        ui.horizontal(|ui| {
            let response = ui.add(Self::new_button(button_text));
            if response.clicked() {
                let filter = Box::new({
                    let ext = Some(OsStr::new(extension));
                    move |path: &Path| -> bool { path.extension() == ext }
                });
                let mut dialog = match dialog_type {
                    DialogType::OpenFile => {
                        FileDialog::open_file(file_path.clone()).show_files_filter(filter)
                    }
                    DialogType::SaveFile => {
                        FileDialog::save_file(file_path.clone()).show_files_filter(filter)
                    }
                    _ => panic!("Unsupported dialog type"),
                };
                dialog.open();
                *file_dialog = Some(dialog);
            }

            if let Some(dialog) = file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        *file_path = Some(file.to_path_buf());
                    }
                }
            }
            response
        })
        .inner
    }

    fn draw_viewport_controls(&mut self, ctx: &Context, ui: &mut Ui) {
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

            let response = Self::draw_file_dialog(
                ctx,
                ui,
                "Save as MacroCell",
                &mut self.saved_file,
                &mut self.save_file_dialog,
                "mc",
                DialogType::SaveFile,
            );
            if let Some(file_path) = self.saved_file.take() {
                let data = self.life_engine.save_into_macrocell();
                std::fs::write(file_path, data).unwrap();
            }
            response
        });

        ui.add_space(Config::WIDGET_GAP);

        ui.label(Self::new_text(&format!(
            "Viewport:\nx: {}\ny: {}\n size: {}",
            NiceInt::from_f64(self.viewport_pos_x),
            NiceInt::from_f64(self.viewport_pos_y),
            NiceInt::from_f64(self.viewport_size)
        )));

        if ui.add(Self::new_button("Reset viewport")).clicked() {
            self.reset_viewport();
        }

        ui.add_space(Config::WIDGET_GAP);

        ui.label(Self::new_text("Recreate field: "));

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.field_source,
                FieldSource::RecursiveOTCA,
                Self::new_text("Recursive OTCA"),
            );
            ui.radio_value(
                &mut self.field_source,
                FieldSource::FileMacroCell,
                Self::new_text("MacroCell"),
            );
            ui.radio_value(
                &mut self.field_source,
                FieldSource::FileRLE,
                Self::new_text("RLE"),
            );
        });
        match self.field_source {
            FieldSource::RecursiveOTCA => {
                ui.horizontal(|ui| {
                    if ui.add(Self::new_button("Recreate with depth:")).clicked() {
                        self.life_engine = Box::new(HashLifeEngine::from_recursive_otca_metapixel(
                            self.field_source_otca_depth,
                            Config::TOP_PATTERN.iter().map(|row| row.to_vec()).collect(),
                        ));
                        self.reset_viewport();
                    }
                    ui.add(DragValue::new(&mut self.field_source_otca_depth).range(1..=5));
                });
            }
            FieldSource::FileMacroCell => {
                Self::draw_file_dialog(
                    ctx,
                    ui,
                    "Open MacroCell",
                    &mut self.opened_file,
                    &mut self.open_file_dialog,
                    "mc",
                    DialogType::OpenFile,
                );
                if let Some(file_path) = self.opened_file.take() {
                    let data = std::fs::read(file_path).unwrap();
                    self.life_engine = Box::new(HashLifeEngine::from_macrocell(&data));
                    self.reset_viewport();
                }
            }
            FieldSource::FileRLE => {
                Self::draw_file_dialog(
                    ctx,
                    ui,
                    "Open RLE",
                    &mut self.opened_file,
                    &mut self.open_file_dialog,
                    "rle",
                    DialogType::OpenFile,
                );
                if let Some(file_path) = self.opened_file.take() {
                    let data = std::fs::read(file_path).unwrap();
                    self.life_engine = Box::new(HashLifeEngine::from_rle(&data));
                    self.reset_viewport();
                }
            }
        }

        ui.add_space(Config::WIDGET_GAP);

        ui.label(Self::new_text(&format!(
            "Generation: {}",
            NiceInt::from(self.generation)
        )));

        ui.label(Self::new_text(&format!(
            "\nLast field update: {:.3} ms",
            NiceInt::from((self.last_update_duration * 1e3) as i128)
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
                BrightnessStrategy::Golly,
                Self::new_text("Golly"),
            )
            .on_hover_text(Self::new_text(
                "Pixels that contain any alive cells will have max brighness",
            ));
            ui.radio_value(
                &mut self.brightness_strategy,
                BrightnessStrategy::Linear,
                Self::new_text("Linear"),
            )
            .on_hover_text(Self::new_text("Linear between min and max population"));
            ui.radio_value(
                &mut self.brightness_strategy,
                BrightnessStrategy::Sigmoid,
                Self::new_text("Sigmoid"),
            )
            .on_hover_text(Self::new_text("Sigmoid on shifted normalized population"));
        });

        if matches!(self.brightness_strategy, BrightnessStrategy::Sigmoid) {
            ui.horizontal(|ui| {
                ui.label(Self::new_text("Shift: "));
                ui.add(Slider::new(&mut self.brightness_shift, -2.0..=2.0));
            });
        }

        if ui.add(Self::new_button("Reset config")).clicked() {
            self.reset_appearance();
        }
    }

    fn draw_stats(&mut self, ui: &mut Ui) {
        if ui.add(Self::new_button("Update slow stats")).clicked() {
            self.cached_verbose_stats = self.life_engine.stats_slow();
        }

        ui.label(Self::new_text(
            &(self.life_engine.stats_fast() + &self.cached_verbose_stats),
        ));
    }

    fn draw_controls(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let aw = ui.available_width();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.draw_viewport_controls(ctx, ui);
                });

                // to adjust the bounds
                ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.draw_appearance_controls(ui);

                    ui.add_space(Config::WIDGET_GAP);

                    self.draw_stats(ui);
                });

                // to adjust the bounds
                ui.add_space((Config::CONTROL_PANEL_WIDTH - aw + ui.available_width()).max(0.));
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

        let gray = self.brightness_strategy.transform(
            resolution as usize,
            &self.viewport_buf,
            self.brightness_shift,
        );

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

    pub fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        let area = ui.available_size();

        let size_px = area
            .y
            .min(area.x - Config::CONTROL_PANEL_WIDTH - Config::FRAME_MARGIN * 5.);
        ui.horizontal(|ui| {
            ui.add_sized([Config::CONTROL_PANEL_WIDTH, area.y], |ui: &mut Ui| {
                ui.vertical(|ui| {
                    Frame::default()
                        .fill(Color32::GRAY)
                        .rounding(Config::ROUNDING)
                        .stroke(Stroke::new(Config::STROKE_WIDTH, Color32::DARK_GRAY))
                        .inner_margin(Margin::same(Config::FRAME_MARGIN))
                        .show(ui, |ui| {
                            ScrollArea::vertical()
                                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                                .show(ui, |ui| {
                                    ui.with_layout(
                                        eframe::egui::Layout::top_down(eframe::egui::Align::LEFT)
                                            .with_cross_justify(true),
                                        |ui| {
                                            self.draw_controls(ctx, ui);
                                        },
                                    );
                                });
                        });
                });
                ui.label("")
            });

            ui.add_space(ui.available_width() - size_px);

            ui.vertical_centered(|ui| {
                self.draw_gol_field(ui, size_px);
            });
        });
    }
}
