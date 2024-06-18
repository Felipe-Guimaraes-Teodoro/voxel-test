use crate::{mesh::Vertex, util::rand_betw};

use cgmath::Vector3;
use noise::{core::perlin::{perlin_2d, perlin_3d, perlin_4d}, permutationtable::PermutationTable, Perlin, Vector4 as NVec4};
use tokio::sync::watch;

#[derive(Clone)]
pub struct Voxel {
    id: u8,
}

impl Voxel {
    pub fn air() -> Self {
        Voxel { id: 0 }
    }

    pub fn ground() -> Self {
        Voxel { id: 1 }
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}

#[derive(Clone)]
pub struct Chunk {
    voxels: Vec<Voxel>,
    pos: Vector3<f32>,
    creation_instant: std::time::Instant,
    is_mesh: bool,
}

pub const CHUNK_SIZE: usize = 24;

impl Chunk {
    pub fn new(pos: Vector3<f32>) -> Self {
        let upd_pos = pos * CHUNK_SIZE as f32 * 0.5;

        let creation_instant = std::time::Instant::now();
        let hasher = PermutationTable::new(0);
        let mut voxels = vec![Voxel::air(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        for i in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            let x = i / (CHUNK_SIZE * CHUNK_SIZE);
            let y = (i % (CHUNK_SIZE * CHUNK_SIZE)) / CHUNK_SIZE;
            let z = i % CHUNK_SIZE;

            let perlin = perlin_4d(
                NVec4::new(
                    (x as f64 / 14.0) + upd_pos.x as f64,
                    (y as f64 / 14.0) + upd_pos.y as f64,
                    (z as f64 / 14.0) + upd_pos.z as f64,
                    1.0, // rand_betw(0.0, 0.5)
                ), &hasher) * 32.0;
            if 10 < perlin as usize {
                voxels[i] = Voxel::ground();
            }
        }

        Self {
            pos,
            voxels,
            creation_instant,
            is_mesh: false,
        }
    }

    pub fn destroy_voxel(&mut self, pos: Vector3<f32>) {
        let n_pos = pos - self.pos * CHUNK_SIZE as f32;
        let voxel_index = Chunk::get_voxel(n_pos);

        self.voxels[voxel_index] = Voxel::air();
        self.is_mesh = false; // so the mesh rebuilds
    }
    
    pub fn get_voxel(pos: Vector3<f32>) -> usize {
        let x = ((pos.x as i32 % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize;
        let y = ((pos.y as i32 % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize;
        let z = ((pos.z as i32 % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize;

        x * (CHUNK_SIZE * CHUNK_SIZE) + y * CHUNK_SIZE + z
    }

    pub fn gen_mesh_data_no_culling(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for current_voxel_pos in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            let x = current_voxel_pos / (CHUNK_SIZE * CHUNK_SIZE);
            let y = (current_voxel_pos % (CHUNK_SIZE * CHUNK_SIZE)) / CHUNK_SIZE;
            let z = current_voxel_pos % CHUNK_SIZE;

            let voxel = &self.voxels[current_voxel_pos];

            if voxel.id == 0 { // voxel is air
                continue;
            }

            vertices.push(Vertex::new(x as f32, y as f32, z as f32));
            vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32));
            vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32));
            vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32));
            vertices.push(Vertex::new(x as f32, y as f32, z as f32 + 1.0));
            vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32 + 1.0));
            vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0));
            vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32 + 1.0));

            let voxel_indices = [
                0, 1, 2, 2, 3, 0,
                1, 5, 6, 6, 2, 1,
                5, 4, 7, 7, 6, 5,
                4, 0, 3, 3, 7, 4,
                3, 2, 6, 6, 7, 3,
                4, 5, 1, 1, 0, 4,
            ];

            for &index in &voxel_indices {
                indices.push((index as u32) + index_offset);
            }

            index_offset += 8; // 8 vertices per voxel
        }

        (vertices, indices)
    }

    pub fn gen_mesh_data_culled(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for current_voxel_pos in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            let x = current_voxel_pos / (CHUNK_SIZE * CHUNK_SIZE);
            let y = (current_voxel_pos % (CHUNK_SIZE * CHUNK_SIZE)) / CHUNK_SIZE;
            let z = current_voxel_pos % CHUNK_SIZE;

            let voxel = &self.voxels[current_voxel_pos];

            if voxel.id() == 0 {
                continue;
            }

            let mut visible_faces = [false; 6];

            if self.is_visible(current_voxel_pos, (-1, 0, 0)) { visible_faces[0] = true; }
            if self.is_visible(current_voxel_pos, (1, 0, 0)) { visible_faces[1] = true; }
            if self.is_visible(current_voxel_pos, (0, -1, 0)) { visible_faces[2] = true; }
            if self.is_visible(current_voxel_pos, (0, 1, 0)) { visible_faces[3] = true; }
            if self.is_visible(current_voxel_pos, (0, 0, -1)) { visible_faces[4] = true; }
            if self.is_visible(current_voxel_pos, (0, 0, 1)) { visible_faces[5] = true; }

            for (face_idx, &visible) in visible_faces.iter().enumerate() {
                if !visible {
                    continue;
                }

                let start_vertex_idx = vertices.len() as u32;

                match face_idx {
                    0 => { // left
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32));
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32));
                    },
                    1 => { // right
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32 + 1.0));
                    },
                    2 => { // bottom
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32 + 1.0));
                    },
                    3 => { // top
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32));
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32));
                    },
                    4 => { // back
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32));
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32));
                    },
                    5 => { // front
                        vertices.push(Vertex::new(x as f32, y as f32, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0));
                        vertices.push(Vertex::new(x as f32, y as f32 + 1.0, z as f32 + 1.0));
                    },
                    _ => {}
                }

                indices.push(start_vertex_idx);
                indices.push(start_vertex_idx + 1);
                indices.push(start_vertex_idx + 2);
                indices.push(start_vertex_idx + 2);
                indices.push(start_vertex_idx + 3);
                indices.push(start_vertex_idx);
            }
        }

        (vertices, indices)
    }

    pub fn is_visible(&self, pos: usize, direction: (isize, isize, isize)) -> bool {
        let x = (pos / (CHUNK_SIZE * CHUNK_SIZE)) as isize;
        let y = ((pos % (CHUNK_SIZE * CHUNK_SIZE)) / CHUNK_SIZE) as isize;
        let z = (pos % CHUNK_SIZE) as isize;

        let (dx, dy, dz) = direction;

        let nx = x + dx;
        let ny = y + dy;
        let nz = z + dz;

        if nx < 0 || nx >= CHUNK_SIZE as isize
        || ny < 0 || ny >= CHUNK_SIZE as isize
        || nz < 0 || nz >= CHUNK_SIZE as isize {
            return true;
        }

        let neighbor_pos = nx as usize * (CHUNK_SIZE * CHUNK_SIZE)
                        + ny as usize * CHUNK_SIZE
                        + nz as usize;

        self.voxels[neighbor_pos].id() == 0
    }
}

