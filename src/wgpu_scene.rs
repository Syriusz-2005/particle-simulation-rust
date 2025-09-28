use std::sync::Arc;

use encase::{impl_vector, ShaderType, UniformBuffer};
use graphics::math::Vec2d;
use rand::{rng, Rng};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, ComputePipeline, Device, Queue,
};

use crate::{
    particle_type::ParticleTypeManager, receive_into_slice::receive_into_slice,
    scene_like::SceneLike, vector::random_vec, Particle, SceneSettings,
};

type Vec2df = Vec2d<f32>;

#[derive(ShaderType, Debug)]
struct GlobalUniforms {
    screen_size_x: f32,
    screen_size_y: f32,
    particle_types_count: u32,
}

pub struct WgpuScene {
    settings: SceneSettings,
    pub particle_types: ParticleTypeManager,

    particles_pos: Vec<Vec2df>,
    particles_vel: Vec<Vec2df>,
    particles_type_indexes: Vec<u32>,

    device: Device,
    pipeline: ComputePipeline,
    queue: Queue,

    uniform_bind_group_layout: BindGroupLayout,
    storage_bind_group_layout: BindGroupLayout,
}

impl SceneLike for WgpuScene {
    async fn new(settings: SceneSettings) -> Self {
        let instance = wgpu::Instance::new(&Default::default());
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            });

        let storage_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        count: None,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&storage_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Self {
            settings,
            particle_types: ParticleTypeManager::new(settings.particle_types_count),
            particles_pos: vec![],
            particles_vel: vec![],
            particles_type_indexes: vec![],
            device,
            pipeline,
            queue,
            uniform_bind_group_layout,
            storage_bind_group_layout,
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
            usage: wgpu::BufferUsages::STORAGE,
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
            usage: wgpu::BufferUsages::STORAGE,
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
            usage: wgpu::BufferUsages::STORAGE,
        });
        let uniforms = GlobalUniforms {
            screen_size_x: self.settings.screen_size[0] as f32,
            screen_size_y: self.settings.screen_size[1] as f32,
            particle_types_count: self.settings.particle_types_count as u32,
        };
        let mut encase_uniform_buffer = UniformBuffer::new(Vec::new());
        encase_uniform_buffer
            .write(&uniforms)
            .expect("Uniform buffer should contain uniforms");
        let global_uniforms_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: encase_uniform_buffer.into_inner().as_slice(),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let type_forces_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.particle_types.get_forces_flattened()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let type_radii_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.particle_types.get_radii_flattened()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let type_min_distance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.particle_types.get_min_distance_flattened()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let type_masses_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.particle_types.get_masses()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let type_drag_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.particle_types.get_drag()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform bind group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: type_masses_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: type_drag_buffer.as_entire_binding(),
                },
            ],
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &self.storage_bind_group_layout,
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
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: type_forces_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: type_radii_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: type_min_distance_buffer.as_entire_binding(),
                },
            ],
        });

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            let num_dispatches =
                (self.particles_pos.len() / 64) as u32 + (self.particles_pos.len() / 64 > 0) as u32;
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
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
