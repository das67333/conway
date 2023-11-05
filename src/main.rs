#![warn(clippy::all)]

// Escape   => quit
// Space    => frame step
// P        => pause
// R        => randomize
// F (held) => 100 updates per frame

mod life_fast;
mod life_hash;
mod life_naive;
mod trait_grid;
use trait_grid::Grid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctness() {
        let n = 128;
        let mut life0 = life_naive::ConwayField::random(n, n, Some(42), 0.3);
        let mut life1 = life_fast::ConwayField::random(n, n, Some(42), 0.3);
        let mut life2 = life_hash::ConwayField::random(n, n, Some(42), 0.3);

        life0.update(n);
        life1.update(n);
        life2.update(n);

        for y in 0..n {
            for x in 0..n {
                assert_eq!(life0.get(x, y), life1.get(x, y), "x={x} y={y}");
                assert_eq!(life1.get(x, y), life2.get(x, y), "x={x} y={y}");
            }
        }
    }
}

// fn main() {
//     // Approximate duration, N=1024, sec
//     // update(N)            3.67        0.057       6.7
//     // update(N*10)         36.9        0.58        10.9
//     // update(N*100)        372         5.7         11.0
//     let (w, h) = (1024, 1024);
//     let iters_num = 1_000_000_000 / (w * h) + 1;
//     let mut life = life_fast::ConwayField::random(w, h, None, 0.3);
//     let timer = std::time::Instant::now();
//     life.update(iters_num);
//     println!("{:?}", timer.elapsed());
// }


fn main() {
    use life_fast::ConwayField;
    use pixels::{Pixels, SurfaceTexture};
    use winit::{
        dpi::LogicalSize,
        event::{Event, VirtualKeyCode},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    let (width, height) = (1600, 900);
    let mut life = ConwayField::random(width, height, Some(52), 0.1);

    let event_loop = EventLoop::new();
    let mut input = winit_input_helper::WinitInputHelper::new();
    let window = {
        WindowBuilder::new()
            .with_title("Conway's Game of Life")
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_decorations(false)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    // window.focus_window();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let mut pixels = Pixels::new(width as u32, height as u32, surface_texture).unwrap();
        life.draw(pixels.get_frame_mut());
        pixels.render().unwrap();
        pixels
    };

    let mut paused = false;

    event_loop.run(move |event, _, control_flow| {
        if matches!(event, Event::RedrawRequested(_)) {
            life.draw(pixels.get_frame_mut());
            if matches!(pixels.render(), Err(_)) {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::Space) {
                paused = true;
            }
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_pressed(VirtualKeyCode::R) {
                life = ConwayField::random(width, height, None, 0.3);
            }
            if input.key_held(VirtualKeyCode::F) {
                life.update(100);
            } else if !paused || input.key_pressed(VirtualKeyCode::Space) {
                life.update(1);
            }
            window.request_redraw();
        }
    });
}
