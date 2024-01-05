#![warn(clippy::all)]

// Escape   => quit
// Space    => frame step
// P        => pause
// R        => randomize
// F (held) => 100 updates per frame

use conway::InteractionManager;
use conway::{CellularAutomaton, ConwayFieldSimd2};
use macroquad::prelude::*;

const WINDOW_SIZE: usize = 1 << 9;
const FIELD_SIZE: usize = 1024 * 7;

fn window_conf() -> Conf {
    Conf {
        window_title: "Conway".to_owned(),
        window_width: WINDOW_SIZE as i32,
        window_height: WINDOW_SIZE as i32,
        // window_resizable: false, // is broken
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut life = ConwayFieldSimd2::blank(FIELD_SIZE, FIELD_SIZE);

    let [otca_0, otca_1] = ["res/otca_0.rle", "res/otca_1.rle"].map(|path| {
        use std::fs::File;
        use std::io::Read;
        let mut buf = vec![];
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();
        buf
    });
    // life.randomize(Some(42), 0.3);
    life.paste_rle(2048 * 0 + 0, 0, &otca_0);
    life.paste_rle(2048 * 1 + 5, 0, &otca_1);
    life.paste_rle(2048 * 2 + 5, 0, &otca_1);

    let mut im = InteractionManager::new(1.5);

    for _ in 0..30 {
        clear_background(BLACK);
        let texture = Texture2D::from_rgba8(
            FIELD_SIZE as u16,
            FIELD_SIZE as u16,
            &life
                .get_cells(.., ..)
                .into_iter()
                .flat_map(|x| {
                    let c = x as u8 * 255;
                    [c, c, c, 255]
                })
                .collect::<Vec<_>>(),
        );
        im.update();
        let (x, y, zoom) = im.get_x_y_zoom();
        println!("{:?}", im.get_x_y_zoom());
        draw_texture_ex(
            &texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(WINDOW_SIZE as f32 * zoom)),
                ..Default::default()
            },
        );
        life.update(1);
        println!(
            "FPS: {:.3}    MOUSE_POS: {:?}    DEV: {:?}",
            1. / get_frame_time(),
            mouse_position(),
            "todo"
        );
        if is_key_pressed(KeyCode::Escape) {
            break;
        } else {
            next_frame().await
        }
    }
}
