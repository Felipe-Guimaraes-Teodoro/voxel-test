use cgmath::*;

use glfw::{self, Action, Key};

use crate::{cstr, shader::Shader};

const UP: Vector3<f32> = Vector3 {x: 0.0, y: 1.0, z: 0.0};
const SPEED: f32 = 5.0;
const SENSITIVITY: f32 = 0.001;

pub enum ProjectionType {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub proj: Matrix4<f32>,
    pub view: Matrix4<f32>,

    pub pos_x: Vector3<f32>,
    pub pos_y: Vector3<f32>,
    target: Vector3<f32>,
    direction: Vector3<f32>,
    pub right: Vector3<f32>,
    pub front: Vector3<f32>,
    pub up: Vector3<f32>,

    pitch: f32,
    yaw: f32,

    pub dt: f32,
    last_frame: f32,

    first_mouse: bool,
    last_x: f32,
    last_y: f32,
}

impl Camera {
    pub fn new() -> Self {
        let (pitch, yaw): (f32, f32) = (0.0, 0.0);
        let pos = vec3(0.0, 0.0, 0.0);
        let target = vec3(0.0, 0.0, -1.0);
        let mut direction = Vector3::normalize(pos - target);
        direction.x = Rad::cos(Rad(yaw)) * Rad::cos(Rad(pitch));
        direction.y = Rad::sin(Rad(pitch));
        direction.z = Rad::sin(Rad(yaw)) * Rad::cos(Rad(pitch));
        
        let right = Vector3::normalize(Vector3::cross(UP, direction));
        let up = Vector3::cross(direction, right);
        let front = Vector3::normalize(direction);

        let view = Matrix4::look_at_rh(
            Point3::from_vec(pos),
            Point3::from_vec(pos + front),
            up,
        );

        Self {
            proj: perspective(Deg(70.0), 1.0, 0.1, 100000.0),
            view, 

            pos_x: pos,
            pos_y: pos,
            target,
            direction,
            right,
            front,
            up,
            
            pitch,
            yaw,

            dt: 0.0,
            last_frame: 0.0,

            first_mouse: true,
            last_x: 400.0,
            last_y: 400.0,
        }
    }

    pub fn update(&mut self, y: Vector3<f32>) {
        self.pos_y = y;
        
        self.view = Matrix4::look_at_rh(
            Point3::from_vec(self.pos_y),
            Point3::from_vec(self.pos_y + self.front),
            self.up,
        );
    }

    pub fn input(
        &mut self,
        window: &mut glfw::Window, 
        glfw: &glfw::Glfw
    ) {
        let mut speed = SPEED;
        let curr_frame = glfw.get_time() as f32;
        self.dt = curr_frame - self.last_frame;
        self.last_frame = curr_frame;

        if window.get_key(Key::LeftShift) == Action::Press {
            speed *= 20.0;
        }
        if window.get_key(Key::RightShift) == Action::Press {
            speed *= 20.0;
        }

        if window.get_key(Key::W) == Action::Press {
            self.pos_x += speed * self.dt * self.front; 
        }
        if window.get_key(Key::S) == Action::Press {
            self.pos_x -= speed * self.dt * self.front; 
        }
        if window.get_key(Key::Space) == Action::Press {
            self.pos_x += speed * self.dt * self.up;
        }
        if window.get_key(Key::LeftControl) == Action::Press {
            self.pos_x -= speed * self.dt * self.up;
        }
        if window.get_key(Key::A) == Action::Press {
            self.pos_x -= speed * self.dt * Vector3::cross(self.front, self.up); 
        }
        if window.get_key(Key::D) == Action::Press {
            self.pos_x += speed * self.dt * Vector3::cross(self.front, self.up); 
        }

        let (w, h) = window.get_framebuffer_size();
        let aspect_ratio = w as f32 / h as f32;
        self.proj = perspective(Deg(70.0), aspect_ratio, 0.1, 1000.0);
    }

    pub fn mouse_callback(
        &mut self, 
        xpos: f32, 
        ypos: f32,
        window: &glfw::Window,
    ) {
        if window.get_cursor_mode() != glfw::CursorMode::Disabled {
            self.first_mouse = true;
            return 
        };
        if self.first_mouse { 
            self.last_x = xpos;
            self.last_y = ypos;
            self.first_mouse = false;
        }

        let mut xoffs = xpos - self.last_x;
        let mut yoffs = self.last_y - ypos;

        self.last_x = xpos;
        self.last_y = ypos;

        xoffs *= SENSITIVITY;
        yoffs *= SENSITIVITY;

        self.yaw += xoffs;
        self.pitch += yoffs;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } 
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.direction.x = Rad::cos(Rad(self.yaw)) * Rad::cos(Rad(self.pitch));
        self.direction.y = Rad::sin(Rad(self.pitch));
        self.direction.z = Rad::sin(Rad(self.yaw)) * Rad::cos(Rad(self.pitch));

        self.front = Vector3::normalize(self.direction);
    }


    // RENDERING //
    pub unsafe fn send_uniforms(&self, shader: &Shader) {
        shader.uniform_mat4fv(
            cstr!("view"),
            &self.view
        );

        shader.uniform_mat4fv(
            cstr!("proj"),
            &self.proj
        );
    }

    pub fn set_projection(
        &mut self, 
        projection_type: ProjectionType,
    ) {
        match projection_type {
            ProjectionType::Perspective => {
                self.proj = perspective(Deg(70.0), 1.0, 0.1, 10000.0);
            },
            ProjectionType::Orthographic => {
                self.proj = ortho(-1.0, 1.0, -1.0, 1.0, -100.0, 100.0);
            }
        }
    }
}

