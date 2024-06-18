use core::task;
use std::{ffi::CString, sync::Arc};

use cgmath::vec3;
use glfw::*;
use gl::*;
use gl::types::*;
use lingering_framebuffer::LingeringFramebuffer;
use rand::random;
use tokio::{spawn, sync::{watch, Mutex}};
use util::{rand_betw, SecondOrderDynamics};
use world::{Chunk, World};

use crate::{camera::Camera, mesh::{Mesh, Vertex}, shader::Shader};

mod shader;
mod mesh;
mod shaders;
mod camera;
mod util;
mod world;
mod lingering_framebuffer;

#[tokio::main]
async fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

     // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw.create_window(1000, 1000, "g-fl", glfw::WindowMode::Windowed)
         .expect("Failed to create GLFW window.");

    glfw.window_hint(
        glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core)
    );
    
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(0));
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    
    load_with(|s| window.get_proc_address(s) as * const _); // load gl function pointers

    unsafe {
        Enable(DEPTH_TEST);
        Enable(ALPHA);
        Enable(ONE_MINUS_SRC_ALPHA);
        Enable(ONE_MINUS_DST_ALPHA);
    }

    let mut my_mesh = Mesh::new(
       (0..=2000).map(|i| {
           Vertex::new(rand_betw(0.0, 1.0), rand_betw(0.0, 1.0), rand_betw(0.0, 1.0))
        }).collect(), 
       (0..=2000).filter(|i| i % 2 == 0)
                .flat_map(|i| {
                    let next_row = i + 1;
                    let next_col = i + 2;
                    vec![i, next_row, next_col, i+1]
                }).collect(),
    );

    let mesh_shader_pipeline = Shader::new_pipeline(shaders::MESH_SHADER_VS, shaders::MESH_SHADER_FS);
    let graph_shader_pipeline = Shader::new_pipeline(shaders::GRAPH_SHADER_VS, shaders::GRAPH_SHADER_FS);
    let lingering_framebuffer_shader_pipeline = Shader::new_pipeline(shaders::LINGERING_SHADER_VS, shaders::LINGERING_SHADER_FS);
    //unsafe { mesh_shader_pipeline.use_shader(); };
    
    let mut camera = Camera::new();
    camera.pos_x = vec3(-1000.0, -1000.0, -1000.0); // the game works better here
    
    let mut sod = SecondOrderDynamics::new(2.5, 0.8, 0.5, vec3(0.0, 0.0, 0.0));
    let y = sod.update(0.0016, camera.pos_x);
    
    // let chunk = Chunk::new();
    // let chunk_mesh_data = chunk.gen_mesh_data_culled();
    // let chunk_mesh = Mesh::new(chunk_mesh_data.0, chunk_mesh_data.1);

    let graph_mesh = Mesh::new(
        vec![
            Vertex::new(-1.0, -1.0, 0.0),
        ],
        vec![0]
    );
    
    let fb_size = 0.5;
    let fb_res = 250;
    let lingering_framebuffer = unsafe {LingeringFramebuffer::new(fb_size, fb_size, fb_res, fb_res, &lingering_framebuffer_shader_pipeline)};
    unsafe {lingering_framebuffer_shader_pipeline.uniform_1i(cstr!("screenTexture"), lingering_framebuffer.texture.try_into().unwrap());};

    let mut time = 0.0;
    let mut world_buffer = World::new();
    
    while !window.should_close() {
        let now = std::time::Instant::now();
        window.swap_buffers();
        
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    camera.mouse_callback(x as f32, y as f32, &mut window);
                }
                _ => {},
            }
        }
        
        world_buffer.update().await;

        if window.get_mouse_button(MouseButtonLeft) == Action::Press {
            world_buffer.remove_voxel_raycasting(camera.pos_x, camera.front);
        }

        world_buffer.camera_pos = camera.pos_x;

        unsafe {
            BindFramebuffer(FRAMEBUFFER, 0);
            Enable(DEPTH_TEST); 
            ClearColor(0.1, 0.2, 0.3, 1.0);
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            
            camera.send_uniforms(&mesh_shader_pipeline);
            // my_mesh.draw(&mesh_shader_pipeline);

            world_buffer.draw(&camera);

            /*
            // todo: add the graph in its own class
            BindFramebuffer(FRAMEBUFFER, lingering_framebuffer.fbo);
            Disable(DEPTH_TEST); 

            graph_mesh.draw_points(&graph_shader_pipeline);

            plot_data(
               &graph_shader_pipeline, 
               (time / 10.0) % 1.0, 
               (f32::sin(time) + f32::sin(time * 2.0) + f32::sin(time * 4.0)) / 4.0, 
               fb_size,
               fb_res as f32 / 1000.0,
            );

            BindFramebuffer(FRAMEBUFFER, 0);
            lingering_framebuffer.draw(&lingering_framebuffer_shader_pipeline, 1.0, 1.0);
            */
            // mesh_shader_pipeline.use_shader();   
        }

        camera.update(sod.update(now.elapsed().as_secs_f32(), camera.pos_x));  
        camera.input(&mut window, &glfw);

        time+=now.elapsed().as_secs_f32();
     }
 }

unsafe fn plot_data(shader: &Shader, x: f32, y: f32, size: f32, reso_ratio: f32) {
    let one = reso_ratio * (1.0 / size);

    //let x_ranged = (x + 1.0) * 0.5;
    let y_ranged = (y + 1.0) * 0.5;

    shader.uniform_1f(cstr!("ofsx"), x * one);
    shader.uniform_1f(cstr!("ofsy"), y_ranged * one);
}
