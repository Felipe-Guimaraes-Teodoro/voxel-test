use cgmath::vec3;
use cgmath::Vector3;
use gl::*;
use gl::types::*;
use noise::core::perlin::perlin_3d;
use noise::core::perlin::perlin_4d;
use noise::permutationtable::PermutationTable;
use noise::Vector4 as NVec4;
use std::mem::*;
use std::ffi::*;
use std::ptr;

use crate::shader::Shader;
use crate::util::rand_betw;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let hasher = PermutationTable::new(0);
        let perlin = perlin_3d(
            noise::Vector3::new(
                x as f64 / 5.0,
                y as f64 / 5.0,
                z as f64 / 5.0
            ), &hasher) * 1.5 + 1.5;
            
        Self {
            position: vec3(x, y, z),
            color: (vec3(x, y, z) / 32.0) * perlin as f32,
        }
    }
}


#[derive(Clone)]
pub struct Mesh {
    pub verts: Vec<Vertex>,
    pub inds: Vec<u32>,
    pub VAO: u32,

    VBO: u32,
    EBO: u32,
}

impl Mesh {
    pub fn new(verts: Vec<Vertex>, inds: Vec<u32>) -> Self {
        let mut mesh = Mesh {
            verts, inds,
            VAO: 0, VBO: 0, EBO: 0,
        };

        unsafe { mesh.setup_mesh() }

        mesh
    }

    unsafe fn setup_mesh(&mut self) {
        GenVertexArrays(1, &mut self.VAO);
        GenBuffers(1, &mut self.VBO);
        GenBuffers(1, &mut self.EBO);

        BindVertexArray(self.VAO);

        BindBuffer(ARRAY_BUFFER, self.VBO);

        let size = (self.verts.len() * size_of::<Vertex>()) as isize;
        let data = &self.verts[0] as *const Vertex as *const c_void;
        BufferData(ARRAY_BUFFER, size, data, STATIC_DRAW);

        BindBuffer(ELEMENT_ARRAY_BUFFER, self.EBO);
        let size = (self.inds.len() * size_of::<u32>()) as isize;
        let data = &self.inds[0] as *const u32 as *const c_void;
        BufferData(ELEMENT_ARRAY_BUFFER, size, data, STATIC_DRAW);

        let size = size_of::<Vertex>() as i32;

        EnableVertexAttribArray(0);
        VertexAttribPointer(0, 3, FLOAT, FALSE, size, offset_of!(Vertex, position) as *const c_void);
        EnableVertexAttribArray(1);
        VertexAttribPointer(1, 3, FLOAT, FALSE, size, offset_of!(Vertex, color) as *const c_void);

        BindVertexArray(0);
    }

    pub unsafe fn draw(&self, shader: &Shader) {
        BindVertexArray(self.VAO);
        shader.use_shader();
        DrawElements(TRIANGLES, self.inds.len() as i32, UNSIGNED_INT, ptr::null());
        BindVertexArray(0);
    }

    pub unsafe fn draw_points(&self, shader: &Shader) {
        BindVertexArray(self.VAO);
        shader.use_shader();
        DrawArrays(POINTS, 0, 1);
        BindVertexArray(0);
    }

    pub unsafe fn destroy(&mut self) {
        DeleteVertexArrays(1, &self.VAO);
        DeleteBuffers(1, &self.VBO);
        DeleteBuffers(1, &self.EBO);
        
        self.VAO = 0;
        self.VBO = 0;
        self.EBO = 0;
    }

    pub unsafe fn update(&mut self, verts: Vec<Vertex>, inds: Vec<u32>) {
        self.verts = verts;
        self.inds = inds;

        BindBuffer(ARRAY_BUFFER, self.VBO);
        let size = (self.verts.len() * size_of::<Vertex>()) as isize;
        let data = &self.verts[0] as *const Vertex as *const c_void;
        BufferSubData(ARRAY_BUFFER, 0, size, data);

        BindBuffer(ELEMENT_ARRAY_BUFFER, self.EBO);
        let size = (self.inds.len() * size_of::<u32>()) as isize;
        let data = &self.inds[0] as *const u32 as *const c_void;
        BufferSubData(ELEMENT_ARRAY_BUFFER, 0, size, data);
    }
}
