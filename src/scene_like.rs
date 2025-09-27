use std::sync::Arc;

use crate::{Particle, SceneSettings};

pub trait SceneLike {
    async fn new(settings: SceneSettings) -> Self;
    fn init(&mut self);
    async fn update(&mut self);
    fn get_particles(&self) -> Arc<Vec<Particle>>;
    fn new_world(&mut self);
    fn get_particle_color(&self, type_index: usize) -> [f32; 4];
}
