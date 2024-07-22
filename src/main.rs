#![warn(clippy::all)]

fn main() {
    use eframe::egui::{vec2, ViewportBuilder};

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(vec2(1280., 800.))
            .with_min_inner_size(vec2(640.0, 360.0)),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    eframe::run_native(
        "Conway's Game of Life",
        options,
        Box::new(move |cc| Ok(Box::new(conway::App::new(&cc.egui_ctx)))),
    )
    .unwrap();
}

// async fn f(n: usize) {
//     let mut handles = Vec::with_capacity(n);
//     for h in handles.iter_mut() {
//         *h = tokio::spawn(async {});
//     }
//     tokio::task::
// }

// fn main() {
//     let mut x = 0u64;

//     tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async {
//             println!("Hello world");
//         })
// }
