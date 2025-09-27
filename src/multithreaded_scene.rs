use std::sync::{Arc, Mutex};

use crate::constants::{K, THREAD_COUNT};
use crate::{
    particle_type::ParticleTypeManager,
    scene_like::SceneLike,
    vector::{add, div_scalar, len, mul_scalar, normalize, remap, sub},
    Particle, SceneSettings,
};
use graphics::math::Vec2d;
use rand::{rng, Rng};
use threadpool::{self, ThreadPool};

pub struct MultithreadedScene {
    particles: Arc<Vec<Particle>>,
    settings: Arc<SceneSettings>,
    pub particle_types: Arc<ParticleTypeManager>,
    pool: ThreadPool,
}

impl SceneLike for MultithreadedScene {
    async fn new(settings: SceneSettings) -> Self {
        return MultithreadedScene {
            particles: Arc::new(vec![]),
            settings: Arc::new(settings),
            pool: ThreadPool::new(THREAD_COUNT),
            particle_types: Arc::new(ParticleTypeManager::new(settings.particle_types_count)),
        };
    }

    fn init(&mut self) {
        let random_source = &mut rng();
        self.particles = Arc::new(
            (0..self.settings.particle_count)
                .map(|_| {
                    let mut particle = Particle::new();
                    particle.pos[0] =
                        random_source.random_range(0.0..(self.settings.screen_size[0] as f64));
                    particle.pos[1] =
                        random_source.random_range(0.0..(self.settings.screen_size[1] as f64));
                    particle.type_index =
                        random_source.random_range(0..self.settings.particle_types_count);
                    return particle;
                })
                .collect(),
        );
    }

    async fn update(&mut self) {
        let particles_per_job = self.particles.len() / THREAD_COUNT;
        let particles_mutexes = (0..THREAD_COUNT)
            .map(|job_index| {
                let particles = Arc::clone(&self.particles);
                let particle_types = Arc::clone(&self.particle_types);
                let settings = Arc::clone(&self.settings);
                let screen_size = settings.screen_size;
                let new_particles = Arc::new(Mutex::new(Vec::<Particle>::with_capacity(
                    particles_per_job,
                )));
                let new_particles_in_thread = Arc::clone(&new_particles);
                self.pool.execute(move || {
                    let start_i = job_index * particles_per_job;
                    let end_i = start_i + particles_per_job;
                    // let mut new_particles_local = Vec::<Particle>::with_capacity(particles_per_job);
                    for i in start_i..end_i {
                        let particle = particles[i];
                        let mut total_force: Vec2d = [0.0, 0.0];
                        for j in 0..particles.len() {
                            if i != j {
                                let p = particles[j];
                                let mut direction: Vec2d = p.pos;
                                sub(&mut direction, &particle.pos);
                                if direction[0] > 0.5 * screen_size[0] as f64 {
                                    direction[0] -= screen_size[0] as f64;
                                }
                                if direction[0] < -0.5 * screen_size[0] as f64 {
                                    direction[0] += screen_size[0] as f64;
                                }
                                if direction[1] > 0.5 * screen_size[1] as f64 {
                                    direction[1] -= screen_size[1] as f64;
                                }
                                if direction[1] < -0.5 * screen_size[1] as f64 {
                                    direction[1] += screen_size[1] as f64;
                                }
                                let distance = len(&direction);
                                normalize(&mut direction);
                                if distance
                                    < particle_types
                                        .get_min_distance(particle.type_index, p.type_index)
                                {
                                    let mut force = direction;
                                    mul_scalar(
                                        &mut force,
                                        (particle_types
                                            .get_forces(particle.type_index, p.type_index)
                                            .abs())
                                            * -6.0,
                                    );
                                    mul_scalar(
                                        &mut force,
                                        remap(
                                            distance,
                                            0.0,
                                            particle_types.get_min_distance(
                                                particle.type_index,
                                                p.type_index,
                                            ),
                                            1.1,
                                            0.0,
                                        ),
                                    );
                                    mul_scalar(&mut force, K);
                                    add(&mut total_force, &force);
                                }
                                if distance
                                    < particle_types.get_radii(particle.type_index, p.type_index)
                                {
                                    // apply_forces(
                                    //     &mut total_force,
                                    //     &direction,
                                    //     particle_types
                                    //         .get_forces(particle.type_index, p.type_index),
                                    //     remap(
                                    //         distance,
                                    //         0.0,
                                    //         particle_types
                                    //             .get_radii(particle.type_index, p.type_index),
                                    //         1.0,
                                    //         0.0,
                                    //     ),
                                    // );
                                    let mut force = direction;
                                    mul_scalar(
                                        &mut force,
                                        particle_types
                                            .get_forces(particle.type_index, p.type_index),
                                    );
                                    mul_scalar(
                                        &mut force,
                                        remap(
                                            distance,
                                            0.0,
                                            particle_types
                                                .get_radii(particle.type_index, p.type_index),
                                            1.0,
                                            0.0,
                                        ),
                                    );
                                    mul_scalar(&mut force, K);
                                    add(&mut total_force, &force);
                                }
                            }
                        }
                        let mut new_particle = particle;
                        let mass = particle_types.get_particle_mass(particle.type_index);
                        div_scalar(&mut total_force, mass);
                        add(&mut new_particle.vel, &total_force);
                        add(&mut new_particle.pos, &new_particle.vel);
                        new_particle.pos[0] =
                            (new_particle.pos[0] + screen_size[0] as f64) % screen_size[0] as f64;
                        new_particle.pos[1] =
                            (new_particle.pos[1] + screen_size[1] as f64) % screen_size[1] as f64;
                        mul_scalar(
                            &mut new_particle.vel,
                            particle_types.get_particle_drag(particle.type_index),
                        );
                        new_particles_in_thread.lock().unwrap().push(new_particle);
                    }
                });
                return new_particles;
            })
            .collect::<Vec<_>>();
        self.pool.join();
        let new_particles = particles_mutexes
            .iter()
            .flat_map(|mutex| {
                mutex
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|particle| *particle)
                    .collect::<Vec<Particle>>()
            })
            .collect::<Vec<_>>();
        self.particles = Arc::new(new_particles);
    }

    fn get_particles(&self) -> Arc<Vec<Particle>> {
        return Arc::clone(&self.particles);
    }

    fn new_world(&mut self) {
        self.particle_types =
            Arc::new(ParticleTypeManager::new(self.settings.particle_types_count));
    }

    fn get_particle_color(&self, type_index: usize) -> [f32; 4] {
        return self.particle_types.get_particle_color(type_index);
    }
}