use std::collections::{HashMap, VecDeque};
use crate::mesh::Mesh;
use cgmath::Vector2;
use crate::camera::Camera;
use crate::shaders::*;
use crate::shader::*;
use std::ffi::CString;
use cgmath::prelude::*;
use crate::cstr;
#[derive(Clone)]
pub struct World {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
    pub meshes: HashMap<Vector3<i32>, Mesh>,
    pub mesh_shader: Shader,
    pub camera_pos: Vector3<f32>,
}

impl World {
    pub fn new() -> Self {
        let mut chunks = HashMap::new();
        let chunk = Chunk::new(Vector3::new(0.0, 0.0, 0.0));
        let mesh_shader_pipeline = Shader::new_pipeline(MESH_SHADER_VS, MESH_SHADER_FS);

        chunks.insert(Vector3::new(0, 0, 0), chunk);

        Self {
            meshes: HashMap::new(),
            chunks,
            camera_pos: Vector3::zero(),
            mesh_shader: mesh_shader_pipeline,
        }
    }

    pub async fn update(&mut self) { 
        let chunk_pos = Vector3::new(
            (self.camera_pos.x / CHUNK_SIZE as f32).floor() as i32, 
            (self.camera_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (self.camera_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        let mut queue = VecDeque::new();
        queue.push_back(chunk_pos);

        let directions = [
            Vector3::new(-1, 0, 0),
            Vector3::new(1, 0, 0),
            Vector3::new(0, -1, 0),
            Vector3::new(0, 1, 0),
            Vector3::new(0, 0, 1),
            Vector3::new(0, 0, -1),
        ];

        while let Some(current_pos) = queue.pop_front() {
            if !self.chunks.contains_key(&current_pos) {
                let p1 = Vector3::new(current_pos.x as f32, current_pos.y as f32, current_pos.z as f32);
                let chunk = Chunk::new(p1);
                self.chunks.insert(current_pos, chunk);
            }

            for direction in &directions {
                let new_pos = current_pos + *direction;
                let p1 = Vector3::new(new_pos.x as f32, new_pos.y as f32, new_pos.z as f32);
                let p2 = Vector3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);

                if !self.chunks.contains_key(&new_pos) && p1.distance(p2) <= 2.0 {
                    queue.push_back(new_pos);
                }
            }
        }

        let mut chunks_to_remove = Vec::new();
        for chunk in self.chunks.keys() {
            let pos = *chunk;
            let p1 = Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32);
            let p2 = Vector3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);

            let chunk_data = self.chunks.get(chunk).unwrap();
            let time = (std::time::Instant::now() - chunk_data.creation_instant)
                .as_secs_f32();
            if p1.distance(p2) > 4.0 {
                chunks_to_remove.push(pos);
            }
        }
        for pos in chunks_to_remove {
            self.meshes.remove(&pos);
            self.chunks.remove(&pos);
        }
    }

