use cgmath::{Matrix4, Matrix};
use crate::gl;
use std::ffi::CString;

pub(crate) mod texture;
pub(crate) mod shader;
pub(crate) mod model;
pub mod instance_model;
//pub(crate) mod sprite;
//pub(crate) mod vertex;

pub fn gl_matrix4(gl: &gl::Gl, shader_program:u32,mat4:Matrix4<f32>, name:&str) {
    unsafe {
        let location = gl.GetUniformLocation(shader_program, CString::new(name).unwrap().as_ptr());
        gl.UniformMatrix4fv(
            location,
            1,
            gl::FALSE,
            mat4.as_ptr(),
        );
    }
}

