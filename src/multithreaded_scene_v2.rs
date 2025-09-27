use std::sync::{Arc, Mutex};

use graphics::math::Vec2d;
use rand::{rng, Rng};
use threadpool::ThreadPool;

use crate::{
    constants::{K, THREAD_COUNT},
    particle_type::ParticleTypeManager,
    scene_like::SceneLike,
    vector::{add, div_scalar, len, mul_scalar, normalize, remap, sub},
    Particle, SceneSettings,
};

pub struct MultithreadedSceneV2 {
    settings: Arc<SceneSettings>,
    pub particle_types: Arc<ParticleTypeManager>,
    pool: ThreadPool,

    particles_pos: Arc<Vec<Vec2d>>,
    particles_vel: Arc<Vec<Vec2d>>,
    particles_type_indexes: Arc<Vec<usize>>,
}

impl SceneLike for MultithreadedSceneV2 {
    async fn new(settings: SceneSettings) -> Self {
        return Self {
            settings: Arc::new(settings),
            pool: ThreadPool::new(THREAD_COUNT),
            particle_types: Arc::new(ParticleTypeManager::new(settings.particle_types_count)),

            particles_pos: Arc::new(vec![]),
            particles_vel: Arc::new(vec![]),
            particles_type_indexes: Arc::new(vec![]),
        };
    }

    fn init(&mut self) {
        let random_source = &mut rng();
        self.particles_pos = Arc::new(
            (0..self.settings.particle_count)
                .map(|_| {
                    [
                        random_source.random_range(0.0..(self.settings.screen_size[0] as f64)),
                        random_source.random_range(0.0..(self.settings.screen_size[0] as f64)),
                    ]
                })
                .collect(),
        );
        self.particles_vel = Arc::new(
            (0..self.settings.particle_count)
                .map(|_| [0.0, 0.0])
                .collect(),
        );
        self.particles_type_indexes = Arc::new(
            (0..self.settings.particle_count)
                .map(|_| random_source.random_range(0..self.settings.particle_types_count))
                .collect(),
        );
    }

