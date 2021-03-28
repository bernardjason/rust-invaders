use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::{Window, GLContext};

use crate::{gl, WIDTH, HEIGHT};
use emscripten_main_loop::MainLoopEvent;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::cube::Cube;
use cgmath::{Vector3, Point3, Matrix4, perspective, Deg, vec3, MetricSpace, Zero};
use crate::flying_camera::{Flying_Camera, PERSPECTIVE_ANGLE};
use crate::flying_camera::Flying_Camera_Movement::{UP, DOWN, LEFT, RIGHT, FORWARD};
use crate::gl_helper::model::Model;
use crate::gl_helper::instance_model::ModelInstance;
use crate::alien_army::AlienArmy;
use std::time::Instant;
use crate::bullets::Bullets;
use crate::explosion::Explosions;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::start_javascript_play_sound;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::start_game;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::end_game;
use crate::handle_javascript::{write_stats_data};
use std::ffi::CString;

pub const GRID_WIDTH: i32 = 48;
pub const SCALE: f32 = 0.25;
pub const GROUND: f32 = 0.0;
pub const ROW_SIZE: usize = 5;
const BULLET_RADIUS: f32 = 0.04;
pub const ALIEN_RADIUS: f32 = 0.03;

pub struct Runtime {
    loaded: bool,
    now: Instant,
    last: u128,
    last_rate: f32,
    sdl: Sdl,
    _video: VideoSubsystem,
    window: Window,
    _gl_context: GLContext,
    pub gl: std::rc::Rc<gl::Gl>,
    pub camera: Flying_Camera,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub forward: bool,
    pub fire: bool,
    last_fire_countdown:i32,
    pub ground: Ground,
    pub alien_model: Model,
    alien_army: AlienArmy,
    bullets: Bullets,
    explosions: Explosions,
    exploding: AlienArmy,
    score:i32,
    level:i32,
}


static mut GLOBAL_ID: u128 = 0;

fn get_next_id() -> u128 {
    unsafe {
        GLOBAL_ID = GLOBAL_ID + 1;
        GLOBAL_ID
    }
}

#[derive(Clone)]
pub struct MovementAndCollision {
    pub id: u128,
    pub radius: f32,
    pub position: Vector3<f32>,
    pub been_hit: bool,
    pub moved: bool,
}

impl Default for MovementAndCollision {
    fn default() -> Self {
        MovementAndCollision {
            id: get_next_id(),
            radius: 0.0,
            position: Vector3::zero(),
            been_hit: false,
            moved: false,
        }
    }
}


impl MovementAndCollision {
    pub fn new(radius: f32, position: Vector3<f32>) -> MovementAndCollision {
        MovementAndCollision {
            radius,
            position,
            been_hit: false,
            moved: false,
            ..MovementAndCollision::default()
        }
    }
    pub fn hit_other(&self, other: &MovementAndCollision) -> bool {
        self.position.distance(other.position) < self.radius
    }
}

pub(crate) trait Render {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>);
}

pub(crate) trait Update {
    fn update(&mut self, delta: f32);
}


pub struct Ground {
    cube: Cube
}

impl Ground {
    pub fn new(gl: &gl::Gl) -> Ground {
        let cube = Cube::new(&gl, "resources/ground.png", vec3(2.0, 0.001, 2.0), 40.0);
        Ground {
            cube
        }
    }
}

impl Update for Ground {
    fn update(&mut self, _delta: f32) {}
}

impl Render for Ground {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let matrix = Matrix4::<f32>::from_translation(Vector3::zero());
        self.cube.render(gl, &matrix, view, projection);
    }
}

#[derive(Clone)]
pub struct MovingAlien {
    pub(crate) model_instance: ModelInstance,
    pub(crate) movement_collision: MovementAndCollision,
    pub(crate) spin: f32,
    pub(crate) drop_down: bool,
    pub(crate) landed: bool,
    pub time_to_live: i32,
}

impl PartialEq for MovingAlien {
    fn eq(&self, other: &Self) -> bool {
        self.movement_collision.id == other.movement_collision.id
    }
}

