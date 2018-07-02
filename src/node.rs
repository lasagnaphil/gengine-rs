use storage::{Storage, ResourceID};
use sprite::SpriteData;
use sprite_renderer::SpriteRenderer;
use cgmath::Vector2;
use canvas::TileMap;

pub trait Updateable {
    fn update(&self, dt: f32);
}

pub trait Drawable {
    fn draw(&self, renderer: &SpriteRenderer);
}

pub struct Sprite {
    pos: Vector2<f32>,
    sprite: ResourceID<SpriteData>
}

impl Drawable for Sprite {
    fn draw(&self, renderer: &SpriteRenderer) {
        renderer.draw_sprite_simple(self.sprite, self.pos, None);
    }
}

pub struct Player {
    sprite: Sprite,
    vel: vel,
}

impl Drawable for Sprite {
    fn draw(&self, renderer: &SpriteRenderer) {
        sprite.draw(renderer);
    }
}

pub struct TileMapNode {
    tilemap: TileMap,
}

impl Updateable for TileMapNode {
    fn update(&self, dt: f32) {
        self.tilemap.update();
    }
}

struct GameWorld {
    handles: Vec<Handle>,
    sprites: Storage<Sprite>,
}
