use std::ffi::CString;
use std::ptr;
use std::str;

use std::fs::File;
use std::io::prelude::*;

use gl;
use gl::types::*;

use cgmath::{Matrix4, Vector3};
use cgmath::prelude::*;

pub struct Shader {
    vertex_path: String,
    fragment_path: String,
    program: GLuint
}

fn to_cstring(source: &str) -> CString {
    CString::new(source.as_bytes()).unwrap()
}

fn compile_shader(shader_type: GLenum, source: &str) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        
        gl::ShaderSource(shader, 1, &to_cstring(source).as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::new();
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ShaderInfoLog not valid utf8"));
        }
        return shader;
    }
}

impl Shader {
    pub fn compile(vertex_path: &str, fragment_path: &str) -> Self {
        let program = unsafe { gl::CreateProgram() };

        let mut vertex_file = File::open(vertex_path)
            .expect(&format!("Vertex shader {} not found", vertex_path));
        let mut vertex_code = String::new();
        vertex_file.read_to_string(&mut vertex_code)
            .expect(&format!("Something went wrong reading the vertex shader {}", vertex_path));
        let mut fragment_file = File::open(fragment_path)
            .expect(&format!("Fragment shader {} not found", fragment_path));
        let mut fragment_code = String::new();
        fragment_file.read_to_string(&mut fragment_code)
            .expect(&format!("Something went wrong reading the fragment shader {}", fragment_path));
        
        unsafe {
            let vertex_shader_id = compile_shader(gl::VERTEX_SHADER, &vertex_code);
            let fragment_shader_id = compile_shader(gl::FRAGMENT_SHADER, &fragment_code);
            gl::AttachShader(program, vertex_shader_id);
            gl::AttachShader(program, fragment_shader_id);
            gl::LinkProgram(program);
            let mut success = 0;
            let mut info_log = Vec::new();
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                gl::GetProgramInfoLog(program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
                panic!("{}", str::from_utf8(info_log.as_slice()).ok().expect("PrograminfoLog not valid utf8"));
            }
            gl::DeleteShader(vertex_shader_id);
            gl::DeleteShader(fragment_shader_id);
        }

        Shader {
            vertex_path: vertex_path.to_string(),
            fragment_path: fragment_path.to_string(),
            program: program,
        }
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.program, to_cstring(name).as_ptr()), value as GLint);
        }
    }
    

    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.program, to_cstring(name).as_ptr()), value as GLint);
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(gl::GetUniformLocation(self.program, to_cstring(name).as_ptr()), value as GLfloat);
        }
    }

    pub fn set_mat4(&self, name: &str, value: Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(gl::GetUniformLocation(self.program, to_cstring(name).as_ptr()), 1, gl::FALSE, value.as_ptr());
        }
    }

    pub fn set_vec3(&self, name: &str, value: Vector3<f32>) {
        unsafe {
            gl::Uniform3fv(gl::GetUniformLocation(self.program, to_cstring(name).as_ptr()), 1, value.as_ptr());
        }
    }

}
