use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer};
use storage::ResourceID;
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
    pub fn get_uvs(&self, tex_w: u32, tex_h: u32) -> [f32; 4] {
        let x1 = self.rect.x as f32 / tex_w as f32;
        let x2 = (self.rect.x + self.rect.w) as f32 / tex_w as f32;
        let y1 = self.rect.y as f32 / tex_h as f32;
        let y2 = (self.rect.y + self.rect.h) as f32 / tex_h as f32;
        [x1, x2, y1, y2]
    }
}