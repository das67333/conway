#![warn(clippy::all)]

use conway::CellularAutomaton;
use eframe::egui;
use std::time::{Duration, Instant};

pub struct App<Life: CellularAutomaton> {
    life_size: usize,
    updates_per_frame: usize,
    control_panel_min_width: f32,
    zoom_step: f32,
    scroll_scale: f32,
    zoom: f32,
    life: Life,
    life_rect: Option<egui::Rect>,
    texture: Option<egui::TextureHandle>,
    frame_pos: egui::Pos2,
    frame_timer: Option<Instant>,
    paused: bool,
    iter_idx: usize,
}

fn otca_transform<Life: CellularAutomaton, const N: usize>(life: &mut Life, data: [[u8; N]; N]) {
    assert_eq!(N * 2048, life.size().0);
    assert_eq!(N * 2048, life.size().1);

    let otca = (["res/otca_0.rle", "res/otca_1.rle"]).map(|path| {
        use std::fs::File;
        use std::io::Read;
        let mut buf = vec![];
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();
        buf
    });
    for (i, &row) in data.iter().enumerate() {
        for (j, &state) in row.iter().enumerate() {
            life.paste_rle(j * 2048, i * 2048, &otca[state as usize])
        }
    }
    // for i in 0..N {
    //     for (x, y) in [
    //         (i * 2048 + 3, 3),
    //         (i * 2048 + 2044, 3),
    //         (i * 2048 + 3, N * 2048 - 4),
    //         (i * 2048 + 2044, N * 2048 - 4),
    //         (3, i * 2048 + 3),
    //         (3, i * 2048 + 2044),
    //         (N * 2048 - 4, i * 2048 + 3),
    //         (N * 2048 - 4, i * 2048 + 2044),
    //     ] {
    //         for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
    //             let (sx, sy) = ((x as isize + dx) as usize, (y as isize + dy) as usize);
    //             life.set_cell(sx, sy, false);
    //         }
    //     }
    // }
}

fn main() {
    let field_size = 1024 * 8;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(1000., 600.)),
        ..Default::default()
    };
    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(move |_cc| {
            let mut life = conway::ConwayFieldHash::blank(field_size, field_size);
            otca_transform(&mut life, [[0; 4], [1, 1, 1, 0], [0; 4], [0; 4]]);
            Box::new(App {
                life_size: field_size,
                updates_per_frame: field_size / 2,
                control_panel_min_width: 60.,
                zoom_step: 1.5,
                scroll_scale: -50.,
                zoom: 1.,
                life,
                life_rect: None,
                texture: None,
                frame_pos: egui::Pos2::ZERO,
                frame_timer: None,
                paused: false,
                iter_idx: 0,
            })
        }),
    )
    .unwrap();
}

impl<Life: CellularAutomaton> eframe::App for App<Life> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.iter_idx == 50 {
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
                    let texture = self.texture.get_or_insert_with(|| {
                        ui.ctx().load_texture(
                            "field",
                            egui::ColorImage::new([0, 0], egui::Color32::TEMPORARY_COLOR), // 2000 is fastest?; mb ci?
                            egui::TextureOptions::LINEAR,
                        )
                    });
                    if !self.paused || self.life_rect.is_none() {
                        let pixels = self
                            .life
                            .get_cells(.., ..)
                            .into_iter()
                            .flat_map(|x| [x as u8 * u8::MAX; 3])
                            .collect::<Vec<_>>();
                        let ci = egui::ColorImage::from_rgb([self.life_size; 2], &pixels);
                        texture.set(ci, egui::TextureOptions::NEAREST); // LINEAR

                        self.life.update(self.updates_per_frame);
                    }
                    self.life_rect.replace(
                        ui.add_sized([size; 2], |ui: &mut egui::Ui| {
                            egui::Widget::ui(
                                egui::Image::new(egui::load::SizedTexture::new(texture, [size; 2]))
                                    .uv(egui::Rect::from_points(&[
                                        self.frame_pos,
                                        self.frame_pos + egui::vec2(self.zoom, self.zoom),
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

impl<Life: CellularAutomaton> App<Life> {
    fn update_viewport(&mut self, ctx: &egui::Context, life_rect: egui::Rect) {
        // TODO: apply faster
        ctx.input(|input| {
            if let Some(pos) = input.pointer.latest_pos() {
                if life_rect.contains(pos) {
                    if input.scroll_delta.y != 0. {
                        let zoom_change = self
                            .zoom_step
                            .powf(input.scroll_delta.y / self.scroll_scale);
                        self.frame_pos +=
                            self.zoom * (pos - life_rect.left_top()) * (1. - zoom_change)
                                / life_rect.size();
                        let pp = (pos - life_rect.left_top()) / life_rect.size();
                        println!("zoom_change={}  PP=[{:.3}, {:.3}]", zoom_change, pp.x, pp.y);
                        self.zoom *= zoom_change;
                    }

                    if input.pointer.primary_down() && input.pointer.delta() != egui::Vec2::ZERO {
                        self.frame_pos -= input.pointer.delta() / life_rect.size() * self.zoom;
                    }
                    self.frame_pos = self
                        .frame_pos
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
