use std::time::{Duration, SystemTime};

use char::{AnsiColorMode, CharColor, CharInfo};
use glm::{look_at, make_vec2, make_vec3, mat4_to_mat3, perspective, rotate, scale, vec3, vec4_to_vec3, Vec3};
use raster::{half_block_shader, CharHalf, Framebuf};
use server::SshSession;
use vertex::Vertex;

pub mod raster;
pub mod char;
pub mod vertex;
pub mod clip;
pub mod server;
pub mod client;

extern crate nalgebra_glm as glm;

#[tokio::main]
async fn main() {
    let start_time = SystemTime::now();

    let (models, _materials) = tobj::load_obj("assets/low-poly-skull.obj", &tobj::GPU_LOAD_OPTIONS)
        .expect("Failed to load file");

    let mut fb = Framebuf::new(48, 48);

    let camera_pos = vec3(0.0, -0.5, 4.0);
    let proj_matrix = perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
    let view_matrix = look_at(&camera_pos, &vec3(0.0, -0.5, 0.0), &vec3(0.0, -1.0, 0.0));

    let mut server = SshSession::new();
    let server_clone = server.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(5)).await;

            fb.clear();

            let elapsed_time = start_time.elapsed().unwrap().as_millis() as f32;
            
            let mut model_matrix = rotate(&glm::identity(), elapsed_time / 600.0, &vec3(0.0, 1.0, 0.0));
            //model_matrix = rotate(&model_matrix, elapsed_time / 1500.0, &vec3(1.0, 0.0, 0.0));
            model_matrix = scale(&model_matrix, &vec3(1.8, 1.8, 1.8));

            let vp_matrix = proj_matrix * view_matrix;
            let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());

            fb.draw_model(&models[0], &model_matrix, &vp_matrix, &normal_matrix, &camera_pos, |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf| {
                let light_color = vec3(1.0, 1.0, 1.0);
                let light_pos = vec3(100.0, 0.0, 50.0);
                let light_direction = (light_pos - vec4_to_vec3(&vertex.position)).normalize();
                
                let ambient_strength = 0.0;
                let ambient = ambient_strength * light_color;
                
                let normal = make_vec3(&vertex.attributes[2..5]);
                let diffuse = normal.dot(&light_direction).max(0.0) * light_color;

                let tex_coord = make_vec2(&vertex.attributes[0..2]);
                let size = 8.0;
                let pattern: bool = ((tex_coord[0] * size % 1.0) > 0.5) ^ ((tex_coord[1] * size % 1.0) < 0.5);
                let object_color = if pattern { vec3(0.8, 0.85, 1.0) } else { vec3(0.2, 0.6, 0.8) };
                let result = (ambient + diffuse).component_mul(&object_color) + vec3(0.2, 0.2, 0.2);
                
                half_block_shader(c, &half, 
                    &CharColor { r: (result.x * 255.0) as u8, g: (result.y * 255.0) as u8, b: (result.z * 255.0) as u8 });
            });

            let mut str = "\x1b[?25l\x1b[H".to_owned(); // Make cursor invisible and move cursor to beginning.
            str.push_str(&fb.to_string(&AnsiColorMode::AnsiTrueColor));
            str.push_str(&format!("Users online: {}\r\n", server_clone.num_sessions().await));
            
            server_clone.clone().broadcast_message(str).await;
        }
    });

    println!("SSH server started and listening on port 22");
    server.run().await;
}
