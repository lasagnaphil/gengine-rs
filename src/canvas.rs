use std::collections::HashMap;
use super::storage::{Storage, ResourceID};

struct SpriteID(u32);

pub struct SpriteBounds {
    x: u32, y: u32,
    w: u32, h: u32,
    ox: u32, oy: u32
}

pub struct SpriteData {
    pub name: String,
    pub texture: ResourceID<Texture2D>,
    pub id: SpriteID,
    pub rect: SpriteBounds
}

pub struct TileMap {
    name: String,
    sprites: ResourceID<SpriteData>,
    texture: ResourceID<Texture2D>
}

pub struct Canvas<'a> {
    num_tiles_x: u32,
    num_tiles_y: u32,
    tile_width: u32,
    tile_height: u32,
    canvas_width: u32,
    canvas_height: u32,

    num_vertices: u32,

    tiles: [ResourceID<SpriteData>; MAX_WIDTH * MAX_HEIGHT],

    sprites: &'a Storage<SpriteData>,

    vbo: GLuint,
    ibo: GLuint,
    vao: GLuint
}

impl<'a> Canvas<'a> {

    pub const MAX_WIDTH: usize = 64;
    pub const MAX_HEIGHT: usize = 64;
    pub const MAX_NUM_VERTICES: usize = MAX_WIDTH * MAX_HEIGHT * 4;

    pub fn new(sprites: &'a Storage<SpriteData>, num_tiles_x: u32, num_tiles_y: u32, tile_width: u32, tile_height: u32, num_sprites: ) -> Self {
        assert!(num_tiles_x <= Self::MAX_WIDTH);
        assert!(num_tiles_y <= Self::MAX_HEIGHT);

        let vertices : [f32; 4 * MAX_NUM_VERTICES] = [

        ]
        Canvas {
            num_tiles_x: 0,
            num_tiles_y: 0,
            tile_width: 0,
            tile_height: 0,
            canvas_width: tile_width,
            canvas_height: tile_height,

            num_vertices: num_tiles_x * num_tiles_y * 6,
            tiles: [ResourceID::null(); MAX_WIDTH * MAX_HEIGHT],

            sprites: sprites,

            vbo: 0,
            ibo: 0,
            vao: 0
        }
    }

    pub fn draw(&self) {
        for j in 0..tile_height {
            for i in 0..tile_width {
                let sprite_id = tiles[j * Self::MAX_WIDTH + i];
                let sprite_data = self.sprites.get(sprite_id);
            }
        }
    }
}
