use crate::gl_helper::model::Model;
use crate::game::{MovingAlien, ROW_SIZE, SCALE, Update, GROUND, GRID_WIDTH, Render, ALIEN_RADIUS};
use cgmath::{Vector3, Matrix4, Deg};
use rand::Rng;
use crate::{gl};

pub const SCALE_GRID: f32 = 0.030;
const TOUCH_GROUND: f32 = 0.02;
const MOVE_DOWN_TARGET: f32 = 0.12;

pub struct AlienArmy {
    model: Model,
    moving_down: i32,
    pub all_aliens: Vec<MovingAlien>,
    add_x: f32,
    add_z: f32,
    difficult: f32,
    march_down: f32,
    all_move_down: bool,
    lap: i32,
}

impl AlienArmy {
    pub fn new(gl: &gl::Gl) -> AlienArmy {
        let model = Model::new(gl, "resources/models/anotheralien.obj","resources/models/anotheralien.png");
        AlienArmy {
            model,
            moving_down: 0,
            all_aliens: Vec::new(),
            add_x: 1.0,
            add_z: 0.0,
            difficult: 0.03,
            march_down: 0.3,
            all_move_down: false,
            lap: 0,
        }
    }
    fn create_new_army(&mut self) {
        //self.all_aliens.clear();
        self.march_down = 0.3;
        for i in (0..self.all_aliens.len()).rev() {
            if self.all_aliens.get(i).unwrap().landed == false  {
                self.all_aliens.remove(i);
            }
        }
        for row in 0..ROW_SIZE {
            for col in 0..ROW_SIZE {
                let x: f32 = row as f32 - ROW_SIZE as f32 / 2.0;
                let z: f32 = col as f32 - ROW_SIZE as f32 / 2.0;
                let y = self.march_down;
                let position = Vector3::new(x as f32 * SCALE, y, z as f32 * SCALE);
                let alien = MovingAlien::new(&self.model, position, ALIEN_RADIUS, 0.002);
                self.all_aliens.push(alien);
            }
        }
    }
    pub fn update(&mut self, delta: f32) {
        let down = self.move_down_picker(delta);

        let mut max_x: f32 = 0.0;
        let mut min_x: f32 = 0.0;
        let mut max_z: f32 = 0.0;
        let mut min_z: f32 = 0.0;

        let mut last_y: f32 = 0.0;
        for alien in &mut self.all_aliens {
            alien.update(delta);
            if alien.drop_down == true && alien.movement_collision.been_hit == false && alien.landed == false {
                AlienArmy::move_it_down(down, alien, true)
            }
            if self.all_move_down {
                if alien.drop_down == false && alien.landed == false {
                    AlienArmy::move_it_down(down, alien, false);
                    last_y = alien.movement_collision.position.y;
                }
            } else {
                if alien.drop_down == false && alien.landed == false {
                    alien.movement_collision.moved = true;
                    alien.movement_collision.position.x = alien.movement_collision.position.x - self.add_x * delta * SCALE * self.difficult;
                    alien.movement_collision.position.z = alien.movement_collision.position.z - self.add_z * delta * SCALE * self.difficult;
                    alien.movement_collision.position.y = self.march_down;
                    alien.model_instance.matrix = Matrix4::<f32>::from_translation(alien.movement_collision.position);

                    if alien.movement_collision.position.x > max_x { max_x = alien.movement_collision.position.x }
                    if alien.movement_collision.position.x < min_x { min_x = alien.movement_collision.position.x }
                    if alien.movement_collision.position.z > max_z { max_z = alien.movement_collision.position.z }
                    if alien.movement_collision.position.z < min_z { min_z = alien.movement_collision.position.z }
                }
            }
        }
        if self.add_z == 0.0 && max_x > GRID_WIDTH as f32 * SCALE_GRID {
            self.add_x = 0.0;
            self.add_z = -1.0;
            self.lap = self.lap + 1;
        } else if self.add_x != -1.0 && min_z < -GRID_WIDTH as f32 * SCALE_GRID {
            self.add_x = -1.0;
            self.add_z = 0.0;
            self.lap = self.lap + 1;
        } else if self.add_z != 1.0 && min_x < -GRID_WIDTH as f32 * SCALE_GRID {
            self.add_x = 0.0;
            self.add_z = 1.0;
            self.lap = self.lap + 1;
        } else if self.add_x != 1.0 && max_z > GRID_WIDTH as f32 * SCALE_GRID {
            self.add_x = 1.0;
            self.add_z = 0.0;
            self.lap = self.lap + 1;
        }

        if last_y <= self.march_down {
            self.all_move_down = false;
        }

        if self.lap >= 4 {
            self.lap = 0;
            self.all_move_down = true;
            self.march_down = self.march_down - MOVE_DOWN_TARGET;
        }
    }

    pub fn create_new_army_if_needed(&mut self) -> (bool, usize) {

        let landed = self.all_aliens.iter().filter(|a|a.landed).count();
        let total = self.all_aliens.len();
        if total == 0 || total == landed {
            self.create_new_army();
            return (true,landed);
        }
        return (false,0);
    }

    fn move_it_down(down: f32, alien: &mut MovingAlien,spin:bool) {
        alien.movement_collision.position.y = alien.movement_collision.position.y - down;
        alien.spin = alien.spin + 1.0;
        if spin {
            alien.model_instance.matrix = Matrix4::<f32>::from_translation(alien.movement_collision.position) *
                Matrix4::<f32>::from_angle_y(Deg(alien.spin));
        } else {
            alien.model_instance.matrix = Matrix4::<f32>::from_translation(alien.movement_collision.position) ;
        }
        if alien.movement_collision.position.y - TOUCH_GROUND < GROUND {
            alien.landed = true;
        }
    }

    fn move_down_picker(&mut self, delta: f32) -> f32 {
        let down = 0.01 * delta;
        self.moving_down = 0;
        for alien in &mut self.all_aliens {
            alien.movement_collision.moved = false;
            if alien.drop_down == true && alien.landed == false {
                self.moving_down = self.moving_down + 1;
                alien.movement_collision.moved = true;
            }
        }
        let mut rng = rand::thread_rng();
        let start_drop_maybe = rng.gen_range(0, 100);
        if self.moving_down == 0 && self.all_aliens.len() > 0 && start_drop_maybe > 75 {
            let mut start_from = rng.gen_range(0, self.all_aliens.len());
            let mut total = self.all_aliens.len();
            while total > 0 {
                let alien = self.all_aliens.get_mut(start_from).unwrap();
                if alien.drop_down == false && alien.landed == false {
                    alien.drop_down = true;
                    break;
                }
                total = total - 1;
                start_from = start_from + 1;
                if start_from >= self.all_aliens.len() {
                    start_from = 0;
                }
            }
        }
        down
    }
    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        for alien in &mut self.all_aliens {
            alien.model_instance.render(gl, &view, &projection);
        }
    }
}