use std::{iter::zip, time::SystemTime};

use char::{AnsiColorMode, CharColor, CharInfo};
use glm::{mat4_to_mat3, rotate, vec3, vec4, Mat3};
use itertools::Itertools;
use raster::{half_block_shader, CharHalf, Framebuf};
use vertex::Vertex;

pub mod raster;
pub mod char;
pub mod vertex;
pub mod clip;

extern crate nalgebra_glm as glm;

fn main() {
    let time_start = SystemTime::now();

    let (models, materials) = tobj::load_obj("assets/statue.obj", &tobj::GPU_LOAD_OPTIONS)
        .expect("Failed to load file");
    let mesh = &models[0].mesh;

    let mut fb = Framebuf::new(128, 128);

    // Make cursor invisible.
    print!("\x1b[?25l");

    // Save cursor position.
    print!("\x1b[s");

    let proj_matrix = glm::perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
    let view_matrix = glm::look_at(&vec3(0.0, 20.0, 25.0), &vec3(0.0, 20.0, 0.0), &vec3(0.0, -1.0, 0.0));

    let mut previous_time = SystemTime::now();

    loop {
        let elapsed_time = time_start.elapsed().unwrap().as_millis() as f32;
        let current_time = SystemTime::now();
        let delta_time = current_time.duration_since(previous_time).unwrap().as_millis();
        previous_time = current_time;

        // Restore cursor position.
        print!("\x1b[u");

        fb.clear();
        
        let mut model_matrix = rotate(&glm::identity(), elapsed_time / 1000.0, &vec3(0.0, 1.0, 0.0));
        //model_matrix = rotate(&model_matrix, elapsed_time / 1700.0, &vec3(0.0, 1.0, 0.0));
        //model_matrix = rotate(&model_matrix, elapsed_time / 2400.0, &vec3(1.0, 0.0, 0.0));

        let mvp_matrix = proj_matrix * view_matrix * model_matrix;
        let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());

        for indices in mesh.indices.chunks(3) {            
            let get_position = |i| mvp_matrix * vec4(
                mesh.positions[3 * i as usize],
                mesh.positions[3 * i as usize + 1],
                mesh.positions[3 * i as usize + 2],
                1.0,
            );

            let get_normal = |i| {
                let normal = normal_matrix * vec3(
                    mesh.normals[3 * i as usize], 
                    mesh.normals[3 * i as usize + 1], 
                    mesh.normals[3 * i as usize + 2]
                );
                vec![normal[0], normal[1], normal[2]]
            };

            let v0 = Vertex { position: get_position(indices[0]), attributes: get_normal(indices[0]) };
            let v1 = Vertex { position: get_position(indices[1]), attributes: get_normal(indices[1]) };
            let v2 = Vertex { position: get_position(indices[2]), attributes: get_normal(indices[2]) };

            fb.draw_triangle(&[&v0, &v1, &v2], |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf| {
                let r = ((vertex.attributes[0] + 1.0) / 2.0 * 255.0) as u8;
                let g = ((vertex.attributes[1] + 1.0) / 2.0 * 255.0) as u8;
                let b = ((vertex.attributes[2] + 1.0) / 2.0 * 255.0) as u8;
                half_block_shader(c, &half, &CharColor { r, g, b });
            });
        }

        fb.print(&AnsiColorMode::AnsiTrueColor);

        // Restore cursor position.
        print!("\x1b[u");

        println!("Frame: {delta_time}ms");
    }

    // Make cursor visible.
    //write!("\x1b[?25h");
}
