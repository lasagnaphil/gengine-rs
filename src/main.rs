#![allow(dead_code)]

#[macro_use]
extern crate derivative;

extern crate gl;
extern crate sdl2;
extern crate cgmath;
extern crate stb_image;
extern crate find_folder;

extern crate arrayvec;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate toml;
extern crate serde_json;

#[macro_use] mod big_array;

mod shader;
mod texture;
mod storage;
mod renderer;
mod canvas;
mod resource_tids;

use shader::Shader;
use texture::{Texture, TextureBuilder};
use storage::Storage;
use renderer::Renderer;
use canvas::{Canvas, TileMap, SpriteData, SpriteBounds};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use stb_image::image;

use cgmath::{Vector2, Vector3};

use std::time::Duration;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

fn load_image(filename: &str) -> image::Image<u8> {
    match image::load(filename) {
        image::LoadResult::ImageU8(image) => image,
        image::LoadResult::ImageF32(_) => { panic!("Image loaded as f32"); }
        image::LoadResult::Error(s) => { panic!("Error while loading image: {}", s); }
    }
}

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

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem.window("SDL2 works!", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window
        .gl_create_context()
        .unwrap();

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut sprites = Storage::<SpriteData>::new(16);
    let mut textures = Storage::<Texture>::new(16);
    let mut shaders = Storage::<Shader>::new(16);
    let mut tilemaps = Storage::<TileMap>::new(4);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    // Load shaders
    let shader = Shader::compile(
        assets.join("sprite.vert").to_str().unwrap(),
        assets.join("sprite.frag").to_str().unwrap()
    );
    let projection_mat = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
    shader.use_shader();
    shader.set_int("image", 0);
    shader.set_mat4("projection", projection_mat);
    let shader_id = shaders.insert("sprite.shader", shader);

    // Load textures
    let test_image = load_image(assets.join("awesomeface.png").to_str().unwrap());
    let test_tex = TextureBuilder::new()
        .image_format(gl::RGBA)
        .internal_format(gl::RGBA)
        .image(test_image)
        .build();
    let test_tex_ref = textures.insert("awesomeface.texture", test_tex);

    let spritesheet_image = load_image(assets.join("kenneyrpgpack/Spritesheet/RPGpack_sheet.png").to_str().unwrap());
    let spritesheet_tex = TextureBuilder::new()
        .image_format(gl::RGBA)
        .internal_format(gl::RGBA)
        .image(spritesheet_image)
        .build();
    let spritesheet_tex_ref = textures.insert("rpgpack.texture", spritesheet_tex);

    // Load sprites
    for i in 0..9 {
        let name = format!("grass_with_dirt_{}", i+1);
        sprites.insert(&name, SpriteData {
            name: name.clone(),
            texture: spritesheet_tex_ref,
            rect: SpriteBounds::new((i % 3) * 64, (i / 3) * 64, 64, 64, 0, 0)
        });
    }

    let canvas = Canvas::from_file(&sprites, &textures, &shaders, shader_id, "map_test.json");

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        // update

        // render
        unsafe {
            gl::ClearColor(0.5, 0.5, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        canvas.draw();
        /*
        renderer.draw_texture(
            test_tex_ref, 
            Vector2::new(200.0, 200.0), 
            Vector2::new(300.0, 400.0),
            45.0,
            Vector3::new(0.0, 1.0, 0.0)
        );
        */

        window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
