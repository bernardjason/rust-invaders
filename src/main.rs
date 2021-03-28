use crate::game::Runtime;
#[macro_use]
extern crate lazy_static;

mod game;
mod gl;
mod cube;
mod gl_helper;
mod flying_camera;
mod alien_army;
mod bullets;
mod explosion;
mod handle_javascript;

pub const WIDTH:u32=800;
pub const HEIGHT:u32=600;
pub const SCALE_TO_SCREEN: f32 = 0.043;


static mut GLOBAL_ID: u128 = 1;
pub fn get_next_id() -> u128 {
    unsafe {
        let next = GLOBAL_ID;
        GLOBAL_ID = GLOBAL_ID + 1;
        next
    }
}

fn main() {
    let runtime = Runtime::new();

    emscripten_main_loop::run(runtime);

}
