use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use shader::Shader;
use texture::Texture;

#[derive(Serialize, Deserialize)]
pub struct AssetManager {
    shaders: HashMap<String, String>,
    textures: HashMap<String, String>,
    scripts: HashMap<String, String>,
}

impl AssetManager {
    fn new() -> AssetManager {
        AssetManager {
            shaders: HashMap::new(),
            textures: HashMap::new(),
            scripts: HashMap::new()
        }
    }
    /*
    fn load_shader(&self, shader: &mut Shader) {
        let vertex_path = &shader.vertex_path;
        let fragment_path = &shader.fragment_path;
        shader.compile(self.shaders[vertex_path], self.shaders[fragment_path]);
    }

    fn load_texture(&self, texture: &mut Texture) {
        texture.load_from_path(self.textures[texture.path]);
    }
    */
}