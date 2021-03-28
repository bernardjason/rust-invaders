use crate::cube::Cube;
use cgmath::{vec3, Vector3, Matrix4};
use crate::game::{Update, Render, };
use crate::{gl, };
use rand::Rng;


pub struct Explosions {
    cube: Cube,
    pub instances: Vec<ExplosionInstance>,

}

pub struct ExplosionInstance {
    pub position: Vector3<f32>,
    direction: Vector3<f32>,
    speed: f32,
    ticks: i32,
}

impl Explosions {
    pub fn new(gl: &gl::Gl) -> Explosions {
        let cube = Cube::new(&gl, "resources/fire.png", vec3(0.005, 0.005, 0.005), 1.0);
        Explosions {
            cube,
            instances: Vec::new(),
        }
    }
    pub fn create(&mut self, position: Vector3<f32>) {
        let mut rng = rand::thread_rng();

        for _i in 0..10 {
            let direction: Vector3<f32> = vec3(
                rng.gen_range(-0.1, 0.1),
                rng.gen_range(0.01, 0.15),
                rng.gen_range(-0.1, 0.1));

            let instance = ExplosionInstance {
                direction,
                position,
                speed: rng.gen_range(0.05, 0.2),
                ticks: rng.gen_range(50,150),
            };
            self.instances.push(instance);
        }
    }
}

impl Update for Explosions {
    fn update(&mut self, delta: f32) {
        for i in (0..self.instances.len()).rev() {
            let change = self.instances.get_mut(i).unwrap();

            change.position += change.direction * delta * change.speed;

            change.ticks = change.ticks - 1;
            if change.ticks <= 0 {
                self.instances.remove(i);
            }
            //change.matrix = Matrix4::<f32>::from_translation(change.collision.position);
        }
    }
}

impl Render for Explosions {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        for i in &self.instances {
            let matrix = Matrix4::<f32>::from_translation(i.position);

            self.cube.render(gl, &matrix, view, projection);
        }
    }
}
