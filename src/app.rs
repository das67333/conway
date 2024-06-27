use crate::{PatternObliviousEngine, Engine, HashLifeEngine};
use eframe::egui;
use std::time::Instant;

pub struct App {
    life_size: f64, // Side length of Conway's square field; edges are stitched together.
    updates_per_frame: u64, // Number of Conway's GoL updates per frame.
    control_panel_min_width: f32, // Minimum pixel width of the control panel on the left.
    zoom_step: f32, // Zooming coefficient for one step of the scroll wheel.
    scroll_scale: f32, // Scaling factor for the scroll wheel output.
    supersampling: f64, // Scaling factor for the texture's rendering resolution.
    zoom: f64,      // Current zoom rate.
    life: Box<dyn Engine>, // Conway's GoL engine.
    life_rect: Option<egui::Rect>, // Part of the window displaying Conway's GoL.
    texture: egui::TextureHandle, // Texture handle of Conway's GoL.
    viewport_buf: Vec<f64>,
    viewport_pos_x: f64, // Position (in the Conway's GoL field) of the left top corner of the viewport.
    viewport_pos_y: f64,
    frame_timer: Instant, // Timer to track frame duration.
    paused: bool,         // Flag indicating if the simulation is paused.
    iter_idx: u64,        // Current iteration index.
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.iter_idx == u64::MAX {
            self.life.print_stats();
            println!("FPS: {:.3}", 1e3 / self.frame_timer.elapsed().as_secs_f64());
            std::process::exit(0);
        }
        self.iter_idx += 1;
        // std::thread::sleep(std::time::Duration::from_millis(100));
        // full-window panel
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::LIGHT_GRAY,
                inner_margin: egui::Margin::symmetric(10., 10.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ctx.request_repaint();

                let (w, h) = (ui.available_width(), ui.available_height());
                // size of the viewport in pixels
                let size_px = h.min(w - self.control_panel_min_width);
                ui.horizontal(|ui| {
                    // drawing control panel
                    ui.add_sized([w - size_px, h], |ui: &mut egui::Ui| {
                        ui.vertical(|ui| {
                            ui.label("hi");
                            ui.label("ha")
                        })
                        .inner
                    });
                    ui.add_space(ui.available_width() - size_px);

                    // updating and drawing the field
                    if let Some(life_rect) = self.life_rect {
                        self.update_viewport(ctx, life_rect);
                    }

                    // RETRIEVING A PART OF THE FIELD THAT SLIGHTLY EXCEEDS VIEWPORT
                    // desired size of texture in pixels
                    let mut resolution = size_px as f64 * self.supersampling;
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
                    let ci = egui::ColorImage::from_gray([resolution as usize; 2], &gray);
                    // TODO: NEAREST when close, LINEAR when far away
                    let texture_options = if size_c > resolution {
                        egui::TextureOptions::LINEAR
                    } else {
                        egui::TextureOptions::NEAREST
                    };
                    self.texture.set(ci, texture_options);
                    let vp_x = (self.viewport_pos_x * self.life_size - x) / size_c;
                    let vp_y = (self.viewport_pos_y * self.life_size - y) / size_c;
                    let vp = egui::pos2(vp_x as f32, vp_y as f32);
                    let vp_s = egui::Vec2::splat((self.zoom * self.life_size / size_c) as f32);
                    self.life_rect.replace(
                        ui.add(|ui: &mut egui::Ui| {
                            egui::Widget::ui(
                                egui::Image::new(egui::load::SizedTexture::new(
                                    self.texture.id(),
                                    [size_px as f32; 2],
                                ))
                                .uv(egui::Rect::from_points(&[vp, vp + vp_s])),
                                ui,
                            )
                        })
                        .rect,
                    );
                });

                if !self.paused {
                    self.life.update(self.updates_per_frame as u32);
                }
                // updating frame counter
                let dur = self.frame_timer.elapsed();
                println!(
                    "FRAMETIME: {:>5} ms \tFPS: {:.3}",
                    dur.as_millis(),
                    1. / dur.as_secs_f64()
                );

                self.frame_timer = Instant::now();
            });
    }
}

impl App {
    pub fn new(
        ctx: &egui::Context,
    ) -> Self {
        let life = PatternObliviousEngine::random(6, 0.3, Some(42));
        // let life = HashLifeEngine::from_recursive_otca_metapixel(
        //     2,
        //     [[0; 4], [1, 1, 1, 0], [0; 4], [0; 4]],
        // );
        // life.into_mc("mega.mc");
        // std::process::exit(0);
        App {
            life_size: life.side_length() as f64,
            updates_per_frame: life.side_length() / 2,
            control_panel_min_width: 60.,
            zoom_step: 1.1,
            scroll_scale: -50.,
            supersampling: 0.7,
            zoom: 1.,
            life: Box::new(life),
            life_rect: None,
            texture: ctx.load_texture(
                "Conway's GoL field",
                egui::ColorImage::default(),
                egui::TextureOptions::default(),
            ),
            viewport_buf: vec![],
            viewport_pos_x: 0.,
            viewport_pos_y: 0.,
            frame_timer: Instant::now(),
            paused: true,
            iter_idx: 0,
        }
    }

    fn update_viewport(&mut self, ctx: &egui::Context, life_rect: egui::Rect) {
        ctx.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                if life_rect.contains(pos) {
                    // TODO: use smooth_scroll_delta
                    if input.raw_scroll_delta.y != 0. {
                        let zoom_change = self
                            .zoom_step
                            .powf(input.raw_scroll_delta.y / self.scroll_scale);
                        self.viewport_pos_x += self.zoom
                            * ((pos.x - life_rect.left_top().x) * (1. - zoom_change)
                                / life_rect.size().x) as f64;
                        self.viewport_pos_y += self.zoom
                            * ((pos.y - life_rect.left_top().y) * (1. - zoom_change)
                                / life_rect.size().y) as f64;
                        self.zoom *= zoom_change as f64;
                    }

                    if input.pointer.primary_down() && input.pointer.delta() != egui::Vec2::ZERO {
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
            if input.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
            }
        });
    }
}
