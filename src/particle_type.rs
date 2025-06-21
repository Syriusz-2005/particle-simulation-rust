use graphics::{types::Color, Colored};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug)]
struct ParticleType {
    color: Color,
    mass: f64,
    drag: f64,
}

pub struct ParticleTypeManager {
    particle_types: Vec<ParticleType>,
    forces: Vec<Vec<f64>>,
    min_distances: Vec<Vec<f64>>,
    radii: Vec<Vec<f64>>,
}

impl ParticleTypeManager {
    pub fn new(particle_types_count: usize) -> ParticleTypeManager {
        let mut random_source = ChaCha8Rng::seed_from_u64(1);
        let particle_types: Vec<ParticleType> = (0..particle_types_count)
            .map(|i| {
                let color = Color::from([1.0, 0.0, 0.0, 1.0])
                    .hue_deg(i as f32 / particle_types_count as f32 * 360.0);
                return ParticleType {
                    color,
                    mass: random_source.random_range(0.3..2.0),
                    drag: random_source.random_range(0.9..1.0),
                };
            })
            .collect();
        let forces: Vec<Vec<f64>> = (0..particle_types_count)
            .map(|_i| {
                (0..particle_types_count)
                    .map(|_j| random_source.random_range(-1.0..1.0))
                    .collect()
            })
            .collect();
        let min_distances: Vec<Vec<f64>> = (0..particle_types_count)
            .map(|_i| {
                (0..particle_types_count)
                    .map(|_j| random_source.random_range(10.0..30.0))
                    .collect()
            })
            .collect();
        let radii: Vec<Vec<f64>> = (0..particle_types_count)
            .map(|_i| {
                (0..particle_types_count)
                    .map(|_j| random_source.random_range(90.0..350.0))
                    .collect()
            })
            .collect();
        let manager = ParticleTypeManager {
            particle_types,
            forces,
            min_distances,
            radii,
        };
        manager.show();
        return manager;
    }

    fn show(&self) {
        println!("=== Current settings ===");
        println!("Particles: {:?}", self.particle_types);
        println!("========================");
    }

    #[inline(always)]
    pub fn get_particle_drag(&self, type_index: usize) -> f64 {
        return self.particle_types[type_index].drag;
    }

    #[inline(always)]
    pub fn get_particle_mass(&self, type_index: usize) -> f64 {
        return self.particle_types[type_index].mass;
    }

    #[inline(always)]
    pub fn get_particle_color(&self, type_index: usize) -> Color {
        return self.particle_types[type_index].color;
    }

    #[inline(always)]
    pub fn get_min_distance(&self, type_a: usize, type_b: usize) -> f64 {
        return self.min_distances[type_a][type_b];
    }

    #[inline(always)]
    pub fn get_forces(&self, type_a: usize, type_b: usize) -> f64 {
        return self.forces[type_a][type_b];
    }

    #[inline(always)]
    pub fn get_radii(&self, type_a: usize, type_b: usize) -> f64 {
        return self.radii[type_a][type_b];
    }
}
