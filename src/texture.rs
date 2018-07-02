use gl;
use gl::types::*;
use stb_image::image::Image;
use std::os::raw::c_void;

pub struct Texture {
    id: GLuint,
    pub width: GLint,
    pub height: GLint,
    internal_format: GLint,
    image_format: GLuint,

    wrap_s: GLint,
    wrap_t: GLint,
    filter_min: GLint,
    filter_max: GLint,

    path: String,

    data: Vec<u8>
}

pub struct TextureBuilder {
    width: GLint,
    height: GLint,
    internal_format: GLint,
    image_format: GLuint,

    wrap_s: GLint,
    wrap_t: GLint,
    filter_min: GLint,
    filter_max: GLint,

    data: Vec<u8>
}

impl TextureBuilder {
    pub fn new() -> Self {
        TextureBuilder {
            width: 0, height: 0,
            internal_format: gl::RGB as GLint, image_format: gl::RGB,
            wrap_s: gl::REPEAT as GLint, wrap_t: gl::REPEAT as GLint,
            filter_min: gl::LINEAR as GLint, filter_max: gl::LINEAR as GLint,
            data: Vec::new()
        }
    }

    pub fn image(mut self, image: Image<u8>) -> Self {
        self.data = image.data;
        self.width = image.width as GLint;
        self.height = image.height as GLint;
        self
    }

    pub fn internal_format(mut self, internal_format: GLuint) -> Self {
        self.internal_format = internal_format as GLint;
        self
    }

    pub fn image_format(mut self, image_format: GLuint) -> Self {
        self.image_format = image_format;
        self
    }

    pub fn build(mut self) -> Texture {
        let mut id = 0;

        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, self.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, self.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, self.filter_min);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, self.filter_max);

            gl::TexImage2D(gl::TEXTURE_2D, 0, self.internal_format, self.width, self.height, 0, self.image_format, gl::UNSIGNED_BYTE, self.data.as_mut_ptr() as *mut c_void);
        }

        Texture {
            id: id,
            width: self.width, height: self.height,
            internal_format: self.internal_format, image_format: self.image_format,
            wrap_s: self.wrap_s, wrap_t: self.wrap_t,
            filter_min: self.filter_min, filter_max: self.filter_max,
            path: String::new(), // TODO
            data: self.data
        }
    }
}

impl Texture {
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}
