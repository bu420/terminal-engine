use std::time::SystemTime;

use char::{AnsiColorMode, CharColor, CharInfo};
use glm::{rotate, vec3, vec4};
use raster::{half_block_shader, CharHalf, Framebuf};
use vertex::Vertex;

pub mod raster;
pub mod char;
pub mod vertex;
pub mod clip;

extern crate nalgebra_glm as glm;

fn main() {
    let time_start = SystemTime::now();

    //let (models, materials) = tobj::load_obj("cube.obj", &tobj::LoadOptions::default())
    //    .expect("Failed to load cube.obj");

    let mut fb = Framebuf::new(48, 48);

    // Make cursor invisible.
    print!("\x1b[?25l");

    // Save cursor position.
    print!("\x1b[s");

    let proj_matrix = glm::perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
    let view_matrix = glm::look_at(
        &glm::vec3(0.0, 0.0, 3.0), 
        &glm::vec3(0.0, 0.0, 0.0), 
        &glm::vec3(0.0, -1.0, 0.0));

    loop {
        // Restore cursor position.
        print!("\x1b[u");

        fb.clear();

        let elapsed_time = time_start.elapsed().unwrap().as_millis() as f32;

        let model_matrix = rotate(&glm::identity(), elapsed_time / 1000.0, &vec3(0.0, 0.0, 1.0));

        let mvp_matrix = proj_matrix * view_matrix * model_matrix;

        let v0 = Vertex {position: mvp_matrix * vec4(0.0, -1.0, 0.0, 1.0), attributes: vec![0.7, 0.4, 0.8] };
        let v1 = Vertex {position: mvp_matrix * vec4(-1.0, 1.0, 0.0, 1.0), attributes: vec![0.3, 0.9, 0.7] };
        let v2 = Vertex {position: mvp_matrix * vec4(1.0, 1.0, 0.0, 1.0), attributes: vec![0.6, 0.7, 0.5] };

        fb.draw_triangle(&[&v0, &v1, &v2], |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf| {
            let r = (vertex.attributes[0] * 255.0) as u8;
            let g = (vertex.attributes[1] * 255.0) as u8;
            let b = (vertex.attributes[2] * 255.0) as u8;
            half_block_shader(c, &half, &CharColor { r, g, b });
        });

        let white_shader = |_: &Vertex, c: &mut CharInfo, half: &CharHalf| 
            half_block_shader(c, &half, &CharColor { r: 255u8, g: 255u8, b: 255u8 });

        fb.draw_line(&v0, &v1, white_shader);
        fb.draw_line(&v1, &v2, white_shader);
        fb.draw_line(&v2, &v0, white_shader);

        fb.print(&AnsiColorMode::AnsiTrueColor);
    }

    // Make cursor visible.
    //write!("\x1b[?25h");
}
