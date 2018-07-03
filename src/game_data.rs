use std;
use find_folder;
use serde_json;
use path::*;

use storage::Storage;
use sprite::SpriteData;
use texture::Texture;
use shader::Shader;

fn load_file(filename: &str) -> String {
    use std::io::Read;
    let mut file = std::fs::File::open(filename)
        .expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    contents
}

fn save_file(filename: &str, content: &[u8]) {
    use std::io::Write;
    let mut f = std::fs::File::create(filename).unwrap();
    f.write_all(content);
}

trait LoadableResource {
    fn load_from_path(path: &str) -> Self;
}

pub struct GameData {
    pub sprites: Storage<SpriteData>,
    pub textures: Storage<Texture>,
    pub shaders: Storage<Shader>,
}

impl GameData {
    pub fn create_new() -> Self {
        let sprites = Storage::new(16);
        let textures = Storage::new(16);
        let shaders = Storage::new(16);

        let game_data = GameData {
            sprites, textures, shaders
        };

        game_data.save();
        game_data
    }

    pub fn save(&self) {
        save_file(&storage_path("sprites.json"),
                  serde_json::to_string_pretty(&self.sprites).unwrap().as_bytes());
        save_file(&storage_path("textures.json"),
                  serde_json::to_string_pretty(&self.textures).unwrap().as_bytes());
        save_file(&storage_path("shaders.json"),
                  serde_json::to_string_pretty(&self.shaders).unwrap().as_bytes());
    }

    pub fn from_file() -> Self {
        let sprite_data = load_file(&storage_path("sprites.json"));
        let texture_data = load_file(&storage_path("textures.json"));
        let shaders_data = load_file(&storage_path("shaders.json"));

        let mut shaders: Storage<Shader> = serde_json::from_str(&shaders_data).unwrap();
        let mut textures: Storage<Texture> = serde_json::from_str(&texture_data).unwrap();
        let sprites: Storage<SpriteData> = serde_json::from_str(&sprite_data).unwrap();

        shaders.iterate_mut(|s| { s.compile(); });
        textures.iterate_mut(|t| { t.load(); });

        GameData {
            sprites, textures, shaders
        }
    }
}
