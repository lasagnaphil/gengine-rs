use super::storage::Resource;
use super::shader::Shader;
use super::texture::Texture;
use super::canvas::SpriteData;
use super::canvas::TileMap;

impl Resource for Shader {
    fn tid() -> u16 { 1 }
}

impl Resource for Texture {
    fn tid() -> u16 { 2 }
}

impl Resource for SpriteData {
    fn tid() -> u16 { 3 }
}

impl Resource for TileMap {
    fn tid() -> u16 { 4 }
}
