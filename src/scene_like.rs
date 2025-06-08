use crate::{Particle, SceneSettings};

pub trait SceneLike {
    fn new(settings: SceneSettings) -> Self;
    fn init(&mut self);
    fn update(&mut self);
    fn get_particles(&self) -> &Vec<Particle>;
    fn new_world(&mut self);
}
