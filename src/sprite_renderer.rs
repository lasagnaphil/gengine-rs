use std::mem;
use std::os::raw::c_void;

use storage::{Storage, ResourceID};
use shader::Shader;
use texture::Texture;
use canvas::SpriteData;

use gl;
use gl::types::*;
use cgmath;
use cgmath::{Vector2, Vector3, Matrix4, One};

pub struct SpriteRenderer<'a> {
    shaders: &'a Storage<Shader>,
    textures: &'a Storage<Texture>,
    sprite_shader: ResourceID<Shader>,
    quad_vao: GLuint
}

impl<'a> SpriteRenderer<'a> {
    pub fn new(shaders: &'a Storage<Shader>, textures: &'a Storage<Texture>) -> Self {
        let mut vbo = 0;
        let mut quad_vao = 0;
        let vertices : [f32; 24] = [
            0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 0.0,

            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of::<[f32; 24]>() as GLsizeiptr, vertices.as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::BindVertexArray(quad_vao);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLint, 0 as *mut c_void);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLint, (2 * mem::size_of::<f32>()) as *mut c_void);
            gl::EnableVertexAttribArray(1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        let (_, sprite_shader_ref) = shaders.get_by_name("sprite.shader").unwrap();

        SpriteRenderer {
            shaders: shaders,
            textures: textures,
            sprite_shader: sprite_shader_ref,
            quad_vao: quad_vao
        }
    }

    pub fn draw_texture(&self, texture_id: ResourceID<Texture>, pos: Vector2<f32>, size: Vector2<f32>, rotate: GLfloat, color: Vector3<f32>) {
        self.draw_texture_with_shader(self.sprite_shader, texture_id, pos, size, rotate, color);
    }

    pub fn draw_texture_with_shader(&self, shader_id: ResourceID<Shader>, texture_id: ResourceID<Texture>, pos: Vector2<f32>, size: Vector2<f32>, rotate: GLfloat, color: Vector3<f32>) {
        let shader = self.shaders.get(shader_id);

        let mut model = Matrix4::<f32>::one();
        model = model * Matrix4::from_translation(Vector3::new(pos.x, pos.y, 0.0));

        model = model * Matrix4::from_translation(Vector3::new(0.5 * size.x, 0.5 * size.y, 0.0));
        model = model * Matrix4::from_angle_z(cgmath::Deg(rotate));
        model = model * Matrix4::from_translation(Vector3::new(-0.5 * size.x, -0.5 * size.y, 0.0));

        model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);

        shader.set_mat4("model", model);
        shader.set_vec3("spriteColor", color);

        let texture = self.textures.get(texture_id);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            texture.bind();

            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}
