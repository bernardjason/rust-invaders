use crate::gl_helper::model::Model;
use cgmath::{Matrix4, vec3};
use crate::gl_helper::gl_matrix4;
use std::ptr;

use crate::gl;
use crate::game::Render;

#[derive(Clone)]
pub struct ModelInstance {
    model: Model,
    pub(crate) matrix: Matrix4<f32>,
    pub(crate) scale: f32,
}


impl ModelInstance {
    pub fn new(model:Model, scale: f32) -> ModelInstance {
        ModelInstance {
            model,
            matrix: Matrix4::from_translation(vec3(0.0,0.0,0.0)),
            //position,
            scale,
        }
    }
}

impl Render for ModelInstance {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>) {

        //self.matrix = self.matrix * Matrix4::<f32>::from_angle_y(Deg(1.0));
        let matrix = self.matrix * Matrix4::from_scale(self.scale);

        for sub_model in &self.model.sub_models {
            unsafe {
                gl.UseProgram(self.model.our_shader);
                gl.ActiveTexture(gl::TEXTURE0);
                gl.BindTexture(gl::TEXTURE_2D, sub_model.texture);
                gl.BindVertexArray(sub_model.vao);

                gl_matrix4(gl, self.model.our_shader, matrix, "transform");
                gl_matrix4(gl, self.model.our_shader, *view, "view");
                gl_matrix4(gl, self.model.our_shader, *projection, "projection");

                gl.DrawElements(gl::TRIANGLES, sub_model.indices_len as i32, gl::UNSIGNED_INT, ptr::null());
                gl.BindVertexArray(0);
            }
        }
    }
}