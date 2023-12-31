use wgpu::util::DeviceExt;

pub struct ConwayField {
    data: Vec<u32>,
    data_is_synced: bool,
    width: usize,
    height: usize,
    width_effective: usize,
    device: wgpu::Device,
    queue: wgpu::Queue,
    idx_active: usize,
    storage_buffers: [wgpu::Buffer; 2],
    bind_groups: [wgpu::BindGroup; 2],
    staging_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
}

impl ConwayField {
    const CELLS_IN_CHUNK: usize = 32;

    fn update_inner(&mut self, iters_cnt: usize) {
        if !self.data_is_synced {
            self.queue.write_buffer(
                &self.storage_buffers[self.idx_active],
                0,
                bytemuck::cast_slice(self.data.as_slice()),
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        for _ in 0..iters_cnt {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups[self.idx_active], &[]);
            compute_pass.dispatch_workgroups(self.height as u32, 1, 1);
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
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
        self.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(receiver.recv_async()).unwrap().unwrap();

        let view = buffer_slice.get_mapped_range();
        self.data.copy_from_slice(bytemuck::cast_slice(&view));
        drop(view);
        self.staging_buffer.unmap();
        self.data_is_synced = true;
    }
}

impl crate::CellularAutomaton for ConwayField {
    fn blank(width: usize, height: usize) -> ConwayField {
        assert!(width % Self::CELLS_IN_CHUNK == 0);
        let width_effective = width / Self::CELLS_IN_CHUNK;
        let instance = wgpu::Instance::default();
        let request_adapter_options: wgpu::RequestAdapterOptionsBase<&wgpu::Surface> = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
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
            size: (width_effective * height * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE // TODO
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        };
        let uniform_size = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[width_effective as u32, height as u32]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let storage_buffers = [0; 2].map(|_| device.create_buffer(&buffer_desc));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                (0, wgpu::BufferBindingType::Uniform),
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
            size: (width_effective * height * 4) as u64,
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
            data: vec![0; width_effective * height],
            data_is_synced: false,
            width,
            height,
            width_effective,
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
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset = x % Self::CELLS_IN_CHUNK;
        self.data[pos] >> offset & 1 != 0
    }

    fn get_cells(&self) -> Vec<bool> {
        self.data
            .iter()
            .flat_map(|x| (0..Self::CELLS_IN_CHUNK).map(|i| (*x >> i & 1 != 0)))
            .collect()
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        self.data_is_synced = false;
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let mask = 1 << x % Self::CELLS_IN_CHUNK;
        if state {
            self.data[pos] |= mask;
        } else {
            self.data[pos] &= !mask;
        }
    }

    fn set_cells(&mut self, states: &[bool]) {
        assert_eq!(states.len(), self.width * self.height);
        self.data_is_synced = false;
        for (dst, src) in self
            .data
            .iter_mut()
            .zip(states.chunks_exact(Self::CELLS_IN_CHUNK))
        {
            *dst = src
                .iter()
                .enumerate()
                .map(|(i, &x)| (x as u32) << i)
                .sum::<u32>();
        }
    }

    fn update(&mut self, iters_cnt: usize) {
        self.update_inner(iters_cnt);
    }
}
