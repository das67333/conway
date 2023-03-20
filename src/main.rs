#![feature(portable_simd)]
#![feature(test)]
#[warn(clippy::all)]
extern crate test;

pub mod grid4;
pub mod grid_common;
use grid4::*;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    const WIDTH: usize = 1600;
    const HEIGHT: usize = 900;

    let event_loop = EventLoop::new();
    let mut input = winit_input_helper::WinitInputHelper::new();

    let window = {
        WindowBuilder::new()
            .with_title("Conway's Game of Life")
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    let mut life = ConwayGrid::<WIDTH, HEIGHT>::random();
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
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_held(VirtualKeyCode::F) {
                for _ in 0..100 {
                    life.update();
                }
            }
            if input.key_pressed_os(VirtualKeyCode::Space) {
                paused = true;
            }
            if input.key_pressed(VirtualKeyCode::R) {
                life.randomize(None, None);
            }
            if !paused || input.key_pressed_os(VirtualKeyCode::Space) {
                life.update();
            }
            window.request_redraw();
        }
    });
}

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[bench]
    fn bench_randomize(b: &mut test::Bencher) {
        let mut life = ConwayGrid::<1600, 900>::random();
        b.iter(|| life.randomize(None, None));
    }

    #[bench]
    fn bench_update(b: &mut test::Bencher) {
        // cpu: 1.4 GHz fixed
        // 0    13.1    ms      simple
        // 1    1.39    ms      addition of 8 byte arrays
        // 2    688     mcs     bit magic
        // 2+   377     mcs     +avx2
        // 3+   280     mcs     single vector
        // 4    223     mcs     portable simd
        let mut life = ConwayGrid::<1600, 900>::random();
        b.iter(|| {
            life.update();
        });
    }

    #[bench]
    fn bench_for_comparison(b: &mut test::Bencher) {
        let mut arr1 = vec![1u8; 1600 * 900 + 2];
        b.iter(|| arr1.fill(1));
    }

    #[test]
    fn test_correctness() {
        use sha2::{Digest, Sha256};

        const WIDTH: usize = 1600;
        const HEIGHT: usize = 900;

        let mut life = ConwayGrid::<WIDTH, HEIGHT>::random();
        {
            let mut hasher = Sha256::new();
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    hasher.update(&[life.get(x, y) as u8]);
                }
            }
            assert_eq!(
                hex::encode(hasher.finalize()),
                "ec1860faadd0b3d5579e1b1d45f44b1a273b348385db0546cb713921016dd16e"
            );
        }
        life.update();
        {
            let mut hasher = Sha256::new();
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    hasher.update(&[life.get(x, y) as u8]);
                }
            }
            assert_eq!(
                hex::encode(hasher.finalize()),
                "f973b5956357d8a540a288a42549fa799b6244c74c060bc339957cfd03779951"
            );
        }
    }
}
