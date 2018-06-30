use std;
use std::os::raw::c_void;
use std::mem;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::marker::PhantomData;

use arrayvec::ArrayVec;
use big_array::BigArray;

use toml;
use serde_json;
use find_folder;
use cgmath::{Vector3, Matrix4, One};
use gl;
use gl::types::*;
use serde::ser::{Serialize, Serializer, SerializeTuple, SerializeSeq};
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error};

use storage::{Storage, ResourceID};
use shader::Shader;
use texture::Texture;

pub struct SpriteBounds {
    x: u32, y: u32,
    w: u32, h: u32,
    ox: u32, oy: u32
}

impl SpriteBounds {
    pub fn new(x: u32, y: u32, w: u32, h: u32, ox: u32, oy: u32) -> Self {
        SpriteBounds { x, y, w, h, ox, oy }
    }
}

impl Serialize for SpriteBounds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        (self.x, self.y, self.w, self.h, self.ox, self.oy).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SpriteBounds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        Deserialize::deserialize(deserializer)
            .map(|(x, y, w, h, ox, oy)| SpriteBounds { x, y, w, h, ox, oy })
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpriteData {
    pub name: String,
    pub texture: ResourceID<Texture>,
    pub rect: SpriteBounds
}

impl SpriteData {
    pub fn new(name: String, texture: ResourceID<Texture>, rect: SpriteBounds) -> Self {
        SpriteData { name, texture, rect }
    }
    fn get_uvs(&self, tex_w: u32, tex_h: u32) -> [f32; 4] {
        let x1 = self.rect.x as f32 / tex_w as f32;
        let x2 = (self.rect.x + self.rect.w) as f32 / tex_w as f32;
        let y1 = self.rect.y as f32 / tex_h as f32;
        let y2 = (self.rect.y + self.rect.h) as f32 / tex_h as f32;
        [x1, x2, y1, y2]
    }
}

pub struct TileMap {
    name: String,
    sprites: ResourceID<SpriteData>,
    texture: ResourceID<Texture>
}

pub const MAX_LAYERS: usize = 4;
pub const MAX_WIDTH: usize = 64;
pub const MAX_HEIGHT: usize = 64;
pub const SCALE: f32 = 64.0;

#[derive(Serialize, Deserialize)]
struct TileMapLayerData(
    #[serde(with = "BigArray")]
    [ResourceID<SpriteData>; MAX_WIDTH * MAX_HEIGHT]
);

big_array! { 4096, }

#[derive(Serialize, Deserialize)]
struct CanvasData {
    name: String,
    num_tiles_x: u32,
    num_tiles_y: u32,
    tile_width: u32,
    tile_height: u32,
    num_layers: usize,
    textures: ArrayVec<[ResourceID<Texture>; MAX_LAYERS]>,
    // Note: We use Vec instead of ArrayVec because of array serializing issue
    data: Vec<ArrayVec<[ResourceID<SpriteData>; MAX_WIDTH * MAX_HEIGHT]>>,
}

pub struct Canvas<'a> {
    num_tiles_x: u32,
    num_tiles_y: u32,
    tile_width: u32,
    tile_height: u32,
    num_layers: usize,
    canvas_width: u32,
    canvas_height: u32,

    layer_index_to_texture: ArrayVec<[ResourceID<Texture>; MAX_LAYERS]>,
    tiles: Vec<ArrayVec<[ResourceID<SpriteData>; MAX_WIDTH * MAX_HEIGHT]>>,
    vertices: [f32; 8*MAX_WIDTH*MAX_HEIGHT],
    uvs: [[f32; 8*MAX_WIDTH*MAX_HEIGHT]; MAX_LAYERS],
    indices: [u32; 6*MAX_WIDTH*MAX_HEIGHT],

    sprites: &'a Storage<SpriteData>,
    textures: &'a Storage<Texture>,
    shaders: &'a Storage<Shader>,

    default_shader: ResourceID<Shader>,

    pos_vbos: ArrayVec<[GLuint; MAX_LAYERS]>,
    uv_vbos: ArrayVec<[GLuint; MAX_LAYERS]>,
    vaos: ArrayVec<[GLuint; MAX_LAYERS]>,
    ebo: GLuint
}