impl MovingAlien {
    pub fn new(model: &Model, position: Vector3<f32>, radius: f32, scale: f32) -> MovingAlien {
        //let alien_model =  Model::new(&gl,"resources/models/alien.obj");

        let alien = ModelInstance::new(model.clone(), scale);
        MovingAlien {
            movement_collision: MovementAndCollision::new(radius, position),
            model_instance: alien,
            spin: 0.0,
            drop_down: false,
            landed: false,
            time_to_live: 0,
        }
    }
}

impl Update for MovingAlien {
    fn update(&mut self, _delta: f32) {}
}

impl Runtime {
    pub(crate) fn new() -> Runtime {
        let sdl = sdl2::init().unwrap();

        let video = sdl.video().unwrap();

        #[cfg(not(target_os = "emscripten"))]
            let context_params = (sdl2::video::GLProfile::Core, 3, 0);
        #[cfg(target_os = "emscripten")]
            let context_params = (sdl2::video::GLProfile::GLES, 3, 0);

        video.gl_attr().set_context_profile(context_params.0);
        video.gl_attr().set_context_major_version(context_params.1);
        video.gl_attr().set_context_minor_version(context_params.2);

        // Create a window
        let window = video
            .window("rust-invaders", WIDTH, HEIGHT)
            .resizable()
            .opengl()
            .position_centered()
            .build().unwrap();


        let gl_context = window.gl_create_context().unwrap();
        let gl_orig: std::rc::Rc<gl::Gl> = std::rc::Rc::new(gl::Gl::load_with(|s| { video.gl_get_proc_address(s) as *const _ }));

        let gl = std::rc::Rc::clone(&gl_orig);

        let ground = Ground::new(&gl);

        let camera = Flying_Camera {
            Position: Point3::new(0.0, 0.05, 0.0),
            ..Flying_Camera::default()
        };

        let alien_model = Model::new(&gl, "resources/models/anotheralien.obj","resources/models/anotheralien.png");

        unsafe { gl.Enable(gl::BLEND); }

        let runtime = Runtime {
            loaded: false,
            now: Instant::now(),
            last: 0,
            last_rate: 0.0,
            sdl,
            _video: video,
            window,
            _gl_context: gl_context,
            gl: gl_orig,
            camera,
            ground,
            left: false,
            right: false,
            up: false,
            down: false,
            forward: false,
            fire: false,
            alien_model,
            alien_army: AlienArmy::new(&gl),
            bullets: Bullets::new(&gl),
            explosions: Explosions::new(&gl),
            exploding: AlienArmy::new(&gl),
            last_fire_countdown: 0,
            score:0,
            level:0,
        };
        runtime
    }
}

impl emscripten_main_loop::MainLoop for Runtime {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        if self.loaded == false {
            self.loaded = true;
            #[cfg(target_os = "emscripten")]
                unsafe {
                start_game();
            }
        }
        let projection: Matrix4<f32> =
            perspective(Deg(PERSPECTIVE_ANGLE), WIDTH as f32 / HEIGHT as f32, 0.01, 40.0);
        let view = self.camera.GetViewMatrix();

        unsafe {
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }

        let start = self.now.elapsed().as_millis();
        let diff = start - self.last;
        self.last = start;
        let new_rate = diff as f32 / 128.0;
        let mut delta = self.last_rate + new_rate / 128.0;
        self.last_rate = new_rate;

        // just for browser, big drop in rate on first load
        if delta > 5.0 {
            delta = 1.0;
        }

        self.ground.render(&self.gl, &view, &projection);
        self.ground.update(delta);

        self.alien_army.update(delta);
        self.alien_army.render(&self.gl, &view, &projection);
        self.bullets.update(delta);
        self.bullets.render(&self.gl, &view, &projection);

        self.exploding.update(delta);
        self.exploding.render(&self.gl, &view, &projection);
        self.explosions.update(delta);
        self.explosions.render(&self.gl, &view, &projection);
        self.window.gl_swap_window();

        self.camera.save_position();
        if self.up { self.camera.processKeyboard(UP, delta); }
        if self.down { self.camera.processKeyboard(DOWN, delta); }
        if self.left { self.camera.processKeyboard(LEFT, delta); }
        if self.right { self.camera.processKeyboard(RIGHT, delta); }
        if self.forward { self.camera.processKeyboard(FORWARD, delta); }

