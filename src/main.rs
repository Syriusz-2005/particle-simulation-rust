extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

mod constants;
mod multithreaded_scene;
mod multithreaded_scene_v2;
mod particle_type;
mod scene_like;
mod vector;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    constants::BENCHMARK_RUNS, multithreaded_scene::MultithreadedScene,
    multithreaded_scene_v2::MultithreadedSceneV2, scene_like::SceneLike,
};
use glutin_window::GlutinWindow as Window;
use graphics::math::Vec2d;
use opengl_graphics::GlGraphics;
use piston::{EventSettings, Events, PressEvent, RenderEvent, WindowSettings};

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

const SCREEN_SIZE: [u32; 2] = [2320, 1280];

fn main() {
    let mut scene = MultithreadedSceneV2::new(SceneSettings {
        screen_size: SCREEN_SIZE,
        particle_count: 4400,
        particle_types_count: 7,
    });
    scene.init();
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    for _ in 0..BENCHMARK_RUNS {
        scene.update();
    }
    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let diff = end - start;
    println!("[Bench] ({}x) {}ms", BENCHMARK_RUNS, diff.as_millis());
    println!(
        "[Bench] Average update time {}ms",
        diff.as_secs_f32() * 1000.0 / BENCHMARK_RUNS as f32
    );
}

#[allow(dead_code)]
fn display(scene: &mut impl SceneLike) {
    let mut window: Window = WindowSettings::new("Simulation window", SCREEN_SIZE)
        .exit_on_esc(true)
        .build()
        .unwrap();
    use graphics::*;
    let mut gl = GlGraphics::new(glutin_window::OpenGL::V3_2);
    let mut events = Events::new(EventSettings::new());
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
                if i == 100 {
                    println!("Average update time {}ms", diff_sum.as_secs_f32() * 10.0);
                }
                for particle in &*scene.get_particles() {
                    let color = &scene.get_particle_color(particle.type_index);
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
