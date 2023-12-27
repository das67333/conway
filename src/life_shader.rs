use wgpu::util::DeviceExt;

pub struct ConwayField {
    data: Vec<u32>,
    data_is_synced: bool,
    width: usize,
    height: usize,
    device: wgpu::Device,
    queue: wgpu::Queue,
    idx_active: usize,
    storage_buffers: [wgpu::Buffer; 2],
    bind_groups: [wgpu::BindGroup; 2],
    staging_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
}

impl crate::CellularAutomaton for ConwayField {
    fn blank(width: usize, height: usize) -> ConwayField {
        let instance = wgpu::Instance::default();
        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        };
        let adapter = pollster::block_on(instance.request_adapter(&request_adapter_options))
            .expect("No adapters found");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .unwrap();
        let buffer_desc = wgpu::BufferDescriptor {
            label: None,
            size: (width * height * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE // TODO
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        };
        let uniform_size = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[width as u32, height as u32]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let storage_buffers = [0; 2].map(|_| device.create_buffer(&buffer_desc));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                (0, wgpu::BufferBindingType::Uniform), // TODO: to separate group for performance
                (1, wgpu::BufferBindingType::Storage { read_only: true }),
                (2, wgpu::BufferBindingType::Storage { read_only: false }),
            ]
            .map(|(binding, ty)| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }),
        });
        let bind_groups = [(0, 1), (1, 0)].map(|(idx_0, idx_1)| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_size.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: storage_buffers[idx_0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: storage_buffers[idx_1].as_entire_binding(),
                    },
                ],
            })
        });
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (width * height * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        ConwayField {
            data: vec![0; width * height],
            data_is_synced: false,
            width,
            height,
            device,
            queue,
            idx_active: 0,
            storage_buffers,
            bind_groups,
            staging_buffer,
            pipeline,
        }
    }

    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_cell(&self, x: usize, y: usize) -> bool {
        self.data[x + y * self.width] != 0
    }

    fn get_cells(&self) -> Vec<bool> {
        self.data.iter().map(|&x| x != 0).collect()
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        self.data_is_synced = false;
        self.data[x + y * self.width] = state as u32;
    }

    fn set_cells(&mut self, states: &[bool]) {
        self.data_is_synced = false;
        let states = states.iter().map(|&x| x as u32).collect::<Vec<_>>();
        self.data.copy_from_slice(&states);
    }

    fn update(&mut self, iters_cnt: usize) {
        self.update_inner(iters_cnt);
    }
}

impl ConwayField {
    fn update_inner(&mut self, iters_cnt: usize) {
        // ...bind_group
        log::info!("Entering update, iter_cnt={iters_cnt}");

        if !self.data_is_synced {
            log::info!("Started syncing the field with gpu");
            self.queue.write_buffer(
                &self.storage_buffers[self.idx_active],
                0,
                bytemuck::cast_slice(self.data.as_slice()),
            );
            log::info!("Synced the field");
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        for _ in 0..iters_cnt {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups[self.idx_active], &[]);
            log::info!("Start dispatch");
            compute_pass.dispatch_workgroups(self.width as u32, self.height as u32, 1);
            log::info!("Finish dispatch");
            drop(compute_pass);
            self.idx_active = 1 - self.idx_active;
        }

        encoder.copy_buffer_to_buffer(
            &self.storage_buffers[self.idx_active],
            0,
            &self.staging_buffer,
            0,
            self.storage_buffers[self.idx_active].size(),
        );

        self.queue.submit(Some(encoder.finish()));
        log::info!("Submitted commands.");
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
        self.device.poll(wgpu::Maintain::Wait);
        log::info!("Device polled.");
        pollster::block_on(receiver.recv_async()).unwrap().unwrap();
        log::info!("Result received.");

        let view = buffer_slice.get_mapped_range();
        self.data.copy_from_slice(bytemuck::cast_slice(&view));
        drop(view);
        log::info!("Results written to local buffer.");
        self.staging_buffer.unmap();
    }
}
