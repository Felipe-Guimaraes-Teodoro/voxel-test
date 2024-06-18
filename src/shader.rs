use gl::*;
use gl::types::*;

use cgmath::*;

use std::ptr;
use std::ffi::CStr;

#[derive(Clone, Copy, Debug)]
pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(shader_type: GLenum, code: &str) -> Self {
        unsafe {
            let shader = CreateShader(shader_type);
            let cstr = std::ffi::CString::new(code.as_bytes()).unwrap();
            ShaderSource(shader, 1, &cstr.as_ptr(), std::ptr::null());
            CompileShader(shader);
            check_shader_error(shader);

            let id = CreateProgram();
            AttachShader(id, shader);
            LinkProgram(id);
            check_shader_link_error(id);

            DeleteShader(shader);

            Self { id }
        }
    }

    pub fn new_pipeline(vs_code: &str, fs_code: &str) -> Self {
        unsafe {
            let vs = CreateShader(VERTEX_SHADER);
            let cstr = std::ffi::CString::new(vs_code.as_bytes()).unwrap();
            ShaderSource(vs, 1, &cstr.as_ptr(), std::ptr::null());
            CompileShader(vs);
            check_shader_error(vs);

            let fs = CreateShader(FRAGMENT_SHADER);
            let cstr = std::ffi::CString::new(fs_code.as_bytes()).unwrap();
            ShaderSource(fs, 1, &cstr.as_ptr(), std::ptr::null());
            CompileShader(fs);
            check_shader_error(fs);

            let id = CreateProgram();
            AttachShader(id, vs);
            AttachShader(id, fs);
            LinkProgram(id);
            check_shader_link_error(id);

            Self { id }
        }
    }

    pub unsafe fn use_shader(&self) {
        UseProgram(self.id);
    }

    pub unsafe fn stop_shader(&self) {

    }

    pub unsafe fn uniform_1f(&self, name: &CStr, val: f32) {
        Uniform1f(GetUniformLocation(self.id, name.as_ptr()), val);
    }

    pub unsafe fn uniform_1i(&self, name: &CStr, val: i32) {
        Uniform1i(GetUniformLocation(self.id, name.as_ptr()), val);
    }

    pub unsafe fn uniform_mat4fv(&self, name: &CStr, mat: &Matrix4<f32>) {
        UniformMatrix4fv(
            GetUniformLocation(self.id, name.as_ptr()), 
            1, 
            FALSE, 
            mat.as_ptr()
        );
    }

    pub unsafe fn uniform_vec3f(&self, name: &CStr, vec: &Vector3<f32>) {
        Uniform3f(
            GetUniformLocation(self.id, name.as_ptr()),
            vec.x, vec.y, vec.z
        );
    }
}

pub unsafe fn check_shader_error(shader: u32) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(54); // skip the trailing null char
    GetShaderiv(shader, COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        GetShaderInfoLog(
            shader,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!(
            "ERROR::SHADER::COMPILATION::FAILED\n{}",
            std::str::from_utf8(&info_log).unwrap()
        );
    }
}

pub unsafe fn check_shader_link_error(program: u32) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        gl::GetProgramInfoLog(
            program,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!(
            "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
            std::str::from_utf8(&info_log).unwrap()
        );
    }
}

#[macro_export]
macro_rules! cstr{
    ($s: expr) => {
        std::ffi::CString::new($s)
            .expect("conversion failed at cstr!() macro")
            .as_c_str()
    }
}
