use crate::cube::Cube;
use cgmath::{vec3, Vector3, Matrix4, };
use crate::game::{Update, Render, GROUND, MovementAndCollision};
use crate::{gl, get_next_id};

const SPEED: f32 = 0.08;//375;

pub struct Bullets {
    cube: Cube,
    pub instances: Vec<BulletInstance>,

}

pub struct BulletInstance {
    pub id: u128,
    pub collision: MovementAndCollision,
    direction: Vector3<f32>,
    ticks: i32,
}

impl Bullets {
    pub fn new(gl: &gl::Gl) -> Bullets {
        let cube = Cube::new(&gl, "resources/fire.png", vec3(0.001, 0.001, 0.001), 1.0);
        Bullets {
            cube,
            instances: Vec::new(),
        }
    }
    pub fn fire(&mut self, mut position: Vector3<f32>, direction: Vector3<f32>, delta: f32, radius: f32) {
        position += direction * delta * SPEED;

        let instance = BulletInstance {
            id: get_next_id(),
            direction,
            collision: MovementAndCollision::new(radius, position),
            ticks: 300,
        };
        self.instances.push(instance);
    }
}

impl Update for Bullets {
    fn update(&mut self, delta: f32) {
        for i in (0..self.instances.len()).rev() {
            let change = self.instances.get_mut(i).unwrap();

            change.collision.position += change.direction * delta * SPEED;

            if change.collision.position.y <= GROUND {
                change.collision.been_hit = true;
            }
            change.ticks = change.ticks -1;
            if change.ticks <= 0  {
                self.instances.remove(i);
            }
            //change.matrix = Matrix4::<f32>::from_translation(change.collision.position);
        }
    }
}

impl Render for Bullets {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        for i in &self.instances {
            let matrix = Matrix4::<f32>::from_translation(i.collision.position);

            self.cube.render(gl, &matrix, view, projection);
        }
    }
}
