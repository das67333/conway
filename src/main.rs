#![warn(clippy::all)]

mod empty_hasher;
mod engine;
mod megapixel;

use eframe::egui;
use engine::ConwayFieldHash256;
use std::time::{Duration, Instant};

pub struct App {
    life_size: usize, // Side length of Conway's square field; edges are stitched together.
    updates_per_frame: usize, // Number of Conway's GoL updates per frame.
    control_panel_min_width: f32, // Minimum pixel width of the control panel on the left.
    zoom_step: f32,   // Zooming coefficient for one step of the scroll wheel.
    scroll_scale: f32, // Scaling factor for the scroll wheel output.
    supersampling: f32, // Scaling factor for the texture's rendering resolution.
    zoom: f32,        // Current zoom rate.
    life: ConwayFieldHash256, // Conway's GoL engine; updates are performed at 256x256 level using simd instructions.
    life_rect: Option<egui::Rect>, // Part of the window displaying Conway's GoL.
    texture: egui::TextureHandle, // Texture handle of Conway's GoL.
    viewport_buf: Vec<f32>,
    viewport_pos: egui::Pos2, // Position (in the Conway's GoL field) of the left top corner of the viewport.
    frame_timer: Option<Instant>, // Timer to track frame duration.
    paused: bool,             // Flag indicating if the simulation is paused.
    iter_idx: usize,          // Current iteration index.
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(1200., 1000.)),
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| {
            // let mut life = ConwayFieldHash256::blank(field_size.ilog2());
            // otca_transform(&mut life, [[0; 4], [1, 1, 1, 0], [0; 4], [0; 4]]);
            // otca_transform(&mut life, [[0]]);
            let life = ConwayFieldHash256::from_recursive_otca_megapixel(2, [[1]]);
            Box::new(App {
                life_size: life.side_length(),
                updates_per_frame: life.side_length() / 2,
                control_panel_min_width: 60.,
                zoom_step: 1.1,
                scroll_scale: -50.,
                supersampling: 1.,
                zoom: 1.,
                life,
                life_rect: None,
                texture: cc.egui_ctx.load_texture(
                    "field",
                    egui::ColorImage::default(),
                    egui::TextureOptions::default(),
                ),
                viewport_buf: vec![],
                viewport_pos: egui::Pos2::ZERO,
                frame_timer: None,
                paused: false,
                iter_idx: 0,
            })
        }),
    )
    .unwrap();
}

#[inline(never)]
fn normalize_brightness(v: &Vec<f32>) -> Vec<u8> {
    // TODO: slow!!!
    let (mut low, mut high) = (0., 0.);
    for &x in v {
        if high < x {
            high = x;
        }
    }
    v.iter()
        .map(|x| ((x - low) / (high - low) * u8::MAX as f32) as u8)
        .collect()
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.iter_idx == 1 {
            std::process::exit(0);
        }
        self.iter_idx += 1;
        // full-window panel
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::LIGHT_GRAY,
                inner_margin: egui::style::Margin::symmetric(10., 10.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ctx.request_repaint();
                // updating frame counter
                if let Some(timer) = self.frame_timer {
                    println!("FPS: {:.1}  ", 1. / timer.elapsed().as_secs_f64());
                }
                self.frame_timer = Some(Instant::now());

                let (w, h) = (ui.available_width(), ui.available_height());
                // size of the viewport in pixels
                let size = h.min(w - self.control_panel_min_width);
                ui.horizontal(|ui| {
                    // drawing control panel
                    ui.add_sized([w - size, h], |ui: &mut egui::Ui| {
                        ui.vertical(|ui| {
                            ui.label("hi");
                            ui.label("ha")
                        })
                        .inner
                    });
                    ui.add_space(ui.available_width() - size);

                    // updating an drawing the field
                    if let Some(life_rect) = self.life_rect {
                        self.update_viewport(ctx, life_rect);
                    }

                    // RETRIEVING A PART OF THE FIELD THAT SLIGHTLY EXCEEDS VIEWPORT
                    // desired size of texture in pixels
                    let mut resolution = (size * self.supersampling).ceil() as usize;
                    // left top viewport coordinate in cells
                    let mut x = (self.life_size as f32 * self.viewport_pos.x) as usize;
                    let mut y = (self.life_size as f32 * self.viewport_pos.y) as usize;
                    // size of viewport in cells
                    let mut side = (self.life_size as f32 * self.zoom).ceil() as usize;
                    self.life.fill_texture(
                        &mut x,
                        &mut y,
                        &mut side,
                        &mut resolution,
                        &mut self.viewport_buf,
                    );

                    let gray = normalize_brightness(&self.viewport_buf);
                    let ci = egui::ColorImage::from_gray([resolution; 2], &gray);
                    // TODO: NEAREST when close, LINEAR when far away
                    let texture_options = if side > resolution {
                        egui::TextureOptions::LINEAR
                    } else {
                        egui::TextureOptions::NEAREST
                    };
                    self.texture.set(ci, texture_options);
                    if !self.paused {
                        self.life.update(self.updates_per_frame);
                    }
                    let mut frame_pos = self.viewport_pos
                        - egui::vec2(
                            x as f32 / self.life_size as f32,
                            y as f32 / self.life_size as f32,
                        );
                    frame_pos.x *= self.life_size as f32 / side as f32;
                    frame_pos.y *= self.life_size as f32 / side as f32;
                    self.life_rect.replace(
                        ui.add(|ui: &mut egui::Ui| {
                            egui::Widget::ui(
                                egui::Image::new(egui::load::SizedTexture::new(
                                    self.texture.id(),
                                    [size as f32; 2],
                                ))
                                .uv(egui::Rect::from_points(&[
                                    frame_pos,
                                    frame_pos
                                        + egui::Vec2::splat(
                                            self.zoom * self.life_size as f32 / side as f32,
                                        ),
                                ])),
                                ui,
                            )
                        })
                        .rect,
                    );
                });

                std::thread::sleep(Duration::from_millis(20));
            });
    }
}

impl App {
    fn update_viewport(&mut self, ctx: &egui::Context, life_rect: egui::Rect) {
        ctx.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                if life_rect.contains(pos) {
                    if input.scroll_delta.y != 0. {
                        let zoom_change = self
                            .zoom_step
                            .powf(input.scroll_delta.y / self.scroll_scale);
                        self.viewport_pos +=
                            self.zoom * (pos - life_rect.left_top()) * (1. - zoom_change)
                                / life_rect.size();
                        self.zoom *= zoom_change;
                    }

                    if input.pointer.primary_down() && input.pointer.delta() != egui::Vec2::ZERO {
                        self.viewport_pos -= input.pointer.delta() / life_rect.size() * self.zoom;
                    }
                    self.viewport_pos = self
                        .viewport_pos
                        .min(egui::pos2(1., 1.) - egui::vec2(self.zoom, self.zoom))
                        .max(egui::Pos2::ZERO);
                    self.zoom = self.zoom.min(1.);
                }
            }
            if input.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
            }
        });
    }
}
