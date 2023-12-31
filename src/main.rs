#![warn(clippy::all)]

// Escape   => quit
// Space    => frame step
// P        => pause
// R        => randomize
// F (held) => 100 updates per frame

mod life_simd;
mod life_hash;
mod life_naive;
mod life_shader;

use conway::CellularAutomaton;
// use wgpu::util::DeviceExt;
// use winit::{
//     dpi::{LogicalSize, PhysicalSize},
//     event::{ElementState, Event, WindowEvent},
//     event_loop::EventLoop,
//     keyboard::{Key, NamedKey},
//     platform::modifier_supplement::KeyEventExtModifierSupplement,
//     window::WindowBuilder,
// };

fn main() {
        // env_logger::builder()
        //     .filter_level(log::LevelFilter::Info)
        //     .format_timestamp_nanos()
        //     .init();

    let (w, h) = (1 << 15, 1 << 15);
    let mut life = life_shader::ConwayField::blank(w, h);
    life.randomize(None, 0.6);
    let timer = std::time::Instant::now();
    life.update(100);
    println!("{:?}", timer.elapsed());
}

// fn main() {
//     env_logger::builder()
//         .filter_level(log::LevelFilter::Info)
//         .format_timestamp_nanos()
//         .init();

//     use life_simd::ConwayField;
//     let (width, height) = (800, 600);
//     let mut life = ConwayField::blank(width, height);
//     life.randomize(None, 0.3);

//     let event_loop = EventLoop::new().unwrap();
//     let window = {
//         WindowBuilder::new()
//             .with_title("Conway's Game of Life")
//             .with_inner_size(PhysicalSize::new(width as f64, height as f64))
//             // .with_decorations(false)
//             .with_resizable(false)
//             .build(&event_loop)
//             .unwrap()
//     };
//     // window.focus_window();

//     // let mut pixels = {
//     //     let window_size = window.inner_size();
//     //     let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
//     //     let mut pixels = Pixels::new(width as u32, height as u32, surface_texture).unwrap();
//     //     life.draw(pixels.frame_mut());
//     //     pixels.render().unwrap();
//     //     pixels
//     // };
//     /////////////////////////////////////////////////////////////////////
//     // Create a wgpu instance and device
//     let instance = wgpu::Instance::default();

//     let surface = unsafe { instance.create_surface(&window).unwrap() };
//     let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
//         power_preference: wgpu::PowerPreference::default(),
//         force_fallback_adapter: false,
//         // Request an adapter which can render to our surface
//         compatible_surface: Some(&surface),
//     }))
//     .expect("Failed to find an appropriate adapter");

//     // Create the logical device and command queue
//     let (device, queue) = pollster::block_on(adapter.request_device(
//         &wgpu::DeviceDescriptor {
//             label: None,
//             features: wgpu::Features::empty(),
//             limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
//         },
//         None,
//     ))
//     .expect("Failed to create device");

//     // Load the shaders from disk
//     let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: None,
//         source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
//     });

//     let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: None,
//         bind_group_layouts: &[],
//         push_constant_ranges: &[],
//     });

//     let swapchain_capabilities = surface.get_capabilities(&adapter);
//     let swapchain_format = swapchain_capabilities.formats[0];

//     let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: None,
//         layout: Some(&pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: &shader,
//             entry_point: "vs_main",
//             buffers: &[],
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &shader,
//             entry_point: "fs_main",
//             targets: &[Some(swapchain_format.into())],
//         }),
//         primitive: wgpu::PrimitiveState::default(),
//         depth_stencil: None,
//         multisample: wgpu::MultisampleState::default(),
//         multiview: None,
//     });

//     let mut config = wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format: swapchain_format,
//         width: width as u32,
//         height: height as u32,
//         present_mode: wgpu::PresentMode::Fifo,
//         alpha_mode: swapchain_capabilities.alpha_modes[0],
//         view_formats: vec![],
//     };

//     surface.configure(&device, &config);

//     let window = &window;
//     event_loop
//         .run(move |event, target| {
//             // Have the closure take ownership of the resources.
//             // `event_loop.run` never returns, therefore we must do this to ensure
//             // the resources are properly cleaned up.
//             let _ = (&instance, &adapter, &shader, &pipeline_layout);

//             if let Event::WindowEvent {
//                 window_id: _,
//                 event,
//             } = event
//             {
//                 match event {
//                     WindowEvent::Resized(new_size) => {
//                         // Reconfigure the surface with the new size
//                         config.width = new_size.width.max(1);
//                         config.height = new_size.height.max(1);
//                         surface.configure(&device, &config);
//                         // On macos the window needs to be redrawn manually after resizing
//                         window.request_redraw();
//                     }
//                     WindowEvent::RedrawRequested => {
//                         let frame = surface
//                             .get_current_texture()
//                             .expect("Failed to acquire next swap chain texture");
//                         let view = frame
//                             .texture
//                             .create_view(&wgpu::TextureViewDescriptor::default());
//                         let mut encoder =
//                             device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                                 label: None,
//                             });
//                         {
//                             let mut rpass =
//                                 encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                                     label: None,
//                                     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                                         view: &view,
//                                         resolve_target: None,
//                                         ops: wgpu::Operations {
//                                             load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
//                                             store: true,
//                                         },
//                                     })],
//                                     depth_stencil_attachment: None,
//                                 });
//                             rpass.set_pipeline(&render_pipeline);
//                             rpass.draw(0..3, 0..1);
//                         }

//                         queue.submit(Some(encoder.finish()));
//                         frame.present();
//                     }
//                     WindowEvent::CloseRequested => target.exit(),
//                     _ => {}
//                 };
//             }
//         })
//         .unwrap();
//     /////////////////////////////////////////////////////////////////////////////////
//     // let mut paused = false;

//     // event_loop
//     //     .run(move |event, elwt| {
//     //         if let Event::WindowEvent { event, .. } = event {
//     //             match event {
//     //                 WindowEvent::CloseRequested => elwt.exit(),
//     //                 WindowEvent::KeyboardInput { event, .. } => {
//     //                     if event.state == ElementState::Pressed {
//     //                         match event.key_without_modifiers().as_ref() {
//     //                             Key::Named(NamedKey::Escape) => elwt.exit(),
//     //                             Key::Character(" ") => paused = true,
//     //                             Key::Character("P") => paused = !paused,
//     //                             Key::Character("R") => life.randomize(None, 0.3),
//     //                             Key::Character("F") => life.update(100),
//     //                             _ => {
//     //                                 if !paused {
//     //                                     life.update(1);
//     //                                     // window.request_redraw();
//     //                                 }
//     //                             }
//     //                         }
//     //                     }
//     //                     if event.state == ElementState::Released {}
//     //                 }
//     //                 WindowEvent::RedrawRequested => {
//     //                     log::info!("Redrawing");
//     //                     // life.draw(pixels.frame_mut());
//     //                     // match pixels.render() {
//     //                     //     Ok(_) => (),
//     //                     //     Err(err) => {
//     //                     //         println!("{err}");
//     //                     //         elwt.exit();
//     //                     //     }
//     //                     // }
//     //                 }
//     //                 _ => (),
//     //             }
//     //         }
//     //     })
//     //     .unwrap();
// }
