use std::mem;
use std::os::raw::c_void;

use storage::{Storage, ResourceID};
use shader::Shader;
use texture::Texture;
use sprite::{SpriteBounds, SpriteData};

use gl;
use gl::types::*;
use cgmath;
use cgmath::{Vector2, Vector3, Matrix4, One};

pub struct SpriteRenderer<'a> {
    shaders: &'a Storage<Shader>,
    textures: &'a Storage<Texture>,
    sprites: &'a Storage<SpriteData>,
    sprite_shader: ResourceID<Shader>,

    vao: GLuint,
    vbo: GLuint,
}

impl<'a> SpriteRenderer<'a> {
    pub fn new(shaders: &'a Storage<Shader>,
               textures: &'a Storage<Texture>,
               sprites: &'a Storage<SpriteData>) -> Self {
        let mut vao = 0;
        let mut vbo = 0;

        let vertices : [f32; 24] = [
            0.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 1.0,

            0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 1.0,
            0.0, 1.0, 0.0, 1.0
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of::<[f32; 24]>() as GLsizeiptr,
                           vertices.as_ptr() as *const c_void,
                           gl::DYNAMIC_DRAW);

            gl::BindVertexArray(vao);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLint, 0 as *mut c_void);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLint, (2 * mem::size_of::<f32>()) as *mut c_void);
            gl::EnableVertexAttribArray(1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        let (_, sprite_shader) = shaders.get_by_name("sprite.shader").unwrap();

        SpriteRenderer {
            shaders,
            textures,
            sprites,
            sprite_shader,
            vao, vbo
        }
    }

    pub fn draw_sprite_with_shader(&self,
                                   shader_id: ResourceID<Shader>,
                                   sprite_id: ResourceID<SpriteData>,
                                   pos: Vector2<f32>, scale: Vector2<f32>, rotate: f32,
                                   color: Vector3<f32>) {
        let shader = self.shaders.get(shader_id);
        let sprite = self.sprites.get(sprite_id);
        let texture = self.textures.get(sprite.texture);
        let size = Vector2::new(scale.x * texture.width as f32, scale.y * texture.height as f32);

        // change uv coordinate buffer before drawing sprite
        let uvs = sprite.get_uvs(texture.width as u32, texture.height as u32);
        let vertices: [f32; 24] = [
            0.0, 0.0, uvs[0], uvs[2],
            1.0, 0.0, uvs[1], uvs[2],
            1.0, 1.0, uvs[1], uvs[3],
            0.0, 0.0, uvs[0], uvs[2],
            1.0, 1.0, uvs[1], uvs[3],
            0.0, 1.0, uvs[0], uvs[3]
        ];

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
            texture.bind();

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER,
                              0 as GLintptr,
                              24 * mem::size_of::<f32>() as isize,
                              vertices.as_ptr() as *const c_void);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }

    // Draw sprite with default shader
    pub fn draw_sprite(&self,
                       sprite_id: ResourceID<SpriteData>,
                       pos: Vector2<f32>, scale: Vector2<f32>, rotate: f32,
                       color: Vector3<f32>) {

        self.draw_sprite_with_shader(self.sprite_shader, sprite_id, pos, scale, rotate, color);
    }

    pub fn draw_sprite_simple(&self,
                              sprite_id: ResourceID<SpriteData>,
                              pos: Vector2<f32>, scale: Vector2<f32>) {
        self.draw_sprite(sprite_id, pos, scale, 0.0, Vector3::new(0.0, 0.0, 0.0));
    }
}