    async fn update(&mut self) {
        let particles_per_job = self.particles_pos.len() / THREAD_COUNT;
        let particles_mutexes = (0..THREAD_COUNT)
            .map(|job_index| {
                let particles_vel = Arc::clone(&self.particles_vel);
                let particles_pos = Arc::clone(&self.particles_pos);
                let particles_type_indexes = Arc::clone(&self.particles_type_indexes);

                let particle_types = Arc::clone(&self.particle_types);
                let settings = Arc::clone(&self.settings);
                let screen_size = [
                    settings.screen_size[0] as f64,
                    settings.screen_size[1] as f64,
                ];

                let new_particles_vel_chunk =
                    Arc::new(Mutex::new(Vec::<Vec2d>::with_capacity(particles_per_job)));
                let new_particles_pos_chunk =
                    Arc::new(Mutex::new(Vec::<Vec2d>::with_capacity(particles_per_job)));

                self.pool.execute({
                    let new_particles_vel_chunk = Arc::clone(&new_particles_vel_chunk);
                    let new_particles_pos_chunk = Arc::clone(&new_particles_pos_chunk);
                    move || {
                        let start_i = job_index * particles_per_job;
                        let end_i = start_i + particles_per_job;
                        let mut velocities = (start_i..end_i)
                            .map(|i| {
                                let p_pos = particles_pos[i];
                                let p_type = particles_type_indexes[i];
                                let p_vel = particles_vel[i];
                                let mut total_force = (0..particles_vel.len())
                                    .filter(|j| i != *j)
                                    .fold([0.0, 0.0], |force_acc, j| {
                                        let mut force_acc = force_acc;
                                        let p2_type = particles_type_indexes[j];
                                        let p2_pos = particles_pos[j];
                                        let mut direction: Vec2d = p2_pos;
                                        sub(&mut direction, &p_pos);
                                        if direction[0] > 0.5 * screen_size[0] {
                                            direction[0] -= screen_size[0];
                                        }
                                        if direction[0] < -0.5 * screen_size[0] {
                                            direction[0] += screen_size[0];
                                        }
                                        if direction[1] > 0.5 * screen_size[1] {
                                            direction[1] -= screen_size[1];
                                        }
                                        if direction[1] < -0.5 * screen_size[1] {
                                            direction[1] += screen_size[1];
                                        }
                                        let distance = len(&direction);
                                        normalize(&mut direction);
                                        let p_min_distance =
                                            particle_types.get_min_distance(p_type, p2_type);
                                        if distance < p_min_distance {
                                            let mut force = direction;
                                            mul_scalar(
                                                &mut force,
                                                (particle_types.get_forces(p_type, p2_type).abs())
                                                    * -6.0,
                                            );
                                            mul_scalar(
                                                &mut force,
                                                remap(distance, 0.0, p_min_distance, 1.1, 0.0),
                                            );
                                            mul_scalar(&mut force, K);
                                            add(&mut force_acc, &force);
                                        }
                                        let p_radii_distance =
                                            particle_types.get_radii(p_type, p2_type);
                                        if distance < p_radii_distance {
                                            let mut force = direction;
                                            mul_scalar(
                                                &mut force,
                                                particle_types.get_forces(p_type, p2_type),
                                            );
                                            mul_scalar(
                                                &mut force,
                                                remap(distance, 0.0, p_radii_distance, 1.0, 0.0),
                                            );
                                            mul_scalar(&mut force, K);
                                            add(&mut force_acc, &force);
                                        }
                                        return force_acc;
                                    });

                                let mass = particle_types.get_particle_mass(p_type);
                                div_scalar(&mut total_force, mass);
                                let mut next_p_vel = p_vel;
                                add(&mut next_p_vel, &total_force);
                                mul_scalar(
                                    &mut next_p_vel,
                                    particle_types.get_particle_drag(p_type),
                                );
                                return next_p_vel;
                            })
                            .collect::<Vec<Vec2d>>();

                        let mut positions = (start_i..end_i)
                            .map(|i| {
                                let p_pos = particles_pos[i];
                                let mut next_p_pos = p_pos;
                                add(&mut next_p_pos, &velocities[i - start_i]);
                                next_p_pos[0] = (next_p_pos[0] + screen_size[0]) % screen_size[0];
                                next_p_pos[1] = (next_p_pos[1] + screen_size[1]) % screen_size[1];
                                return next_p_pos;
                            })
                            .collect::<Vec<Vec2d>>();

                        let mut new_particles_pos_chunk = new_particles_pos_chunk.lock().unwrap();
                        new_particles_pos_chunk.append(&mut positions);
                        let mut new_particles_vel_chunk = new_particles_vel_chunk.lock().unwrap();
                        new_particles_vel_chunk.append(&mut velocities);
                    }
                });

                return (new_particles_vel_chunk, new_particles_pos_chunk);
            })
            .collect::<Vec<_>>();
        self.pool.join();
        let new_particles = particles_mutexes
            .iter()
            .map(|(vel_mutex, pos_mutex)| {
                return (
                    vel_mutex
                        .lock()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect::<Vec<Vec2d>>(),
                    pos_mutex
                        .lock()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect::<Vec<Vec2d>>(),
                );
            })
            .collect::<Vec<(Vec<_>, Vec<_>)>>();
        self.particles_vel = Arc::new(
            new_particles
                .iter()
                .flat_map(|(vel, _)| vel.clone())
                .collect::<Vec<Vec2d>>(),
        );
        self.particles_pos = Arc::new(
            new_particles
                .into_iter()
                .flat_map(|(_, pos)| pos)
                .collect::<Vec<Vec2d>>(),
        );
    }

    fn get_particles(&self) -> Arc<Vec<Particle>> {
        let particles = (0..self.settings.particle_count)
            .map(|i| Particle {
                pos: self.particles_pos[i],
                vel: self.particles_vel[i],
                type_index: self.particles_type_indexes[i],
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
