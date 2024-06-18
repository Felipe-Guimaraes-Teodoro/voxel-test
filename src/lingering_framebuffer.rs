use std::{ffi::c_void, mem, ptr};

use gl::{types::{GLfloat, GLint, GLsizei, GLsizeiptr, GLvoid}, *};

use crate::{cstr, mesh::{Mesh, Vertex}, shader::Shader};

pub struct LingeringFramebuffer {
    rect_vao: u32,
    pub fbo: u32,
    pub texture: u32,
    rbo: u32,
}

//todo: add motion blur
impl LingeringFramebuffer {
    pub unsafe fn new(width: f32, height: f32, resolution_x: i32, resolution_y: i32, shader: &Shader) -> Self {
        let quad_vertices: [GLfloat; 24] = [
            // positions          // texCoords
            0.0, height,        0.0, 1.0,
            0.0, 0.0,           0.0, 0.0,
            width, 0.0,         1.0, 0.0,
            0.0, height,        0.0, 1.0,
            width, 0.0,         1.0, 0.0,
            width, height,      1.0, 1.0
        ];

        // screen quad VAO
        let mut rect_vao = 0;
        let mut quad_vbo = 0;
        gl::GenVertexArrays(1, &mut rect_vao);
        gl::GenBuffers(1, &mut quad_vbo);
        gl::BindVertexArray(rect_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quad_vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            quad_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );


        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * std::mem::size_of::<GLfloat>() as GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * std::mem::size_of::<GLfloat>() as GLsizei,
            (2 * std::mem::size_of::<GLfloat>()) as *const c_void,
        );
        let mut fbo = 0;
        gl::GenFramebuffers(1, &mut fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            resolution_x,
            resolution_y,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture,
            0,
        );

        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            resolution_x,
            resolution_y,
        );
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            rbo,
        );

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        Self {
            rect_vao,
            fbo,
            rbo,
            texture,
        }
    }

    pub unsafe fn draw(&self, shader: &Shader, width: f32, height: f32) {
        //shader.uniform_1i(cstr!("screenTex"), 0);
        Disable(DEPTH_TEST);
        shader.use_shader();
        //shader.uniform_1f(cstr!("width"), width);
        //shader.uniform_1f(cstr!("height"), height);
        BindVertexArray(self.rect_vao);
        BindTexture(TEXTURE_2D, self.texture);
        DrawArrays(TRIANGLES, 0, 6);

        BindVertexArray(0);
        BindFramebuffer(FRAMEBUFFER, 0);
    }

    pub unsafe fn go_back_to_sleep(&mut self) {
        DeleteFramebuffers(1, &mut self.fbo);
    }
}