impl<'a> Canvas<'a> {
    pub fn from_file(sprites: &'a Storage<SpriteData>,
                     textures: &'a Storage<Texture>,
                     shaders: &'a Storage<Shader>,
                     default_shader: ResourceID<Shader>,
                     filename: &str) -> Self {

        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();

        let mut file = File::open(assets.join(filename).to_str().unwrap())
            .expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");
        let canvas_data = serde_json::from_str::<CanvasData>(&contents).unwrap();

        let num_tiles_x = canvas_data.num_tiles_x;
        let num_tiles_y = canvas_data.num_tiles_y;
        let tile_width = canvas_data.tile_width;
        let tile_height = canvas_data.tile_height;
        let num_layers = canvas_data.num_layers;

        assert!(num_tiles_x <= MAX_WIDTH as u32);
        assert!(num_tiles_y <= MAX_HEIGHT as u32);

        // Create vertices array
        let mut vertices: [f32; 8*MAX_WIDTH*MAX_HEIGHT] =
            unsafe { std::mem::uninitialized() };

        for i in 0..MAX_WIDTH*MAX_HEIGHT {
            vertices[8*i] = (i % MAX_WIDTH) as f32 * SCALE;
            vertices[8*i + 1] = (i / MAX_WIDTH) as f32 * SCALE;
            vertices[8*i + 2] = ((i % MAX_WIDTH) + 1) as f32 * SCALE;
            vertices[8*i + 3] = (i / MAX_WIDTH) as f32 * SCALE;
            vertices[8*i + 4] = ((i % MAX_WIDTH) + 1) as f32 * SCALE;
            vertices[8*i + 5] = ((i / MAX_WIDTH) + 1) as f32 * SCALE;
            vertices[8*i + 6] = (i % MAX_WIDTH) as f32 * SCALE;
            vertices[8*i + 7] = ((i / MAX_WIDTH) + 1) as f32 * SCALE;
        }

        let mut uvs: [[f32; 8*MAX_WIDTH*MAX_HEIGHT]; MAX_LAYERS] =
            unsafe { std::mem::uninitialized() };

        for layer_idx in 0..num_layers {
            let texture_id = canvas_data.textures[layer_idx as usize];
            let texture = textures.get(texture_id);
            for i in 0..MAX_WIDTH*MAX_HEIGHT{
                let sprite_id = canvas_data.data[layer_idx as usize][i];
                let sprite = sprites.get(sprite_id);
                let sprite_uvs = sprite.get_uvs(texture.width as u32, texture.height as u32);

                uvs[layer_idx][8*i] = sprite_uvs[0];
                uvs[layer_idx][8*i + 1] = sprite_uvs[2];
                uvs[layer_idx][8*i + 2] = sprite_uvs[1];
                uvs[layer_idx][8*i + 3] = sprite_uvs[2];
                uvs[layer_idx][8*i + 4] = sprite_uvs[1];
                uvs[layer_idx][8*i + 5] = sprite_uvs[3];
                uvs[layer_idx][8*i + 6] = sprite_uvs[0];
                uvs[layer_idx][8*i + 7] = sprite_uvs[3];
            }
        }

        let mut indices: [GLuint; 6 * MAX_WIDTH*MAX_HEIGHT] = unsafe { std::mem::uninitialized() };
        for i in 0..MAX_WIDTH * MAX_HEIGHT {
            indices[6*i] = (4*i) as u32;
            indices[6*i+1] = (4*i + 1) as u32;
            indices[6*i+2] = (4*i + 2) as u32;
            indices[6*i+3] = (4*i) as u32;
            indices[6*i+4] = (4*i + 2) as u32;
            indices[6*i+5] = (4*i + 3) as u32;
        }

        let mut pos_vbos = ArrayVec::<[GLuint; MAX_LAYERS]>::new();
        let mut uv_vbos = ArrayVec::<[GLuint; MAX_LAYERS]>::new();
        let mut vaos = ArrayVec::<[GLuint; MAX_LAYERS]>::new();
        let mut ebo = 0;

        unsafe {
            // Create buffers
            gl::GenBuffers(1, &mut ebo);
            for i in 0..num_layers {
                let mut vao = 0;
                let mut pos_vbo = 0;
                let mut uv_vbo = 0;
                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut pos_vbo);
                gl::GenBuffers(1, &mut uv_vbo);
                vaos.push(vao);
                pos_vbos.push(pos_vbo);
                uv_vbos.push(uv_vbo);
            }

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (6*MAX_WIDTH*MAX_HEIGHT*mem::size_of::<f32>()) as GLsizeiptr,
                           indices.as_ptr() as *const c_void,
                           gl::STATIC_DRAW);

            // Bind vbos and ebo to vaos
            for i in 0..num_layers {
                gl::BindVertexArray(vaos[i]);

                gl::BindBuffer(gl::ARRAY_BUFFER, pos_vbos[i]);
                gl::BufferData(gl::ARRAY_BUFFER,
                               (8*MAX_WIDTH*MAX_HEIGHT*mem::size_of::<f32>()) as GLsizeiptr,
                               vertices.as_ptr() as *const c_void,
                               gl::STATIC_DRAW);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE,
                                        0 as GLint, 0 as *const c_void);
                gl::EnableVertexAttribArray(0);

                gl::BindBuffer(gl::ARRAY_BUFFER, uv_vbos[i]);
                gl::BufferData(gl::ARRAY_BUFFER,
                               (8*MAX_WIDTH*MAX_HEIGHT*mem::size_of::<f32>()) as GLsizeiptr,
                               uvs.as_ptr() as *const c_void,
                               gl::STATIC_DRAW);

                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE,
                                        0 as GLint, 0 as *const c_void);
                gl::EnableVertexAttribArray(1);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            }

        }

        Canvas {
            num_tiles_x,
            num_tiles_y,
            tile_width,
            tile_height,
            num_layers,
            canvas_width: tile_width * num_tiles_x,
            canvas_height: tile_height * num_tiles_y,

            layer_index_to_texture: canvas_data.textures,
            tiles: canvas_data.data,

            vertices,
            indices,
            uvs,

            sprites,
            textures,
            shaders,
            default_shader,

            pos_vbos,
            uv_vbos,
            vaos,
            ebo
        }
    }

    pub fn draw(&self) {
        let shader = self.shaders.get(self.default_shader);
        shader.use_shader();
        shader.set_mat4("model", Matrix4::<f32>::one());
        shader.set_vec3("spriteColor", Vector3::<f32>::new(1.0, 1.0, 1.0));

        for layer_idx in 0..self.num_layers {
            let texture_id = self.layer_index_to_texture[layer_idx];
            let texture = self.textures.get(texture_id);

            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                texture.bind();

                gl::BindVertexArray(self.vaos[layer_idx]);
                gl::DrawElements(gl::TRIANGLES, (6 * MAX_WIDTH * MAX_HEIGHT) as GLint, gl::UNSIGNED_INT, 0 as *const c_void);
                gl::BindVertexArray(0);
            }
        }
    }
}

impl<'a> Drop for Canvas<'a> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.num_layers {
                gl::DeleteVertexArrays(1, &self.vaos[i]);
                gl::DeleteBuffers(1, &self.pos_vbos[i]);
                gl::DeleteBuffers(1, &self.uv_vbos[i]);
            }
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}