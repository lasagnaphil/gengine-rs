extern crate gl;
extern crate cgmath;
extern crate find_folder;

use std::mem;
use std::os::raw::c_void;

use storage::{Storage, ResourceID};
use shader::Shader;
use texture::Texture;

use gl::types::*;
use cgmath::{Vector2, Vector3, Matrix4, One};

pub struct Renderer<'a> {
    shaders: &'a mut Storage<Shader>,
    textures: &'a mut Storage<Texture>,
    sprite_shader: ResourceID<Shader>,
    quad_vao: GLuint
}

impl<'a> Renderer<'a> {
    pub fn new(shaders: &'a mut Storage<Shader>, textures: &'a mut Storage<Texture>) -> Self {
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
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLint, 0 as *mut c_void);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        let shader = Shader::compile(
            assets.join("sprite.vert").to_str().unwrap(), 
            assets.join("sprite.frag").to_str().unwrap()
        );
        let sprite_shader_ref = {
            let (sprite_shader, sprite_shader_ref) = shaders.insert("sprite.shader", shader);
            // TODO: move screen width / height settings to separate file
            let projection_mat = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
            sprite_shader.use_shader();
            sprite_shader.set_int("image", 0);
            sprite_shader.set_mat4("projection", projection_mat);
            sprite_shader_ref
        };

        Renderer {
            shaders: shaders,
            textures: textures,
            sprite_shader: sprite_shader_ref,
            quad_vao: quad_vao
        }
    }

    pub fn draw_texture(&self, texture: ResourceID<Texture>, pos: Vector2<f32>, size: Vector2<f32>, rotate: GLfloat, color: Vector3<f32>) {
        let shader = self.shaders.get(self.sprite_shader);
        let mut model = Matrix4::<f32>::one();
        model = model * Matrix4::from_translation(Vector3::new(pos.x, pos.y, 0.0));

        model = model * Matrix4::from_translation(Vector3::new(0.5 * size.x, 0.5 * size.y, 0.0));
        model = model * Matrix4::from_angle_z(cgmath::Deg(rotate));
        model = model * Matrix4::from_translation(Vector3::new(-0.5 * size.x, -0.5 * size.y, 0.0));

        model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);

        shader.set_mat4("model", model);
        shader.set_vec3("spriteColor", color);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            self.textures.get(texture).bind();

            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}
