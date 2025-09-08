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

pub struct CubeScene {
    model: Model,
}

impl CubeScene {
    pub fn new() -> Self {
        let (models, _materials) = tobj::load_obj("assets/cube.obj", &tobj::GPU_LOAD_OPTIONS)
            .expect("Failed to load file");
        Self {
            model: models[0].clone(),
        }
    }
}

impl Scene for CubeScene {
    fn run(&self, fb: &mut Framebuf, elapsed_time: f32) {
        let camera_pos = vec3(0.0, -0.5, 8.5);
        let proj_matrix = perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
        let view_matrix = look_at(&camera_pos, &vec3(0.0, -0.5, 0.0), &vec3(0.0, -1.0, 0.0));

        let angle_y = (elapsed_time as f32 / 600.0).sin() * std::f32::consts::PI; // -π to π
        let angle_x = (elapsed_time as f32 / 1500.0).sin() * std::f32::consts::FRAC_PI_2; // -π/2 to π/2

        let mut model_matrix = rotate(&glm::identity(), angle_y, &vec3(0.0, 1.0, 0.0));
        model_matrix = rotate(&model_matrix, angle_x, &vec3(1.0, 0.0, 0.0));
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
                let light_color = vec3(1.0, 1.0, 1.0);
                let light_pos = vec3(100.0, 0.0, 50.0);
                let light_direction = (light_pos - vec4_to_vec3(&vertex.position)).normalize();

                let ambient_strength = 0.0;
                let ambient = ambient_strength * light_color;

                let normal = make_vec3(&vertex.attributes[2..5]);
                let diffuse = normal.dot(&light_direction).max(0.0f32) * light_color;

                let tex_coord = make_vec2(&vertex.attributes[0..2]);
                let size = 8.0;
                let pattern: bool =
                    ((tex_coord[0] * size % 1.0) > 0.5) ^ ((tex_coord[1] * size % 1.0) < 0.5);
                let object_color = if pattern {
                    vec3(0.8, 0.3, 0.5)
                } else {
                    vec3(0.2, 0.6, 0.8)
                };
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