        self.last_fire_countdown = self.last_fire_countdown -1;
        if self.last_fire_countdown < -10000 {
            self.last_fire_countdown = -1;
        }
        if self.fire && self.last_fire_countdown <= 0 {
            self.last_fire_countdown = 30;
            let direction = vec3(self.camera.Front.x, self.camera.Front.y, self.camera.Front.z);
            let here = vec3(self.camera.Position.x, self.camera.Position.y, self.camera.Position.z);
            self.bullets.fire(here, direction, delta, BULLET_RADIUS);
        }


        let end_status = self.handle_keyboard();

        let camera_collision = MovementAndCollision::new(0.07, vec3(self.camera.Position.x, self.camera.Position.y, self.camera.Position.z));


        let mut alien_remove: Vec<usize> = Vec::new();
        let mut alien_collide: Vec<usize> = Vec::new();
        for i in (0..self.alien_army.all_aliens.len()).rev() {
            let s = self.alien_army.all_aliens.get(i).unwrap();
            if camera_collision.hit_other(&s.movement_collision) {
                self.camera.rollback();
            }


            for bullet_index in (0..self.bullets.instances.len()).rev() {
                let b = self.bullets.instances.get(bullet_index).unwrap();
                if b.collision.hit_other(&s.movement_collision) {
                    self.bullets.instances.remove(bullet_index);
                    alien_remove.push(i);
                    self.score = self.score +1;
                }
            }
            for other_aliens in (0..self.alien_army.all_aliens.len()).rev() {
                let o = self.alien_army.all_aliens.get(other_aliens).unwrap();
                if o.movement_collision.id != s.movement_collision.id {
                    if o.movement_collision.hit_other(&s.movement_collision) {
                        alien_remove.push(i);
                        alien_collide.push(other_aliens);
                    }
                }
            }
        }
        for i in alien_collide {
            let a = self.alien_army.all_aliens.get_mut(i).unwrap();
            a.drop_down = true;
        }
        for i in alien_remove {
            let mut alien = self.alien_army.all_aliens.get(i).unwrap().clone();
            alien.time_to_live = 60;
            let position = alien.movement_collision.position.clone();
            self.exploding.all_aliens.push(alien);

            self.alien_army.all_aliens.remove(i);
            self.explosions.create(position);
            #[cfg(target_os = "emscripten")]
                unsafe {
                start_javascript_play_sound(1);
            }
        }
        for i in (0..self.exploding.all_aliens.len()).rev() {
            let mut alien = self.exploding.all_aliens.get_mut(i).unwrap();
            alien.time_to_live = alien.time_to_live - 1;
            alien.model_instance.scale = alien.model_instance.scale * 0.9;
            if alien.time_to_live <= 0 {
                self.exploding.all_aliens.remove(i);
            }
        }

        let (create,landed) = self.alien_army.create_new_army_if_needed() ;
        if  create {
            self.score = self.score - landed as i32;
            self.level = self.level +1;
        }

        let mut list: Vec<String> = Vec::new();
        list.push(format!("level {} score {}",self.level, self.score));
        let update:String =list.join("\n") ;

        #[cfg(not(target_os = "emscripten"))]
        if self.last_fire_countdown % 60 == 0 {
            println!("{}",update);
        }

        write_stats_data(CString::new(update).to_owned().unwrap().as_ptr());

        match end_status {
            MainLoopEvent::Terminate => {
                #[cfg(target_os = "emscripten")]
                    unsafe {
                    end_game();
                }
            }
            MainLoopEvent::Continue => {}
        }

        end_status
    }
}

impl Runtime {
    fn handle_keyboard(&mut self) -> MainLoopEvent {
        let mut return_status = emscripten_main_loop::MainLoopEvent::Continue;
        let mut events = self.sdl.event_pump().unwrap();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return_status = emscripten_main_loop::MainLoopEvent::Terminate;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    self.left = true;
                    self.right = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    self.right = true;
                    self.left = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    self.up = true;
                    self.down = false
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    self.down = true;
                    self.up = false
                }
                Event::KeyDown { keycode: Some(Keycode::LShift), .. } => {
                    self.forward = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    self.fire = true;
                }
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => { self.left = false; }
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => { self.right = false; }
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => { self.up = false }
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => { self.down = false }
                Event::KeyUp { keycode: Some(Keycode::LShift), .. } => { self.forward = false }
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => { self.fire = false }

                _ => {}
            }
        }
        return_status
    }
}
