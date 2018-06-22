#![allow(dead_code)]

#[macro_use]
extern crate derivative;

extern crate gl;
extern crate sdl2;
extern crate cgmath;
extern crate stb_image;
extern crate find_folder;

mod shader;
mod texture;
mod storage;
mod renderer;
mod canvas;

use shader::Shader;
use texture::{Texture, TextureBuilder};
use storage::Storage;
use renderer::Renderer;
use canvas::{Canvas, TileMap};

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

    let mut textures = Storage::<Texture>::new(16);
    let mut shaders = Storage::<Shader>::new(16);
    let mut tilemaps = Storage::<TileMap>::new(4);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();


    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let shader = Shader::compile(
        assets.join("sprite.vert").to_str().unwrap(),
        assets.join("sprite.frag").to_str().unwrap()
    );
    let _ = shaders.insert("sprite.shader", shader);

    let test_image = match image::load(assets.join("awesomeface.png").to_str().unwrap()) {
        image::LoadResult::ImageU8(image) => image,
        image::LoadResult::ImageF32(_) => { panic!("Image loaded as f32"); }
        image::LoadResult::Error(s) => { panic!("Error while loading image: {}", s); }
    };
    let test_tex = TextureBuilder::new()
        .image_format(gl::RGBA)
        .internal_format(gl::RGBA)
        .image(test_image)
        .build();

    let (_, test_tex_ref) = textures.insert("awesomeface.texture", test_tex);

    let renderer = Renderer::new(&mut shaders, &mut textures);

    let canvas = Canvas::new(&tilemaps, 16, 16, 64, 64);

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

        renderer.draw_texture(
            test_tex_ref, 
            Vector2::new(200.0, 200.0), 
            Vector2::new(300.0, 400.0),
            45.0,
            Vector3::new(0.0, 1.0, 0.0)
        );

        window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
