extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

mod multithreaded_scene;
mod particle_type;
mod scene_like;
mod vector;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use glutin_window::GlutinWindow as Window;
use graphics::math::Vec2d;
use opengl_graphics::GlGraphics;
use particle_type::ParticleTypeManager;
use piston::{EventSettings, Events, PressEvent, RenderEvent, WindowSettings};
use rand::{rng, Rng};

use vector::{add, len, mul_scalar, normalize, remap, sub};

use crate::{multithreaded_scene::MultithreadedScene, scene_like::SceneLike};

#[derive(Debug, Clone, Copy)]
struct Particle {
    pos: Vec2d,
    vel: Vec2d,
    type_index: usize,
}

impl Particle {
    fn new() -> Particle {
        return Particle {
            pos: Vec2d::default(),
            vel: Vec2d::default(),
            type_index: 0,
        };
    }
}

#[derive(Debug, Clone, Copy)]
struct SceneSettings {
    screen_size: [u32; 2],
    particle_count: usize,
    particle_types_count: usize,
}

fn main() {
    let screen_size = [2320, 1280];
    let mut window: Window = WindowSettings::new("Simulation window", screen_size)
        .exit_on_esc(true)
        .build()
        .unwrap();
    use graphics::*;
    let mut gl = GlGraphics::new(glutin_window::OpenGL::V3_2);
    let mut events = Events::new(EventSettings::new());
    // let mut scene = Scene::new(SceneSettings {
    //     screen_size: screen_size,
    //     particle_count: 1400,
    //     particle_types_count: 5,
    // });
    let mut scene = MultithreadedScene::new(SceneSettings {
        screen_size: screen_size,
        particle_count: 4400,
        particle_types_count: 7,
    });
    scene.init();
    let mut i = 0;
    let mut diff_sum = Duration::new(0, 0);
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.press_args() {
            println!("New world!");
            scene.new_world();
        }
        if let Some(args) = e.render_args() {
            i += 1;
            gl.draw(args.viewport(), |c, gl| {
                clear([0.0; 4], gl);
                let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                scene.update();
                let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                diff_sum += end - start;
                if i % 10 == 0 {
                    println!("Diff {}ms", (end - start).as_millis());
                }
                if i == 100 {
                    println!("Average update time {}ms", diff_sum.as_secs_f32() * 10.0);
                }
                for particle in scene.get_particles() {
                    let color = &scene.particle_types.get_particle_color(particle.type_index);
                    ellipse(
                        *color,
                        rectangle::centered_square(particle.pos[0], particle.pos[1], 3.0),
                        c.transform,
                        gl,
                    );
                }
            });
        }
    }
}