    pub fn draw(&mut self, camera: &Camera) {
        let chunk_pos = Vector3::new(
            (camera.pos_x.x / CHUNK_SIZE as f32).floor() as i32, 
            (camera.pos_x.y / CHUNK_SIZE as f32).floor() as i32, 
            (camera.pos_x.z / CHUNK_SIZE as f32).floor() as i32
        );

        for chunk in &mut self.chunks {
            let pos = chunk.0;

            let p1 = Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32);
            let p2 = Vector3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);

            if !chunk.1.is_mesh {
                let mesh_data = chunk.1.gen_mesh_data_culled();
                let mesh = Mesh::new(mesh_data.0, mesh_data.1);
                chunk.1.is_mesh = true;
                
                if let Some(mesh) = self.meshes.get_mut(pos) {
                    unsafe { mesh.destroy(); }
                }
                self.meshes.remove(pos);
                self.meshes.insert(*pos, mesh);
            }

            if let Some(mesh) = self.meshes.get(pos) {
                unsafe {
                    camera.send_uniforms(&self.mesh_shader);
                    self.mesh_shader.uniform_vec3f(cstr!("chunkPos"), 
                        &Vector3::new(
                            (pos.x * CHUNK_SIZE as i32) as f32, 
                            (pos.y * CHUNK_SIZE as i32) as f32, 
                            (pos.z * CHUNK_SIZE as i32) as f32
                        )
                    );
                    if p2.distance(p1) > 8.0 { 
                    } else {
                        mesh.draw(&self.mesh_shader);
                    }
                }
            }
        }
    }

    pub fn chunk_in_camera(&mut self, pos: Vector3<f32>) -> Option<&mut Chunk> {
        let chunk_pos = Vector3::new(
            (pos.x / CHUNK_SIZE as f32).floor() as i32, 
            (pos.y / CHUNK_SIZE as f32).floor() as i32,
            (pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        self.chunks.get_mut(&chunk_pos)
    }

    pub fn remove_voxel_raycasting(&mut self, cam_pos: Vector3<f32>, dir: Vector3<f32>) {
        let mut curr_pos = cam_pos;
        let step_size = 0.1;
        let max_steps = (5.0 / step_size) as usize; // maximum distance is 5

        for _ in 0..max_steps {
            let chunk_pos = Vector3::new(
                (curr_pos.x / CHUNK_SIZE as f32).floor() as i32,
                (curr_pos.y / CHUNK_SIZE as f32).floor() as i32,
                (curr_pos.z / CHUNK_SIZE as f32).floor() as i32,
            );

            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                let local_pos = curr_pos - chunk_pos.cast::<f32>().unwrap() * CHUNK_SIZE as f32;
                let voxel_idx = Chunk::get_voxel(local_pos);

                if chunk.voxels[voxel_idx].id != 0 {
                    chunk.destroy_voxel(local_pos);
                    return;
                }
            }

            curr_pos += dir * step_size;
        }
    }
}