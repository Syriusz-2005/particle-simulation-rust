use std::sync::Arc;

use graphics::math::Vec2d;
use rand::{rng, Rng};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    ComputePipeline, Device, Queue,
};

use crate::{
    particle_type::ParticleTypeManager, receive_into_slice::receive_into_slice,
    scene_like::SceneLike, vector::random_vec, Particle, SceneSettings,
};

type Vec2df = Vec2d<f32>;

pub struct WgpuScene {
    settings: SceneSettings,
    pub particle_types: ParticleTypeManager,

    particles_pos: Vec<Vec2df>,
    particles_vel: Vec<Vec2df>,
    particles_type_indexes: Vec<u32>,

    device: Device,
    pipeline: ComputePipeline,
    queue: Queue,
}

impl SceneLike for WgpuScene {
    async fn new(settings: SceneSettings) -> Self {
        let instance = wgpu::Instance::new(&Default::default());
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        WgpuScene {
            settings,
            particle_types: ParticleTypeManager::new(settings.particle_types_count),
            particles_pos: vec![],
            particles_vel: vec![],
            particles_type_indexes: vec![],
            device,
            pipeline,
            queue,
        }
    }

    fn init(&mut self) {
        let random_source = &mut rng();
        self.particles_pos = (0..self.settings.particle_count)
            .map(|_| random_vec(random_source, 0.0..(self.settings.screen_size[0] as f32)))
            .collect();
        self.particles_vel = (0..self.settings.particle_count)
            .map(|_| [0.0, 0.0])
            .collect();
        self.particles_type_indexes = (0..self.settings.particle_count)
            .map(|_| random_source.random_range(0..self.settings.particle_types_count as u32))
            .collect();
    }

    async fn update(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute encoder"),
            });

        let input_positions_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&self.particles_pos),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let output_positions_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output"),
            size: input_positions_buffer.size(),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let temp_buffer_positions = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("temp"),
            size: input_positions_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let input_velocities_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&self.particles_vel),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let output_velocities_buffers = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output"),
            size: input_velocities_buffer.size(),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let temp_buffer_velocities = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("temp"),
            size: input_velocities_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let input_type_indexes = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&self.particles_type_indexes),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_positions_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_velocities_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: input_type_indexes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output_positions_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: output_velocities_buffers.as_entire_binding(),
                },
            ],
        });

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            let num_dispatches =
                (self.particles_pos.len() / 64) as u32 + (self.particles_pos.len() / 64 > 0) as u32;
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(num_dispatches, 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &output_positions_buffer,
            0,
            &temp_buffer_positions,
            0,
            output_positions_buffer.size(),
        );

        encoder.copy_buffer_to_buffer(
            &output_velocities_buffers,
            0,
            &temp_buffer_velocities,
            0,
            output_velocities_buffers.size(),
        );

        self.queue.submit([encoder.finish()]);

        receive_into_slice(&self.device, temp_buffer_positions, &mut self.particles_pos).await;
        receive_into_slice(
            &self.device,
            temp_buffer_velocities,
            &mut self.particles_vel,
        )
        .await;
    }

    fn get_particles(&self) -> std::sync::Arc<Vec<crate::Particle>> {
        let particles = (0..self.settings.particle_count)
            .map(|i| Particle {
                pos: self.particles_pos[i].map(|v| v as f64),
                vel: self.particles_vel[i].map(|v| v as f64),
                type_index: self.particles_type_indexes[i] as usize,
            })
            .collect::<Vec<Particle>>();
        return Arc::new(particles);
    }

    fn new_world(&mut self) {
        todo!()
    }

    fn get_particle_color(&self, type_index: usize) -> [f32; 4] {
        return self.particle_types.get_particle_color(type_index);
    }
}
