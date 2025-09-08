use glm::{
    look_at, make_vec2, make_vec3, mat4_to_mat3, perspective, rotate, scale, vec3, vec4_to_vec3,
};
use tobj::Model;

use crate::{
    char::{CharColor, CharInfo},
    raster::{half_block_shader, CharHalf, Framebuf},
    scene::Scene,
    vertex::Vertex,
};

pub struct BullScene {
    model: Model,
}

impl BullScene {
    pub fn new() -> Self {
        let (models, _materials) = tobj::load_obj("assets/bull.obj", &tobj::GPU_LOAD_OPTIONS)
            .expect("Failed to load file");
        Self {
            model: models[0].clone(),
        }
    }
}

impl Scene for BullScene {
    fn run(&self, fb: &mut Framebuf, elapsed_time: f32) {
        let camera_pos = vec3(0.0, 0.5, 6.0);
        let proj_matrix = perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
        let view_matrix = look_at(&camera_pos, &vec3(0.0, -0.5, 0.0), &vec3(0.0, -1.0, 0.0));

        let mut model_matrix = rotate(&glm::identity(), elapsed_time / 200.0, &vec3(0.0, 1.0, 0.0));
        model_matrix = scale(&model_matrix, &vec3(1.8, 1.8, 1.8));

        let vp_matrix = proj_matrix * view_matrix;
        let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());

        fb.draw_model(
            &self.model,
            &model_matrix,
            &vp_matrix,
            &normal_matrix,
            &camera_pos,
            elapsed_time,
            |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf, _elapsed_time: f32| {
                // Rotating light
                let angle = _elapsed_time / 1000.0;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let base_light_pos = vec3(50.0, 0.0, 20.0);
                let rotated_light_pos = vec3(
                    cos_a * base_light_pos.x - sin_a * base_light_pos.y,
                    sin_a * base_light_pos.x + cos_a * base_light_pos.y,
                    base_light_pos.z,
                );

                let light_color = vec3(1.0, 1.0, 1.0);
                let light_dir = (rotated_light_pos - vec4_to_vec3(&vertex.position)).normalize();

                let normal = make_vec3(&vertex.attributes[2..5]);
                let diffuse = normal.dot(&light_dir).max(0.0f32) * light_color;
                let ambient = 0.2 * light_color;

                // Spot pattern based on UV coordinates
                let tex_coord = make_vec2(&vertex.attributes[0..2]);
                let scale = 15.0; // size of spots
                let pattern = ((tex_coord[0] * scale).sin() * (tex_coord[1] * scale).sin()) > 0.0;

                // Bull/cow colors
                let base_color = vec3(0.8, 0.6, 0.4); // brown
                let spot_color = vec3(0.1, 0.1, 0.1); // dark spots
                let object_color = if pattern { spot_color } else { base_color };

                let result = (ambient + diffuse).component_mul(&object_color) + vec3(0.2, 0.2, 0.2);

                half_block_shader(
                    c,
                    &half,
                    &CharColor {
                        r: (result.x * 255.0) as u8,
                        g: (result.y * 255.0) as u8,
                        b: (result.z * 255.0) as u8,
                    },
                );
            },
        );
    }
}
