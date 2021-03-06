#![allow(dead_code)]

// #[cfg(use_gl_crate)]
// extern crate gl;

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate lazy_static;

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

#[macro_use]
extern crate wren;

#[macro_use] mod big_array;

mod shader;
mod texture;
mod storage;
mod sprite_renderer;
mod canvas;
mod resource_tids;
mod sprite;
mod input_manager;
mod asset_manager;
mod game_data;
mod path;

#[cfg(not(use_gl_crate))]
mod gl;

use shader::Shader;
use texture::{Texture, TextureBuilder};
use storage::{Storage, ResourceID};
use sprite_renderer::SpriteRenderer;
use canvas::Canvas;
use sprite::{SpriteData, SpriteBounds};
use input_manager::{InputManager, Key};
use game_data::GameData;
use path::{asset_path, storage_path};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use stb_image::image;

use cgmath::{Vector2, Vector3};

use std::time::Duration;
use std::collections::HashMap;

/*
struct App<'a> {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    window: sdl2::Window,

    sprite_renderer: SpriteRenderer<'a>,
    event_pump: sdl2::EventPump,
    input_manager: InputManager,

    test_tex_ref: ResourceID<Texture>,
    spritesheet_tex_ref: ResourceID<Texture>,
    sprite_id: ResourceID<SpriteData>,
}
*/

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}


lazy_static! {
    static ref FOREIGN_METHODS: HashMap<&'static str, wren::ForeignMethodFn> = {
        let mut map = HashMap::new();
        map
    };
}

lazy_static! {
    static ref FOREIGN_CLASSES: HashMap<&'static str, wren::ForeignClassMethods> = {
        let mut map = HashMap::new();
        /*
        let mut vec3_class_methods = wren::ForeignClassMethods::new();
        vec3_class_methods.set_allocate_fn(wren_foreign_method_fn!(vec3_allocate));
        vec3_class_methods.set_finalize_fn(wren_finalizer_fn!(vec3_finalize));

        map.insert("vectorVec3", vec3_class_methods);
        */
        map
    };
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

    let ctx = window
        .gl_create_context()
        .unwrap();

    window.gl_make_current(&ctx);

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    /*
    let mut game_data = GameData::create_new();

    // Load shaders
    let mut shader = Shader::new("sprite.vert".to_string(), "sprite.frag".to_string());
    shader.compile();
    let shader_id = game_data.shaders.insert("sprite.shader", shader);

    // Load textures
    let test_tex = TextureBuilder::new()
        .load_file("awesomeface.png")
        .image_format(gl::RGBA)
        .internal_format(gl::RGBA)
        .build();
    let test_tex_ref = game_data.textures.insert("awesomeface.texture", test_tex);

    let spritesheet_tex = TextureBuilder::new()
        .load_file("kenneyrpgpack/Spritesheet/RPGpack_sheet.png")
        .image_format(gl::RGBA)
        .internal_format(gl::RGBA)
        .build();
    let spritesheet_tex_ref = game_data.textures.insert("rpgpack.texture", spritesheet_tex);

    // Load sprites
    for i in 0..9 {
        let name = format!("grass_with_dirt_{}", i+1);
        game_data.sprites.insert(&name, SpriteData {
            name: name.clone(),
            texture: spritesheet_tex_ref,
            rect: SpriteBounds::new((i % 3) * 64, (i / 3) * 64, 64, 64, 0, 0)
        });
    }

    let sprite_id = game_data.sprites.insert("smiley_face.sprite", SpriteData {
        name: "smiley_face".to_string(),
        texture: test_tex_ref,
        rect: SpriteBounds::new(64, 64, 384, 384, 0, 0)
    });

    game_data.save();
    */

    let mut game_data = GameData::from_file();

    let (shader, shader_id) = game_data.shaders.get_by_name("sprite.shader").unwrap();
    let projection_mat = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
    shader.use_shader();
    shader.set_int("image", 0);
    shader.set_mat4("projection", projection_mat);

    let (_, test_tex_ref) = game_data.textures.get_by_name("awesomeface.texture").unwrap();
    let (_, spritesheet_tex_ref) = game_data.textures.get_by_name("rpgpack.texture").unwrap();
    let (_, sprite_id) = game_data.sprites.get_by_name("smiley_face.sprite").unwrap();

    // Load Wren VM
    fn bind_method(_: &mut wren::VM,
                   module: &str,
                   class_name: &str,
                   is_static: bool,
                   signature: &str) -> wren::ForeignMethodFn {
        let full_signature = format!("{}{}{}{}",
                                     module,
                                     class_name,
                                     signature,
                                     if is_static { "s" } else { "" });
        *FOREIGN_METHODS.get::<str>(&full_signature).unwrap_or(&None)
    }

    fn bind_class(_: &mut wren::VM,
                  module: &str,
                  class_name: &str) -> wren::ForeignClassMethods {
        let full_signature = format!("{}{}", module, class_name);
        let methods = FOREIGN_CLASSES.get::<str>(&full_signature);
        if let Some(methods) = methods {
            return *methods;
        }
        panic!("Failed to bind foreign class");
    }

    fn load_module(_: &mut wren::VM, name: &str) -> Option<String> {
        use std::path::Path;
        use std::fs::File;
        use std::io::Read;

        let mut path = Path::new("scripts").join(&name);
        path.set_extension("wren");
        let mut buffer = String::new();
        if File::open(path)
            .map(|mut f| f.read_to_string(&mut buffer))
            .is_ok() {
            Some(buffer)
        } else {
            None
        }
    }

    let mut wren_cfg = wren::Configuration::new();
    wren_cfg.set_bind_foreign_method_fn(wren_bind_foreign_method_fn!(bind_method));
    wren_cfg.set_bind_foreign_class_fn(wren_bind_foreign_class_fn!(bind_class));
    wren_cfg.set_load_module_fn(wren_load_module_fn!(load_module));
    let mut vm = wren::VM::new(wren_cfg);
    // vm.interpret(source);

    let sprite_renderer = SpriteRenderer::new(&game_data.shaders, &game_data.textures, &game_data.sprites);
    let canvas = Canvas::from_file(&game_data.sprites, &game_data.textures, &game_data.shaders, shader_id, "map_test.json");

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut input_mgr = InputManager::new();
    let (mut x, mut y) = (100.0f32, 100.0f32);

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
        input_mgr.update(&event_pump);

        if input_mgr.is_key_pressed(Key::Left) {
            x -= 10.0;
        }
        if input_mgr.is_key_pressed(Key::Right) {
            x += 10.0;
        }
        if input_mgr.is_key_pressed(Key::Up) {
            y -= 10.0;
        }
        if input_mgr.is_key_pressed(Key::Down) {
            y += 10.0;
        }

        // render
        unsafe {
            gl::ClearColor(0.5, 0.5, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        canvas.draw();

        sprite_renderer.draw_sprite(
            sprite_id,
            Vector2::new(x, y),
            Vector2::new(0.25, 0.25),
            0.0,
            Vector3::new(0.0, 1.0, 0.0)
        );

        window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
